//! Real CEF C API integration. Initialize, pump, close and shut down on the same game thread.
use crate::{
    input::{Button, Input, InputQueue, PageTarget},
    paint::{Mailbox, Pixels, Rect as PopupRect, paint_length},
};
use cef::{args::Args, *};
use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::ThreadId,
    time::{Duration, Instant},
};

pub const CEF_VERSION: &str = "152.0.6+g708dc14+chromium-152.0.7977.83";
static INITIALIZED_ONCE: AtomicBool = AtomicBool::new(false);

fn cef_path(path: &std::path::Path) -> Result<String, String> {
    let absolute = path.canonicalize().map_err(|e| e.to_string())?;
    let value = absolute.to_string_lossy();
    // Chromium expects ordinary absolute Windows paths, not Rust's extended-length prefix.
    Ok(if let Some(unc) = value.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        value.strip_prefix(r"\\?\").unwrap_or(&value).to_string()
    })
}

#[derive(Clone)]
pub struct Config {
    /// The original built theme served by liquidbounce-interop; lb_code goes in this URL.
    pub url: String,
    /// Fixed static route for the cached standalone ClickGUI. Tests may reuse the shared URL.
    pub click_gui_url: Option<String>,
    pub helper_executable: PathBuf,
    pub cache_directory: PathBuf,
    /// Directory containing CEF .pak files and the locales/ folder.
    pub resources_directory: PathBuf,
}

/// Actual Chromium callbacks, available to the game and smoke runner without DOM polling.
/// A completed document load does not establish that the game's REST/WebSocket actor works.
#[derive(Clone, Debug, Default)]
pub struct PageState {
    pub loading: bool,
    pub completed_loads: u64,
    pub http_status: Option<i32>,
    pub title: String,
    pub load_errors: Vec<String>,
    pub console: Vec<String>,
}

fn append_bounded(messages: &mut Vec<String>, message: String) {
    if messages.len() == 128 {
        messages.remove(0);
    }
    messages.push(message.chars().take(4096).collect());
}

wrap_load_handler! {
    struct PageLoads { state: Arc<Mutex<PageState>>, }
    impl LoadHandler {
        fn on_loading_state_change(&self, _browser: Option<&mut Browser>, loading: i32,
            _back: i32, _forward: i32) {
            self.state.lock().unwrap_or_else(|e| e.into_inner()).loading = loading != 0;
        }
        fn on_load_end(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, status: i32) {
            if frame.is_some_and(|f| f.is_main() != 0) {
                let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                state.http_status = Some(status);
                state.completed_loads += 1;
            }
        }
        fn on_load_error(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>,
            code: Errorcode, text: Option<&CefString>, _url: Option<&CefString>) {
            if frame.is_some_and(|f| f.is_main() != 0) {
                let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                append_bounded(&mut state.load_errors, format!("{code:?}: {}", text.map(ToString::to_string).unwrap_or_default()));
            }
        }
    }
}

wrap_display_handler! {
    struct PageDisplay { state: Arc<Mutex<PageState>>, }
    impl DisplayHandler {
        fn on_title_change(&self, _browser: Option<&mut Browser>, title: Option<&CefString>) {
            self.state.lock().unwrap_or_else(|e| e.into_inner()).title =
                title.map(ToString::to_string).unwrap_or_default();
        }
        fn on_console_message(&self, _browser: Option<&mut Browser>, level: LogSeverity,
            message: Option<&CefString>, _source: Option<&CefString>, line: i32) -> i32 {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            append_bounded(&mut state.console, format!("{level:?} line {line}: {}", message.map(ToString::to_string).unwrap_or_default()));
            0
        }
    }
}

wrap_browser_process_handler! {
    struct ProcessHandler { ready: Arc<AtomicBool>, }
    impl BrowserProcessHandler {
        fn on_context_initialized(&self) { self.ready.store(true, Ordering::Release); }
    }
}

wrap_app! {
    struct Application { ready: Arc<AtomicBool>, }
    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(ProcessHandler::new(self.ready.clone()))
        }
    }
}

