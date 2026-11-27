#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{
    self, Align2, Color32, FontData, FontFamily, FontId, Pos2, Rect, RichText, Sense, Stroke,
    TextureHandle, TextureOptions, Vec2,
};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
mod auth;

const SIZE: Vec2 = Vec2::new(300.0, 450.0);
const BG: Color32 = Color32::from_black_alpha(153);
const SURFACE: Color32 = Color32::from_black_alpha(67);
const HOVER: Color32 = Color32::from_rgb(34, 34, 39);
const TEXT: Color32 = Color32::from_rgb(250, 250, 254);
const MUTED: Color32 = Color32::from_rgb(159, 164, 183);
const ACCENT: Color32 = Color32::from_rgb(70, 119, 255);

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    player_name: String,
    resources: String,
}

#[derive(Default)]
struct Cli {
    package: Option<PathBuf>,
    resources: Option<PathBuf>,
    launch_check: Option<PathBuf>,
    smoke_report: Option<PathBuf>,
}

fn main() -> eframe::Result {
    let cli = parse_cli();
    let root = cli.package.clone().unwrap_or_else(package_root);
    if let Some(report) = cli.launch_check.as_ref() {
        let settings = load_settings(&root, cli.resources.as_deref());
        let checks = launch_plan(&root, &settings);
        let passed = checks["passed"].as_bool().unwrap_or(false);
        let _ = write_json(report, &checks);
        std::process::exit(if passed { 0 } else { 1 });
    }
    let icon = image::load_from_memory(include_bytes!("../assets/mark.png"))
        .ok()
        .map(|i| {
            let rgba = i.to_rgba8();
            egui::IconData {
                width: rgba.width(),
                height: rgba.height(),
                rgba: rgba.into_raw(),
            }
        })
        .unwrap_or_default();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(SIZE)
            .with_min_inner_size(SIZE)
            .with_max_inner_size(SIZE)
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_title("LiquidBounce - Rust")
            .with_icon(icon),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "LiquidBounce - Rust",
        options,
        Box::new(move |cc| Ok(Box::new(Launcher::new(cc, root, cli)))),
    )
}

struct Launcher {
    root: PathBuf,
    settings: Settings,
    mark: TextureHandle,
    avatar: TextureHandle,
    account: auth::Account,
    auth_task: Option<auth::AuthTask>,
    auth_code: Option<(String, String, String)>,
    auth_status: String,
    account_ready: bool,
    pending_launch: bool,
    offline_name: String,
    status: String,
    account_open: bool,
    resources_open: bool,
    child: Option<Child>,
    instance_lock: Option<File>,
    smoke_report: Option<PathBuf>,
    started: Instant,
    frames: u64,
}

