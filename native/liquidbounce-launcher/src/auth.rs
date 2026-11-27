// SPDX-License-Identifier: GPL-3.0-or-later
//! Microsoft / Xbox / Minecraft authentication matching LiquidLauncher.
//!
//! Reference: CCBlueX/LiquidLauncher a2420d4e93dc2700bfedca68ecbef143dfb66c2b,
//! src-tauri/src/minecraft/auth.rs, using minecraft-auth 0.1.1's JavaAuthManager.
//! The separate auth.liquidbounce.net application ID is NOT a Microsoft ID.

use minecraft_auth::java::JavaAuthManager;
use minecraft_auth::msa::{constants::SCOPE_TITLE_AUTH, MsaApplicationConfig};
use reqwest::Client;
use serde_json::{json, Value};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

/// This is the actual Microsoft title ID selected by current LiquidLauncher's
/// dependency, not its unrelated LiquidBounce account OAuth application ID.
pub const MICROSOFT_APPLICATION_ID: &str = "00000000402b5328";
pub const AUTH_SOURCE_COMMIT: &str = "a2420d4e93dc2700bfedca68ecbef143dfb66c2b";

#[derive(Clone)]
pub struct Account {
    pub name: String,
    pub uuid: String,
    pub premium: bool,
    pub avatar_png: Option<Vec<u8>>,
    /// Only transfer through the child's stdin; never serialize into settings.
    pub session_json: Option<String>,
    pub expires_at: u64,
}

impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("name", &self.name)
            .field("uuid", &self.uuid)
            .field("premium", &self.premium)
            .field("session_json", &"<redacted>")
            .finish()
    }
}

pub enum AuthEvent {
    DeviceCode {
        user_code: String,
        verification_uri: String,
        direct_uri: String,
    },
    Status(String),
    Complete(Account),
    Error(String),
    Cancelled,
}

pub struct AuthTask {
    pub receiver: Receiver<AuthEvent>,
    control: Arc<AuthControl>,
}

#[derive(Default)]
struct AuthControl {
    cancelled: AtomicBool,
    commit_lock: Mutex<()>,
}

impl AuthTask {
    pub fn cancel(&self) {
        self.control.cancelled.store(true, Ordering::Release);
        // After this lock is released, callers can remove or replace the cache:
        // an old worker can never commit again after cancel() has returned.
        let _commit = self
            .control
            .commit_lock
            .lock()
            .unwrap_or_else(|e| e.into_inner());
    }
}

impl Drop for AuthTask {
    fn drop(&mut self) {
        self.cancel();
    }
}

pub fn offline_account(name: &str) -> Result<Account, String> {
    let name = name.trim();
    if name.is_empty()
        || name.len() > 16
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("玩家名需为 1–16 位英文字母、数字或下划线".into());
    }
    let mut bytes = md5::compute(format!("OfflinePlayer:{name}")).0;
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(Account {
        name: name.into(),
        uuid: uuid::Uuid::from_bytes(bytes).hyphenated().to_string(),
        premium: false,
        avatar_png: None,
        session_json: None,
        expires_at: 0,
    })
}

pub fn start_login(cache_path: PathBuf) -> AuthTask {
    start(cache_path, false)
}

pub fn start_refresh(cache_path: PathBuf) -> AuthTask {
    start(cache_path, true)
}

fn start(cache_path: PathBuf, refresh: bool) -> AuthTask {
    let (sender, receiver) = mpsc::channel();
    let control = Arc::new(AuthControl::default());
    let worker_control = control.clone();
    let worker_sender = sender.clone();
    let spawn = thread::Builder::new()
        .name("Microsoft sign-in".into())
        .spawn(move || {
            let sender = worker_sender;
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(_) => {
                    let _ = sender.send(AuthEvent::Error("无法创建登录任务，请重试".into()));
                    return;
                }
            };
            let result = runtime.block_on(async {
                tokio::select! {
                    result = authenticate(&cache_path, refresh, &sender, &worker_control) => Some(result),
                    _ = async {
                        while !worker_control.cancelled.load(Ordering::Acquire) {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    } => None,
                }
            });
            let event = if worker_control.cancelled.load(Ordering::Acquire) {
                AuthEvent::Cancelled
            } else {
                match result {
                    Some(Ok(account)) => AuthEvent::Complete(account),
                    Some(Err(message)) => AuthEvent::Error(message),
                    None => AuthEvent::Cancelled,
                }
            };
            let _ = sender.send(event);
        });
    if spawn.is_err() {
        let _ = sender.send(AuthEvent::Error("无法启动登录任务，请重试".into()));
    }
    AuthTask { receiver, control }
}