wrap_render_handler! {
    struct Renderer { mailbox: Arc<Mailbox>, surface: Arc<Mailbox>, active: Arc<AtomicBool>, }
    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                let size = self.mailbox.desired_size();
                rect.x = 0; rect.y = 0; rect.width = size[0] as i32; rect.height = size[1] as i32;
            }
        }
        fn screen_info(&self, _browser: Option<&mut Browser>, info: Option<&mut ScreenInfo>) -> i32 {
            let Some(info) = info else { return 0; };
            let size = self.mailbox.desired_size();
            // As the original MCEF browser: CSS dimensions equal the physical browser surface.
            // Minecraft GUI scaling is applied to input before dispatch, never to these pixels.
            info.device_scale_factor = 1.;
            info.depth = 32; info.depth_per_component = 8;
            info.rect = Rect { x: 0, y: 0, width: size[0] as i32, height: size[1] as i32 };
            info.available_rect = info.rect.clone();
            1
        }
        fn on_popup_show(&self, _browser: Option<&mut Browser>, show: i32) {
            self.surface.popup_visible(show != 0);
            if self.active.load(Ordering::Acquire) { self.mailbox.popup_visible(show != 0); }
        }
        fn on_popup_size(&self, _browser: Option<&mut Browser>, rect: Option<&Rect>) {
            if let Some(rect) = rect {
                if rect.width > 0 && rect.height > 0 {
                    let rect = PopupRect { x: rect.x, y: rect.y,
                        width: rect.width as u32, height: rect.height as u32 };
                    self.surface.popup_rect(rect);
                    if self.active.load(Ordering::Acquire) { self.mailbox.popup_rect(rect); }
                }
            }
        }
        fn on_paint(&self, _browser: Option<&mut Browser>, type_: PaintElementType,
            _dirty: Option<&[Rect]>, buffer: *const u8, width: i32, height: i32) {
            if buffer.is_null() || width <= 0 || height <= 0 { return; }
            let Some(length) = paint_length(width as u32, height as u32) else { return; };
            // SAFETY: CEF owns a width*height*4 BGRA buffer for this callback's duration.
            // Check dimensions/overflow first, then copy before returning; no pointer escapes.
            let source = unsafe { std::slice::from_raw_parts(buffer, length) };
            if let Ok(pixels) = Pixels::copy(width as u32, height as u32, source) {
                let popup = type_ == PaintElementType::POPUP;
                self.surface.publish(popup, pixels.clone());
                // Background and asynchronously closing pages retain their own pixels only.
                if self.active.load(Ordering::Acquire) { self.mailbox.publish(popup, pixels); }
            }
        }
    }
}

wrap_life_span_handler! {
    struct LifeSpan { closed: Arc<AtomicBool>, }
    impl LifeSpanHandler {
        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            self.closed.store(true, Ordering::Release);
        }
    }
}

wrap_client! {
    struct ClientHandlers { renderer: RenderHandler, life: LifeSpanHandler, load: LoadHandler, display: DisplayHandler, }
    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> { Some(self.renderer.clone()) }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> { Some(self.life.clone()) }
        fn load_handler(&self) -> Option<LoadHandler> { Some(self.load.clone()) }
        fn display_handler(&self) -> Option<DisplayHandler> { Some(self.display.clone()) }
    }
}

/// Native browser identities and lifetime counts, independent of document load state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageStatus {
    pub active: PageTarget,
    pub shared_open: bool,
    pub click_gui_open: bool,
    pub shared_browser_id: Option<i32>,
    pub click_gui_browser_id: Option<i32>,
    pub shared_fps: Option<u32>,
    pub click_gui_fps: Option<u32>,
    pub retired_pages: usize,
    pub created_pages: u64,
    pub closed_pages: u64,
    pub fps: u32,
    pub hud_open: bool,
    pub hud_browser_id: Option<i32>,
    pub hud_fps: Option<u32>,
    pub hud_visible: bool,
    pub hud_generation: u64,
}

