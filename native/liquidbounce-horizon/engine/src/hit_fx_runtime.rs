// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientHitFxEvent, Host};
use liquidbounce_behaviors::{hit_fx, random::with_thread_random};
use serde_json::{Value, json};
impl LiquidBounceHooks {
    pub(crate) fn hit_fx_attack(&self, host: &dyn Host, id: i32, living: bool) {
        let Ok(context) = serde_json::from_str::<Value>(&host.client_hit_fx_context(id)) else {
            return;
        };
        let s = self.state();
        let mut m = s.behaviors.borrow_mut();
        if m.query_running(&s.registry, "HitFX") {
            m.hit_fx.attack(id, living, context["alive"] == true);
        }
    }
    pub(crate) fn hit_fx(&self, host: &dyn Host, event: &mut ClientHitFxEvent) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let target = {
            let s = self.state();
            let m = s.behaviors.borrow();
            if !m.query_running(&s.registry, "HitFX") {
                return;
            }
            m.hit_fx.last_target
        };
        if input["action"] == "self" {
            let s = self.state();
            let m = s.behaviors.borrow();
            let sound = with_thread_random(|r| hit_fx::sound(m.hit_fx.settings.self_sound, r));
            *event.output = json!({"sound":sound}).to_string();
            return;
        }
        let Some(target) = target else {
            return;
        };
        let Ok(context) = serde_json::from_str::<Value>(&host.client_hit_fx_context(target)) else {
            return;
        };
        let (Ok(position), Ok(at)) = (
            serde_json::from_value::<[f64; 3]>(context["position"].clone()),
            serde_json::from_value::<[f64; 3]>(input["at"].clone()),
        ) else {
            return;
        };
        let s = self.state();
        let mut m = s.behaviors.borrow_mut();
        if !m.query_running(&s.registry, "HitFX") {
            return;
        }
        let effect = with_thread_random(|r| {
            m.hit_fx.packet(
                input["name"].as_str().unwrap_or(""),
                input["source"].as_str().unwrap_or(""),
                target,
                context["alive"] == true,
                position,
                at,
                r,
            )
        });
        if let Some(e) = effect {
            *event.output =
                json!({"target":e.target,"particles":e.particles,"sound":e.sound}).to_string();
        }
    }
}