async fn authenticate(
    cache_path: &Path,
    refresh: bool,
    sender: &Sender<AuthEvent>,
    control: &AuthControl,
) -> Result<Account, String> {
    let client = Client::builder()
        .user_agent("LiquidBounce-Rust-Launcher/0.1.0")
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(35))
        .build()
        .map_err(|_| "无法初始化安全网络连接".to_string())?;
    let manager = if refresh {
        let _ = sender.send(AuthEvent::Status("正在刷新正版登录…".into()));
        let protected =
            fs::read(cache_path).map_err(|_| "没有已保存的正版账号，请先登录".to_string())?;
        let state = unprotect(&protected)?;
        let mut saved: Value =
            serde_json::from_slice(&state).map_err(|_| "登录缓存已损坏，请重新登录".to_string())?;
        // The library normally reuses a token right up to its exact expiry.
        // Launchers refresh a minute early, so explicitly refetch this stage.
        // Other still-valid Microsoft/Xbox credentials remain reusable.
        saved
            .as_object_mut()
            .ok_or("登录缓存已损坏，请重新登录")?
            .insert("minecraft_token".into(), Value::Null);
        JavaAuthManager::from_json(client.clone(), &saved).map_err(auth_error)?
    } else {
        let _ = sender.send(AuthEvent::Status("正在获取微软登录代码…".into()));
        let callbacks = sender.clone();
        JavaAuthManager::builder(client.clone())
            .application_config(MsaApplicationConfig::new(
                MICROSOFT_APPLICATION_ID,
                SCOPE_TITLE_AUTH,
            ))
            .login_device_code(move |code| {
                let _ = callbacks.send(AuthEvent::DeviceCode {
                    user_code: code.user_code.clone(),
                    verification_uri: code.verification_uri.clone(),
                    direct_uri: code.direct_verification_uri(),
                });
            })
            .await
            .map_err(auth_error)?
    };
    let _ = sender.send(AuthEvent::Status("正在验证 Xbox 与 Minecraft 账号…".into()));
    let token = manager.minecraft_token().await.map_err(auth_error)?;
    let profile = manager.profile().await.map_err(auth_error)?;
    let session = game_session_json(&profile.name, profile.id, &token.access_token);
    // A skin download is cosmetic; account authentication is valid if it fails.
    // Complete this optional wait before committing so cancel can still discard
    // the new account without changing the account saved on disk.
    let avatar_png = tokio::time::timeout(
        Duration::from_secs(8),
        load_avatar(&client, &token.access_token),
    )
    .await
    .ok()
    .flatten();
    let _ = sender.send(AuthEvent::Status("正在保存加密登录状态…".into()));
    let state = manager.to_json().await.map_err(auth_error)?;
    let plain = serde_json::to_vec(&state).map_err(|_| "无法保存登录状态".to_string())?;
    commit_saved_login(cache_path, &plain, control)?;
    Ok(Account {
        name: profile.name,
        uuid: profile.id.hyphenated().to_string(),
        premium: true,
        avatar_png,
        session_json: Some(session),
        expires_at: (token.expire_time_ms.max(0) as u64) / 1000,
    })
}

/// Horizon reads this exact structure from stdin; no session file is created.
fn game_session_json(name: &str, uuid: uuid::Uuid, access_token: &str) -> String {
    json!({
        "name": name,
        "uuid": uuid.simple().to_string(),
        "access_token": access_token,
        "user_type": "msa",
    })
    .to_string()
}