struct Page {
    client: Client,
    browser: Option<Browser>,
    closed: Arc<AtomicBool>,
    active: Arc<AtomicBool>,
    state: Arc<Mutex<PageState>>,
    surface: Arc<Mailbox>,
    url: String,
}

impl Page {
    fn new(url: String, mailbox: Arc<Mailbox>, selected: bool) -> Self {
        let closed = Arc::new(AtomicBool::new(false));
        let active = Arc::new(AtomicBool::new(selected));
        let state = Arc::new(Mutex::new(PageState::default()));
        let surface = Arc::new(Mailbox::new(mailbox.desired_size()));
        let client = ClientHandlers::new(
            Renderer::new(mailbox, surface.clone(), active.clone()),
            LifeSpan::new(closed.clone()),
            PageLoads::new(state.clone()),
            PageDisplay::new(state.clone()),
        );
        Self {
            client,
            browser: None,
            closed,
            active,
            state,
            surface,
            url,
        }
    }

    fn open(&mut self, fps: u32) -> Result<bool, String> {
        if self.browser.is_some() {
            return Ok(false);
        }
        let window = WindowInfo::default().set_as_windowless(Default::default());
        let settings = BrowserSettings {
            windowless_frame_rate: fps as i32,
            background_color: 0,
            ..Default::default()
        };
        self.browser = browser_host_create_browser_sync(
            Some(&window),
            Some(&mut self.client),
            Some(&self.url.as_str().into()),
            Some(&settings),
            None,
            None,
        );
        if self.browser.is_none() {
            return Err("CEF could not create its offscreen browser".into());
        }
        Ok(true)
    }

    fn identifier(&self) -> Option<i32> {
        self.browser
            .as_ref()
            .filter(|_| !self.closed.load(Ordering::Acquire))
            .map(|b| b.identifier())
    }

    fn frame_rate(&self) -> Option<u32> {
        self.browser
            .as_ref()
            .filter(|_| !self.closed.load(Ordering::Acquire))
            .and_then(|browser| browser.host())
            .and_then(|host| u32::try_from(host.windowless_frame_rate()).ok())
    }

    fn current_url(&self) -> String {
        self.browser
            .as_ref()
            .and_then(|b| b.main_frame())
            .map(|f| cef::CefString::from(&f.url()).to_string())
            .filter(|url| !url.is_empty())
            .unwrap_or_else(|| self.url.clone())
    }

    fn request_close(&self) {
        // Disable publication before CEF begins its asynchronous close callbacks.
        self.active.store(false, Ordering::Release);
        if let Some(host) = self.browser.as_ref().and_then(|b| b.host()) {
            host.set_focus(0);
            host.was_hidden(1);
            host.close_browser(1);
        } else {
            self.closed.store(true, Ordering::Release);
        }
    }

    fn restore_pixels(&self, mailbox: &Mailbox) {
        let snapshot = self.surface.snapshot();
        mailbox.popup_visible(false);
        if let Some(view) = snapshot.view {
            mailbox.publish(false, (*view).clone());
        } else {
            // Never display the previous document while the selected page loads.
            let [width, height] = mailbox.desired_size();
            if let Some(length) = paint_length(width, height) {
                mailbox.publish(
                    false,
                    Pixels {
                        width,
                        height,
                        bgra: vec![0; length].into(),
                    },
                );
            }
        }
        if let Some((rect, pixels)) = snapshot.popup {
            mailbox.popup_rect(rect);
            mailbox.popup_visible(true);
            mailbox.publish(true, (*pixels).clone());
        }
    }
}

pub struct Runtime {
    owner: ThreadId,
    config: Config,
    mailbox: Arc<Mailbox>,
    input: InputQueue,
    ready: Arc<AtomicBool>,
    _application: App,
    shared: Option<Page>,
    click_gui: Option<Page>,
    hud: Option<Page>,
    hud_mailbox: Arc<Mailbox>,
    hud_fps: u32,
    hud_visible: bool,
    hud_generation: u64,
    // Retain Browser/Client/callback state until that page's OnBeforeClose.
    retired: Vec<Page>,
    active: PageTarget,
    size: [u32; 2],
    visible: bool,
    focus: bool,
    fps: u32,
    created_pages: u64,
    closed_pages: u64,
    closing: bool,
}

