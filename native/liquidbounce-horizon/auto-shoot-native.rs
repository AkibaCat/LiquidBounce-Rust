// SPDX-License-Identifier: GPL-3.0-or-later
//! Real throwable slots and GENERIC projectile extrapolation for AutoShoot.
use super::{
    Game,
    auto_bow_math::{self as math, Trajectory, V},
};
use horizon_core::geometry::Vec3;
use serde_json::{Value, json};
impl Game<'_> {
    pub(super) fn auto_shoot_query(&mut self, text: &str) -> String {
        let Ok(q) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        if q["op"] == "point" || q["op"] == "angle" || q["op"] == "use" {
            return self.auto_rod_query(text);
        }
        if q["op"] == "facts" {
            let Ok(mut facts) = serde_json::from_str::<Value>(&self.auto_rod_query(text)) else {
                return "null".into();
            };
            if !facts.is_object() {
                return "null".into();
            }
            let slots = (0..9).chain(std::iter::once(40)).map(|slot| {
                let stack = self.stack_in(slot);
                let egg = stack.is_some_and(|s| Self::client_air_place_item_class(&s.item,"EggItem") || Self::client_air_place_item_class(&s.item,"SnowballItem"));
                json!({"item":stack.map_or("minecraft:air",|s|s.item.as_ref()),"count":stack.map_or(0,|s|s.count),"projectile":egg})
            }).collect::<Vec<_>>();
            facts["slots"] = json!(slots);
            return facts.to_string();
        }
        if q["op"] != "projectile" {
            return "null".into();
        }
        let Some(eye) = self.eye_position().map(|a| V(a[0], a[1], a[2])) else {
            return "null".into();
        };
        let entities = super::auto_bow::entities(self);
        let Some(e) = entities
            .iter()
            .find(|e| q["id"].as_i64() == Some(i64::from(e.id)))
        else {
            return "null".into();
        };
        let forecast = (e.weight == 0)
            .then(|| super::auto_bow_forecast::positions(self, e.id, 30))
            .flatten();
        let predicted = |t: f64| {
            if let Some(f) = forecast.as_ref() {
                let i = (t + 0.5).floor().clamp(0., 30.) as usize;
                let at = f[i.min(f.len() - 1)];
                V(at[0], at[1], at[2])
            } else {
                V(e.position.x, e.position.y, e.position.z)
                    .fma(t, V(e.velocity.x, e.velocity.y, e.velocity.z))
            }
        };
        let info = Trajectory {
            velocity: 1.5,
            gravity: 0.03,
            drag: 0.99,
            water_drag: 0.8,
            radius: 0.25,
            copies_velocity: true,
        };
        math::calculate(info, eye, e.dimensions, predicted, |from, to| {
            super::auto_bow_ray::clip(
                self,
                Vec3::new(from.0, from.1, from.2),
                Vec3::new(to.0, to.1, to.2),
                true,
            )
            .is_some_and(|hit| hit.0)
        })
        .map_or_else(|| "null".into(), |a| json!(a).to_string())
    }
}
