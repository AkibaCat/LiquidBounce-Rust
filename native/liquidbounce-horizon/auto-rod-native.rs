// SPDX-License-Identifier: GPL-3.0-or-later
//! Fishing-hook, held-component and real living-entity facts for AutoRod.
use super::{
    Game, Hand, Screen,
    auto_bow_math::{self as math, B, Trajectory, V},
};
use horizon_core::geometry::Vec3;
use serde_json::{Value, json};
fn v(p: Vec3) -> V {
    V(p.x, p.y, p.z)
}
fn p(v: V) -> Vec3 {
    Vec3::new(v.0, v.1, v.2)
}
fn b(e: &super::auto_bow::Entity) -> B {
    B {
        lo: v(e.bounds.min_position()),
        hi: v(e.bounds.max_position()),
    }
}
impl Game<'_> {
    pub(super) fn auto_rod_animation(
        &self,
        hand: Hand,
    ) -> horizon_client::first_person::SwingAnimation {
        use horizon_client::first_person::SwingAnimation;
        let slot = if hand == Hand::Off {
            super::shield::OFFHAND_SLOT
        } else {
            self.client_hotbar_slot()
        };
        let Some(stack) = self.stack_in(slot) else {
            return SwingAnimation::DEFAULT;
        };
        let view = horizon_client::item_render::stack_view(stack);
        let components = self.pack.item_renderer.models.components_view(&view);
        let Some(a) = components.get("minecraft:interact_animation") else {
            return SwingAnimation::DEFAULT;
        };
        SwingAnimation {
            kind: match a["type"].as_str().unwrap_or("whack") {
                "none" => 0,
                "stab" => 2,
                _ => 1,
            },
            duration: a["duration"].as_i64().unwrap_or(6) as i32,
        }
    }
    pub(super) fn auto_rod_query(&mut self, text: &str) -> String {
        let Ok(q) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        if q["op"] == "release" {
            if self.remote.is_some() {
                self.release_remote_use();
            } else if let Some((slot, ticks)) = self.using.take()
                && let Some(stack) = self.stack_in(slot).cloned()
            {
                let hand = if slot == super::shield::OFFHAND_SLOT {
                    Hand::Off
                } else {
                    Hand::Main
                };
                self.with_hand(hand, |g| g.release_used_item(&stack, ticks, false));
            }
            return "true".into();
        }
        if q["op"] == "use" {
            let hand = q["hand"].as_i64().unwrap_or(-1) as i32;
            let yaw = q["rotation"][0].as_f64().unwrap_or(f64::from(self.yaw)) as f32;
            let pitch = q["rotation"][1].as_f64().unwrap_or(f64::from(self.pitch)) as f32;
            let previous = self.client_auto_rod_swing.replace(q["hideSwing"] == true);
            let used = self.native_client_use_item(hand, yaw, pitch);
            self.client_auto_rod_swing = previous;
            return json!(used).to_string();
        }
        if q["op"] == "angle" {
            return self.auto_bow_query(text);
        }
        let Some(player) = self.player.as_ref() else {
            return "null".into();
        };
        let at = player.position;
        let Some(eye) = self.eye_position().map(|a| V(a[0], a[1], a[2])) else {
            return "null".into();
        };
        if q["op"] == "slot" {
            let slot = q["slot"].as_u64().unwrap_or(255) as usize;
            return json!(
                self.stack_in(slot)
                    .is_some_and(|s| s.count > 0 && s.item.as_ref() == "minecraft:fishing_rod")
            )
            .to_string();
        }
        let hook = self.hook.as_ref().map(|h| h.auto_rod_facts());
        if q["op"] == "hook" {
            return json!(hook).to_string();
        }
        let entities = super::auto_bow::entities(self);
        if q["op"] == "distance" {
            let id = q["id"].as_i64().unwrap_or(i64::MIN);
            let position = entities
                .iter()
                .find(|e| i64::from(e.id) == id)
                .map(|e| e.position)
                .or_else(|| {
                    serde_json::from_value::<[f64; 3]>(q["position"].clone())
                        .ok()
                        .map(|a| Vec3::new(a[0], a[1], a[2]))
                });
            return position.map_or_else(
                || "null".into(),
                |p| json!({"distance":p.distance_squared(at),"position":[p.x,p.y,p.z]}).to_string(),
            );
        }
        if q["op"] == "point" || q["op"] == "projectile" {
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
                    v(e.position).fma(t, v(e.velocity))
                }
            };
            if q["op"] == "point" {
                let bounds = b(e)
                    .moved(predicted(1.).sub(v(e.position)))
                    .inflate(self.client_pick_margin(e.id, 0.));
                return json!({"eye":eye.array(),"bounds":[bounds.lo.0,bounds.lo.1,bounds.lo.2,bounds.hi.0,bounds.hi.1,bounds.hi.2]}).to_string();
            }
            let info = Trajectory {
                velocity: 1.5,
                gravity: 0.04,
                drag: 0.92,
                water_drag: 0.92,
                radius: 0.25,
                copies_velocity: true,
            };
            return math::calculate(info, eye, e.dimensions, predicted, |from, to| {
                super::auto_bow_ray::clip(self, p(from), p(to), true).is_some_and(|hit| hit.0)
            })
            .map_or_else(|| "null".into(), |a| json!(a).to_string());
        }
        if q["op"] != "facts" {
            return "null".into();
        }
        let view = v(Vec3::direction_from_rotation(player.pitch, player.yaw));
        let visible = super::criticals_target::candidates(self, f32::MAX, true);
        let candidates = entities.into_iter().map(|e|json!({"id":e.id,"target":e.target,"type":e.weight,"health":e.health,"hurt":e.hurt,"age":e.age,"distance":b(&e).distance(eye),"cosine":math::max_cos(eye,view,b(&e)),"rangeSquared":e.position.distance_squared(at),"position":[e.position.x,e.position.y,e.position.z],"visible":visible.iter().any(|a|a["id"].as_i64()==Some(i64::from(e.id)))})).collect::<Vec<_>>();
        let main = self.stack_in(self.client_hotbar_slot());
        let component = |stack: &horizon_client::items::ItemStack, name: &str| {
            self.pack
                .item_renderer
                .models
                .components_view(&horizon_client::item_render::stack_view(stack))
                .contains_key(name)
        };
        let weapon = main.is_some_and(|s| {
            self.items().item_in_tag("minecraft:swords", &s.item)
                // The pinned client speaks 26.3. Its legacy combat *pictures*
                // preference does not change the negotiated protocol version.
                || self.items().item_in_tag("minecraft:axes", &s.item)
                || Self::client_air_place_item_class(&s.item, "MaceItem")
                || s.enchantment_level("minecraft:knockback") > 0
        });
        let consumable = main
            .into_iter()
            .chain(self.offhand.as_ref())
            .any(|s| component(s, "minecraft:consumable"));
        let rods = (0..9)
            .chain(std::iter::once(40))
            .map(|slot| {
                self.stack_in(slot)
                    .is_some_and(|s| s.count > 0 && s.item.as_ref() == "minecraft:fishing_rod")
            })
            .collect::<Vec<_>>();
        let attack_key = self.settings.keys.get("key.attack");
        json!({"hook":hook,"rods":rods,"selected":self.client_hotbar_slot(),"health":player.health.health(),"mainItem":main.map_or("minecraft:air",|s|s.item.as_ref()),"weapon":weapon,"vanillaName":main.is_none_or(|s|!component(s,"minecraft:custom_name")),"notBreaking":!self.remote.as_ref().map_or(self.clicks.destroying.active,|r|r.is_destroying),"inventoryOpen":matches!(self.screen,Screen::Inventory|Screen::Creative),"using":self.using.is_some(),"consumable":consumable,"attackKey":attack_key,"candidates":candidates}).to_string()
    }
}