impl Runtime {
    pub fn initialize(
        config: Config,
        mailbox: Arc<Mailbox>,
        input: InputQueue,
    ) -> Result<Self, String> {
        if !config.helper_executable.is_file() {
            return Err("CEF subprocess helper does not exist".into());
        }
        if !config.resources_directory.join("icudtl.dat").is_file() {
            return Err("CEF resources_directory must contain icudtl.dat and locales/".into());
        }
        std::fs::create_dir_all(&config.cache_directory).map_err(|e| e.to_string())?;
        let helper = cef_path(&config.helper_executable)?;
        let cache = cef_path(&config.cache_directory)?;
        let resources = cef_path(&config.resources_directory)?;
        if INITIALIZED_ONCE.swap(true, Ordering::AcqRel) {
            return Err("CEF may initialize only once in one process".into());
        }
        let ready = Arc::new(AtomicBool::new(false));
        let mut application = Application::new(ready.clone());
        let args = Args::new();
        // Select the stable C API represented by these fixed Rust bindings before initialization.
        let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
        let settings = Settings {
            no_sandbox: 1,
            windowless_rendering_enabled: 1,
            external_message_pump: 1,
            multi_threaded_message_loop: 0,
            browser_subprocess_path: helper.as_str().into(),
            root_cache_path: cache.as_str().into(),
            cache_path: PathBuf::from(&cache)
                .join("browser")
                .to_string_lossy()
                .as_ref()
                .into(),
            resources_dir_path: resources.as_str().into(),
            locales_dir_path: PathBuf::from(&resources)
                .join("locales")
                .to_string_lossy()
                .as_ref()
                .into(),
            log_file: PathBuf::from(&cache)
                .join("chromium.log")
                .to_string_lossy()
                .as_ref()
                .into(),
            user_agent_product: "MCEF/2".into(),
            background_color: 0,
            ..Default::default()
        };
        if initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(&mut application),
            std::ptr::null_mut(),
        ) != 1
        {
            return Err(
                "cef_initialize failed; inspect chromium.log and matching libcef.dll".into(),
            );
        }
        let size = mailbox.desired_size();
        let shared = Some(Page::new(config.url.clone(), mailbox.clone(), true));
        Ok(Self {
            owner: std::thread::current().id(),
            config,
            mailbox,
            input,
            ready,
            _application: application,
            shared,
            click_gui: None,
            hud: None,
            hud_mailbox: Arc::new(Mailbox::new(size)),
            hud_fps: 60,
            hud_visible: false,
            hud_generation: 0,
            retired: Vec::new(),
            active: PageTarget::Shared,
            size,
            visible: true,
            focus: false,
            fps: 60,
            created_pages: 0,
            closed_pages: 0,
            closing: false,
        })
    }

    fn page(&self, target: PageTarget) -> Option<&Page> {
        match target {
            PageTarget::Shared => self.shared.as_ref(),
            PageTarget::ClickGui => self.click_gui.as_ref(),
        }
    }

    fn page_mut(&mut self, target: PageTarget) -> Option<&mut Page> {
        match target {
            PageTarget::Shared => self.shared.as_mut(),
            PageTarget::ClickGui => self.click_gui.as_mut(),
        }
    }

    fn ensure_page(&mut self, target: PageTarget) {
        if self.page(target).is_some() {
            return;
        }
        let url = match target {
            PageTarget::Shared => &self.config.url,
            PageTarget::ClickGui => self
                .config
                .click_gui_url
                .as_ref()
                .unwrap_or(&self.config.url),
        };
        let page = Page::new(url.clone(), self.mailbox.clone(), self.active == target);
        match target {
            PageTarget::Shared => self.shared = Some(page),
            PageTarget::ClickGui => self.click_gui = Some(page),
        }
    }

    fn update_visibility(&self) {
        // All callbacks use the same CEF UI thread. Disable both producers before changing hosts.
        for page in [&self.shared, &self.click_gui].into_iter().flatten() {
            page.active.store(false, Ordering::Release);
        }
        for (target, page) in [
            (PageTarget::Shared, &self.shared),
            (PageTarget::ClickGui, &self.click_gui),
        ] {
            if let Some(page) = page {
                let selected = target == self.active;
                page.active.store(selected, Ordering::Release);
                if let Some(host) = page.browser.as_ref().and_then(|b| b.host()) {
                    host.set_focus((selected && self.visible && self.focus) as i32);
                    host.was_hidden(!(selected && self.visible) as i32);
                    if selected && self.visible {
                        host.invalidate(PaintElementType::VIEW);
                    }
                }
            }
        }
    }

    fn open_pages(&mut self) -> Result<(), String> {
        if !self.ready.load(Ordering::Acquire) || self.closing {
            return Ok(());
        }
        let mut opened = false;
        for page in [&mut self.shared, &mut self.click_gui]
            .into_iter()
            .flatten()
        {
            if page.open(self.fps)? {
                self.created_pages += 1;
                opened = true;
            }
        }
        if opened {
            self.update_visibility();
        }
        Ok(())
    }

    fn select_page(&mut self, target: PageTarget) -> Result<(), String> {
        let existed = self.page(target).is_some();
        self.ensure_page(target);
        self.open_pages()?;
        if self.active == target && existed {
            return Ok(());
        }
        for page in [&self.shared, &self.click_gui].into_iter().flatten() {
            page.active.store(false, Ordering::Release);
        }
        self.active = target;
        if let Some(page) = self.page(target) {
            page.restore_pixels(&self.mailbox);
        }
        self.update_visibility();
        Ok(())
    }

    fn retire_page(&mut self, page: Page) {
        page.request_close();
        if page.browser.is_some() {
            self.retired.push(page);
        }
    }

    fn close_page(&mut self, target: PageTarget) -> Result<(), String> {
        let page = match target {
            PageTarget::Shared => self.shared.take(),
            PageTarget::ClickGui => self.click_gui.take(),
        };
        let Some(page) = page else {
            return Ok(());
        };
        self.retire_page(page);
        if self.active == target {
            if target == PageTarget::ClickGui {
                self.select_page(PageTarget::Shared)?;
            } else if self.click_gui.is_some() {
                self.select_page(PageTarget::ClickGui)?;
            } else {
                self.mailbox.popup_visible(false);
                let [width, height] = self.mailbox.desired_size();
                if let Some(length) = paint_length(width, height) {
                    self.mailbox.publish(
                        false,
                        Pixels {
                            width,
                            height,
                            bgra: vec![0; length].into(),
                        },
                    );
                }
            }
        }
        Ok(())
    }

    fn restart_pages(&mut self, fps: u32) -> Result<(), String> {
        self.fps = fps.clamp(1, i32::MAX as u32);
        for target in [PageTarget::Shared, PageTarget::ClickGui] {
            let slot = match target {
                PageTarget::Shared => &mut self.shared,
                PageTarget::ClickGui => &mut self.click_gui,
            };
            if let Some(page) = slot.take() {
                let url = page.current_url();
                self.retire_page(page);
                let replacement = Page::new(url, self.mailbox.clone(), target == self.active);
                match target {
                    PageTarget::Shared => self.shared = Some(replacement),
                    PageTarget::ClickGui => self.click_gui = Some(replacement),
                }
            }
        }
        if let Some(page) = self.page(self.active) {
            page.restore_pixels(&self.mailbox);
        }
        self.open_pages()?;
        self.update_visibility();
        Ok(())
    }

    fn hud_overlay(
        &mut self,
        url: String,
        fps: u32,
        visible: bool,
        close: bool,
        generation: u64,
    ) -> Result<(), String> {
        let fps = fps.clamp(1, i32::MAX as u32);
        let replace = close
            || fps != self.hud_fps
            || generation != self.hud_generation
            || self.hud.as_ref().is_some_and(|p| p.url != url);
        if replace {
            if let Some(page) = self.hud.take() {
                self.retire_page(page);
            }
            // A recreated document may take several frames to paint.  Keep
            // the independent overlay transparent during that interval rather
            // than exposing pixels from the retired document.
            self.clear_hud_pixels();
        }
        self.hud_fps = fps;
        self.hud_visible = visible && !close;
        self.hud_generation = generation;
        if !close && self.hud.is_none() {
            self.hud = Some(Page::new(url, self.hud_mailbox.clone(), true));
        }
        if close {
            self.clear_hud_pixels();
        }
        self.open_hud()
    }
    fn clear_hud_pixels(&self) {
        self.hud_mailbox.popup_visible(false);
        let [width, height] = self.hud_mailbox.desired_size();
        if let Some(length) = paint_length(width, height) {
            self.hud_mailbox.publish(
                false,
                Pixels {
                    width,
                    height,
                    bgra: vec![0; length].into(),
                },
            );
        }
    }
    fn open_hud(&mut self) -> Result<(), String> {
        if !self.ready.load(Ordering::Acquire) || self.closing {
            return Ok(());
        }
        if let Some(page) = &mut self.hud {
            if page.open(self.hud_fps)? {
                self.created_pages += 1;
            }
            if let Some(host) = page.browser.as_ref().and_then(|b| b.host()) {
                host.set_focus(0);
                host.was_hidden(!self.hud_visible as i32);
            }
        }
        Ok(())
    }
    fn reap_closed_pages(&mut self) {
        let before = self.retired.len();
        self.retired
            .retain(|page| !page.closed.load(Ordering::Acquire));
        self.closed_pages += (before - self.retired.len()) as u64;
    }

    fn status(&self) -> PageStatus {
        let shared_browser_id = self.shared.as_ref().and_then(Page::identifier);
        let click_gui_browser_id = self.click_gui.as_ref().and_then(Page::identifier);
        PageStatus {
            active: self.active,
            shared_open: shared_browser_id.is_some(),
            click_gui_open: click_gui_browser_id.is_some(),
            shared_browser_id,
            click_gui_browser_id,
            shared_fps: self.shared.as_ref().and_then(Page::frame_rate),
            click_gui_fps: self.click_gui.as_ref().and_then(Page::frame_rate),
            retired_pages: self.retired.len(),
            created_pages: self.created_pages,
            closed_pages: self.closed_pages,
            fps: self.fps,
            hud_open: self
                .hud
                .as_ref()
                .is_some_and(|page| page.browser.is_some() && !page.closed.load(Ordering::Acquire)),
            hud_browser_id: self.hud.as_ref().and_then(Page::identifier),
            hud_fps: self.hud.as_ref().and_then(Page::frame_rate),
            hud_visible: self.hud_visible,
            hud_generation: self.hud_generation,
        }
    }

    pub fn pump(&mut self) -> Result<(), String> {
        if self.owner != std::thread::current().id() {
            return Err("CEF pump must run on its initializing thread".into());
        }
        do_message_loop_work();
        self.reap_closed_pages();
        if self.closing || !self.ready.load(Ordering::Acquire) {
            return Ok(());
        }
        let size = self.mailbox.desired_size();
        if size != self.size {
            self.size = size;
            self.hud_mailbox.resize(size);
            for page in [&self.shared, &self.click_gui, &self.hud]
                .into_iter()
                .flatten()
            {
                if let Some(host) = page.browser.as_ref().and_then(|b| b.host()) {
                    host.was_resized();
                }
            }
        }
        for event in self.input.drain() {
            match event {
                Input::SelectPage(target) => self.select_page(target)?,
                Input::EnsurePage(target) => {
                    self.ensure_page(target);
                    self.open_pages()?;
                }
                Input::ClosePage(target) => self.close_page(target)?,
                Input::ReloadPage(target) => {
                    if let Some(browser) = self.page(target).and_then(|p| p.browser.as_ref()) {
                        browser.reload();
                    }
                }
                Input::RestartPages { fps } => self.restart_pages(fps)?,
                Input::HudOverlay {
                    url,
                    fps,
                    visible,
                    close,
                    generation,
                } => self.hud_overlay(url, fps, visible, close, generation)?,
                Input::Focus(focus) => {
                    self.focus = focus;
                    self.update_visibility();
                }
                Input::Visible(visible) => {
                    self.visible = visible;
                    self.update_visibility();
                }
                event => {
                    self.open_pages()?;
                    self.dispatch_browser_input(event);
                }
            }
        }
        self.open_hud()?;
        self.open_pages()
    }

    fn dispatch_browser_input(&mut self, event: Input) {
        let Some(page) = self.page_mut(self.active) else {
            return;
        };
        let Some(browser) = page.browser.as_ref() else {
            return;
        };
        let Some(host) = browser.host() else {
            return;
        };
        match event {
            Input::MouseMove {
                x,
                y,
                modifiers,
                leave,
            } => host.send_mouse_move_event(Some(&MouseEvent { x, y, modifiers }), leave as i32),
            Input::MouseButton {
                x,
                y,
                modifiers,
                button,
                up,
                clicks,
            } => {
                let button = match button {
                    Button::Left => MouseButtonType::LEFT,
                    Button::Middle => MouseButtonType::MIDDLE,
                    Button::Right => MouseButtonType::RIGHT,
                };
                host.send_mouse_click_event(
                    Some(&MouseEvent { x, y, modifiers }),
                    button,
                    up as i32,
                    i32::from(clicks.max(1)),
                );
            }
            Input::Wheel {
                x,
                y,
                modifiers,
                delta_x,
                delta_y,
            } => {
                host.send_mouse_wheel_event(Some(&MouseEvent { x, y, modifiers }), delta_x, delta_y)
            }
            Input::Key {
                virtual_key,
                native_code,
                modifiers,
                up,
                system,
            } => host.send_key_event(Some(&KeyEvent {
                type_: if up {
                    KeyEventType::KEYUP
                } else {
                    KeyEventType::RAWKEYDOWN
                },
                modifiers,
                windows_key_code: virtual_key,
                native_key_code: native_code,
                is_system_key: system as i32,
                ..Default::default()
            })),
            Input::Text { utf16, modifiers } => {
                for character in utf16 {
                    host.send_key_event(Some(&KeyEvent {
                        type_: KeyEventType::CHAR,
                        modifiers,
                        windows_key_code: i32::from(character),
                        character,
                        unmodified_character: character,
                        ..Default::default()
                    }));
                }
            }
            Input::Focus(focus) => host.set_focus(focus as i32),
            Input::Visible(visible) => host.was_hidden(!visible as i32),
            Input::Navigate(url) => {
                page.url = url.clone();
                if let Some(frame) = browser.main_frame() {
                    frame.load_url(Some(&url.as_str().into()));
                }
            }
            Input::Reload => browser.reload(),
            Input::Repaint => host.invalidate(PaintElementType::VIEW),
            Input::SelectPage(_)
            | Input::EnsurePage(_)
            | Input::ClosePage(_)
            | Input::ReloadPage(_)
            | Input::RestartPages { .. }
            | Input::HudOverlay { .. } => unreachable!("page command routed above"),
        }
    }

    fn request_close(&mut self) {
        self.closing = true;
        if let Some(page) = self.shared.take() {
            self.retire_page(page);
        }
        if let Some(page) = self.click_gui.take() {
            self.retire_page(page);
        }
        if let Some(page) = self.hud.take() {
            self.retire_page(page);
        }
        self.hud_visible = false;
    }
}

