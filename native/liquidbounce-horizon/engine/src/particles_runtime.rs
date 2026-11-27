// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientParticlesEvent, Host};
use liquidbounce_behaviors::targeting::VisualCamera;
use serde_json::Value;
fn wall() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
impl LiquidBounceHooks {
    pub(crate) fn particles_attack(&self, host: &dyn Host, target: i32) {
        let snapshot = crate::targeting::snapshot(&host.client_targeting_entity(target));
        let Ok(context) = serde_json::from_str::<Value>(&host.client_particles_context(target))
        else {
            return;
        };
        let (eligible, rotation) = {
            let mut state = self.state();
            let m = state.behaviors.borrow();
            let active = m.query_running(&state.registry, "Particles");
            let rotation = m.rotations.lock().unwrap().current;
            let free_look = m.query_running(&state.registry, "FreeLook");
            drop(m);
            let eligible = active
                && snapshot.as_ref().is_some_and(|s| {
                    let info = state.target_info(target, s);
                    state.global_targets.visual.should_show(
                        &s.entity,
                        info,
                        VisualCamera {
                            raw_first_person: context["firstPerson"].as_bool().unwrap_or(true),
                            free_cam: false,
                            free_look,
                        },
                    )
                });
            (eligible, rotation)
        };
        if !eligible {
            return;
        }
        let Ok(eye) = serde_json::from_value::<[f64; 3]>(context["eye"].clone()) else {
            return;
        };
        let Some(distance) = context["distance"].as_f64() else {
            return;
        };
        let (yaw, pitch) = rotation.map_or_else(
            || {
                (
                    context["yaw"].as_f64().unwrap_or(0.) as f32,
                    context["pitch"].as_f64().unwrap_or(0.) as f32,
                )
            },
            |r| (r.yaw, r.pitch),
        );
        let direction = host.client_particles_direction(yaw, pitch);
        self.state()
            .behaviors
            .borrow_mut()
            .particles
            .attack(wall(), eye, distance, direction);
    }
    pub(crate) fn particles(&self, host: &dyn Host, event: &mut ClientParticlesEvent) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut m = state.behaviors.borrow_mut();
        let active = m.query_running(&state.registry, "Particles");
        if !active || v["action"] == "reset" {
            m.particles.clear();
            return;
        }
        match v["action"].as_str() {
            Some("tick") => {
                if let Ok(camera) = serde_json::from_value::<[f64; 3]>(v["camera"].clone()) {
                    m.particles
                        .tick(wall(), camera, |p| !host.block_collision(p).is_empty());
                }
            }
            Some("frame") => {
                if let (Ok(right), Ok(up)) = (
                    serde_json::from_value::<[f32; 3]>(v["right"].clone()),
                    serde_json::from_value::<[f32; 3]>(v["up"].clone()),
                ) {
                    *event.geometry = m
                        .particles
                        .geometry(
                            wall(),
                            v["partial"].as_f64().unwrap_or(0.) as f32,
                            right,
                            up,
                        )
                        .to_string();
                }
            }
            _ => {}
        }
    }
}