impl Launcher {
    fn new(cc: &eframe::CreationContext<'_>, root: PathBuf, cli: Cli) -> Self {
        configure(&cc.egui_ctx);
        let settings = load_settings(&root, cli.resources.as_deref());
        let account =
            auth::offline_account(&sanitize(&settings.player_name)).expect("valid offline account");
        let cache = root.join("user-data/microsoft-account.bin");
        let restoring = cache.is_file();
        let auth_task = restoring.then(|| auth::start_refresh(cache));
        let offline_name = account.name.clone();
        Self {
            settings,
            root,
            account,
            auth_task,
            auth_code: None,
            auth_status: if restoring {
                "正在恢复正版登录…".into()
            } else {
                String::new()
            },
            account_ready: !restoring,
            pending_launch: false,
            offline_name,
            mark: texture(
                &cc.egui_ctx,
                "mark",
                include_bytes!("../assets/mark.png"),
                TextureOptions::LINEAR,
            ),
            avatar: texture(
                &cc.egui_ctx,
                "avatar",
                include_bytes!("../assets/avatar.png"),
                TextureOptions::NEAREST,
            ),
            status: String::new(),
            account_open: false,
            resources_open: false,
            child: None,
            instance_lock: None,
            smoke_report: cli.smoke_report,
            started: Instant::now(),
            frames: 0,
        }
    }
    fn save(&self) {
        let mut settings = self.settings.clone();
        if Path::new(&settings.resources) == self.root.join("resources/26.3") {
            settings.resources.clear();
        }
        let _ = write_json(&self.root.join("user-data/launcher.json"), &settings);
    }
    fn launch(&mut self) {
        if self.child.is_some() {
            return;
        }
        if self.auth_task.is_some() {
            self.status = "正在处理登录，请稍候".into();
            self.account_open = true;
            return;
        }
        if !self.account_ready {
            self.status = "请重新登录或明确选择离线账户".into();
            self.account_open = true;
            return;
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if self.account.premium && self.account.expires_at <= now.saturating_add(60) {
            self.pending_launch = true;
            self.auth_status = "正在刷新正版登录，完成后启动游戏…".into();
            self.status = self.auth_status.clone();
            self.auth_task = Some(auth::start_refresh(
                self.root.join("user-data/microsoft-account.bin"),
            ));
            return;
        }
        let plan = launch_plan(&self.root, &self.settings);
        if plan["passed"] != true {
            self.status = plan["error"]
                .as_str()
                .unwrap_or("游戏文件不完整")
                .to_owned();
            if plan["resourcesValid"] != true {
                self.resources_open = true;
            }
            return;
        }
        let data = self.root.join("user-data");
        for part in [
            "",
            "home",
            "launcher",
            "game",
            "player",
            "data",
            "config",
            "trash",
            "clipboard",
            "desktop",
            "browser-cache",
            "liquidbounce",
        ] {
            if let Err(e) = fs::create_dir_all(data.join(part)) {
                self.status = format!("无法创建游戏目录：{e}");
                return;
            }
        }
        let mut options = fs::OpenOptions::new();
        options.create(true).read(true).write(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0);
        }
        let lock = match options.open(data.join("instance.lock")) {
            Ok(f) => f,
            Err(_) => {
                self.status = "游戏已运行，或用户目录被占用".to_owned();
                return;
            }
        };
        let log = match fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(data.join("launcher-game.log"))
        {
            Ok(f) => f,
            Err(e) => {
                self.status = format!("无法创建日志：{e}");
                return;
            }
        };
        let mut command = Command::new(self.root.join("runtime/liquidbounce-native-game.exe"));
        command
            .args([
                "--resources",
                &self.settings.resources,
                "--name",
                &self.account.name,
                "--width",
                "1280",
                "--height",
                "720",
                "--render-distance",
                "4",
            ])
            .arg("--home")
            .arg(data.join("home"))
            .arg("--game-dir")
            .arg(data.join("game"))
            .arg("--player-dir")
            .arg(data.join("player"))
            .arg("--control-socket")
            .arg(data.join("horizon-control.json"))
            .current_dir(data.join("game"))
            .env("HORIZON_HOME", data.join("home"))
            .env("HORIZON_LAUNCHER_HOME", data.join("launcher"))
            .env("XDG_DATA_HOME", data.join("data"))
            .env("XDG_CONFIG_HOME", data.join("config"))
            .env("HORIZON_LAUNCHER_TRASH", data.join("trash"))
            .env("HORIZON_LAUNCHER_CLIPBOARD", data.join("clipboard"))
            .env("HORIZON_LAUNCHER_DESKTOP", data.join("desktop"))
            .env("HORIZON_LAUNCHER_NO_SELF_UPDATE", "1")
            .env("LIQUIDBOUNCE_THEME_ROOT", self.root.join("theme"))
            .env("LIQUIDBOUNCE_CEF_RESOURCES", self.root.join("runtime"))
            .env("LIQUIDBOUNCE_CONFIG_DIR", data.join("liquidbounce"))
            .env("LIQUIDBOUNCE_CACHE_DIR", data.join("browser-cache"))
            .stdin(if self.account.session_json.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(log.try_clone().unwrap())
            .stderr(log);
        if self.account.session_json.is_some() {
            command.arg("--session-stdin");
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        match command.spawn() {
            Ok(mut child) => {
                if let (Some(session), Some(mut stdin)) =
                    (self.account.session_json.as_ref(), child.stdin.take())
                {
                    if let Err(error) = stdin.write_all(session.as_bytes()) {
                        let _ = child.kill();
                        self.status = format!("无法传递登录会话：{error}");
                        return;
                    }
                }
                self.child = Some(child);
                self.instance_lock = Some(lock);
                self.status = "游戏正在运行".to_owned();
                self.save();
            }
            Err(e) => self.status = format!("启动失败：{e}"),
        }
    }
    fn poll_auth(&mut self, ctx: &egui::Context) {
        let events: Vec<_> = self
            .auth_task
            .as_ref()
            .map(|task| task.receiver.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                auth::AuthEvent::DeviceCode {
                    user_code,
                    verification_uri,
                    direct_uri,
                } => {
                    self.auth_code = Some((user_code, verification_uri, direct_uri));
                    self.account_open = true;
                }
                auth::AuthEvent::Status(text) => self.auth_status = text,
                auth::AuthEvent::Complete(account) => {
                    self.settings.player_name = account.name.clone();
                    self.offline_name = account.name.clone();
                    if let Some(bytes) = account.avatar_png.as_deref() {
                        if let Ok(image) = image::load_from_memory(bytes) {
                            let image = image.to_rgba8();
                            self.avatar = ctx.load_texture(
                                "account-avatar",
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [image.width() as usize, image.height() as usize],
                                    image.as_raw(),
                                ),
                                TextureOptions::NEAREST,
                            );
                        }
                    }
                    self.account = account;
                    self.account_ready = true;
                    self.auth_status.clear();
                    self.auth_code = None;
                    self.auth_task = None;
                    self.account_open = false;
                    self.status = "登录成功".into();
                    self.save();
                    if std::mem::take(&mut self.pending_launch) {
                        self.launch();
                    }
                }
                auth::AuthEvent::Error(error) => {
                    self.status = error.clone();
                    self.auth_status = error;
                    self.auth_task = None;
                    self.auth_code = None;
                    self.pending_launch = false;
                    self.account_open = true;
                }
                auth::AuthEvent::Cancelled => {
                    self.auth_task = None;
                    self.auth_code = None;
                    self.auth_status = "已取消登录".into();
                    self.pending_launch = false;
                }
            }
        }
        if self.auth_task.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    fn account_dialog(&mut self, ctx: &egui::Context) {
        if !self.account_open {
            return;
        }
        egui::Window::new("切换账户")
            .collapsible(false)
            .resizable(false)
            .fixed_size([252.0, 292.0])
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(RichText::new("选择登录方式").size(16.0));
                ui.add_space(12.0);
                if ui
                    .add_enabled(
                        self.auth_task.is_none(),
                        egui::Button::new("微软正版登录")
                            .min_size([228.0, 34.0].into())
                            .fill(ACCENT),
                    )
                    .clicked()
                {
                    self.auth_task = Some(auth::start_login(
                        self.root.join("user-data/microsoft-account.bin"),
                    ));
                    self.auth_status = "正在连接微软…".into();
                }
                if let Some((code, uri, direct)) = self.auth_code.as_ref() {
                    ui.add_space(8.0);
                    ui.label("在微软页面输入代码：");
                    if ui
                        .button(RichText::new(code).size(18.0).strong())
                        .on_hover_text("复制登录代码")
                        .clicked()
                    {
                        ctx.copy_text(code.clone());
                    }
                    if ui.link(uri).clicked() {
                        ctx.open_url(egui::OpenUrl::new_tab(direct));
                    }
                }
                if !self.auth_status.is_empty() {
                    ui.label(RichText::new(&self.auth_status).size(11.0).color(MUTED));
                }
                ui.add_space(14.0);
                ui.label(RichText::new("离线登录").color(MUTED));
                ui.add(
                    egui::TextEdit::singleline(&mut self.offline_name)
                        .desired_width(228.0)
                        .hint_text("1–16 位字母、数字或下划线"),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("使用离线账户").clicked() {
                        match auth::offline_account(&self.offline_name) {
                            Ok(account) => {
                                if let Some(task) = self.auth_task.take() {
                                    task.cancel();
                                }
                                let _ = auth::clear_saved_login(
                                    &self.root.join("user-data/microsoft-account.bin"),
                                );
                                self.settings.player_name = account.name.clone();
                                self.account = account;
                                self.account_ready = true;
                                self.pending_launch = false;
                                self.account_open = false;
                                self.status.clear();
                                self.auth_status.clear();
                                self.auth_code = None;
                                self.avatar = texture(
                                    ctx,
                                    "avatar",
                                    include_bytes!("../assets/avatar.png"),
                                    TextureOptions::NEAREST,
                                );
                                self.save();
                            }
                            Err(error) => self.auth_status = error,
                        }
                    }
                    if ui.button("取消").clicked() {
                        if let Some(task) = self.auth_task.take() {
                            task.cancel();
                        }
                        self.auth_code = None;
                        self.pending_launch = false;
                        self.auth_status.clear();
                        self.account_open = false;
                    }
                });
            });
    }
    fn resources_dialog(&mut self, ctx: &egui::Context) {
        if !self.resources_open {
            return;
        }
        egui::Window::new("游戏资源")
            .collapsible(false)
            .resizable(false)
            .fixed_size([252.0, 170.0])
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Minecraft 26.3 resources");
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.settings.resources).desired_width(230.0),
                );
                ui.add_space(8.0);
                if ui.button("选择文件夹…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("选择已准备的 26.3 resources 目录")
                        .pick_folder()
                    {
                        self.settings.resources = path.to_string_lossy().into_owned();
                    }
                }
                ui.add_space(10.0);
                if ui.button("完成").clicked() {
                    self.resources_open = false;
                    self.save();
                }
            });
    }
}