struct PendingRuntime {
    config: Config,
    mailbox: Arc<Mailbox>,
    input: InputQueue,
}

thread_local! {
    static CURRENT: RefCell<Option<Runtime>> = const { RefCell::new(None) };
    static PENDING: RefCell<Option<PendingRuntime>> = const { RefCell::new(None) };
}

/// Register before Horizon starts, then initialize on the first real frame.
/// Renderer/world startup can exceed Chromium's 15-second child connection
/// timeout; no Chromium child should be created until the frame pump can run.
pub fn initialize_on_first_frame(
    config: Config,
    mailbox: Arc<Mailbox>,
    input: InputQueue,
) -> Result<(), String> {
    if !config.helper_executable.is_file() {
        return Err("CEF subprocess helper does not exist".into());
    }
    if !config.resources_directory.join("icudtl.dat").is_file() {
        return Err("CEF resources_directory must contain icudtl.dat and locales/".into());
    }
    if INITIALIZED_ONCE.load(Ordering::Acquire) || CURRENT.with_borrow(Option::is_some) {
        return Err("CEF runtime is already initialized on this thread".into());
    }
    PENDING.with_borrow_mut(|pending| {
        if pending.is_some() {
            return Err("CEF runtime is already scheduled on this thread".into());
        }
        *pending = Some(PendingRuntime {
            config,
            mailbox,
            input,
        });
        Ok(())
    })
}

