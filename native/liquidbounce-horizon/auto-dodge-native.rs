// SPDX-License-Identifier: GPL-3.0-or-later
//! Source projectile forecasting over the actual scoped player body and collider shapes.
use super::{
    Game,
    auto_walk_player::NativePlayer,
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NavigationRegistry},
};
use crate::{
    auto_dodge as model,
    navigation::{
        input::Input,
        walk_control::body::{SimulatedInput, WalkSimulation},
    },
};
use horizon_core::{
    geometry::{Vec3, VoxelShape},
    position::BlockPos,
};
use horizon_world::{
    registry::BlockStateId,
    world_ray::{RayBlockMode, RayFluidMode, RayWorld, clip_world},
};
use serde_json::{Value, json};
use std::io;
type V = [f64; 3];
fn vec(v: V) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}
fn point(v: Vec3) -> V {
    [v.x, v.y, v.z]
}
struct ColliderRay<'a, 'r>(&'a Game<'r>);
impl RayWorld for ColliderRay<'_, '_> {
    type Error = io::Error;
    type Block = BlockStateId;
    type Fluid = ();
    fn block_state(&mut self, p: BlockPos) -> io::Result<BlockStateId> {
        Ok(self.0.world.block(p.x, p.y, p.z))
    }
    fn fluid_state(&mut self, _: BlockPos) -> io::Result<()> {
        Ok(())
    }
    fn block_shape(
        &mut self,
        s: BlockStateId,
        p: BlockPos,
        _: RayBlockMode,
    ) -> io::Result<VoxelShape> {
        let mut scene = horizon_client::collisions::SceneCollisions {
            world: &self.0.world,
            binding: self.0.collision,
            subject: None,
        };
        self.0
            .collision
            .collision_shape_with_context(
                &mut scene,
                s,
                p,
                &horizon_world::collision_context::CollisionContext::empty(false),
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))
    }
    fn interaction_shape(&mut self, _: BlockStateId, _: BlockPos) -> io::Result<VoxelShape> {
        Ok(VoxelShape::empty())
    }
    fn fluid_is_source(&mut self, _: ()) -> io::Result<bool> {
        Ok(false)
    }
    fn fluid_is_empty(&mut self, _: ()) -> io::Result<bool> {
        Ok(true)
    }
    fn fluid_is_water(&mut self, _: ()) -> io::Result<bool> {
        Ok(false)
    }
    fn fluid_shape(&mut self, _: (), _: BlockPos) -> io::Result<VoxelShape> {
        Ok(VoxelShape::empty())
    }
}
fn clip(from: V, to: V) -> Option<V> {
    world::read(|g| {
        let visits = (model::length(model::sub(to, from)).ceil() as usize)
            .saturating_mul(4)
            .saturating_add(64);
        let result = clip_world(
            &mut ColliderRay(g),
            vec(from),
            vec(to),
            RayBlockMode::Collider,
            RayFluidMode::None,
            visits,
        )
        .expect("AutoDodge actual collider ray");
        (!result.miss).then(|| point(result.hit.location))
    })
}
fn arrows() -> Vec<model::Arrow> {
    world::read(|g| {
        let mut arrows = Vec::new();
        for p in &g.creatures.projectiles {
            if matches!(
                p.kind,
                horizon_client::projectiles::ProjectileKind::Arrow { .. }
                    | horizon_client::projectiles::ProjectileKind::Trident
            ) && !p.arrow.as_ref().is_some_and(|a| a.in_ground)
            {
                arrows.push((
                    p.join,
                    model::Arrow {
                        id: p.id,
                        position: point(p.position),
                        velocity: point(p.velocity),
                        stopped: false,
                    },
                ));
            }
        }
        for p in &g.tridents {
            if !p.in_ground && p.returning == 0 {
                arrows.push((
                    p.join,
                    model::Arrow {
                        id: p.id,
                        position: point(p.position),
                        velocity: point(p.velocity),
                        stopped: false,
                    },
                ));
            }
        }
        arrows.sort_by_key(|(join, _)| *join);
        arrows.into_iter().map(|(_, a)| a).collect()
    })
}
fn pos(v: &Value) -> Option<V> {
    let a = v.as_array()?;
    if a.len() != 3 {
        return None;
    }
    Some([a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?])
}
impl Game<'_> {
    pub(super) fn auto_dodge_query(&mut self, text: &str) -> String {
        let Ok(v) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        if self.player.is_none() {
            return "null".into();
        }
        world::with_scope(self, || {
            let arrows = arrows();
            if arrows.is_empty() {
                return "null".into();
            }
            if v["mode"] == "blink" {
                let positions: Vec<V> = v["positions"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(pos)
                    .collect();
                let threatened = positions.first().is_some_and(|p| {
                    model::inflicted(std::iter::repeat_n(*p, 40), &arrows, 0.7, clip).is_some()
                });
                if !threatened {
                    return json!({"threatened":false}).to_string();
                }
                let player = world::read(|g| point(g.player.as_ref().unwrap().position));
                let evading = model::avoiding(&positions, player, |p| {
                    model::inflicted(std::iter::repeat_n(p, 40), &arrows, 0.7, clip)
                });
                return json!({"threatened":true,"index":evading.map(|p|p.0),"impact":evading.is_some_and(|p|p.1)}).to_string();
            }
            let ignore = v["ignore"].as_u64().unwrap_or(7) as u32;
            let blocked = world::read(|g| {
                let inventory =
                    matches!(g.screen, super::Screen::Inventory | super::Screen::Creative);
                (ignore & 1 == 0 && (inventory || v["inventoryOpen"] == true))
                    || (ignore & 2 == 0 && g.using.is_some())
                    || (ignore & 4 == 0 && v["scaffold"] == true)
            });
            if blocked {
                return "null".into();
            }
            let epoch = world::read(world::current_epoch);
            let registry = world::read(NavigationRegistry::capture);
            let player = NativePlayer::new(NativeLevel::new(registry, epoch));
            let keys = &v["keys"];
            let key = |i: usize| keys[i] == true;
            let yaw = world::read(|g| g.yaw);
            let (jump, shift, sprint) = world::read(|g| {
                let p = g.player.as_ref().unwrap();
                use horizon_client::actions::Action;
                (
                    g.keys.held(Action::Jump),
                    p.shift_key_down,
                    p.sprinting || g.keys.held(Action::Sprint),
                )
            });
            let input = Input {
                forward: key(0),
                backward: key(1),
                left: key(2),
                right: key(3),
                jump,
                shift,
                sprint,
            };
            let mut input = SimulatedInput::new(input);
            input.force_stay_on_ground_surface =
                horizon_client::navigation_physics::staying_on_surface(false);
            let mut simulation = NativeSimulation::new(&player, input, yaw);
            let positions = (0..80).map(|_| {
                simulation.tick();
                let p = simulation.predicted_position();
                [p.x, p.y, p.z]
            });
            let Some(hit) = model::inflicted(positions, &arrows, 1.5, clip) else {
                return "null".into();
            };
            let player_pos = player.position();
            let velocity = player.motion();
            let current_yaw = v["currentYaw"].as_f64().map_or(yaw, |x| x as f32);
            let walk = |base: V, to: V| {
                [0.6, 1.6]
                    .into_iter()
                    .map(|h| {
                        let from = [base[0], player_pos.y + h, base[2]];
                        let to = [to[0], player_pos.y + h, to[2]];
                        model::length(model::sub(clip(from, to).unwrap_or(to), from))
                    })
                    .fold(f64::INFINITY, f64::min)
            };
            let plan = model::plan(
                [player_pos.x, player_pos.y, player_pos.z],
                [velocity.x, velocity.y, velocity.z],
                yaw,
                current_yaw,
                v["rotation"] == true,
                hit,
                walk,
                |angle| super::client_rotation_direction(angle, 0.),
            );
            let Some(plan) = plan else {
                return "null".into();
            };
            if let Some(yaw) = plan.yaw {
                world::write(|g| {
                    g.yaw = yaw;
                    if let Some(p) = g.player.as_mut() {
                        p.yaw = yaw;
                    }
                });
            }
            json!({"keys":plan.keys,"jump":plan.jump&&player.on_ground(),"timer":plan.timer,"hitTick":hit.tick,"arrow":hit.id}).to_string()
        })
    }
}
