// SPDX-License-Identifier: GPL-3.0-or-later
//! AutoMobHeal reads the actual render mobs, stack components and client result.
use super::{Game, Hand, InteractTarget, Screen};
use horizon_client::items::ItemStack;
use horizon_core::geometry::Vec3;
use serde_json::{Value, json};
fn kind(id: &str) -> Option<&'static str> {
    Some(match id {
        "minecraft:iron_golem" => "IronGolem",
        "minecraft:wolf" => "Wolf",
        "minecraft:cat" => "Cat",
        "minecraft:horse" | "minecraft:donkey" | "minecraft:mule" => "HorseFamily",
        "minecraft:zombie_horse" => "ZombieHorse",
        "minecraft:llama" | "minecraft:trader_llama" => "Llama",
        "minecraft:camel" => "Camel",
        "minecraft:camel_husk" => "CamelHusk",
        "minecraft:nautilus" | "minecraft:zombie_nautilus" => "Nautilus",
        _ => return None,
    })
}
impl Game<'_> {
    pub(super) fn auto_mob_heal_query(&mut self, text: &str) -> String {
        let Ok(q) = serde_json::from_str::<Value>(text) else { return "null".into(); };
        if q["op"] == "interact" {
            let Some(id) = q["id"].as_i64().and_then(|id| i32::try_from(id).ok()) else { return "false".into(); };
            let Some(i) = self.mobs.iter().position(|m| m.id as i32 == id && !m.removed) else { return "false".into(); };
            let at = self.mobs[i].position;
            if !self.border.contains(at.x.floor(), at.z.floor()) { return "false".into(); }
            let hand = if q["offhand"] == true { Hand::Off } else { Hand::Main };
            self.remote_send_carried();
            let hide = q["swingMode"] == "HideForClient";
            // Interaction swings are client animation only; no Punch packet.
            let consumed = if self.remote.is_some() {
                self.send(horizon_protocol::packets::play::Serverbound::Interact(horizon_protocol::packets::play::Interact {
                    id, hand: if hand == Hand::Off { 1 } else { 0 }, location: [0.; 3], secondary: self.remote_shift_down(),
                }));
                let takes = self.remote_mob_takes(i, hand);
                if takes {
                    self.remote_feed_particles(i, hand);
                    if !hide && self.remote_mob_swings(i, hand) {
                        self.swing_hand(hand, self.auto_rod_animation(hand));
                    }
                }
                takes
            } else {
                let previous = self.client_auto_rod_swing.replace(hide);
                let result = self.with_hand(hand, |g| g.interact_entity(Some(InteractTarget { id, location: [0.; 3] }))).unwrap_or(false);
                self.client_auto_rod_swing = previous;
                result
            };
            return json!(consumed).to_string();
        }
        if q["op"] != "facts" { return "null".into(); }
        let entity_range = self.interaction_range("minecraft:entity_interaction_range", 3., 5.);
        let Some(eye) = self.eye_position().map(|p| Vec3::new(p[0], p[1], p[2])) else { return "null".into(); };
        let owner = self.acting_uuid();
        let entities = self.mobs.iter().filter(|m| !m.removed).filter_map(|m| {
            let kind = kind(&m.kind.id)?;
            let mut armor = m.extra.body.as_ref().map(|id| ItemStack::new(id.as_ref(), 1));
            if let Some(armor) = armor.as_mut() {
                armor.damage = m.extra.body_damage.max(0) as u32;
                armor.extra = m.extra.body_extra.clone();
            }
            let slots = std::iter::once(40).chain(0..9).filter_map(|index| {
                let stack = self.stack_in(index).filter(|s| s.count > 0 && &*s.item != "minecraft:air")?;
                let view = horizon_client::item_render::stack_view(stack);
                let components = self.pack.item_renderer.models.components_view(&view);
                let repairs = armor.as_ref().is_some_and(|a| horizon_client::stations::is_valid_repair_item(&self.stations().enchantments, self.items(), a, stack));
                Some(json!({"index":index,"item":stack.item.as_ref(),"nutrition":components.get("minecraft:food").and_then(|f| f["nutrition"].as_i64()),"wolfFood":self.items().item_in_tag("minecraft:wolf_food", &stack.item),"catFood":self.items().item_in_tag("minecraft:cat_food", &stack.item),"repairsArmor":repairs}))
            }).collect::<Vec<_>>();
            let target_eye = Vec3::new(m.position.x, m.position.y + m.eye_height(), m.position.z);
            let visible = super::spear_kill_ray::clip(self, eye, target_eye).is_some_and(|hit| hit.0);
            Some(json!({"id":m.id,"kind":kind,"alive":m.alive(),"visible":visible,"health":m.health,"maxHealth":m.attrs.max_health,"distanceSquared":m.bounds().distance_squared_to_point(eye),"baby":m.is_baby(),"tame":m.extra.tame,"owned":m.extra.owner==Some(owner),"sitting":m.extra.sitting_pose,"wearingBodyArmor":armor.is_some(),"armorEmpty":armor.as_ref().is_none_or(|a| a.count==0||&*a.item=="minecraft:air"),"armorMaxDamage":armor.as_ref().map_or(0,|a| self.items().stack_max_damage(a)),"armorDamage":m.extra.body_damage,"slots":slots}))
        }).collect::<Vec<_>>();
        json!({"entities":entities,"selected":self.client_hotbar_slot(),"entityRange":entity_range,"secondaryUse":self.player.as_ref().is_some_and(|p| p.shift_key_down),"usingItem":self.using.is_some(),"screenOpen":self.screen!=Screen::Playing}).to_string()
    }
}