impl eframe::App for Launcher {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root_ui.ctx().clone();
        self.frames += 1;
        self.poll_auth(&ctx);
        if let Some(child) = self.child.as_mut() {
            if let Ok(Some(exit)) = child.try_wait() {
                self.status = if exit.success() {
                    "游戏已关闭".to_owned()
                } else {
                    "游戏已退出，请查看 user-data/launcher-game.log".to_owned()
                };
                self.child = None;
                self.instance_lock = None;
            }
            ctx.request_repaint_after(Duration::from_millis(250));
        }
        if self.child.is_some() && ctx.input(|input| input.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(root_ui, |ui| {
                let r = ui.max_rect();
                let p = r.min;
                let full = Rect::from_min_size(p, SIZE);
                ui.painter().rect_filled(full, 0, BG);
                // Match LiquidLauncher's black 60% surface over a transparent viewport.
                ui.painter().rect_filled(
                    Rect::from_min_size(p, [300.0, 36.0].into()),
                    0,
                    Color32::from_black_alpha(35),
                );
                let title = Rect::from_min_size(p, [236.0, 36.0].into());
                if ui
                    .interact(title, ui.scope_id().with("drag"), Sense::drag())
                    .drag_started()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                ui.painter().image(
                    self.mark.id(),
                    Rect::from_min_size(p + egui::vec2(8.0, 8.0), [20.0, 20.0].into()),
                    Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    TEXT,
                );
                ui.painter().text(
                    p + egui::vec2(32.0, 18.0),
                    Align2::LEFT_CENTER,
                    "LiquidBounce - Rust",
                    FontId::proportional(13.0),
                    TEXT,
                );
                let min_rect = Rect::from_min_size(p + egui::vec2(238.0, 4.0), [26.0, 28.0].into());
                if ui
                    .put(
                        min_rect,
                        egui::Button::new(RichText::new("−").size(20.0).color(MUTED)).frame(false),
                    )
                    .on_hover_text("最小化")
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
                let close_rect =
                    Rect::from_min_size(p + egui::vec2(268.0, 4.0), [26.0, 28.0].into());
                if ui
                    .put(
                        close_rect,
                        egui::Button::new(RichText::new("×").size(20.0).color(MUTED)).frame(false),
                    )
                    .on_hover_text(if self.child.is_some() {
                        "游戏运行时最小化"
                    } else {
                        "关闭启动器"
                    })
                    .clicked()
                {
                    ctx.send_viewport_cmd(if self.child.is_some() {
                        egui::ViewportCommand::Minimized(true)
                    } else {
                        egui::ViewportCommand::Close
                    });
                }
                ui.painter().text(
                    p + egui::vec2(150.0, 70.0),
                    Align2::CENTER_CENTER,
                    if !self.account_ready && self.auth_task.is_some() {
                        "正在恢复正版登录"
                    } else if self.account.premium {
                        "当前为正版登录"
                    } else {
                        "当前为离线登录"
                    },
                    FontId::proportional(16.0),
                    TEXT,
                );
                let avatar_rect =
                    Rect::from_min_size(p + egui::vec2(110.0, 110.0), [80.0, 80.0].into());
                ui.painter().image(
                    self.avatar.id(),
                    avatar_rect,
                    Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    TEXT,
                );
                let name_rect =
                    Rect::from_min_size(p + egui::vec2(30.0, 211.0), [240.0, 36.0].into());
                let name = ui.put(
                    name_rect,
                    egui::Button::new(RichText::new(&self.account.name).size(24.0).color(TEXT))
                        .frame(false),
                );
                if name.clicked() {
                    self.offline_name = self.account.name.clone();
                    self.account_open = true;
                }
                name.on_hover_text("点击切换正版或离线账户");
                ui.painter().text(
                    p + egui::vec2(150.0, 363.0),
                    Align2::CENTER_CENTER,
                    "Minecraft Rust Edition 26.3",
                    FontId::proportional(14.0),
                    TEXT,
                );
                let launch = ui.put(
                    Rect::from_min_size(p + egui::vec2(40.0, 383.0), [220.0, 38.0].into()),
                    egui::Button::new(
                        RichText::new(if self.child.is_some() {
                            "游戏运行中"
                        } else {
                            "启动游戏"
                        })
                        .size(18.0)
                        .strong(),
                    )
                    .fill(ACCENT)
                    .corner_radius(6),
                );
                if launch.clicked() {
                    self.launch();
                }
                if !self.status.is_empty() {
                    ui.painter().text(
                        p + egui::vec2(150.0, 341.0),
                        Align2::CENTER_CENTER,
                        &self.status,
                        FontId::proportional(10.0),
                        MUTED,
                    );
                }
                if ui
                    .put(
                        Rect::from_min_size(p + egui::vec2(112.0, 425.0), [76.0, 20.0].into()),
                        egui::Button::new(RichText::new("资源设置").size(10.0).color(MUTED))
                            .frame(false),
                    )
                    .clicked()
                {
                    self.resources_open = true;
                }
            });
        self.account_dialog(&ctx);
        self.resources_dialog(&ctx);
        if self.smoke_report.is_some() {
            ctx.request_repaint_after(Duration::from_millis(50));
            if self.frames >= 4 && self.started.elapsed() > Duration::from_secs(2) {
                let report = self.smoke_report.take().unwrap();
                let _ = write_json(
                    &report,
                    &serde_json::json!({"passed":true,"windowSize":[300,450],"frames":self.frames,"embeddedAssets":true,"gameLaunched":false,"packageRoot":self.root}),
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

fn configure(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Inter".into(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../assets/Inter-Regular.ttf"
        ))),
    );
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "Inter".into());
    for path in ["C:/Windows/Fonts/msyh.ttc", "C:/Windows/Fonts/simhei.ttf"] {
        if let Ok(bytes) = fs::read(path) {
            fonts.font_data.insert(
                "Chinese".into(),
                std::sync::Arc::new(FontData::from_owned(bytes)),
            );
            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .push("Chinese".into());
            break;
        }
    }
    ctx.set_fonts(fonts);
    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.window_fill = Color32::from_rgb(23, 23, 27);
    style.visuals.panel_fill = BG;
    style.visuals.widgets.inactive.bg_fill = SURFACE;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.widgets.inactive.corner_radius = 6.into();
    style.visuals.widgets.hovered.bg_fill = HOVER;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    style.visuals.widgets.active.bg_fill = ACCENT;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.selection.bg_fill = ACCENT;
    style.visuals.window_corner_radius = 12.into();
    style.spacing.button_padding = egui::vec2(10.0, 7.0);
    ctx.set_style_of(egui::Theme::Dark, style);
}
fn texture(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
    options: TextureOptions,
) -> TextureHandle {
    let image = image::load_from_memory(bytes)
        .expect("embedded image")
        .to_rgba8();
    ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.as_raw(),
        ),
        options,
    )
}
fn load_settings(root: &Path, resources: Option<&Path>) -> Settings {
    let mut s: Settings = fs::read(root.join("user-data/launcher.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    if s.player_name.is_empty() {
        s.player_name = "PortTest".into();
    }
    if let Some(p) = resources {
        s.resources = p.to_string_lossy().into_owned();
    } else if s.resources.is_empty() {
        s.resources = root.join("resources/26.3").to_string_lossy().into_owned();
    }
    s
}
fn sanitize(value: &str) -> String {
    let s: String = value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(16)
        .collect();
    if s.is_empty() {
        "PortTest".into()
    } else {
        s
    }
}
fn package_root() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}
fn parse_cli() -> Cli {
    let mut cli = Cli::default();
    let mut a = env::args().skip(1);
    while let Some(arg) = a.next() {
        match arg.as_str() {
            "--package" => cli.package = a.next().map(PathBuf::from),
            "--resources" => cli.resources = a.next().map(PathBuf::from),
            "--launch-check" => cli.launch_check = a.next().map(PathBuf::from),
            "--smoke-report" => cli.smoke_report = a.next().map(PathBuf::from),
            "--help" => {
                println!("LiquidBounce Rust Launcher\n  --package PATH\n  --resources PATH\n  --launch-check REPORT.json\n  --smoke-report REPORT.json");
                std::process::exit(0)
            }
            _ => {}
        }
    }
    cli
}
fn write_json(path: &Path, value: &impl Serialize) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        if !p.as_os_str().is_empty() {
            fs::create_dir_all(p)?;
        }
    }
    let mut f = File::create(path)?;
    f.write_all(serde_json::to_string_pretty(value)?.as_bytes())
}
fn launch_plan(root: &Path, settings: &Settings) -> serde_json::Value {
    let required = [
        "runtime/liquidbounce-native-game.exe",
        "runtime/liquidbounce-cef-helper.exe",
        "runtime/libcef.dll",
        "runtime/icudtl.dat",
        "runtime/resources.pak",
        "runtime/locales/en-US.pak",
        "theme/index.html",
    ];
    let missing: Vec<_> = required
        .iter()
        .filter(|p| !root.join(p).is_file())
        .collect();
    let resources = Path::new(&settings.resources);
    let resources_exists = resources.is_dir();
    let metadata: Option<serde_json::Value> = fs::read(resources.join("horizon-resources.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let minecraft = metadata
        .as_ref()
        .and_then(|value| value["minecraft"].as_str());
    let portable: Option<serde_json::Value> = fs::read(resources.join("portable-assets.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let portable_verified = portable
        .as_ref()
        .is_some_and(|value| value["allReferencesVerified"] == true && value["missingCount"] == 0);
    let asset_index: Option<serde_json::Value> = fs::read(resources.join("asset-index.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let objects = asset_index.as_ref().and_then(|value| value["objects"].as_object());
    let mut missing_objects = 0;
    if let Some(objects) = objects {
        for object in objects.values() {
            let valid = object["hash"].as_str().filter(|hash| {
                hash.len() == 40 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            }).is_some_and(|hash| {
                fs::metadata(resources.join("assets/objects").join(&hash[..2]).join(hash))
                    .is_ok_and(|info| info.is_file() && Some(info.len()) == object["size"].as_u64())
            });
            if !valid { missing_objects += 1; }
        }
    }
    let objects_valid = objects.is_some_and(|objects| !objects.is_empty()) && missing_objects == 0;
    let required_assets = [
        "asset-index.json",
        "assets/indexes/34.json",
        "assets/minecraft/sounds.json",
        "assets/minecraft/textures/gui/title/background/panorama_0.png",
        "assets/minecraft/font/unifont.zip",
    ];
    let missing_assets: Vec<_> = required_assets
        .iter()
        .filter(|path| !resources.join(path).is_file())
        .collect();
    let resources_valid = resources_exists && minecraft == Some("26.3")
        && portable_verified && objects_valid && missing_assets.is_empty();
    let passed = missing.is_empty() && resources_valid;
    serde_json::json!({
        "passed": passed,
        "packageRoot": root,
        "missing": missing,
        "resources": settings.resources,
        "resourcesExists": resources_exists,
        "resourcesMetadataMinecraft": minecraft,
        "portableAssetsVerified": portable_verified,
        "assetObjectReferences": objects.map(|objects| objects.len()).unwrap_or(0),
        "missingAssetObjects": missing_objects,
        "missingAssets": missing_assets,
        "resourcesValid": resources_valid,
        "playerName": sanitize(&settings.player_name),
        "gameLaunched": false,
        "error": if !resources_exists {
            "请选择 Minecraft 26.3 resources 目录"
        } else if !resources_valid {
            "Minecraft 26.3 资源不完整，请补齐资源索引、objects 和音效等文件"
        } else if !missing.is_empty() {
            "发布包缺少运行文件"
        } else { "" }
    })
}