pub fn initialize_current_thread(
    config: Config,
    mailbox: Arc<Mailbox>,
    input: InputQueue,
) -> Result<(), String> {
    if PENDING.with_borrow(Option::is_some) {
        return Err("CEF runtime is already scheduled on this thread".into());
    }
    CURRENT.with_borrow_mut(|current| {
        if current.is_some() {
            return Err("CEF runtime is already initialized on this thread".into());
        }
        *current = Some(Runtime::initialize(config, mailbox, input)?);
        Ok(())
    })
}

pub fn pump_current_thread() -> Result<(), String> {
    if let Some(pending) = PENDING.with_borrow_mut(Option::take) {
        initialize_current_thread(pending.config, pending.mailbox, pending.input)?;
    }
    CURRENT.with_borrow_mut(|current| match current {
        Some(runtime) => runtime.pump(),
        None => Ok(()),
    })
}

pub fn page_state_current_thread() -> Option<PageState> {
    CURRENT.with_borrow(|current| {
        current.as_ref().and_then(|runtime| {
            runtime
                .page(runtime.active)
                .map(|page| page.state.lock().unwrap_or_else(|e| e.into_inner()).clone())
        })
    })
}

pub fn hud_snapshot_current_thread() -> Option<crate::paint::Snapshot> {
    CURRENT.with_borrow(|current| {
        current
            .as_ref()
            .filter(|r| r.hud_visible && r.hud.is_some())
            .map(|r| r.hud_mailbox.snapshot())
    })
}
pub fn hud_page_state_current_thread() -> Option<PageState> {
    CURRENT.with_borrow(|current| {
        current
            .as_ref()
            .and_then(|r| r.hud.as_ref())
            .map(|p| p.state.lock().unwrap_or_else(|e| e.into_inner()).clone())
    })
}
pub fn page_status_current_thread() -> Option<PageStatus> {
    CURRENT.with_borrow(|current| current.as_ref().map(Runtime::status))
}