fn auth_error(error: minecraft_auth::Error) -> String {
    use minecraft_auth::Error;
    match error {
        Error::ProfileNotFound => {
            "此微软账号没有 Minecraft Java 版档案；请确认已购买游戏并完成档案设置".into()
        }
        Error::DeviceCodeTimedOut => "登录代码已过期，请重新发起登录".into(),
        Error::Cancelled => "已取消登录".into(),
        Error::Msa { error, .. } => match error.as_str() {
            "access_denied" | "authorization_declined" => "你取消了微软授权".into(),
            "expired_token" => "登录代码已过期，请重试".into(),
            "invalid_grant" => "登录状态已失效，请重新登录微软账号".into(),
            _ => "微软登录失败，请稍后重试".into(),
        },
        Error::Xbl { error_code, .. } => match error_code {
            2148916233 => "此账号尚未建立 Xbox 档案，请先到 xbox.com 完成设置".into(),
            2148916238 => "此账号的 Xbox 家庭或年龄设置阻止了登录，请检查家庭权限".into(),
            _ => "Xbox 账号验证失败，请检查账号与家庭设置".into(),
        },
        Error::Http(e) => {
            if e.is_timeout() {
                "连接登录服务超时，请检查网络并重试".into()
            } else {
                "无法连接登录服务，请检查网络或代理设置".into()
            }
        }
        Error::MinecraftServices { .. } => "Minecraft 服务验证失败，请稍后重试".into(),
        _ => "登录状态无法读取，请重新登录".into(),
    }
}

async fn load_avatar(client: &Client, token: &str) -> Option<Vec<u8>> {
    let profile: Value = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(token)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    let skins = profile.get("skins")?.as_array()?;
    let skin = skins
        .iter()
        .find(|s| s.get("state").and_then(Value::as_str) == Some("ACTIVE"))
        .or_else(|| skins.first())?;
    let url = skin.get("url")?.as_str()?;
    let url = url.replacen(
        "http://textures.minecraft.net/",
        "https://textures.minecraft.net/",
        1,
    );
    if !url.starts_with("https://textures.minecraft.net/texture/") {
        return None;
    }
    let response = client.get(url).send().await.ok()?.error_for_status().ok()?;
    if response.content_length().is_some_and(|n| n > 2_000_000) {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    if bytes.len() > 2_000_000 {
        return None;
    }
    let skin = image::load_from_memory(&bytes).ok()?.into_rgba8();
    if skin.width() != 64 || !matches!(skin.height(), 32 | 64) {
        return None;
    }
    let mut face = image::imageops::crop_imm(&skin, 8, 8, 8, 8).to_image();
    let overlay = image::imageops::crop_imm(&skin, 40, 8, 8, 8).to_image();
    image::imageops::overlay(&mut face, &overlay, 0, 0);
    let mut encoded = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(face)
        .write_to(&mut encoded, image::ImageFormat::Png)
        .ok()?;
    Some(encoded.into_inner())
}

fn commit_saved_login(path: &Path, bytes: &[u8], control: &AuthControl) -> Result<(), String> {
    let _commit = control
        .commit_lock
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if control.cancelled.load(Ordering::Acquire) {
        return Err("已取消登录".into());
    }
    let encrypted = protect(bytes)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| "无法创建账号缓存目录".to_string())?;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&temporary, encrypted).map_err(|_| "无法保存加密账号缓存".to_string())?;
    fs::rename(&temporary, path).map_err(|_| "无法更新加密账号缓存".to_string())
}

pub fn clear_saved_login(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("无法删除已保存的登录状态".into()),
    }
}

