// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleClickGui settings and lifecycle decisions from LiquidBounce ae0eb396.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub scale: f32,
    pub search_auto_focus: bool,
    pub snapping: bool,
    pub grid_size: i32,
    pub cache: bool,
    pub fps: i32,
    pub sync_game_fps: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            scale: 1.,
            search_auto_focus: true,
            snapping: true,
            grid_size: 10,
            cache: true,
            fps: 0,
            sync_game_fps: true,
        }
    }
}
impl Settings {
    pub fn current_fps(&self, refresh_hz: u32) -> u32 {
        if self.fps <= 0 {
            refresh_hz.max(1)
        } else {
            self.fps as u32
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    EnsureCache,
    SelectCache,
    SelectShared,
    CloseCache,
    ReloadCache,
    RestartPages(u32),
    OpenScreen,
    CloseScreen,
}
#[derive(Clone, Debug)]
pub struct ClickGui {
    pub settings: Settings,
    pub refresh_hz: u32,
    pub open_requested: bool,
    pub close_requested: bool,
    pub sync_requested: bool,
    pub invalidate_requested: bool,
    pub restart_requested: bool,
    disconnect_requested: bool,
    sync_at: Option<u64>,
    last_fps: Option<u32>,
}
impl Default for ClickGui {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            refresh_hz: 60,
            open_requested: false,
            close_requested: false,
            sync_requested: false,
            invalidate_requested: false,
            restart_requested: false,
            disconnect_requested: false,
            sync_at: None,
            last_fps: None,
        }
    }
}
impl ClickGui {
    pub fn enabled(&mut self, in_game: bool) {
        if in_game {
            self.open_requested = true;
        }
    }
    pub fn cache_changed(&mut self) {
        self.open_requested = true;
    }
    pub fn renderer_changed(&mut self) {
        self.restart_requested = true;
    }
    pub fn level_loaded(&mut self, now_ms: u64) {
        if self.settings.cache {
            self.sync_at = Some(now_ms.saturating_add(1000));
        }
    }
    pub fn disconnected(&mut self) {
        self.disconnect_requested = true;
        self.sync_at = None;
        self.open_requested = false;
    }
    pub fn in_search_bar(typing: bool, click_gui_open: bool) -> bool {
        typing && click_gui_open
    }
    pub fn frame(
        &mut self,
        now_ms: u64,
        in_game: bool,
        ready: bool,
        visible: bool,
        cache_exists: bool,
        refresh_hz: u32,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.disconnect_requested {
            self.disconnect_requested = false;
            if visible {
                actions.push(Action::CloseScreen);
            }
            // Also cancels an EnsureCache already queued before the disconnect.
            actions.push(Action::CloseCache);
            return actions;
        }
        if self.close_requested {
            self.close_requested = false;
            self.open_requested = false;
            if visible {
                actions.push(Action::CloseScreen);
            }
            return actions;
        }
        if !ready {
            return actions;
        }
        let fps = self.settings.current_fps(refresh_hz);
        let restart = self.restart_requested || self.last_fps != Some(fps);
        self.refresh_hz = refresh_hz.max(1);
        if restart {
            self.restart_requested = false;
            if cache_exists {
                self.invalidate_requested = true;
            }
        }
        self.last_fps = Some(fps);
        let mut cache_exists = cache_exists;
        if self.invalidate_requested {
            self.invalidate_requested = false;
            if cache_exists {
                if visible {
                    actions.push(Action::CloseScreen);
                    self.open_requested = in_game;
                }
                actions.push(Action::CloseCache);
                cache_exists = false;
            }
        }
        if restart {
            actions.push(Action::RestartPages(fps));
        }
        if self.sync_requested {
            self.sync_requested = false;
            if cache_exists {
                actions.push(Action::ReloadCache);
            }
        }
        if self.sync_at.is_some_and(|at| now_ms >= at) {
            self.sync_at = None;
            if in_game && self.settings.cache {
                actions.push(if cache_exists {
                    Action::ReloadCache
                } else {
                    Action::EnsureCache
                });
                cache_exists = true;
            }
        }
        if self.open_requested {
            self.open_requested = false;
            if in_game {
                if self.settings.cache {
                    if !cache_exists {
                        actions.push(Action::EnsureCache);
                    }
                    actions.push(Action::SelectCache);
                } else {
                    if cache_exists {
                        actions.push(Action::CloseCache);
                    }
                    actions.push(Action::SelectShared);
                }
                actions.push(Action::OpenScreen);
            }
        }
        actions
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn menu_gates_world_delay_and_cache_reuse() {
        let mut gui = ClickGui::default();
        gui.enabled(false);
        assert!(!gui.open_requested);
        gui.level_loaded(10);
        assert_eq!(
            gui.frame(1009, true, true, false, false, 144),
            [Action::RestartPages(144)]
        );
        assert_eq!(
            gui.frame(1010, true, true, false, false, 144),
            [Action::EnsureCache]
        );
        gui.enabled(true);
        assert_eq!(
            gui.frame(1011, true, true, false, true, 144),
            [Action::SelectCache, Action::OpenScreen]
        );
        gui.level_loaded(2000);
        assert_eq!(
            gui.frame(3000, true, true, true, true, 144),
            [Action::ReloadCache]
        );
        gui.settings.cache = false;
        gui.cache_changed();
        assert_eq!(
            gui.frame(3001, true, true, true, true, 144),
            [Action::CloseCache, Action::SelectShared, Action::OpenScreen]
        );
    }
    #[test]
    fn invalidate_keeps_closed_cache_lazy_and_reopens_active_screen() {
        let mut gui = ClickGui::default();
        gui.invalidate_requested = true;
        assert_eq!(
            gui.frame(0, true, true, false, true, 60),
            [Action::CloseCache, Action::RestartPages(60)]
        );
        gui.invalidate_requested = true;
        assert_eq!(
            gui.frame(1, true, true, true, true, 60),
            [
                Action::CloseScreen,
                Action::CloseCache,
                Action::EnsureCache,
                Action::SelectCache,
                Action::OpenScreen
            ]
        );
        gui.disconnected();
        assert_eq!(
            gui.frame(2, false, false, true, true, 60),
            [Action::CloseScreen, Action::CloseCache]
        );
        assert!(gui.frame(3000, false, true, false, false, 60).is_empty());
    }
    #[test]
    fn renderer_restart_waits_for_ready_and_search_is_screen_scoped() {
        let mut gui = ClickGui::default();
        gui.renderer_changed();
        gui.enabled(true);
        assert!(gui.frame(0, true, false, false, false, 165).is_empty());
        assert_eq!(
            gui.frame(1, true, true, false, false, 165),
            [
                Action::RestartPages(165),
                Action::EnsureCache,
                Action::SelectCache,
                Action::OpenScreen
            ]
        );
        assert_eq!(gui.settings.current_fps(0), 1);
        gui.settings.fps = 30;
        assert_eq!(gui.settings.current_fps(165), 30);
        assert!(!ClickGui::in_search_bar(true, false));
        assert!(!ClickGui::in_search_bar(false, true));
        assert!(ClickGui::in_search_bar(true, true));
    }
}
