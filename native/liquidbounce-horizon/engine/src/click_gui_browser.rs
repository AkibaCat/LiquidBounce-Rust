// SPDX-License-Identifier: GPL-3.0-or-later
//! Native ClickGUI screen policy, real Chromium documents and legacy settings migration.
use super::*;
use liquidbounce_behaviors::click_gui::{Action, ClickGui};
use liquidbounce_webview::input::PageTarget;
impl Browser {
    pub(super) fn click_gui_frame(
        &mut self,
        hooks: &LiquidBounceHooks,
        host: &mut dyn Host,
        client: &NativeClientState,
    ) {
        let refresh = host.client_click_gui_refresh_rate().max(1);
        let status = liquidbounce_webview::runtime::page_status_current_thread();
        let ready = status.as_ref().is_some_and(|s| s.shared_open);
        let cached = status.as_ref().is_some_and(|s| s.click_gui_open);
        let now = hooks.started.elapsed().as_millis().min(u64::MAX as u128) as u64;
        let actions = {
            let state = hooks.state();
            let mut b = state.behaviors.borrow_mut();
            b.click_gui.frame(
                now,
                client.logged_in && client.world.is_some(),
                ready,
                self.visible,
                cached,
                refresh,
            )
        };
        for action in actions {
            match action {
                Action::EnsureCache => push(self, Input::EnsurePage(PageTarget::ClickGui)),
                Action::SelectCache => push(self, Input::SelectPage(PageTarget::ClickGui)),
                Action::SelectShared => push(self, Input::SelectPage(PageTarget::Shared)),
                Action::CloseCache => push(self, Input::ClosePage(PageTarget::ClickGui)),
                Action::ReloadCache => push(self, Input::ReloadPage(PageTarget::ClickGui)),
                Action::RestartPages(fps) => push(self, Input::RestartPages { fps }),
                Action::OpenScreen => {
                    host.mod_screen_open(MOD_ID);
                    self.screen(host.mod_screen() == Some(MOD_ID));
                }
                Action::CloseScreen => {
                    host.mod_screen_close(MOD_ID);
                    self.screen(false);
                }
            }
        }
        horizon_client::inventory_move::SEARCH_FOCUSED.store(
            ClickGui::in_search_bar(self.ui.typing, self.visible),
            std::sync::atomic::Ordering::Release,
        );
    }
    pub(super) fn migrate_click_gui(&self, actor: &mut Actor<'_>) {
        // A saved actor module takes precedence over the older presentation-only file.
        let migrate = match std::fs::read(&self.modules_file) {
            Ok(bytes) => serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|v| {
                    v["value"]
                        .as_array()
                        .map(|groups| !groups.iter().any(|g| g["name"] == "ClickGUI"))
                })
                .unwrap_or(false),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => false,
        };
        if migrate {
            use liquidbounce_interop::protocol::ModuleAccess;
            if let Err(e) = actor.set_settings("ClickGUI", &self.ui.clickgui_settings()) {
                eprintln!("[LiquidBounce] legacy ClickGUI migration: {e}");
            }
        }
    }
}
impl LiquidBounceHooks {
    pub(super) fn click_gui_bound(&self, key: liquidbounce_core::InputKey, modifiers: u32) -> bool {
        self.state()
            .registry
            .modules()
            .find(|m| m.definition().name == "ClickGUI")
            .and_then(|m| m.definition().bind)
            .is_some_and(|b| {
                b.key == key && crate::input::modifiers(modifiers).includes(b.modifiers)
            })
    }
    pub(crate) fn click_gui_limit(
        &self,
        host: &dyn Host,
        event: &mut horizon_client::mod_api::FramerateLimitEvent,
    ) {
        if self.browser.is_none() || host.mod_screen() != Some(MOD_ID) {
            return;
        }
        let state = self.state();
        let b = state.behaviors.borrow();
        if b.click_gui.settings.sync_game_fps {
            event.limit = event.limit.min(
                b.click_gui
                    .settings
                    .current_fps(host.client_click_gui_refresh_rate())
                    .min(i32::MAX as u32) as i32,
            );
        }
    }
}
