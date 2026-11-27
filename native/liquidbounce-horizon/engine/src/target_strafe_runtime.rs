// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientDirectionalInputEvent, Host};
use liquidbounce_behaviors::{
    auto_bow::{self, Candidate},
    target_strafe::Settings,
};
use serde_json::{Value, json};
#[derive(Default)]
pub struct TargetStrafeRuntime {
    pub settings: Settings,
    pub input_from_features: bool,
}
fn query(host: &mut dyn Host, value: Value) -> Value {
    serde_json::from_str(&host.client_target_strafe_query(&value.to_string()))
        .unwrap_or(Value::Null)
}
impl LiquidBounceHooks {
    pub(crate) fn target_strafe_input(
        &self,
        host: &mut dyn Host,
        event: &mut ClientDirectionalInputEvent,
    ) {
        let (settings, running, speed, killaura, locked, features) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            (
                b.target_strafe.settings.clone(),
                b.query_running(&state.registry, "TargetStrafe"),
                b.query_running(&state.registry, "Speed"),
                b.query_running(&state.registry, "KillAura"),
                b.kill_aura.behavior.target.or(b.aimbot.behavior.target),
                b.target_strafe.input_from_features,
            )
        };
        if !running {
            query(host, json!({"op":"configure","running":false}));
            return;
        }
        let facts = query(host, json!({"op":"facts"}));
        let met = settings.requirements_met([
            facts["jump"] == true,
            speed,
            killaura,
            facts["ground"] == true,
        ]);
        let mut candidates = vec![];
        if locked.is_none() {
            for row in facts["candidates"].as_array().into_iter().flatten() {
                let Some(id) = row["id"].as_i64().map(|i| i as i32) else {
                    continue;
                };
                let target = if row["target"].is_object() {
                    row["target"].to_string()
                } else {
                    host.client_targeting_entity(id)
                };
                let Some(t) = crate::targeting::snapshot(&target) else {
                    continue;
                };
                if !t.entity.kind.living {
                    continue;
                }
                let mut state = self.state();
                let info = state.target_info(id, &t);
                if !state
                    .global_targets
                    .combat
                    .should_be_attacked(&t.entity, info)
                {
                    continue;
                }
                let distance = row["distance"].as_f64().unwrap_or(f64::INFINITY);
                if !(distance <= f64::from(settings.range * settings.range)) {
                    continue;
                }
                candidates.push(Candidate {
                    id,
                    type_weight: row["type"].as_i64().unwrap_or(i64::from(i32::MAX)) as i32,
                    health: row["health"].as_f64().unwrap_or(0.) as f32,
                    distance,
                    cosine: row["cosine"].as_f64().unwrap_or(-1.),
                    hurt: row["hurt"].as_i64().unwrap_or(0) as i32,
                    age: row["age"].as_i64().unwrap_or(0) as i32,
                });
            }
        }
        let selected = locked.or_else(|| {
            auto_bow::targets(
                &auto_bow::Settings {
                    fov: settings.fov,
                    hurt_time: settings.hurt_time,
                    priority: settings
                        .priority
                        .iter()
                        .map(|i| auto_bow::PRIORITIES[*i])
                        .collect(),
                    ..Default::default()
                },
                candidates,
            )
            .first()
            .map(|c| c.id)
        });
        let input = u8::from(event.forward)
            | u8::from(event.backward) << 1
            | u8::from(event.left) << 2
            | u8::from(event.right) << 3;
        let result = query(
            host,
            json!({"op":"configure","running":met,"settings":settings.wire(),"target":selected,"input":input,"features":features,"speed":speed}),
        );
        if let Some(input) = result["input"].as_u64() {
            let input = input as u8;
            event.forward = input & 1 != 0;
            event.backward = input & 2 != 0;
            event.left = input & 4 != 0;
            event.right = input & 8 != 0;
        }
    }
}