#[cfg(windows)]
mod dpapi {
    use std::{ffi::c_void, ptr};
    #[repr(C)]
    struct Blob {
        len: u32,
        data: *mut u8,
    }
    #[link(name = "crypt32")]
    unsafe extern "system" {
        fn CryptProtectData(
            input: *const Blob,
            description: *const u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut Blob,
        ) -> i32;
        fn CryptUnprotectData(
            input: *const Blob,
            description: *mut *mut u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut Blob,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
    }
    const MAGIC: &[u8] = b"LBACCT1\0";
    pub fn run(bytes: &[u8], encrypt: bool) -> Result<Vec<u8>, String> {
        let bytes = if encrypt {
            bytes
        } else {
            bytes
                .strip_prefix(MAGIC)
                .ok_or("账号缓存格式无效，请重新登录")?
        };
        if bytes.len() > 4_000_000 {
            return Err("账号缓存大小无效，请重新登录".into());
        }
        let input = Blob {
            len: bytes.len() as u32,
            data: bytes.as_ptr() as *mut u8,
        };
        let mut output = Blob {
            len: 0,
            data: ptr::null_mut(),
        };
        // No LOCAL_MACHINE flag: only this Windows user can decrypt the cache.
        let success = unsafe {
            if encrypt {
                CryptProtectData(
                    &input,
                    ptr::null(),
                    ptr::null(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    1,
                    &mut output,
                )
            } else {
                CryptUnprotectData(
                    &input,
                    ptr::null_mut(),
                    ptr::null(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    1,
                    &mut output,
                )
            }
        };
        if success == 0 {
            return Err("无法读取本 Windows 用户的加密账号状态，请重新登录".into());
        }
        let mut result = if encrypt { MAGIC.to_vec() } else { Vec::new() };
        unsafe {
            result.extend_from_slice(std::slice::from_raw_parts(output.data, output.len as usize));
            LocalFree(output.data.cast());
        }
        Ok(result)
    }
}

fn protect(bytes: &[u8]) -> Result<Vec<u8>, String> {
    #[cfg(windows)]
    {
        dpapi::run(bytes, true)
    }
    #[cfg(not(windows))]
    {
        let _ = bytes;
        Err("当前平台未提供安全账号存储".into())
    }
}

fn unprotect(bytes: &[u8]) -> Result<Vec<u8>, String> {
    #[cfg(windows)]
    {
        dpapi::run(bytes, false)
    }
    #[cfg(not(windows))]
    {
        let _ = bytes;
        Err("当前平台未提供安全账号存储".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn offline_uuid_matches_java_name_uuid() {
        let account = offline_account("Notch").unwrap();
        assert_eq!(account.uuid, "b50ad385-829d-3141-a216-7e7d7539ba7f");
        assert!(!account.premium);
        assert!(account.session_json.is_none());
    }
    #[test]
    fn invalid_names_are_rejected() {
        for name in [
            "",
            "玩家",
            "hello world",
            "12345678901234567",
            "../../account",
        ] {
            assert!(offline_account(name).is_err());
        }
    }
    #[test]
    fn account_debug_redacts_credentials() {
        let mut account = offline_account("Player").unwrap();
        account.session_json = Some("a-secret-token".into());
        assert!(!format!("{account:?}").contains("a-secret-token"));
    }
    #[test]
    fn premium_session_matches_horizon_stdin_contract() {
        let uuid = uuid::Uuid::parse_str("b50ad385-829d-3141-a216-7e7d7539ba7f").unwrap();
        let session: Value =
            serde_json::from_str(&game_session_json("Notch", uuid, "synthetic-token")).unwrap();
        assert_eq!(session["name"], "Notch");
        assert_eq!(session["uuid"], "b50ad385829d3141a2167e7d7539ba7f");
        assert_eq!(session["access_token"], "synthetic-token");
        assert_eq!(session["user_type"], "msa");
        assert_eq!(session.as_object().unwrap().len(), 4);
    }
    #[test]
    fn cancelled_worker_cannot_overwrite_or_recreate_cache() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("lb-auth-cancel-{}-{unique}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("account.bin");
        fs::write(&path, b"new-account-cache").unwrap();
        let (_, receiver) = mpsc::channel();
        let task = AuthTask {
            receiver,
            control: Arc::new(AuthControl::default()),
        };
        task.cancel();
        assert!(commit_saved_login(&path, b"old-account-secret", &task.control).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"new-account-cache");
        clear_saved_login(&path).unwrap();
        assert!(commit_saved_login(&path, b"old-account-secret", &task.control).is_err());
        assert!(!path.exists());
        fs::remove_dir(&root).unwrap();
    }
    #[cfg(windows)]
    #[test]
    fn protected_cache_roundtrip_is_not_plaintext() {
        let secret = b"{\"refresh_token\":\"synthetic-test-token\"}";
        let encrypted = protect(secret).unwrap();
        assert!(!encrypted.windows(secret.len()).any(|w| w == secret));
        assert_eq!(unprotect(&encrypted).unwrap(), secret);
    }
    #[cfg(windows)]
    #[test]
    fn plaintext_or_corrupt_cache_is_rejected() {
        assert!(unprotect(b"{\"refresh_token\":\"plaintext\"}").is_err());
        assert!(unprotect(b"LBACCT1\0invalid-ciphertext").is_err());
    }
}
