// SPDX-License-Identifier: GPL-3.0-or-later
use crate::hud_components::HudComponents;
use liquidbounce_behaviors::hud::Hud;
#[derive(Default)]
pub struct HudRuntime {
    pub behavior: Hud,
    pub components: HudComponents,
    pub refresh_rate: u32,
    pub names_event: Option<bool>,
    pub published_generation: u64,
}
impl HudRuntime {
    pub fn names_changed(&mut self, value: bool) {
        if self.behavior.space_seperated_names != value {
            self.behavior.space_seperated_names = value;
            self.names_event = Some(value);
        }
    }
}
