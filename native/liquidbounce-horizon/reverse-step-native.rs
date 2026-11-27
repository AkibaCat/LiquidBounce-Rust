// SPDX-License-Identifier: GPL-3.0-or-later
//! ReverseStep over the actual scoped body and contextual block collision.
use super::*;
use super::{
    auto_walk_player::NativePlayer,
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NavigationRegistry},
};
use crate::navigation::{
    aabb::Aabb as Box3,
    geometry::Vec3 as V,
    input::Input,
    simulation::player_motion::MotionPlayer,
    terrain::{TerrainLevel, TerrainState},
    walk_control::body::{SimulatedInput, WalkPlayer, WalkSimulation},
};
use crate::reverse_step::{self as model, Mode, Settings};
use horizon_protocol::packets::play::Serverbound as S;
use std::cell::RefCell;
thread_local! {static LEVEL:RefCell<Option<NativeLevel>>=const{RefCell::new(None)};}
fn level(epoch: u64) -> NativeLevel {
    LEVEL.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.as_ref().is_none_or(|l| l.epoch != epoch) {
            *slot = Some(NativeLevel::new(
                world::read(NavigationRegistry::capture),
                epoch,
            ));
        }
        slot.as_ref().expect("reverse-step level").clone()
    })
}
fn held(player: &NativePlayer, directional: bool, jump: bool) -> SimulatedInput {
    let k = crate::reverse_step_bridge::keys();
    let keys = Input {
        forward: directional && k[0],
        backward: directional && k[1],
        left: directional && k[2],
        right: directional && k[3],
        jump: jump && k[4],
        shift: player.read(|p| p.shift_key_down),
        sprint: k[6] || player.is_sprinting(),
    };
    SimulatedInput {
        keys,
        force_stay_on_ground_surface: horizon_client::navigation_physics::staying_on_surface(false),
        ignores_edge_back_off: false,
    }
}
fn unwanted(player: &NativePlayer, yaw: f32) -> bool {
    let mut simulation = NativeSimulation::new(player, held(player, true, false), yaw);
    for _ in 0..20 {
        WalkSimulation::tick(&mut simulation);
        if simulation.player().on_ground() {
            return simulation.player().supporting_block().is_some_and(|p| {
                matches!(
                    player.inventory.level.block_state(p).name(),
                    "minecraft:water"
                        | "minecraft:cobweb"
                        | "minecraft:powder_snow"
                        | "minecraft:hay_block"
                        | "minecraft:slime_block"
                )
            });
        }
    }
    false
}
fn collision_below(player: &NativePlayer, distance: f32) -> bool {
    let b = player.read(|p| super::auto_walk_player::box_(p.bounds()));
    let b = Box3::new(
        V::new(b.min.x, b.min.y - f64::from(distance), b.min.z),
        V::new(b.max.x, b.max.y - f64::from(distance), b.max.z),
    );
    player.inventory.level.game(|game| {
        let p = game.player.as_ref().expect("reverse-step collision player");
        let context = CollisionContext::for_entity(
            CollisionEntity {
                identity: horizon_world::entity_obstruction::EntityIdentity(0),
                kind: horizon_world::collision_context::EntityContextKind::Player,
                y: p.position.y,
                descending: p.shift_key_down,
                main_hand: Some(CollisionItem::EMPTY),
            },
            false,
        );
        let mut world = horizon_client::collisions::SceneCollisions {
            world: &game.world,
            binding: game.collision,
            subject: Some(p.powder_snow_subject()),
        };
        horizon_world::block_collision_scan::BlockCollisionScan::any_collision(
            &mut world,
            context,
            super::auto_walk_player::core_box(b),
            false,
            1_000_000,
        )
        .expect("reverse-step contextual block collision")
    })
}

impl model::Simulation for NativeSimulation {
    fn sample(&self) -> model::Sample {
        let p = self.player();
        let v = p.position();
        model::Sample {
            position: [v.x, v.y, v.z],
            on_ground: p.on_ground(),
            horizontal_collision: p.horizontal_collision(),
        }
    }
    fn tick(&mut self) {
        WalkSimulation::tick(self);
    }
}
pub(super) fn clear() {
    LEVEL.with(|l| *l.borrow_mut() = None);
    crate::reverse_step_bridge::clear();
}
impl Game<'_> {
    pub(super) fn reverse_step_tick(&mut self, value: &str) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(value) else {
            return;
        };
        if !self.native_client_slot_owner() {
            clear();
            return;
        }
        let running = v["running"] == true;
        let p = self.player.as_ref().expect("reverse-step owner player");
        let state = crate::reverse_step_bridge::observe(running, p.velocity.y, p.ground.on_ground);
        if !running || state.initiated_jump || p.ground.on_ground {
            return;
        }
        let s = Settings {
            mode: Mode::parse(v["mode"].as_str().unwrap_or("Instant")).unwrap_or_default(),
            maximum_fall_distance: v["distance"].as_f64().unwrap_or(1.) as f32,
            ticks: v["ticks"].as_i64().unwrap_or(20).clamp(1, 40) as i32,
            simulate_falling: v["simulate"] == true,
            motion: v["motion"].as_f64().unwrap_or(1.) as f32,
            factor: v["factor"].as_f64().unwrap_or(1.) as f32,
        };
        if p.fall_distance > f64::from(s.maximum_fall_distance) {
            return;
        }
        let epoch = world::current_epoch(self);
        world::with_scope(self, || {
            let player = NativePlayer::new(level(epoch));
            let yaw = player.read(|p| p.yaw);
            if !collision_below(&player, s.maximum_fall_distance) || unwanted(&player, yaw) {
                return;
            }
            if s.mode == Mode::Instant {
                let mut simulation =
                    NativeSimulation::new(&player, held(&player, false, true), yaw);
                if let Some(landing) = model::instant(s, &mut simulation) {
                    world::write(|g| {
                        for sample in landing.packets {
                            let [x, y, z] = sample.position;
                            g.send(S::MovePlayerPos(
                                horizon_protocol::packets::play::MovePlayerPos {
                                    x,
                                    y,
                                    z,
                                    flags: u8::from(sample.on_ground)
                                        | (u8::from(sample.horizontal_collision) << 1),
                                },
                            ));
                        }
                        if let Some(p) = g.player.as_mut() {
                            let [x, y, z] = landing.position;
                            p.position = Vec3::new(x, y, z);
                        }
                    });
                }
            } else if let Some([x, y, z]) = model::velocity(
                s,
                player.read(|p| [p.velocity.x, p.velocity.y, p.velocity.z]),
            ) {
                world::write(|g| {
                    if let Some(p) = g.player.as_mut() {
                        p.velocity = Vec3::new(x, y, z);
                    }
                });
            }
        });
    }
}
