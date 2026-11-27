// SPDX-License-Identifier: GPL-3.0-or-later
//! The original menu routes follow real Horizon screens on the shared CEF page.
use super::*;
use liquidbounce_webview::input::PageTarget;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct MenuState {
    native: Option<&'static str>,
    overlay: Option<&'static str>,
    ready: bool,
    acknowledged_paint: Option<u64>,
    browser_id: Option<i32>,
    server_pings: BTreeMap<String, Value>,
    last_server_poll: Option<Instant>,
}

pub(super) fn route(native: Option<&str>) -> Option<&'static str> {
    match native? {
        "title" => Some("title"),
        "world_select" => Some("singleplayer"),
        "multiplayer" => Some("multiplayer"),
        _ => None,
    }
}

impl MenuState {
    fn screen(&self) -> Option<&'static str> {
        self.overlay.or(self.native)
    }

    fn invalidate(&mut self) {
        self.ready = false;
        self.acknowledged_paint = None;
    }

    fn accepts(&self, native: Option<&str>) -> bool {
        self.ready && self.native.is_some() && self.native == route(native)
    }
}

impl Browser {
    pub(super) fn sync_screens(&mut self, click_gui: bool, native: Option<&str>) {
        let next = route(native);
        if self.menu.native != next {
            self.menu.native = next;
            self.menu.overlay = None;
            self.menu.invalidate();
        }
        self.screen(click_gui);
    }

    pub(super) fn sync_route(&mut self) {
        let next = if self.visible { Some("clickgui") } else { self.menu.screen() };
        if self.ui.screen() != next {
            self.ui.set_screen(next);
            self.buttons = 0;
            self.menu.invalidate();
            if !self.visible {
                push(self, Input::SelectPage(PageTarget::Shared));
            }
            self.server.broadcast("virtualScreen", &json!({
                "action": if next.is_some() { "open" } else { "close" },
                "screenName": next.unwrap_or("none"),
            }));
        }
        self.refresh_surface();
    }

    fn refresh_surface(&self) {
        // CEF must render while pending; the compositor withholds stale pixels.
        self.layer.set_visible(self.visible || self.menu.screen().is_some());
        self.layer.set_compositing(self.visible || self.menu.ready);
    }

    pub(super) fn menu_frame(&mut self) {
        if self.visible || self.menu.screen().is_none() {
            self.refresh_surface();
            return;
        }
        let status = liquidbounce_webview::runtime::page_status_current_thread();
        let selected = status.as_ref().is_some_and(|s| s.active == PageTarget::Shared && s.shared_open);
        let browser_id = status.as_ref().and_then(|s| s.shared_browser_id);
        if self.menu.browser_id != browser_id {
            self.menu.browser_id = browser_id;
            self.menu.invalidate();
        }
        let page = liquidbounce_webview::runtime::page_state_current_thread();
        let loaded = page.as_ref().is_some_and(|p|
            !p.loading && p.completed_loads > 0 && p.http_status.is_some_and(|s| (200..400).contains(&s)));
        if !selected || !loaded {
            self.menu.invalidate();
        } else if let Some(paint) = self.menu.acknowledged_paint {
            let snapshot = self.layer.mailbox.snapshot();
            self.menu.ready = self.layer.mailbox.view_paint_count() > paint
                && snapshot.view.as_ref().is_some_and(|pixels| pixels.width > 0 && pixels.height > 0);
        }
        self.refresh_surface();
    }

    pub(super) fn menu_acknowledge(&mut self, request: &ApiRequest, response: &ApiResponse) {
        if self.visible || !response.status.is_success() || request.method != Method::POST
            || request.path != "/api/v1/client/virtualScreen" {
            return;
        }
        let Ok(payload) = serde_json::from_slice::<Value>(&request.body) else { return; };
        if self.menu.screen().is_some() && payload["name"].as_str() == self.menu.screen() {
            self.menu.browser_id = liquidbounce_webview::runtime::page_status_current_thread()
                .and_then(|s| s.shared_browser_id);
            self.menu.acknowledged_paint = Some(self.layer.mailbox.view_paint_count());
            self.menu.ready = false;
            push(self, Input::Repaint);
            self.refresh_surface();
        }
    }

