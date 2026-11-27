// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleExtinguish: real buckets, body replay, placement context and verified click.
use super::{
    auto_walk_player::NativePlayer,
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NavigationRegistry},
};
use super::{
    block_placement::{self as placement, Target},
    *,
};
use crate::navigation::{
    input::Input,
    walk_control::body::{SimulatedInput, WalkSimulation},
};
use serde_json::{Value, json};
const REQUESTER: u64 = 0x4558_5449_4e47_5549;
fn nearest(game: &Game<'_>, item: &str) -> Option<usize> {
    let selected = game.client_hotbar_slot();
    std::iter::once(40usize)
        .chain(0..9)
        .filter(|slot| {
            game.stack_in(*slot)
                .is_some_and(|s| s.count > 0 && &*s.item == item)
        })
        .min_by_key(|slot| {
            if *slot == selected {
                i32::MIN
            } else if *slot == 40 {
                i32::MIN + 1
            } else {
                (*slot as i32 - selected as i32).abs()
            }
        })
}
fn write_target(t: &Target, slot: usize) -> Value {
    json!({"interacted":t.interacted,"placed":t.placed,"direction":t.direction,"point":t.point,"minY":t.min_y,"rotation":t.rotation,"slot":slot})
}
fn read_target(v: &Value) -> Option<Target> {
    Some(Target {
        interacted: serde_json::from_value(v["interacted"].clone()).ok()?,
        placed: serde_json::from_value(v["placed"].clone()).ok()?,
        direction: serde_json::from_value(v["direction"].clone()).ok()?,
        point: serde_json::from_value(v["point"].clone()).ok()?,
        min_y: v["minY"].as_f64()?,
        rotation: serde_json::from_value(v["rotation"].clone()).ok()?,
    })
}
impl Game<'_> {
    pub(super) fn extinguish_query(&mut self, text: &str) -> Option<String> {
        if !self.native_client_slot_owner() {
            return None;
        }
        let q: Value = serde_json::from_str(text).ok()?;
        match q["op"].as_str()? {
            "facts" => {
                let p = self.player.as_ref()?;
                Some(json!({"onFire":p.fire_ticks>0,"fireResistance":p.status.fire_resistance,"evaporates":self.dimension_type().water_evaporates}).to_string())
            }
            "plan" => {
                let pickup: Option<[i32; 3]> = serde_json::from_value(q["pickup"].clone()).ok();
                let slot = nearest(
                    self,
                    if pickup.is_some() {
                        "minecraft:bucket"
                    } else {
                        "minecraft:water_bucket"
                    },
                )?;
                let stack = self.stack_in(slot)?.clone();
                let predicted = if pickup.is_some() {
                    None
                } else {
                    Some(world::with_scope(self, || {
                        let epoch = world::read(world::current_epoch);
                        let registry = world::read(NavigationRegistry::capture);
                        let player = NativePlayer::new(NativeLevel::new(registry, epoch));
                        let key = |i: usize| q["keys"][i] == true;
                        let mut input = SimulatedInput::new(Input {
                            forward: key(0),
                            backward: key(1),
                            left: key(2),
                            right: key(3),
                            jump: key(4),
                            shift: key(5),
                            sprint: key(6),
                        });
                        input.force_stay_on_ground_surface =
                            horizon_client::navigation_physics::staying_on_surface(false);
                        let mut sim = NativeSimulation::new(
                            &player,
                            input,
                            q["movementYaw"].as_f64().unwrap_or(0.) as f32,
                        );
                        for tick in 0..=20 {
                            if tick != 0 {
                                sim.tick();
                            }
                            if sim.player().on_ground() {
                                return Some(
                                    sim.player()
                                        .read(|p| [p.position.x, p.position.y, p.position.z]),
                                );
                            }
                        }
                        None
                    })?)
                };
                let pos = pickup.or_else(|| predicted.map(|p| p.map(|v| v.floor() as i32)))?;
                let rotation = serde_json::from_value(q["serverRotation"].clone()).ok()?;
                let location = placement::capture_location(self, predicted, rotation)?;
                let mut options = placement::Options::center(&stack, location);
                let target = placement::find(self, pos, &mut options)?;
                Some(write_target(&target, slot).to_string())
            }
            "execute" => {
                let plan = &q["plan"];
                let target = read_target(plan)?;
                let rotation: [f32; 2] = serde_json::from_value(q["rotation"].clone()).ok()?;
                let range = self.block_reach.max(self.entity_reach);
                let hit = placement::verify(
                    self,
                    &target,
                    rotation,
                    range,
                    placement::ClickTrace::World,
                    placement::FailedClick::Nothing,
                )?;
                let slot = plan["slot"].as_u64()? as usize;
                if slot != 40
                    && !self
                        .native_client_select_slot_silently(REQUESTER, slot as i32, 1)
                        .ok()?
                {
                    return Some("false".into());
                }
                let hand = if slot == 40 { Hand::Off } else { Hand::Main };
                let placed = target.placed;
                let accepted =
                    self.client_do_placement(hit, rotation, hand, "DoNotHide", |g, _| {
                        g.native_client_extinguish_success(placed);
                        true
                    });
                Some(json!(accepted).to_string())
            }
            _ => None,
        }
    }
}
