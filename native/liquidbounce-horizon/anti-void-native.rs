// SPDX-License-Identifier: GPL-3.0-or-later
//! AntiVoid reads actual bodies, collision shapes and a shared local forecast.
use super::{
    auto_walk_player::NativePlayer,
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NavigationRegistry},
    *,
};
use crate::navigation::{
    input::Input,
    walk_control::body::{SimulatedInput, WalkSimulation},
};
use horizon_core::geometry::Axis;
use horizon_world::{
    block_collision_scan::BlockCollisionScan,
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_obstruction::EntityIdentity,
};
use serde_json::{Value, json};
impl Game<'_> {
    fn anti_void_column_empty(&self, position: Vec3, void_level: i32) -> bool {
        let Some(player) = self.player.as_ref() else {
            return false;
        };
        // MC26.3 AABB.setMinY calls the Math.min/Math.max constructor.
        let bounds = player
            .bounds()
            .move_by(player.position.vector_to(position))
            .with_min(Axis::Y, f64::from(void_level));
        let held = self
            .inventory
            .get(self.client_hotbar_slot())
            .and_then(Option::as_ref);
        let context = CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(
                    self.remote
                        .as_ref()
                        .map_or(0, |r| r.entity_id as u32 as u64),
                ),
                kind: EntityContextKind::Player,
                y: player.position.y,
                descending: player.shift_key_down,
                main_hand: Some(held.map_or(CollisionItem::EMPTY, |stack| CollisionItem {
                    item: &stack.item,
                    count: stack.count as i32,
                })),
            },
            false,
        );
        let mut scene = horizon_client::collisions::SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: Some(player.powder_snow_subject()),
        };
        !BlockCollisionScan::any_collision(&mut scene, context, bounds, false, usize::MAX)
            .expect("AntiVoid actual block collision column")
    }
    pub(super) fn anti_void_clear(&self, void_level: i32) -> bool {
        if !self.native_client_slot_owner() {
            return false;
        }
        self.player
            .as_ref()
            .is_some_and(|p| self.anti_void_column_empty(p.position, void_level))
    }
    pub(super) fn anti_void_raise(&mut self, height: f64) {
        if !self.native_client_slot_owner() {
            return;
        }
        if let Some(p) = self.player.as_mut() {
            p.position.y += height;
        }
    }
    pub(super) fn anti_void_query(&mut self, text: &str) -> Option<String> {
        if !self.native_client_slot_owner() {
            return None;
        }
        let v: Value = serde_json::from_str(text).ok()?;
        let p = self.player.as_ref()?;
        let (position, fall_distance, motion_y, dead) = (
            p.position,
            p.fall_distance,
            p.velocity.y,
            p.health.dead_or_dying(),
        );
        horizon_world::anti_void::player_dead(dead);
        let void_level = v["voidLevel"].as_i64()? as i32;
        let slots = std::iter::once((40, self.offhand.as_ref()))
            .chain((0..9).map(|i| (i as i32, self.inventory.get(i).and_then(Option::as_ref))))
            .map(|(slot, stack)| json!([slot, stack.map_or("minecraft:air", |s| &*s.item)]))
            .collect::<Vec<_>>();
        let selected = self.client_hotbar_slot();
        let (likely, reaches_ground) = world::with_scope(self, || {
            let epoch = world::read(world::current_epoch);
            let registry = world::read(NavigationRegistry::capture);
            let player = NativePlayer::new(NativeLevel::new(registry, epoch));
            let key = |i: usize| v["keys"][i] == true;
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
            let yaw = v["movementYaw"].as_f64().unwrap_or(0.) as f32;
            let mut simulation = NativeSimulation::new(&player, input, yaw);
            let mut tick = 0;
            let mut ground = false;
            let likely = horizon_world::block_walk::without_shape_changes(|| {
                for index in 0..10 {
                    if index != 0 {
                        simulation.tick();
                        tick += 1;
                    }
                    let (position, fall, grounded) = simulation
                        .player()
                        .read(|p| (p.position, p.fall_distance, p.ground.on_ground));
                    if index != 0 {
                        ground |= grounded;
                    }
                    if fall > 0. {
                        return world::read(|game| {
                            game.anti_void_column_empty(position, void_level)
                        });
                    }
                }
                false
            });
            if likely
                && v["useItem"] == true
                && !(motion_y
                    >= f64::from(f32::from_bits(
                        v["thresholdBits"]
                            .as_u64()
                            .unwrap_or((-0.5_f32).to_bits() as u64) as u32,
                    )))
            {
                // Extend the same forecast, preserving its cached early snapshots.
                // Source UseItem's extension itself does not suppress shape changes.
                while tick < 40 && !ground {
                    simulation.tick();
                    tick += 1;
                    ground |= simulation.player().on_ground();
                    if ground {
                        break;
                    }
                }
            }
            (likely, ground)
        });
        Some(
            json!({"likely":likely,"reachesGround":reaches_ground,
            "position":[position.x,position.y,position.z],"fallDistance":fall_distance,
            "motionY":motion_y,"dead":dead,"slots":slots,"selected":selected})
            .to_string(),
        )
    }
}
