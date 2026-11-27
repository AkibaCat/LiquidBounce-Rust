// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use serde_json::json;
impl Game<'_> {
    pub(super) fn client_particles_reset(&self) {
        self.native_client_particles("{\"action\":\"reset\"}", &mut String::new());
    }
    pub(super) fn client_particles_tick(&self) {
        if !self.native_window() {
            return;
        }
        let Some(camera) = self.native_camera() else {
            return;
        };
        self.native_client_particles(
            &json!({"action":"tick","camera":camera.position}).to_string(),
            &mut String::new(),
        );
    }
    pub(super) fn client_particles_frame(&self, partial: f64) {
        if !self.native_window() {
            return;
        }
        let q = self.camera_orientation();
        let right = q * glam::Vec3::X;
        let up = q * glam::Vec3::Y;
        let input =
            json!({"action":"frame","partial":partial as f32,"right":right.to_array(),"up":up.to_array()})
                .to_string();
        let mut geometry = String::new();
        self.native_client_particles(&input, &mut geometry);
        horizon_client::gfx::particles::begin_frame(&geometry);
    }
}