/// Pump until every page's OnBeforeClose, then release CEF references before cef_shutdown. A timeout leaves
/// the runtime owned by its thread; calling cef_shutdown while a browser is alive is invalid.
pub fn shutdown_current_thread(timeout: Duration) -> Result<bool, String> {
    // Startup may fail before the first frame. Dropping an uninitialized
    // registration requires no cef_shutdown and must never initialize CEF.
    PENDING.with_borrow_mut(Option::take);
    CURRENT.with_borrow_mut(|current| {
        let Some(runtime) = current.as_mut() else {
            return Ok(true);
        };
        if runtime.owner != std::thread::current().id() {
            return Err("CEF shutdown thread changed".into());
        }
        runtime.request_close();
        let deadline = Instant::now() + timeout;
        runtime.reap_closed_pages();
        while !runtime.retired.is_empty() {
            runtime.pump()?;
            if Instant::now() >= deadline {
                return Ok(false);
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        // Drop Browser and Client handles before shutting down the native runtime.
        *current = None;
        shutdown();
        Ok(true)
    })
}

pub fn execute_subprocess() -> i32 {
    let args = Args::new();
    let mut application = Application::new(Arc::new(AtomicBool::new(false)));
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    execute_process(
        Some(args.as_main_args()),
        Some(&mut application),
        std::ptr::null_mut(),
    )
}
