// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientTargetRendererEvent, Host};
use liquidbounce_behaviors::target_renderer::{Camera, Entity, Frame};
use serde_json::{Value, json};
fn entity(input: &Value, id: i32) -> Option<Entity> {
    let e = input["entities"]
        .as_array()?
        .iter()
        .find(|e| e["id"].as_i64() == Some(i64::from(id)))?;
    Some(Entity {
        id,
        position: serde_json::from_value(e["position"].clone()).ok()?,
        interpolated: serde_json::from_value(e["interpolated"].clone()).ok()?,
        height: e["height"].as_f64()? as f32,
        health: e["health"].as_f64().unwrap_or(0.) as f32,
        max_health: e["maxHealth"].as_f64().unwrap_or(1.) as f32,
        absorption: e["absorption"].as_f64().unwrap_or(0.) as f32,
        hurt_time: e["hurt"].as_i64().unwrap_or(0) as i32,
        tick_count: e["age"].as_i64().unwrap_or(0) as i32,
        living: e["living"].as_bool().unwrap_or(false),
    })
}
fn camera(input: &Value) -> Option<Camera> {
    Some(Camera {
        partial: input["partial"].as_f64()? as f32,
        right: serde_json::from_value(input["right"].clone()).ok()?,
        up: serde_json::from_value(input["up"].clone()).ok()?,
        yaw: input["yaw"].as_f64()? as f32,
        pitch: input["pitch"].as_f64()? as f32,
    })
}
impl LiquidBounceHooks {
    pub(crate) fn target_renderer(
        &self,
        _host: &dyn Host,
        event: &mut ClientTargetRendererEvent<'_>,
    ) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut m = state.behaviors.borrow_mut();
        let targets_only = input["phase"] == "targets";
        let c = camera(&input);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis().min(u64::MAX as u128) as u64);
        let elapsed = self.started.elapsed().as_millis().min(u64::MAX as u128) as u64;
        let mut ids = Vec::new();
        let mut frame = Frame::default();
        macro_rules! owner {
            ($module:literal, $field:ident, $extra:expr) => {{
                let running = m.query_running(&state.registry, $module) && $extra;
                let owner = &mut m.$field;
                if running && owner.target_renderer.boolean("Enabled") {
                    if let Some(target) = owner.behavior.target {
                        if targets_only {
                            ids.push(target);
                        } else if let (Some(e), Some(c)) = (entity(&input, target), c.as_ref()) {
                            let rendered = owner.target_render_runtime.render(
                                &owner.target_renderer,
                                &e,
                                c,
                                now,
                                elapsed,
                            );
                            frame.quads.extend(rendered.quads);
                            frame.overlay.extend(rendered.overlay);
                        }
                    }
                }
            }};
        }
        owner!("AutoBow", auto_bow, m.auto_bow.behavior.settings.aimbot);
        owner!("AutoRod", auto_rod, true);
        owner!("Aimbot", aimbot, true);
        owner!("AutoShoot", auto_shoot, true);
        owner!("KillAura",kill_aura,!(m.query_running(&state.registry,"ElytraTarget")&&m.elytra_target.target_renderer.boolean("Enabled")&&m.kill_aura.behavior.target==m.elytra_target.behavior.target));
        owner!("ElytraTarget",elytra_target,true);
        // TARGET_RENDERER_OWNERS_END
        drop(m);drop(state);
        if !targets_only{self.kill_aura_render(&input,&mut frame);}
        if targets_only {
            ids.sort_unstable();
            ids.dedup();
            *event.geometry = json!(ids).to_string();
        } else {
            *event.geometry = frame.json().to_string();
        }
    }
}
