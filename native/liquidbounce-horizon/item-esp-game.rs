// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{arrows::Pickup, projectiles::ProjectileKind};
use horizon_core::geometry::Vec3;
use serde_json::{Value, json};
thread_local! {static CAPTURED:std::cell::RefCell<Vec<Value>>=const{std::cell::RefCell::new(Vec::new())};}
pub(super) fn clear() {
    CAPTURED.with_borrow_mut(Vec::clear);
    horizon_client::gfx::item_esp::begin_frame("null");
}
fn entry(
    id: u32,
    identity: u64,
    kind: &str,
    p: Vec3,
    previous: Vec3,
    height: f64,
    eye_height: f64,
    alpha: f64,
) -> Value {
    let at = [p.x, p.y, p.z];
    let old = [previous.x, previous.y, previous.z];
    json!({"id":id as i32,"identity":identity,"kind":kind,"position":at,"previous":old,"interpolated":std::array::from_fn::<_,3,_>(|i|old[i]+(at[i]-old[i])*alpha),"eye":[p.x,p.y+eye_height,p.z],"height":height})
}
impl Game<'_> {
    pub(super) fn client_item_esp_entities(&self, alpha: f64) -> Vec<Value> {
        let mut entries = vec![];
        for item in self.dropped.iter() {
            let mut v = entry(
                item.id,
                (item.join.get() << 2) | 0,
                "item",
                item.position,
                item.previous,
                0.25,
                0.2125,
                alpha,
            );
            v["item"] = json!(item.stack.item.as_ref());
            entries.push(v);
        }
        for a in self.creatures.projectiles.iter() {
            let kind = if a.is_spectral_arrow() {
                "spectral"
            } else if a.is_arrow() {
                "arrow"
            } else if matches!(a.kind, ProjectileKind::Trident) {
                "trident"
            } else {
                continue;
            };
            let mut v = entry(
                a.id,
                (a.join.get() << 2) | 1,
                kind,
                a.position,
                a.previous,
                0.5,
                0.13,
                alpha,
            );
            v["allowed"] = json!(
                a.arrow
                    .as_ref()
                    .is_some_and(|s| s.pickup == Pickup::Allowed)
            );
            v["effectColor"] = json!(a.arrow.as_ref().map_or(-1, |s| s.effect_color));
            entries.push(v);
        }
        for t in self.tridents.iter() {
            entries.push(entry(
                t.id,
                (t.join.get() << 2) | 2,
                "trident",
                t.position,
                t.previous,
                0.5,
                0.13,
                alpha,
            ));
        }
        entries.sort_by_key(|e| e["identity"].as_u64().unwrap_or(0) >> 2);
        entries
    }
    pub(super) fn client_item_esp_tick(&self) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let Some(camera) = self.native_camera() else {
            return;
        };
        let entries = self.client_item_esp_entities(1.);
        let mut output = String::new();
        self.native_client_item_esp(
            &json!({"tick":true,"camera":camera.position,"entities":entries}).to_string(),
            &mut output,
        );
        let Ok(v) = serde_json::from_str::<Value>(&output) else {
            return;
        };
        CAPTURED.with_borrow_mut(|captured| {
            *captured = entries
                .into_iter()
                .filter(|e| {
                    v["selected"]
                        .as_array()
                        .is_some_and(|ids| ids.contains(&e["identity"]))
                })
                .collect();
        });
    }
    /// Called after OutlineGlow.begin_frame and before extracting model geometry.
    pub(super) fn client_item_esp_frame(&self, alpha: f64, camera: glam::DVec3) {
        horizon_client::gfx::item_esp::begin_frame("null");
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let entries = self.client_item_esp_entities(alpha);
        let captured = CAPTURED.with_borrow_mut(|captured| {
            for e in captured.iter_mut() {
                if let Some(live) = entries.iter().find(|v| v["identity"] == e["identity"]) {
                    *e = live.clone();
                } else {
                    let p = serde_json::from_value::<[f64; 3]>(e["position"].clone())
                        .unwrap_or([0.; 3]);
                    let old =
                        serde_json::from_value::<[f64; 3]>(e["previous"].clone()).unwrap_or(p);
                    e["interpolated"] = json!(std::array::from_fn::<_, 3, _>(
                        |i| old[i] + (p[i] - old[i]) * alpha
                    ));
                }
            }
            captured.clone()
        });
        let orientation = self.camera_orientation();
        let eye = (orientation * -glam::Vec3::Z).as_dvec3().to_array();
        let yaw = self.native_camera().map_or(self.yaw, |c| c.yaw);
        let input = json!({"tick":false,"camera":camera.to_array(),"eyeVector":eye,"yaw":yaw,"entities":entries,"captured":captured});
        let mut output = String::new();
        self.native_client_item_esp(&input.to_string(), &mut output);
        if let Ok(v) = serde_json::from_str::<Value>(&output) {
            horizon_client::gfx::outline_glow::submit(
                horizon_client::gfx::outline_glow::Owner::ItemEsp,
                &v["glow"].to_string(),
            );
            horizon_client::gfx::item_esp::begin_frame(&v["plain"].to_string());
            CAPTURED.with_borrow_mut(|c| {
                c.retain(|e| {
                    v["selected"]
                        .as_array()
                        .is_some_and(|ids| ids.contains(&e["identity"]))
                })
            });
        }
    }
}
