// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual GodBridge and Breezily world queries and persistent technique state.
use super::{Game, block_placement as placement, scaffold_special_rules as rules};
use horizon_core::{geometry::trig::DEG_TO_RAD, jmath};
use serde_json::{Value, json};
use std::cell::RefCell;
thread_local! {
    static GOD_BRIDGE:RefCell<rules::GodBridge>=RefCell::new(Default::default());
    static BREEZILY:RefCell<rules::Breezily>=RefCell::new(Default::default());
}
fn target(value: &Value) -> Option<placement::Target> {
    Some(placement::Target {
        interacted: serde_json::from_value(value["interacted"].clone()).ok()?,
        placed: serde_json::from_value(value["placed"].clone()).ok()?,
        direction: serde_json::from_value(value["direction"].clone()).ok()?,
        point: serde_json::from_value(value["point"].clone()).ok()?,
        min_y: value["minY"].as_f64()?,
        rotation: serde_json::from_value(value["rotation"].clone()).ok()?,
    })
}
fn mode(value: &Value) -> Option<rules::LedgeMode> {
    rules::LedgeMode::ALL
        .into_iter()
        .find(|m| Some(m.tag()) == value.as_str())
}
fn action(a: rules::LedgeAction) -> String {
    json!({"jump":a.jump,"sneakTime":a.sneak_time,"stopInput":a.stop_input,"stepBack":a.step_back})
        .to_string()
}
impl Game<'_> {
    pub(super) fn scaffold_special_technique_query(&mut self, text: &str) -> Option<String> {
        let request: Value = serde_json::from_str(text).ok()?;
        match request["op"].as_str()? {
            "plan" | "crosshair" => self.scaffold_technique_query(text),
            "rotation" => {
                let raw: rules::Input = serde_json::from_value(request["rawInput"].clone()).ok()?;
                if raw == [false; 4] {
                    return Some(
                        json!(rules::no_input_rotation(
                            target(&request["target"]).map(|t| t.rotation)
                        ))
                        .to_string(),
                    );
                }
                let p = self.player.as_ref()?;
                let moving_yaw =
                    rules::moving_yaw(super::scaffold_planner_rules::movement_angle(p.yaw, raw));
                if request["technique"] == "Breezily" {
                    return Some(json!(rules::Breezily::rotation(moving_yaw)).to_string());
                }
                if request["technique"] != "GodBridge" {
                    return None;
                }
                let position = [p.position.x, p.position.y, p.position.z];
                let on_ground = p.ground.on_ground;
                let radians = moving_yaw * DEG_TO_RAD;
                let cos_sin = [
                    jmath::cos(f64::from(radians)) as f32,
                    jmath::sin(f64::from(radians)) as f32,
                ];
                let below = position.map(|v| v.floor() as i32);
                let relative = rules::cardinal_offset(f64::from(moving_yaw));
                let ahead = std::array::from_fn::<_, 3, _>(|i| {
                    (position[i] + f64::from(relative[i]) * 0.6).floor() as i32
                });
                let is_air = |at: [i32; 3]| {
                    self.res.facts().is_air(self.world.block(
                        at[0],
                        at[1].wrapping_sub(1),
                        at[2],
                    ))
                };
                let rotation = GOD_BRIDGE.with_borrow_mut(|s| {
                    s.rotation(
                        moving_yaw,
                        on_ground,
                        position,
                        cos_sin,
                        is_air(below),
                        is_air(ahead),
                    )
                });
                Some(json!(rotation).to_string())
            }
            "breezilyInput" => {
                let input: rules::Input = serde_json::from_value(request["input"].clone()).ok()?;
                let distance: [f32; 2] =
                    serde_json::from_value(request["edgeDistance"].clone()).ok()?;
                let p = self.player.as_ref()?;
                let at = p.position;
                let below = self.world.block(
                    at.x.floor() as i32,
                    (at.y.floor() as i32).wrapping_sub(1),
                    at.z.floor() as i32,
                );
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_millis() as i64;
                let input = BREEZILY.with_borrow_mut(|s| {
                    s.input(
                        input,
                        p.shift_key_down,
                        self.res.facts().is_air(below),
                        now,
                        [at.x, at.y, at.z],
                        p.yaw,
                        || {
                            if distance[0] >= distance[1] {
                                distance[0]
                            } else {
                                super::edge_random::with_thread_random(|r| {
                                    r.next_f32_between(distance[0], distance[1].next_up())
                                })
                            }
                        },
                    )
                });
                Some(json!(input).to_string())
            }
            "godbridgeLedge" => {
                let input: rules::Input = serde_json::from_value(request["input"].clone()).ok()?;
                let p = self.player.as_ref()?;
                let eye_height = f64::from(p.dimensions.eye_height);
                let raw = horizon_client::reverse_step_bridge::keys();
                let keys = horizon_client::sprint::Keys {
                    forward: input[0],
                    backward: input[1],
                    left: input[2],
                    right: input[3],
                    jump: raw[4],
                    shift: p.shift_key_down,
                    sprint: p.sprinting || raw[6],
                    analog: None,
                };
                let mut snapshot = None;
                // This Game helper runs before Horizon's native base tick. It must
                // advance that base once; the partial-tick replay API is reserved
                // for callers already inside the native aiStep phase.
                super::falling_forecast::visit_with_keys_probe(
                    self,
                    1,
                    0.0,
                    keys,
                    false,
                    |body, _, backed| {
                        snapshot = Some((body.position, backed));
                        true
                    },
                )
                .ok()?;
                let (position, backed) = snapshot?;
                if !backed {
                    return Some(action(Default::default()));
                }
                let Some(target) = target(&request["target"]) else {
                    return Some(action(Default::default()));
                };
                let rotation: [f32; 2] =
                    serde_json::from_value(request["rotation"].clone()).ok()?;
                let range = self
                    .interaction_range("minecraft:block_interaction_range", 4.5, 5.0)
                    .max(self.interaction_range("minecraft:entity_interaction_range", 3.0, 5.0));
                let start = [position.x, position.y + eye_height, position.z];
                if let Some(click) = placement::trace_from_point(self, start, rotation, range) {
                    let h = click.hit;
                    let eye = self.eye_position()?;
                    let minimum = request["minDist"].as_f64()? as f32;
                    let valid = if h.face[1] != 0 {
                        true
                    } else {
                        let axis = if h.face[2] != 0 { 2 } else { 0 };
                        (h.point[axis] - eye[axis]).abs() >= f64::from(minimum)
                    };
                    if target.matches(true, h.pos, h.face, h.point) && valid {
                        return Some(action(Default::default()));
                    }
                }
                let power = self.scaffold_with_motion(|motion| {
                    horizon_world::living_motion::jump_power(motion, 1.0)
                        .expect("GodBridge jump power")
                })?;
                let block_count = i32::try_from(request["blockCount"].as_i64()?).ok()?;
                let force = i32::try_from(request["forceSneakBelowCount"].as_i64()?).ok()?;
                let mut modes = request["modes"]
                    .as_array()?
                    .iter()
                    .map(mode)
                    .collect::<Option<Vec<_>>>()?;
                modes.sort_by_key(|m| rules::LedgeMode::ALL.iter().position(|v| v == m).unwrap());
                modes.dedup();
                if modes.is_empty() {
                    return None;
                }
                let sneak: [i32; 2] = serde_json::from_value(request["sneakTime"].clone()).ok()?;
                if sneak[0] > sneak[1] {
                    return None;
                }
                let mode = rules::ledge_mode(block_count, force, &modes, power, |count| {
                    super::edge_random::with_thread_random(|r| {
                        r.next_i32_bound(count as i32) as usize
                    })
                });
                Some(action(rules::action(mode, || {
                    super::edge_random::random_i32_inclusive(sneak[0], sneak[1])
                })))
            }
            _ => None,
        }
    }
}
