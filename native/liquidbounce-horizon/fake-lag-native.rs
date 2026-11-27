// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game, Screen};
use horizon_core::geometry::{Aabb, Vec3};
use serde_json::{Value, json};
fn cube(at: Vec3, r: f64) -> Aabb {
    Aabb::new(
        Vec3::new(at.x - r, at.y - r, at.z - r),
        Vec3::new(at.x + r, at.y + r, at.z + r),
    )
}
impl Game<'_> {
    pub(super) fn fake_lag_query(&self, text: &str) -> String {
        let Ok(q) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        let Some(p) = self.player.as_ref() else {
            return "null".into();
        };
        let old = serde_json::from_value::<[f64; 3]>(q["old"].clone())
            .ok()
            .map(|a| Vec3::new(a[0], a[1], a[2]));
        let range: [f32; 2] = serde_json::from_value(q["range"].clone()).unwrap_or([2., 5.]);
        let eye = Vec3::new(
            p.position.x,
            p.position.y + f64::from(p.dimensions.eye_height),
            p.position.z,
        );
        let radius = f64::from(range[1]);
        let high = f64::from(range[1] * range[1]);
        let low = f64::from(range[0] * range[0]);
        let all = super::auto_bow::entities(self);
        let targets = super::criticals_target::candidates(self, f32::MAX, false);
        let entities = all
            .into_iter()
            .filter_map(|e| {
                let target = targets
                    .iter()
                    .find(|t| t["id"].as_i64() == Some(i64::from(e.id)))?["target"]
                    .clone();
                let nearby = cube(eye, radius).intersects(e.bounds)
                    && (low..=high).contains(&e.bounds.distance_squared_to_point(eye));
                let in_range = old.is_some_and(|at| {
                    cube(at, radius).intersects(e.bounds)
                        && e.bounds.distance_squared_to_point(at) <= radius * radius
                });
                Some(
                    json!({"id":e.id,"target":target,"nearby":nearby,"inRange":in_range,
                "intersects":old.is_some_and(|at|e.bounds.intersects(p.dimensions.bounds(at))),
                "serverDistance":old.map(|at|e.position.distance_squared(at).sqrt()),
                "clientDistance":e.position.distance_squared(p.position).sqrt()}),
                )
            })
            .collect::<Vec<_>>();
        let using_consumable = self
            .using
            .and_then(|(slot, _)| self.stack_in(slot))
            .is_some_and(|s| {
                self.pack
                    .item_renderer
                    .models
                    .components_view(&horizon_client::item_render::stack_view(s))
                    .contains_key("minecraft:consumable")
            });
        let screen = self.screen != Screen::Playing
            || self.title.is_some()
            || self.dialog_open()
            || self.book.is_some()
            || self.sign_edit.is_some()
            || self
                .remote
                .as_ref()
                .is_some_and(|r| r.level_load.is_active());
        json!({"dead":p.health.dead_or_dying(),"water":p.fluids.was_touching_water,"screen":screen,"usingConsumable":using_consumable,"entities":entities}).to_string()
    }
}
