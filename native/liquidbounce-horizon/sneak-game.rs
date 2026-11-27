// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use super::{
    auto_walk_player::NativePlayer,
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NavigationRegistry},
};
use crate::navigation::{
    aabb::Aabb as B,
    geometry::{BlockPos as P, Vec3 as V},
    input::Input,
    terrain::{TerrainLevel, TerrainState},
    walk_control::body::{SimulatedInput, WalkSimulation},
};
use std::cell::RefCell;
thread_local! {static LEVEL:RefCell<Option<NativeLevel>>=const{RefCell::new(None)};}
pub(super) fn clear() {
    LEVEL.with(|level| *level.borrow_mut() = None);
}
fn on_magma(player: &NativePlayer) -> bool {
    let bounds = player.read(|p| super::auto_walk_player::box_(p.bounds()));
    let b = B::new(
        V::new(bounds.min.x, bounds.min.y - 0.2, bounds.min.z),
        bounds.max,
    );
    let level = &player.inventory.level;
    let y = b.min.y.floor() as i32;
    for z in b.min.z.floor() as i32..=b.max.z.floor() as i32 {
        for x in b.min.x.floor() as i32..=b.max.x.floor() as i32 {
            let p = P::new(x, y, z);
            let state = level.block_state(p);
            if state.has_class("MagmaBlock")
                && state.collision_shape(level, p).aabbs().iter().any(|shape| {
                    B::new(
                        V::new(
                            shape.min.x + x as f64,
                            shape.min.y + y as f64,
                            shape.min.z + z as f64,
                        ),
                        V::new(
                            shape.max.x + x as f64,
                            shape.max.y + y as f64,
                            shape.max.z + z as f64,
                        ),
                    )
                    .intersects(b)
                })
            {
                return true;
            }
        }
    }
    false
}
impl Game<'_> {
    pub(super) fn sneak_moving(&self) -> Option<bool> {
        if !self.native_client_slot_owner() {
            return None;
        }
        self.player.as_ref().map(|p| p.move_vector != [0.; 2])
    }
    pub(super) fn sneak_context(&self) -> Option<[bool; 2]> {
        let moving = self.sneak_moving()?;
        let p = self.player.as_ref()?;
        let immune = p.status.fire_resistance
            || p.status.resistance.is_some_and(|level| level >= 4)
            || self.creative
            || p.spectator
            || self.armor[3]
                .as_ref()
                .is_some_and(|s| s.enchantment_level("minecraft:frost_walker") > 0);
        Some([moving, immune])
    }
    pub(super) fn sneak_magma(&mut self, raw: [bool; 7]) -> bool {
        if self.sneak_moving().is_none() {
            return false;
        }
        let epoch = world::current_epoch(self);
        world::with_scope(self, || {
            let level = LEVEL.with(|l| {
                let mut l = l.borrow_mut();
                if l.as_ref().is_none_or(|l| l.epoch != epoch) {
                    *l = Some(NativeLevel::new(
                        world::read(NavigationRegistry::capture),
                        epoch,
                    ));
                }
                l.as_ref().unwrap().clone()
            });
            let player = NativePlayer::new(level);
            let mut input = SimulatedInput::new(Input {
                forward: raw[0],
                backward: raw[1],
                left: raw[2],
                right: raw[3],
                jump: false,
                shift: player.read(|p| p.shift_key_down),
                sprint: raw[6] || player.read(|p| p.sprinting),
            });
            input.ignores_edge_back_off = true;
            let mut simulation = NativeSimulation::new(&player, input, world::read(|g| g.yaw));
            simulation.tick();
            let first = on_magma(simulation.player());
            simulation.tick();
            first || on_magma(simulation.player())
        })
    }
}
