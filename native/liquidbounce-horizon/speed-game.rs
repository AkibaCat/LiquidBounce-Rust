// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::actions::Action;
use serde_json::{Value, json};

fn bits(v: [f32; 2]) -> u8 {
    let mut out = 0;
    if v[1] > 0.01 { out |= 1; }
    if v[1] < -0.01 { out |= 2; }
    if v[0] < -0.01 { out |= 4; }
    if v[0] > 0.01 { out |= 8; }
    out
}

impl Game<'_> {
    /// Facts are sampled only from the real client window and local player.
    pub(super) fn client_speed_facts(&self) -> String {
        if !self.native_window() || !self.native_client_slot_owner() {
            return json!({"inGame":false}).to_string();
        }
        let Some(p) = self.player.as_ref().filter(|p| p.native_client_queries_allowed()) else {
            return json!({"inGame":false}).to_string();
        };
        json!({
            "inGame": true,
            "ground": p.ground.on_ground,
            "velocity": [p.velocity.x, p.velocity.y, p.velocity.z],
            "yaw": p.yaw,
            "input": bits(p.applied_input),
            "physicalJump": self.keys.held(Action::Jump),
            "sprinting": p.sprinting,
            "sneaking": p.local_crouching || self.keys.held(Action::Sneak),
            "usingItem": p.item_use_speed.is_some(),
            "fallFlying": p.fall_flying,
            "inLiquid": p.fluids.was_touching_water || p.is_in_lava(),
            "fly": p.flying,
            "towering": false,
            "hurt": p.health.tick_count,
            "fallDistance": p.fall_distance,
            "flyDistance": 0.0,
            "horizontalCollision": p.collision.horizontal,
            "bottomCollision": p.collision.below,
            "groundWithin": p.ground.on_ground,
            "jumpBlocked": p.collision.vertical,
            "leatherBoots": p.leather_boots,
            "collisions": [0, 0, 0],
            "now": p.health.tick_count as i64,
        }).to_string()
    }

    /// Apply only movement effects produced for this local client tick.
    pub(super) fn client_speed_apply(&mut self, text: &str) -> String {
        if !self.native_window() || !self.native_client_slot_owner() { return "{}".into(); }
        let Ok(value) = serde_json::from_str::<Value>(text) else { return "{}".into(); };
        let Some(p) = self.player.as_mut().filter(|p| p.native_client_queries_allowed()) else {
            return "{}".into();
        };
        if let Some(v) = value["velocity"].as_array() {
            if v.len() >= 3 {
                p.velocity.x = v[0].as_f64().unwrap_or(p.velocity.x);
                p.velocity.y = v[1].as_f64().unwrap_or(p.velocity.y);
                p.velocity.z = v[2].as_f64().unwrap_or(p.velocity.z);
            }
        }
        if value["cancel"].as_bool().unwrap_or(false) { p.velocity.y = 0.0; }
        if let Some(power) = value["jumpPower"].as_f64() { p.velocity.y = power; }
        json!({"applied":true,"velocity":[p.velocity.x,p.velocity.y,p.velocity.z]}).to_string()
    }
}
