// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game, GuestUse, Hand, Hit};
use horizon_client::{blocks::Fluid, items::ItemStack, world_primitives::WorldPrimitives};
use serde_json::Value;
impl Game<'_> {
    fn client_air_place_config(&self) -> Option<Value> {
        if !self.native_window() || self.player.is_none() || self.spectator() {
            return None;
        }
        let mut configuration = String::new();
        self.native_client_air_place_configuration(&mut configuration);
        let config = serde_json::from_str::<Value>(&configuration).ok()?;
        (config["enabled"] == true).then_some(config)
    }
    fn client_air_place_stack(&mut self, stack: &ItemStack, hit: &Hit) -> bool {
        if stack.count <= 0 || &*stack.item == "minecraft:air" {
            return false;
        }
        let view = horizon_client::item_render::stack_view(stack);
        if self
            .pack
            .item_renderer
            .models
            .components_view(&view)
            .contains_key("minecraft:consumable")
        {
            return false;
        }
        let is = |class: &str| Self::client_air_place_item_class(&stack.item, class);
        if is("BlockItem") {
            let Some(state) = self.items().get(&stack.item).and_then(|i| i.block) else {
                return false;
            };
            let pos = horizon_core::position::BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]);
            return self.with_level(|b, level, ticks| b.can_survive(level, ticks, pos, state));
        }
        if is("SpawnEggItem") || is("ArmorStandItem") {
            return true;
        }
        if is("FireworkRocketItem") {
            return !self.armor[1].as_ref().is_some_and(|chest| {
                let view = horizon_client::item_render::stack_view(chest);
                self.pack
                    .item_renderer
                    .models
                    .components_view(&view)
                    .contains_key("minecraft:glider")
            });
        }
        false
    }
    fn client_air_place_can(&mut self, hit: &Hit, config: &Value) -> bool {
        let state = self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
        let facts = self.res.facts();
        if !(facts.is_air(state)
            || (config["placeInLiquid"] == true
                && config["liquidPlace"] != true
                && facts.get(state).fluid != Fluid::None))
        {
            return false;
        }
        let main = self.with_hand(Hand::Main, |g| g.held().cloned());
        let off = self.offhand.clone();
        main.as_ref()
            .is_some_and(|s| self.client_air_place_stack(s, hit))
            || off
                .as_ref()
                .is_some_and(|s| self.client_air_place_stack(s, hit))
    }
    fn client_air_place_hit(&mut self, config: &Value) -> Option<Hit> {
        if self.pick_entity_hit().is_some() {
            return None;
        }
        let mut hit = self.pick().or_else(|| {
            let (eye, dir) = self.client_block_aim()?;
            let point = std::array::from_fn(|i| eye[i] + dir[i] * self.block_reach);
            let mut face = [0, -1, 0];
            let mut dot = f64::NEG_INFINITY;
            for candidate in [
                [0, -1, 0],
                [0, 1, 0],
                [0, 0, -1],
                [0, 0, 1],
                [-1, 0, 0],
                [1, 0, 0],
            ] {
                let next = (0..3)
                    .map(|i| -f64::from(candidate[i]) * dir[i])
                    .sum::<f64>();
                if next > dot {
                    dot = next;
                    face = candidate;
                }
            }
            Some(Hit {
                pos: point.map(|x| x.floor() as i32),
                face,
                point,
            })
        })?;
        if !self.client_air_place_can(&hit, config) {
            return None;
        }
        if config["customRange"] == true {
            hit.point = crate::air_place_target::target(
                self.eye_position()?,
                hit.point,
                config["range"].as_f64()? as f32,
            );
            hit.pos = hit.point.map(|v| v.floor() as i32);
            if !self.client_air_place_can(&hit, config) {
                return None;
            }
        }
        Some(hit)
    }
    /// MultiPlayerGameMode.useItem HEAD: a valid target cancels with PASS,
    /// including useItemOn failures. The hand is the actual event hand.
    pub(super) fn client_air_place_item(&mut self, hand: Hand) -> bool {
        let Some(config) = self.client_air_place_config() else {
            return false;
        };
        let Some(hit) = self.client_air_place_hit(&config) else {
            return false;
        };
        if self.remote.is_some() {
            let old = self.forced_hit;
            self.forced_hit = Some(hit);
            self.remote_use_item_on(hand);
            self.forced_hit = old;
        } else {
            self.with_hand(hand, |g| {
                let old = g.guest_use;
                g.guest_use = Some(GuestUse::OnBlock(hand));
                let _ = g.use_hand(Some(&hit));
                g.guest_use = old;
            });
        }
        true
    }
    pub(super) fn client_air_place_geometry(&mut self, primitives: &mut WorldPrimitives) {
        let Some(config) = self
            .client_air_place_config()
            .filter(|c| c["preview"] == true)
        else {
            return;
        };
        let Some(hit) = self.client_air_place_hit(&config) else {
            return;
        };
        let min = hit.pos.map(f64::from);
        let max = min.map(|v| v + 1.);
        let fill = config["color"].as_u64().unwrap_or(0) as u32;
        let outline = config["outlineColor"].as_u64().unwrap_or(0) as u32;
        if config["outlineOnly"] != true && fill >> 24 != 0 {
            primitives.filled_box(min, max, fill).through_walls();
        }
        if outline >> 24 != 0 {
            primitives
                .box_outline(min, max, outline, 1.)
                .through_walls();
        }
    }
}
