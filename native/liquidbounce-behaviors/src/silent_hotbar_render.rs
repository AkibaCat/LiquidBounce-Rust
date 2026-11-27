// SPDX-License-Identifier: GPL-3.0-or-later
//! The cosmetic ModuleSilentHotbar; the slot service has a separate lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SilentHotbarRender {
    pub no_cooldown_progress: bool,
}
impl Default for SilentHotbarRender {
    fn default() -> Self {
        Self {
            no_cooldown_progress: true,
        }
    }
}
impl SilentHotbarRender {
    pub fn render(
        &self,
        running: bool,
        modified: bool,
        clientside_slot: u8,
        visible_slot: &mut i32,
        swap_scale: &mut f32,
    ) {
        if running {
            *visible_slot = i32::from(clientside_slot);
            if self.no_cooldown_progress && modified {
                *swap_scale = 1.0;
            }
        }
    }
}
