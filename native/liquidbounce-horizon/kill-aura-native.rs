// SPDX-License-Identifier: GPL-3.0-or-later
//! KillAura's live entity, ray, held-item and game-mode action boundary.
use super::{
    Game,
    auto_bow_math::{B, V},
};
use horizon_client::{first_person::Hand, mod_api::ServerboundPlayPacket as S};
use horizon_core::geometry::{Aabb, Vec3};
use horizon_protocol::packets::play;
use horizon_world::world_ray::{RayBlockMode, RayFluidMode};
use serde_json::{Value, json};
fn v(p: Vec3) -> V {
    V(p.x, p.y, p.z)
}
fn p(v: V) -> Vec3 {
    Vec3::new(v.0, v.1, v.2)
}
fn bounds(b: Aabb) -> B {
    B {
        lo: v(b.min_position()),
        hi: v(b.max_position()),
    }
}
fn outline(g: &Game<'_>, a: V, b: V) -> bool {
    super::client_ghost_hand::clip(
        g,
        &mut super::no_web_ray::NoWebRay(g),
        p(a),
        p(b),
        RayBlockMode::Outline,
        RayFluidMode::None,
        4096,
    )
    .is_ok_and(|h| h.miss)
}
fn angles(g: &Game<'_>, q: &Value) -> (f32, f32) {
    (
        q["rotation"][0].as_f64().unwrap_or(f64::from(g.yaw)) as f32,
        q["rotation"][1].as_f64().unwrap_or(f64::from(g.pitch)) as f32,
    )
}
impl Game<'_> {
    fn kill_aura_block_animation(&self, stack: &horizon_client::items::ItemStack) -> bool {
        let view = horizon_client::item_render::stack_view(stack);
        let components = self.pack.item_renderer.models.components_view(&view);
        let generic = match &*stack.item {
            "minecraft:bow"
            | "minecraft:bundle"
            | "minecraft:white_bundle"
            | "minecraft:orange_bundle"
            | "minecraft:magenta_bundle"
            | "minecraft:light_blue_bundle"
            | "minecraft:yellow_bundle"
            | "minecraft:lime_bundle"
            | "minecraft:pink_bundle"
            | "minecraft:gray_bundle"
            | "minecraft:light_gray_bundle"
            | "minecraft:cyan_bundle"
            | "minecraft:purple_bundle"
            | "minecraft:blue_bundle"
            | "minecraft:brown_bundle"
            | "minecraft:green_bundle"
            | "minecraft:red_bundle"
            | "minecraft:black_bundle"
            | "minecraft:spyglass"
            | "minecraft:trident"
            | "minecraft:crossbow"
            | "minecraft:goat_horn"
            | "minecraft:brush" => false,
            _ => true,
        };
        let mut animation = if stack.item.as_ref() == "minecraft:shield" {
            horizon_world::consumable::UseAnimation::Block
        } else if generic {
            components.get("minecraft:consumable").map_or(
                horizon_world::consumable::UseAnimation::None,
                horizon_client::first_person::consumable_animation,
            )
        } else {
            horizon_world::consumable::UseAnimation::None
        };
        self.native_use_animation(stack, &mut animation);
        animation == horizon_world::consumable::UseAnimation::Block
    }
    fn kill_aura_interact_pick(&self, q: &Value) -> Option<super::ClientReachPick> {
        let raycast = q["raycast"].as_u64().unwrap_or(2);
        if raycast == 0 {
            return None;
        }
        let eye = self.eye_position().map(|a| V(a[0], a[1], a[2]))?;
        let (yaw, pitch) = angles(self, q);
        let range = q["range"].as_f64().unwrap_or(3.);
        let end = eye.fma(range, v(Vec3::direction_from_rotation(pitch, yaw)));
        self.kill_aura_candidates()
            .into_iter()
            .filter(|c| {
                raycast != 1
                    || q["eligible"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .any(|v| v.as_i64() == Some(i64::from(c.id)))
            })
            .filter_map(|c| {
                let b =
                    bounds(c.bounds).inflate(self.client_pick_margin(c.id, c.pick_radius as f32));
                let point = if b.contains(eye) {
                    Some(eye)
                } else if c.same_root_vehicle {
                    None
                } else {
                    b.clip(eye, end)
                }?;
                let distance_squared = point.sub(eye).len2();
                Some(super::ClientReachPick {
                    kind: c.kind,
                    index: c.index,
                    id: c.id,
                    local_id: c.local_id,
                    distance: distance_squared.sqrt(),
                    distance_squared,
                    point: point.array(),
                    location: std::array::from_fn(|i| point.array()[i] - c.position[i]),
                })
            })
            .min_by(|a, b| a.distance_squared.total_cmp(&b.distance_squared))
    }
    fn kill_aura_item_enabled(&self, stack: &horizon_client::items::ItemStack) -> bool {
        let name = stack.item.as_ref();
        let flags = super::client_vomit_blocks::CLIENT_ITEM_FEATURES
            .binary_search_by_key(&name, |(name, _)| *name)
            .ok()
            .map_or(&[][..], |i| {
                super::client_vomit_blocks::CLIENT_ITEM_FEATURES[i].1
            });
        let enabled = self
            .remote
            .as_ref()
            .map_or(&self.level.enabled_features, |r| &r.features);
        flags.iter().all(|flag| enabled.iter().any(|v| v == flag))
    }
    fn kill_aura_block_hand(&self, assume: bool) -> Option<i32> {
        for (hand, slot) in [
            (0, self.client_hotbar_slot()),
            (1, super::shield::OFFHAND_SLOT),
        ] {
            let Some(s) = self.stack_in(slot) else {
                continue;
            };
            if !self.kill_aura_item_enabled(s) {
                continue;
            }
            let default = self
                .items()
                .get(&s.item)
                .and_then(|i| i.use_cooldown.clone());
            let group = horizon_client::cooldowns::group(s, default.as_ref());
            if self.kill_aura_block_animation(s)
                && self.shield_cooldown == 0
                && !self.cooldowns.is_on_cooldown(&group)
            {
                return Some(hand);
            }
        }
        // The compatibility fallback applies only when blocks_attacks does
        // not exist; the actual negotiated 26.3 protocol has that component.
        let _ = assume;
        None
    }
    fn kill_aura_box(&self, id: i32) -> Option<(B, bool)> {
        if let Some(e) = super::auto_bow::entities(self)
            .into_iter()
            .find(|e| e.id == id)
        {
            return Some((bounds(e.bounds), true));
        }
        self.kill_aura_candidates()
            .into_iter()
            .find(|c| c.id == id)
            .map(|c| {
                (
                    bounds(c.bounds),
                    self.native_entity(id as u32)
                        .and_then(|e| self.native_living(e))
                        .is_some(),
                )
            })
    }
    fn kill_aura_hit(
        &self,
        id: i32,
        eye: V,
        rotation: (f32, f32),
        range: f64,
        walls: f64,
    ) -> Option<(bool, f64, Option<f64>)> {
        let (physical, _) = self.kill_aura_box(id)?;
        // EnderDragon itself is not pickable in 26.3. Its real parts own
        // the entity ray; the parent remains an attack-only target alias.
        if self
            .dragon
            .as_ref()
            .is_some_and(|dragon| dragon.id as i32 == id)
            || self.mobs.iter().any(|mob| {
                mob.id as i32 == id
                    && horizon_client::command::args::normalize(&mob.kind.id)
                        == "minecraft:ender_dragon"
            })
        {
            return Some((false, physical.distance(eye), None));
        }
        let b = physical.inflate(self.client_pick_margin(id, 0.));
        let end = eye.fma(
            range,
            v(Vec3::direction_from_rotation(rotation.1, rotation.0)),
        );
        let shared = self.riding.is_some_and(|r| {
            self.client_reach_root_vehicle(r) == self.client_reach_root_vehicle(id as u32)
        });
        let hit = if b.contains(eye) {
            Some(eye)
        } else if shared {
            None
        } else {
            b.clip(eye, end)
        };
        Some((
            hit.is_some_and(|h| {
                let d = h.sub(eye).len2();
                d <= walls * walls || d <= range * range && outline(self, eye, h)
            }),
            physical.distance(eye),
            hit.map(|h| h.sub(eye).len2()),
        ))
    }
    fn kill_aura_held_range(&self, distance: f64, increase: f32) -> bool {
        let range = self
            .stack_in(self.client_hotbar_slot())
            .and_then(|s| self.stack_attack_range(s));
        let value = |key: &str, default: f32| {
            range
                .as_ref()
                .and_then(|r| r[key].as_f64())
                .map_or(default, |v| v as f32)
        };
        let (mut min, mut max, mut cmin, mut cmax) = (
            value("min_reach", 0.),
            value("max_reach", 3.),
            value("min_creative_reach", 0.),
            value("max_creative_reach", 5.),
        );
        self.native_client_attack_range(&mut min, &mut max, &mut cmin, &mut cmax);
        let margin = value("hitbox_margin", 0.3);
        let (min, max) = if self.creative {
            (cmin, cmax)
        } else {
            (min, max)
        };
        distance >= f64::from(min.max(0.) - margin)
            && distance <= f64::from(max + increase + margin)
    }
    fn kill_aura_use(&mut self, q: &Value) -> bool {
        let (yaw, pitch) = angles(self, q);
        let Some(hand) = q["hand"]
            .as_i64()
            .filter(|h| [0, 1].contains(h))
            .map(|h| h as i32)
        else {
            return false;
        };
        let hands = if q["vanilla"] == true {
            vec![0, 1]
        } else {
            vec![hand]
        };
        let interact = q["mode"].as_u64() == Some(1);
        let picked = interact.then(|| self.kill_aura_interact_pick(q)).flatten();
        if picked.is_some_and(|hit| {
            !self.border.contains(
                (hit.point[0] - hit.location[0]).floor(),
                (hit.point[2] - hit.location[2]).floor(),
            )
        }) {
            return false;
        }
        let old_pick = self.client_kill_aura_pick;
        let old_ignore = self.client_kill_aura_ignore_entities;
        let old_block_reach = self.block_reach;
        if interact {
            self.client_kill_aura_pick = picked;
            self.client_kill_aura_ignore_entities = true;
            self.block_reach = if picked.is_some() {
                f64::MAX
            } else {
                self.block_reach.max(self.entity_reach)
            };
        }
        let tick_hit = self.tick_hit.take();
        let tick_mob = self.tick_mob.take();
        let used = self.native_with_item_rotation(yaw, pitch, |g| {
            for use_hand in hands {
                g.use_failed = false;
                let hand_enum = if use_hand == 0 { Hand::Main } else { Hand::Off };
                let block = (interact && picked.is_none()).then(|| g.pick()).flatten();
                let facing = picked.is_some() || block.is_some();
                let interacted = if interact && g.remote.is_some() {
                    let entity = picked.is_some();
                    if let Some(picked) =
                        picked.filter(|hit| matches!(hit.kind, 10 | 11 | KILL_AURA_GUEST))
                    {
                        // Players, EndCrystals and dragon parts pass the
                        // client interaction. remote_object_hit's generic
                        // consuming answer does not apply to these types.
                        g.send(S::Interact(play::Interact {
                            id: picked.id,
                            hand: use_hand,
                            location: picked.location,
                            secondary: g.player.as_ref().is_some_and(|p| p.shift_key_down),
                        }));
                        false
                    } else if entity {
                        g.remote_use_entity(hand_enum)
                    } else if block.is_some() {
                        g.remote_use_item_on(hand_enum)
                    } else {
                        false
                    }
                } else if interact {
                    if let Some(picked) = picked {
                        g.with_hand(hand_enum, |g| {
                            g.interact_entity(Some(super::InteractTarget {
                                id: picked.id,
                                location: picked.location,
                            }))
                            .unwrap_or(false)
                        })
                    } else if let Some(hit) = block.as_ref() {
                        // Separate useItemOn from Item.use, as the upstream
                        // strict interaction helper does for each hand.
                        let guest = g.guest_use;
                        g.guest_use = Some(super::GuestUse::OnBlock(hand_enum));
                        let result =
                            g.with_hand(hand_enum, |g| g.use_hand(Some(hit)).unwrap_or(false));
                        g.guest_use = guest;
                        result
                    } else {
                        false
                    }
                } else {
                    false
                };
                if interacted {
                    return use_hand == hand && q["vanilla"] != true && !g.use_failed;
                }
                if facing && q["vanilla"] != true {
                    return false;
                }
                if g.native_client_use_item(use_hand, yaw, pitch) {
                    return use_hand == hand;
                }
            }
            false
        });
        self.tick_hit = tick_hit;
        self.tick_mob = tick_mob;
        self.client_kill_aura_pick = old_pick;
        self.client_kill_aura_ignore_entities = old_ignore;
        self.block_reach = old_block_reach;
        used
    }
    fn kill_aura_keep_visual(&mut self, id: i32, area: Aabb, critical: bool) {
        let kind = self
            .level_entity(id)
            .map(|e| self.target_type(e))
            .unwrap_or_default();
        let spin = self.player.as_ref().map_or(0., |p| p.auto_spin_attack_dmg);
        let base = if spin > 0. {
            spin
        } else {
            self.attack_damage()
        };
        let speed = self
            .player
            .as_mut()
            .and_then(|p| p.attribute("minecraft:attack_speed").ok())
            .unwrap_or_else(|| self.main_hand_attack_speed());
        let strength = self.hand_strength.scale(0.5, speed);
        let magic = (self.enchanted_damage(&kind, base) - base) * strength;
        let damage = base * (0.2 + strength * strength * 0.8);
        if damage > 0. || magic > 0. {
            if magic > 0. {
                self.area_critical_hit(id, area, true);
            }
            if critical {
                let at = self
                    .player
                    .as_ref()
                    .map(|p| [p.position.x, p.position.y, p.position.z]);
                self.audio
                    .play("minecraft:entity.player.attack.crit", at, 1., 1.);
                self.area_critical_hit(id, area, false);
            }
        }
    }
    fn kill_aura_attack(&mut self, id: u32, keep: bool, critical: bool) -> bool {
        if self.client_projectile_puncher_piercing() {
            return true;
        }
        let Some(c) = self.kill_aura_attack_candidate(id) else {
            return false;
        };
        let living = self
            .native_entity(id)
            .and_then(|e| self.native_living(e))
            .is_some();
        if !self.client_weapon_before_attack(id, living) {
            return false;
        }
        let old_keep = self.client_kill_aura_keep_sprint;
        let old_hook = self.client_kill_aura_attack_hook;
        self.client_kill_aura_keep_sprint = keep;
        self.client_kill_aura_attack_hook = Some(id);
        if horizon_core::version::PROTOCOL_VERSION <= 47 {
            self.client_projectile_puncher_swing(0);
        }
        self.native_client_ensure_carried();
        if self.remote.is_some() {
            self.send(S::Attack(play::Attack { id: id as i32 }));
            if keep {
                self.kill_aura_keep_visual(id as i32, c.bounds, critical);
            }
            if !self.spectator() {
                let is_player = self
                    .mobs
                    .iter()
                    .find(|m| m.id == id)
                    .is_some_and(|m| m.kind.id.as_str() == "minecraft:player");
                self.native_client_attack(id, is_player);
                if let Some(i) = self.mobs.iter().position(|m| m.id == id) {
                    if !keep {
                        self.remote_player_attack(i);
                    }
                }
            }
        } else {
            // Scope the existing exact native object/mob actions to this
            // chosen EntityHitResult, preserving integrated-world simulation.
            let point = bounds(c.bounds).center();
            let eye = self
                .eye_position()
                .map(|a| V(a[0], a[1], a[2]))
                .unwrap_or(point);
            let d = point.sub(eye).len2();
            let old_pick = self.client_kill_aura_pick;
            let tick_hit = self.tick_hit.take();
            let tick_mob = self.tick_mob.take();
            let old_range = (self.block_reach, self.entity_reach);
            self.client_kill_aura_pick = Some(super::ClientReachPick {
                kind: c.kind,
                index: c.index,
                id: c.id,
                local_id: c.local_id,
                distance: d.sqrt(),
                distance_squared: d,
                point: point.array(),
                location: std::array::from_fn(|i| point.array()[i] - c.position[i]),
            });
            let old_implicit = self.client_projectile_puncher_hide_implicit;
            self.client_projectile_puncher_hide_implicit = true;
            self.block_reach = f64::MAX;
            self.entity_reach = f64::MAX;
            if self.kill_aura_extra_local_attack(c).is_none() {
                let is_player = self
                    .mobs
                    .iter()
                    .find(|m| m.id == id)
                    .is_some_and(|m| m.kind.id.as_str() == "minecraft:player");
                self.native_client_attack(id, is_player);
                let _ = self.attack();
            }
            self.block_reach = old_range.0;
            self.entity_reach = old_range.1;
            self.client_kill_aura_pick = old_pick;
            self.client_projectile_puncher_hide_implicit = old_implicit;
            self.tick_hit = tick_hit;
            self.tick_mob = tick_mob;
        }
        self.reset_attack_strength();
        if horizon_core::version::PROTOCOL_VERSION > 47 {
            self.client_projectile_puncher_swing(0);
        }
        self.client_kill_aura_keep_sprint = old_keep;
        self.client_kill_aura_attack_hook = old_hook;
        true
    }
    pub(super) fn kill_aura_query(&mut self, text: &str) -> String {
        let Ok(q) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        if q["op"] == "raytrace" || q["op"] == "point" {
            return self.aimbot_query(text);
        }
        if q["op"] == "unblock" {
            match q["mode"].as_u64().unwrap_or(0) {
                1 => {
                    let current = self.selected as i16;
                    let next = (current + 1) % 9;
                    self.send(S::SetCarriedItem(play::SetCarriedItem { slot: next }));
                    self.send(S::SetCarriedItem(play::SetCarriedItem { slot: current }));
                }
                2 => {
                    for _ in 0..2 {
                        self.send(S::PlayerAction(play::PlayerAction {
                            action: 7,
                            pos: [0; 3],
                            direction: 0,
                            sequence: 0,
                        }));
                    }
                }
                _ => {
                    self.auto_rod_query("{\"op\":\"release\"}");
                }
            }
            return "true".into();
        }
        if q["op"] == "inventory" {
            if q["open"] != true {
                self.native_client_inventory_close(0, false);
            }
            // Current 26.3 has no serverbound OPEN_INVENTORY command.
            // Keep the local screen in place; closing is the only network action.
            return "true".into();
        }
        if q["op"] == "rotation" {
            let (y, p) = angles(self, &q);
            self.native_client_auto_buff_packet(y, p);
            return "true".into();
        }
        if q["op"] == "block" {
            return json!(self.kill_aura_use(&q)).to_string();
        }
        if q["op"] == "attack" {
            return json!(q["id"].as_i64().is_some_and(|id| self.kill_aura_attack(
                id as u32,
                q["keepSprint"] == true,
                q["critical"] == true
            )))
            .to_string();
        }
        if q["op"] == "using" {
            return json!(self.using.is_some()).to_string();
        }
        if q["op"] == "blocking" {
            return json!(self.using.is_some_and(|(slot, _)| {
                self.stack_in(slot)
                    .is_some_and(|s| self.blocks_attacks(s).is_some())
            }))
            .to_string();
        }
        if q["op"] == "physicalUse" {
            return json!(self.right).to_string();
        }
        if q["op"] == "blockHand" {
            return json!(self.kill_aura_block_hand(q["assume"] == true)).to_string();
        }
        if q["op"] == "canAttack" {
            return json!(
                !self.spectator()
                    && self
                        .stack_in(self.client_hotbar_slot())
                        .is_none_or(|s| self.kill_aura_item_enabled(s))
                    && !self.cannot_attack_with_item(self.hand_strength.ticker, 0)
            )
            .to_string();
        }
        let Some(player) = self.player.as_ref() else {
            return "null".into();
        };
        let Some(eye) = self.eye_position().map(|a| V(a[0], a[1], a[2])) else {
            return "null".into();
        };
        if q["op"] == "facts" {
            let mut facts = serde_json::from_str::<Value>(&self.aimbot_query("{\"op\":\"facts\"}"))
                .unwrap_or(Value::Null);
            if !facts.is_object() {
                return "null".into();
            }
            let (dead, vehicle) = self.player.as_ref().map_or((true, false), |p| {
                (p.health.health() <= 0., self.riding.is_some())
            });
            if let Some(rows) = facts["candidates"].as_array_mut() {
                for row in rows {
                    if let Some(id) = row["id"].as_i64() {
                        row["wouldBlock"] = json!(self.client_weapon_would_block_hit(id as u32));
                        row["void"] = json!(
                                q["ground"] == true
                                    && self.kill_aura_box(id as i32).is_none_or(
                                        |(b, _)| !self.kill_aura_ground_below(id as i32, b)
                                    )
                            );
                    }
                }
            }
            let attribute = self.interaction_range("minecraft:entity_interaction_range", 3., 5.);
            let main = self.stack_in(self.client_hotbar_slot());
            facts["attributeRange"] = json!(attribute);
            facts["dead"] = json!(dead);
            facts["spectator"] = json!(self.spectator());
            facts["vehicle"] = json!(vehicle);
            facts["creative"] = json!(self.creative);
            facts["axe"] =
                json!(main.is_some_and(|s| self.items().item_in_tag("minecraft:axes", &s.item)));
            facts["blockAnimation"] =
                json!(main.is_some_and(|s| self.kill_aura_block_animation(s)));
            let range = main.and_then(|s| self.stack_attack_range(s));
            let value = |key: &str, default: f32| {
                range
                    .as_ref()
                    .and_then(|r| r[key].as_f64())
                    .map_or(default, |v| v as f32)
            };
            let (mut min, mut max, mut cmin, mut cmax) = (
                value("min_reach", 0.),
                value("max_reach", 3.),
                value("min_creative_reach", 0.),
                value("max_creative_reach", 5.),
            );
            self.native_client_attack_range(&mut min, &mut max, &mut cmin, &mut cmax);
            let (min, max) = if self.creative {
                (cmin, cmax)
            } else {
                (min, max)
            };
            facts["itemMinRange"] = json!(min.max(0.));
            facts["itemMaxRange"] = json!(max);
            return facts.to_string();
        }
        if q["op"] == "hit" {
            let id = q["id"].as_i64().unwrap_or(i64::MIN) as i32;
            let range = q["range"].as_f64().unwrap_or(3.);
            let walls = q["walls"].as_f64().unwrap_or(0.);
            return self.kill_aura_hit(id,eye,angles(self,&q),range,walls).map_or_else(||"null".into(),|(hit,d,ray)|json!({"hit":hit,"inRange":ray.is_some_and(|distance|self.kill_aura_held_range(distance.sqrt(),q["increase"].as_f64().unwrap_or(0.)as f32)),"distance":d}).to_string());
        }
        if q["op"] == "crosshair" {
            let rotation = angles(self, &q);
            let range = q["range"].as_f64().unwrap_or(3.);
            let end = eye.fma(
                range,
                v(Vec3::direction_from_rotation(rotation.1, rotation.0)),
            );
            let mut hits = self
                .kill_aura_candidates()
                .into_iter()
                .filter_map(|c| {
                    let b = bounds(c.bounds)
                        .inflate(self.client_pick_margin(c.id, c.pick_radius as f32));
                    let point = if b.contains(eye) {
                        Some(eye)
                    } else if c.same_root_vehicle {
                        None
                    } else {
                        b.clip(eye, end)
                    }?;
                    let d = point.sub(eye).len2();
                    (d <= range * range).then(|| {
                        (
                            d,
                            c.id,
                            self.native_entity(c.id as u32)
                                .and_then(|e| self.native_living(e))
                                .is_some(),
                        )
                    })
                })
                .collect::<Vec<_>>();
            hits.sort_by(|a, b| a.0.total_cmp(&b.0));
            return json!(
                hits.into_iter()
                    .map(|(_, id, living)| json!({"id":id,"living":living}))
                    .collect::<Vec<_>>()
            )
            .to_string();
        }
        if q["op"] == "danger" {
            let range = q["range"].as_f64().unwrap_or(3.);
            let walls = q["walls"].as_f64().unwrap_or(0.);
            let own = bounds(player.bounds()).inflate(q["tolerance"].as_f64().unwrap_or(0.3));
            let force = serde_json::from_value::<[f64; 2]>(q["force"].clone()).unwrap_or([0., 1.]);
            let danger = q["targets"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_i64)
                .any(|id| {
                    let Some((b, _)) = self.kill_aura_box(id as i32) else {
                        return false;
                    };
                    if b.distance(eye) > range * range {
                        return false;
                    }
                    let Some(e) = self.native_entity(id as u32) else {
                        return false;
                    };
                    let Some(a) = self.native_entity_eye_position(e) else {
                        return false;
                    };
                    let Some([yaw, pitch]) = self.native_entity_rotation(e) else {
                        return false;
                    };
                    let origin = V(a[0], a[1], a[2]);
                    let Some(hit) = super::aimbot_ray::first_hit(
                        own,
                        origin,
                        origin.fma(range, v(Vec3::direction_from_rotation(pitch, yaw))),
                    ) else {
                        return false;
                    };
                    let d = hit.sub(origin).len2().sqrt();
                    d >= force[0] && d <= force[1]
                        || d <= walls
                        || d <= range && outline(self, origin, hit)
                });
            return json!(danger).to_string();
        }
        if q["op"] == "exiting" {
            let id = q["id"].as_i64().unwrap_or(i64::MIN) as i32;
            let ticks = q["ticks"].as_f64().unwrap_or(1.);
            let Some(e) = super::auto_bow::entities(self)
                .into_iter()
                .find(|e| e.id == id)
            else {
                return "false".into();
            };
            let t = ticks.ceil().clamp(0., 40.) as i32;
            let future = super::auto_bow_forecast::positions(self, id, t)
                .and_then(|v| v.last().copied())
                .unwrap_or([
                    e.position.x + e.velocity.x * ticks,
                    e.position.y + e.velocity.y * ticks,
                    e.position.z + e.velocity.z * ticks,
                ]);
            let own = self.kill_aura_local_future(t, angles(self, &q).0);
            let b = bounds(e.bounds).moved(V(
                future[0] - e.position.x,
                future[1] - e.position.y,
                future[2] - e.position.z,
            ));
            return json!(!self.kill_aura_held_range(
                b.distance(own).sqrt(),
                q["increase"].as_f64().unwrap_or(0.) as f32
            ))
            .to_string();
        }
        if q["op"] == "failEligible" {
            let range = q["range"].as_f64().unwrap_or(3.);
            let target = q["id"].as_i64().map(|i| i as i32);
            let candidate = target
                .and_then(|id| self.kill_aura_box(id).map(|(b, _)| (id, b)))
                .or_else(|| {
                    q["eligible"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_i64)
                        .filter_map(|id| self.kill_aura_box(id as i32).map(|(b, _)| (id as i32, b)))
                        .filter(|(_, b)| b.distance(eye) <= range * range)
                        .min_by(|a, b| a.1.distance(eye).total_cmp(&b.1.distance(eye)))
                });
            let Some((id, b)) = candidate else {
                return "null".into();
            };
            if b.distance(eye) > range * range || self.client_crosshair_entity().is_some() {
                return "null".into();
            }
            if self.pick().is_some_and(|hit| {
                self.res
                    .facts()
                    .is_air(self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]))
            }) {
                return "null".into();
            }
            return json!({"id":id}).to_string();
        }
        if q["op"] == "failSwing" {
            let rotation = angles(self, &q);
            let Some(center) = q["id"]
                .as_i64()
                .and_then(|id| self.kill_aura_box(id as i32))
                .map(|(b, _)| b.center())
            else {
                return "null".into();
            };
            let block = self.pick().filter(|h| {
                !self
                    .res
                    .facts()
                    .is_air(self.world.block(h.pos[0], h.pos[1], h.pos[2]))
            });
            if let Some(block) = block {
                if self.remote.is_some() {
                    self.remote_mining(true);
                } else {
                    let face = match block.face {
                        [0, -1, 0] => 0,
                        [0, 1, 0] => 1,
                        [0, 0, -1] => 2,
                        [0, 0, 1] => 3,
                        [-1, 0, 0] => 4,
                        _ => 5,
                    };
                    self.client_kill_aura_start_destroy(block.pos, face);
                }
            } else if !self.creative {
                self.clicks.miss_time = 10;
            }
            self.client_projectile_puncher_swing(0);
            if q["sound"] == true {
                self.audio.play(
                    "minecraft:block.note_block.bass",
                    None,
                    q["volume"].as_f64().unwrap_or(0.5) as f32,
                    q["pitch"].as_f64().unwrap_or(0.8) as f32,
                );
            }
            return json!({"eye":eye.array(),"center":center.array(),"direction":v(Vec3::direction_from_rotation(rotation.1,rotation.0)).array()}).to_string();
        }
        "null".into()
    }
}
