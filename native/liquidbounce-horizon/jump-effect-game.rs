// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
impl Game<'_> {
    pub(super) fn client_jump_effect_errors(&mut self) {
        if let Some(error) = horizon_client::gfx::jump_effect::take_error() {
            horizon_client::mod_api::Host::chat(self, &error);
        }
    }
    pub(super) fn client_jump_effect_frame(&self, partial: f64) {
        let mut geometry = "null".to_owned();
        if self.native_window() {
            self.native_client_jump_effect(
                &serde_json::json!({"partial":partial as f32}).to_string(),
                &mut geometry,
            );
        }
        horizon_client::gfx::jump_effect::begin_frame(&geometry);
    }
}
