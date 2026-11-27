// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientTotemEffectEvent, Host};
use liquidbounce_behaviors::{targeting::VisualCamera, totem_effect::Snapshot};
use serde_json::Value;
impl LiquidBounceHooks {
    pub(crate) fn totem_effect_tick(&self) {
        let s = self.state();
        let mut m = s.behaviors.borrow_mut();
        if m.query_running(&s.registry, "TotemEffect") {
            m.totem_effect.game_tick();
        }
    }
    pub(crate) fn totem_effect(&self, _host: &dyn Host, event: &mut ClientTotemEffectEvent) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let snapshot = crate::targeting::snapshot(&v["target"].to_string());
        let mut state = self.state();
        if !state
            .behaviors
            .borrow()
            .query_running(&state.registry, "TotemEffect")
        {
            return;
        }
        if v["action"] == "pop" {
            let Some(id) = v["id"].as_i64().map(|n| n as i32) else {
                return;
            };
            let Some(snapshot) = snapshot else {
                return;
            };
            let free_look = {
                let m = state.behaviors.borrow();
                m.query_running(&state.registry, "FreeLook")
            };
            let info = state.target_info(id, &snapshot);
            if !state.global_targets.visual.should_show(
                &snapshot.entity,
                info,
                VisualCamera {
                    raw_first_person: v["firstPerson"].as_bool().unwrap_or(true),
                    free_cam: false,
                    free_look,
                },
            ) {
                return;
            }
            let Ok(position) = serde_json::from_value::<[f64; 3]>(v["position"].clone()) else {
                return;
            };
            let (Some(x), Some(y), Some(height)) =
                (v["xRot"].as_f64(), v["yRot"].as_f64(), v["height"].as_f64())
            else {
                return;
            };
            state.behaviors.borrow_mut().totem_effect.pop(Snapshot {
                position,
                x_rot: x as f32,
                y_rot: y as f32,
                height: height as f32,
            });
        } else if v["action"] == "frame" {
            *event.geometry = state
                .behaviors
                .borrow()
                .totem_effect
                .geometry(v["partial"].as_f64().unwrap_or(0.) as f32)
                .to_string();
        }
    }
}
