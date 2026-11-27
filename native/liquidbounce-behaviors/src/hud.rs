// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleHud ae0eb396: independent overlay lifetime and F1/editor policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    None,
    Other,
    ClickGui,
    Disconnected,
    LevelLoading,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overlay {
    #[default]
    Closed,
    Hidden,
    Visible,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Hud {
    pub blur: bool,
    pub sigma: f32,
    pub alpha_blend_range: [f32; 2],
    pub space_seperated_names: bool,
    pub fps: i32,
    pub renderer_ready: bool,
    pub hud_editor_selected: bool,
    pub last_hud_hidden: bool,
    pub overlay: Overlay,
    pub reopen_generation: u64,
}
impl Default for Hud {
    fn default() -> Self {
        Self {
            blur: true,
            sigma: 5.,
            alpha_blend_range: [0., 0.75],
            space_seperated_names: true,
            fps: 60,
            renderer_ready: false,
            hud_editor_selected: false,
            last_hud_hidden: false,
            overlay: Overlay::Closed,
            reopen_generation: 0,
        }
    }
}
impl Hud {
    pub fn running(enabled: bool, destructed: bool) -> bool {
        enabled && !destructed
    }
    pub fn blur_active(&self, hidden: bool, screen: Screen) -> bool {
        self.blur && !(hidden && screen == Screen::None)
    }
    pub fn current_fps(&self, refresh_rate: i32) -> i32 {
        if self.fps <= 0 {
            refresh_rate
        } else {
            self.fps
        }
    }
    pub fn update(
        &mut self,
        enabled: bool,
        in_game: bool,
        hidden: bool,
        screen: Screen,
    ) -> Overlay {
        self.last_hud_hidden = hidden;
        if !enabled || !in_game {
            self.overlay = Overlay::Closed;
            return self.overlay;
        }
        let show = !hidden
            && !matches!(screen, Screen::Disconnected | Screen::LevelLoading)
            && !(self.hud_editor_selected && screen == Screen::ClickGui);
        // Setting visible=false on a nonexistent CustomOverlay does not open it.
        self.overlay = if show {
            Overlay::Visible
        } else if self.overlay == Overlay::Closed {
            Overlay::Closed
        } else {
            Overlay::Hidden
        };
        self.overlay
    }
    pub fn reopen(&mut self) {
        self.overlay = Overlay::Closed;
        self.reopen_generation = self.reopen_generation.wrapping_add(1);
    }
    pub fn disconnect(&mut self) {
        self.overlay = Overlay::Closed;
    }
}
/// NativeHudComponent uses a fixed scale 2 for editor offsets, irrespective of GUI scale.
pub fn alignment_bounds(
    horizontal: &str,
    vertical: &str,
    offset: [i32; 2],
    size: [f32; 2],
    screen: [f32; 2],
    base_scale: f32,
) -> [f32; 4] {
    let [ox, oy] = [offset[0] as f32 / base_scale, offset[1] as f32 / base_scale];
    let x = match horizontal {
        "Right" => screen[0] - size[0] - ox,
        "CenterTranslated" => screen[0] / 2. - size[0] / 2. + ox,
        "Center" => screen[0] / 2. + ox,
        _ => ox,
    };
    let y = match vertical {
        "Bottom" => screen[1] - size[1] - oy,
        "CenterTranslated" => screen[1] / 2. - size[1] / 2. + oy,
        "Center" => screen[1] / 2. + oy,
        _ => oy,
    };
    [x, y, x + size[0], y + size[1]]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_overlay_creation_visibility_and_f1_editor_lifetime() {
        let mut h = Hud::default();
        assert!(Hud::running(true, false));
        assert_eq!(h.update(true, true, true, Screen::Other), Overlay::Closed);
        assert_eq!(h.update(true, true, false, Screen::None), Overlay::Visible);
        assert_eq!(h.update(true, true, true, Screen::Other), Overlay::Hidden);
        assert!(h.blur_active(true, Screen::Other));
        assert!(!h.blur_active(true, Screen::None));
        h.hud_editor_selected = true;
        assert_eq!(
            h.update(true, true, false, Screen::ClickGui),
            Overlay::Hidden
        );
        assert_eq!(h.update(true, true, false, Screen::Other), Overlay::Visible);
        assert_eq!(h.update(true, false, false, Screen::Other), Overlay::Closed);
        assert_eq!(
            h.update(true, true, false, Screen::Disconnected),
            Overlay::Closed
        );
        h.reopen();
        assert_eq!(h.reopen_generation, 1);
        assert_eq!(h.current_fps(144), 60);
        h.fps = 0;
        assert_eq!(h.current_fps(144), 144);
    }
    #[test]
    fn native_editor_offsets_use_fixed_two() {
        assert_eq!(
            alignment_bounds("Left", "Top", [7, 180], [96., 96.], [640., 360.], 2.),
            [3.5, 90., 99.5, 186.]
        );
        assert_eq!(
            alignment_bounds("Right", "Bottom", [10, 20], [100., 50.], [640., 360.], 2.),
            [535., 300., 635., 350.]
        );
    }
}
