// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientJumpEffectEvent;
use serde_json::Value;
impl LiquidBounceHooks {
    pub(crate) fn jump_effect_tick(&self) {
        let s = self.state();
        let mut m = s.behaviors.borrow_mut();
        if m.query_running(&s.registry, "JumpEffect") {
            m.jump_effect.game_tick();
        }
    }
    pub(crate) fn jump_effect_jump(&self, position: [f64; 3]) {
        let s = self.state();
        let mut m = s.behaviors.borrow_mut();
        if m.query_running(&s.registry, "JumpEffect") {
            m.jump_effect.jump(position);
        }
    }
    pub(crate) fn jump_effect(&self, event: &mut ClientJumpEffectEvent) {
        let s = self.state();
        let m = s.behaviors.borrow();
        if !m.query_running(&s.registry, "JumpEffect") {
            return;
        }
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        *event.geometry = m
            .jump_effect
            .geometry(v["partial"].as_f64().unwrap_or(0.) as f32)
            .to_string();
    }
}
