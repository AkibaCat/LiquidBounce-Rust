// SPDX-License-Identifier: GPL-3.0-or-later
//! ClientFunctions.kt desktop routes run on the interop worker, outside the
//! game actor queue. A native modal dialog may remain open for any duration.
use crate::{ApiRequest, ApiResponse, Backend, Method, StatusCode, localization};
use serde::Deserialize;
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DialogMode {
    OpenFile,
    SaveFile,
    OpenDirectory,
}
impl DialogMode {
    fn title(self, language: &str) -> String {
        let (key, fallback) = match self {
            Self::OpenFile => ("openFile", "Open File"),
            Self::SaveFile => ("saveFile", "Save File As"),
            Self::OpenDirectory => ("openDirectory", "Select Folder"),
        };
        localization::translate(language, &format!("liquidbounce.fileDialog.mode.{key}"))
            .unwrap_or(fallback)
            .to_owned()
    }
}

pub trait Desktop: Send + Sync + 'static {
    fn select(
        &self,
        mode: DialogMode,
        title: &str,
        root: &Path,
        extensions: &[String],
    ) -> Result<Option<PathBuf>, String>;
    fn browse(&self, directory: &Path) -> Result<(), String>;
}

pub struct SystemDesktop;
impl Desktop for SystemDesktop {
    fn select(
        &self,
        mode: DialogMode,
        title: &str,
        root: &Path,
        extensions: &[String],
    ) -> Result<Option<PathBuf>, String> {
        #[cfg(windows)]
        {
            let mut dialog = rfd::FileDialog::new().set_title(title).set_directory(root);
            if mode != DialogMode::OpenDirectory && !extensions.is_empty() {
                dialog = dialog.add_filter(
                    extensions
                        .iter()
                        .map(|ext| format!("*.{ext}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    extensions,
                );
            }
            Ok(match mode {
                DialogMode::OpenFile => dialog.pick_file(),
                DialogMode::SaveFile => dialog.save_file(),
                DialogMode::OpenDirectory => dialog.pick_folder(),
            })
        }
        #[cfg(not(windows))]
        {
            let _ = (mode, title, root, extensions);
            Err("Native file dialogs are unavailable on this build target".into())
        }
    }
    fn browse(&self, directory: &Path) -> Result<(), String> {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            std::process::Command::new("explorer.exe")
                .arg(directory)
                .creation_flags(0x0800_0000)
                .spawn()
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        #[cfg(not(windows))]
        {
            let _ = directory;
            Err("Native directory browsing is unavailable on this build target".into())
        }
    }
}

pub struct DesktopBackend {
    inner: Arc<dyn Backend>,
    desktop: Arc<dyn Desktop>,
    root: PathBuf,
    language: Arc<RwLock<String>>,
}
impl DesktopBackend {
    pub fn new(
        inner: Arc<dyn Backend>,
        desktop: Arc<dyn Desktop>,
        root: PathBuf,
        language: Arc<RwLock<String>>,
    ) -> Self {
        Self {
            inner,
            desktop,
            root,
            language,
        }
    }
    fn dialog(&self, request: &ApiRequest) -> ApiResponse {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Body {
            mode: DialogMode,
            supported_extensions: Option<Vec<String>>,
        }
        let Ok(body) = serde_json::from_slice::<Body>(&request.body) else {
            return ApiResponse::error(StatusCode::BAD_REQUEST, "No dialog mode provided");
        };
        let extensions = if body.mode == DialogMode::OpenDirectory {
            Vec::new()
        } else {
            body.supported_extensions
                .unwrap_or_default()
                .iter()
                .map(|extension| {
                    let extension = extension.trim();
                    extension.strip_prefix("*.").unwrap_or(extension).to_owned()
                })
                .filter(|extension| !extension.is_empty())
                .collect()
        };
        let title = body.mode.title(
            &self
                .language
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        match self
            .desktop
            .select(body.mode, &title, &self.root, &extensions)
        {
            Ok(Some(file)) => ApiResponse::json(json!({"file":file.to_string_lossy()})),
            Ok(None) => ApiResponse::json(json!({})),
            Err(error) => ApiResponse::error(StatusCode::INTERNAL_SERVER_ERROR, error),
        }
    }
    fn browse(&self, request: &ApiRequest) -> ApiResponse {
        let value = serde_json::from_slice::<serde_json::Value>(&request.body).ok();
        let Some(path) = value.as_ref().and_then(|value| value["path"].as_str()) else {
            return ApiResponse::error(StatusCode::BAD_REQUEST, "No file specified");
        };
        let file = Path::new(path);
        let file = if file.is_absolute() {
            file.to_path_buf()
        } else {
            self.root.join(file)
        };
        if !file.exists() {
            return ApiResponse::error(StatusCode::NOT_FOUND, "File not exists");
        }
        let directory = if file.is_dir() {
            file.as_path()
        } else if file.is_file() {
            let Some(parent) = file.parent() else {
                return ApiResponse::error(StatusCode::FORBIDDEN, "Cannot access root directory");
            };
            parent
        } else {
            return ApiResponse::error(StatusCode::FORBIDDEN, "Invalid file type");
        };
        match self.desktop.browse(directory) {
            Ok(()) => ApiResponse::no_content(),
            Err(error) => ApiResponse::error(StatusCode::INTERNAL_SERVER_ERROR, error),
        }
    }
}
impl Backend for DesktopBackend {
    fn request(&self, request: ApiRequest) -> ApiResponse {
        if request.method == Method::POST {
            match request.path.as_str() {
                "/api/v1/client/fileDialog" => return self.dialog(&request),
                "/api/v1/client/browsePath" => return self.browse(&request),
                _ => {}
            }
        }
        self.inner.request(request)
    }
}

/// FileValue normalizes only paths beneath the actual client config root.
pub fn normalize_to_client_folder(root: &Path, file: &str) -> String {
    Path::new(file)
        .strip_prefix(root)
        .unwrap_or(Path::new(file))
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, sync::Mutex};
    type DialogCall = (DialogMode, String, PathBuf, Vec<String>);
    #[derive(Default)]
    struct FakeDesktop {
        dialogs: Mutex<Vec<DialogCall>>,
        browsed: Mutex<Vec<PathBuf>>,
        selected: Mutex<Option<PathBuf>>,
    }
    impl Desktop for FakeDesktop {
        fn select(
            &self,
            mode: DialogMode,
            title: &str,
            root: &Path,
            extensions: &[String],
        ) -> Result<Option<PathBuf>, String> {
            self.dialogs
                .lock()
                .unwrap()
                .push((mode, title.into(), root.into(), extensions.into()));
            Ok(self.selected.lock().unwrap().clone())
        }
        fn browse(&self, directory: &Path) -> Result<(), String> {
            self.browsed.lock().unwrap().push(directory.into());
            Ok(())
        }
    }
    struct Fallback;
    impl Backend for Fallback {
        fn request(&self, request: ApiRequest) -> ApiResponse {
            ApiResponse::json(json!({"forwarded":request.path}))
        }
    }
    fn request(path: &str, value: serde_json::Value) -> ApiRequest {
        ApiRequest {
            method: Method::POST,
            path: format!("/api/v1/client/{path}"),
            query: HashMap::new(),
            body: serde_json::to_vec(&value).unwrap().into(),
        }
    }
    #[test]
    fn dialogs_preserve_modes_filters_cancel_unicode_and_live_language() {
        let root = tempfile::tempdir().unwrap();
        let desktop = Arc::new(FakeDesktop::default());
        let language = Arc::new(RwLock::new("zh_cn".into()));
        let backend = DesktopBackend::new(
            Arc::new(Fallback),
            desktop.clone(),
            root.path().into(),
            language.clone(),
        );
        assert_eq!(
            backend
                .request(request(
                    "fileDialog",
                    json!({"mode":"OPEN_FILE","supportedExtensions":[" *.png ","","jpg"]})
                ))
                .body,
            Some(json!({}))
        );
        let selected = root.path().join("中文图片.png");
        *desktop.selected.lock().unwrap() = Some(selected.clone());
        *language.write().unwrap() = "zh_tw".into();
        assert_eq!(
            backend
                .request(request("fileDialog", json!({"mode":"SAVE_FILE"})))
                .body,
            Some(json!({"file":selected}))
        );
        backend.request(request(
            "fileDialog",
            json!({"mode":"OPEN_DIRECTORY","supportedExtensions":["png"]}),
        ));
        let calls = desktop.dialogs.lock().unwrap();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0].0, DialogMode::OpenFile);
        assert_eq!(calls[0].2, root.path());
        assert_eq!(calls[0].3, ["png", "jpg"]);
        assert_eq!(calls[0].1, "打开文件");
        assert_eq!(calls[1].0, DialogMode::SaveFile);
        assert_eq!(calls[1].1, "另存新檔");
        assert_eq!(calls[2].0, DialogMode::OpenDirectory);
        assert_eq!(calls[2].1, "選擇資料夾");
        assert!(calls[2].3.is_empty());
    }
    #[test]
    fn desktop_routes_validate_inputs_and_browse_real_parent_directories() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("图像.png"), []).unwrap();
        let desktop = Arc::new(FakeDesktop::default());
        let backend = DesktopBackend::new(
            Arc::new(Fallback),
            desktop.clone(),
            root.path().into(),
            Arc::new(RwLock::new("en_us".into())),
        );
        for value in [
            json!({}),
            json!({"mode":"UNKNOWN"}),
            json!({"mode":"OPEN_FILE","supportedExtensions":[2]}),
        ] {
            assert_eq!(
                backend.request(request("fileDialog", value)).status,
                StatusCode::BAD_REQUEST
            );
        }
        assert_eq!(
            backend.request(request("browsePath", json!({}))).status,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            backend
                .request(request("browsePath", json!({"path":"missing"})))
                .status,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            backend
                .request(request("browsePath", json!({"path":"图像.png"})))
                .status,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            backend
                .request(request("browsePath", json!({"path":root.path()})))
                .status,
            StatusCode::NO_CONTENT
        );
        assert_eq!(*desktop.browsed.lock().unwrap(), [root.path(), root.path()]);
        assert_eq!(
            backend.request(request("modules", json!({}))).body,
            Some(json!({"forwarded":"/api/v1/client/modules"}))
        );
        assert_eq!(
            normalize_to_client_folder(
                root.path(),
                &root.path().join("图像.png").to_string_lossy()
            ),
            "图像.png"
        );
        assert_eq!(
            normalize_to_client_folder(root.path(), "relative.png"),
            "relative.png"
        );
    }

    #[test]
    fn modal_dialog_leaves_the_actual_actor_request_queue_available() {
        use std::{sync::mpsc, thread, time::Duration};
        struct BlockingDialog {
            entered: mpsc::Sender<()>,
            release: Mutex<mpsc::Receiver<()>>,
        }
        impl Desktop for BlockingDialog {
            fn select(
                &self,
                _: DialogMode,
                _: &str,
                _: &Path,
                _: &[String],
            ) -> Result<Option<PathBuf>, String> {
                self.entered.send(()).unwrap();
                self.release.lock().unwrap().recv().unwrap();
                Ok(None)
            }
            fn browse(&self, _: &Path) -> Result<(), String> {
                Ok(())
            }
        }
        let (queued, inbox) = crate::main_thread::QueuedBackend::channel(2, Duration::from_secs(2));
        let (entered, started) = mpsc::channel();
        let (release, wait) = mpsc::channel();
        let backend = Arc::new(DesktopBackend::new(
            queued,
            Arc::new(BlockingDialog {
                entered,
                release: Mutex::new(wait),
            }),
            PathBuf::new(),
            Arc::new(RwLock::new("en_us".into())),
        ));
        let modal_backend = backend.clone();
        let modal = thread::spawn(move || {
            modal_backend.request(request("fileDialog", json!({"mode":"OPEN_FILE"})))
        });
        started.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(inbox.try_next().is_none());
        let ordinary = thread::spawn(move || backend.request(request("modules", json!({}))));
        let pending = loop {
            if let Some(pending) = inbox.try_next() {
                break pending;
            }
            thread::yield_now();
        };
        assert_eq!(pending.request.path, "/api/v1/client/modules");
        pending.reply(ApiResponse::json(json!({"actor":"responsive"})));
        assert_eq!(
            ordinary.join().unwrap().body,
            Some(json!({"actor":"responsive"}))
        );
        release.send(()).unwrap();
        assert_eq!(modal.join().unwrap().body, Some(json!({})));
    }
}
