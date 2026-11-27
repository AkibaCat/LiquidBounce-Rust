// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientPotionFxEvent, Host};
use liquidbounce_behaviors::potion_fx::{Cloud, Player};
use serde_json::Value;
impl LiquidBounceHooks {
    pub(crate) fn potion_fx_tick(&self) {
        let s = self.state();
        let mut m = s.behaviors.borrow_mut();
        if m.query_running(&s.registry, "PotionFX") {
            m.potion_fx.game_tick();
        }
    }
    pub(crate) fn potion_fx(&self, _: &dyn Host, e: &mut ClientPotionFxEvent) {
        let Ok(v) = serde_json::from_str::<Value>(e.input) else {
            return;
        };
        let s = self.state();
        let mut m = s.behaviors.borrow_mut();
        if !m.query_running(&s.registry, "PotionFX") {
            return;
        }
        if v["action"] == "splash" {
            if let (Some(color), Ok(position)) = (
                v["color"].as_i64(),
                serde_json::from_value::<[f64; 3]>(v["position"].clone()),
            ) {
                m.potion_fx.splash(color as i32, position);
            }
        } else if v["action"] == "frame" {
            let players = v["players"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Player::from_json)
                .collect::<Vec<_>>();
            let clouds = v["clouds"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Cloud::from_json)
                .collect::<Vec<_>>();
            let right = serde_json::from_value(v["right"].clone()).unwrap_or([1., 0., 0.]);
            let up = serde_json::from_value(v["up"].clone()).unwrap_or([0., 1., 0.]);
            *e.geometry = m
                .potion_fx
                .geometry(
                    v["partial"].as_f64().unwrap_or(0.) as f32,
                    &players,
                    &clouds,
                    right,
                    up,
                )
                .to_string();
        }
    }
}