    /// Main-menu ClickGUI uses Shared, preserving the in-world cached document.
    pub(super) fn menu_request(&mut self, request: &ApiRequest) -> Option<ApiResponse> {
        if self.visible || self.menu.native.is_none() || request.path != "/api/v1/client/screen" {
            return None;
        }
        if request.method == Method::PUT {
            let payload: Value = serde_json::from_slice(&request.body).ok()?;
            if payload["name"] == "clickgui" {
                self.menu.overlay = Some("clickgui");
                self.sync_route();
                return Some(ApiResponse::no_content());
            }
            if self.menu.overlay.take().is_some() {
                self.sync_route();
            }
        }
        if request.method == Method::DELETE && self.menu.overlay.is_some() {
            self.menu.overlay = None;
            self.sync_route();
            return Some(ApiResponse::no_content());
        }
        None
    }

    pub(super) fn menu_escape(&mut self) -> bool {
        if !self.visible && self.menu.overlay.is_some() && !self.ui.page_keeps_escape {
            self.menu.overlay = None;
            self.sync_route();
            return true;
        }
        false
    }

    pub(super) fn accepts_menu(&self, native: Option<&str>) -> bool {
        !self.visible && self.menu.accepts(native)
    }

    pub(super) fn invalidate_menu(&mut self) {
        self.menu.invalidate();
        self.refresh_surface();
    }

    pub(super) fn menu_server_updates(&mut self, updates: Vec<Value>) {
        let next: BTreeMap<_, _> = updates.into_iter().filter_map(|row|
            row.get("id").map(|id| (id.to_string(), row.clone()))).collect();
        for (id, row) in &next {
            if self.menu.server_pings.get(id) != Some(row) {
                self.server.broadcast("serverPinged", &json!({"server":row}));
            }
        }
        self.menu.server_pings = next;
    }

    pub(super) fn menu_poll_servers(&mut self) -> bool {
        if self.menu.native != Some("multiplayer")
            || self.menu.last_server_poll.is_some_and(|last| last.elapsed() < Duration::from_millis(250)) {
            return false;
        }
        self.menu.last_server_poll = Some(Instant::now());
        true
    }

    pub(super) fn screen_descriptor(&self) -> Value {
        match self.ui.screen() {
            Some("title") => json!({"class":"net.minecraft.client.gui.screen.TitleScreen","title":"Title"}),
            Some("singleplayer") => json!({"class":"net.minecraft.client.gui.screen.world.SelectWorldScreen","title":"Select World"}),
            Some("multiplayer") => json!({"class":"net.minecraft.client.gui.screen.multiplayer.MultiplayerScreen","title":"Multiplayer"}),
            _ => clickgui_screen(),
        }
    }

    pub(super) fn menu_status(&self) -> Value {
        json!({"native":self.menu.native,"overlay":self.menu.overlay,
            "route":self.menu.screen(),"ready":self.menu.ready,
            "acknowledgedPaint":self.menu.acknowledged_paint,
            "viewPaintCount":self.layer.mailbox.view_paint_count()})
    }
}

impl LiquidBounceHooks {
    pub(super) fn browser_accepts_input(&self, host: &dyn Host) -> bool {
        let Some(browser) = &self.browser else { return false; };
        if host.mod_screen() == Some(MOD_ID) {
            return true;
        }
        let native = host.client_menu_screen();
        browser.lock().unwrap_or_else(|e| e.into_inner()).accepts_menu(native.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_real_supported_title_screens_capture_menu_input() {
        assert_eq!(route(Some("world_select")), Some("singleplayer"));
        assert_eq!(route(Some("multiplayer")), Some("multiplayer"));
        assert_eq!(route(Some("options")), None);
        let mut menu = MenuState { native: Some("title"), ..Default::default() };
        assert!(!menu.accepts(Some("title")));
        menu.ready = true;
        assert!(menu.accepts(Some("title")));
        assert!(!menu.accepts(Some("world_select")));
        menu.invalidate();
        assert!(!menu.accepts(Some("title")));
    }
}
