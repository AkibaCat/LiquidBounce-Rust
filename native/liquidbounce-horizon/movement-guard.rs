// SPDX-License-Identifier: GPL-3.0-or-later
//! Real world, body prediction and HoleTracker adapter for Anchor/AvoidHazards.
use super::*;
use super::{
    auto_walk_player::NativePlayer,
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NativeState, NavigationRegistry},
};
use crate::navigation::{
    aabb::Aabb as NavBox,
    geometry::{BlockPos as Pos, Vec3 as V},
    input::Input,
    terrain::{TerrainLevel, TerrainState},
    walk_control::body::{SimulatedInput, WalkPlayer, WalkSimulation},
};
use crate::{
    anchor::{self, Anchor},
    avoid_hazards::{self as hazards, Flags},
};
use serde_json::Value;
use std::cell::RefCell;
thread_local! {
 static CONFIG:RefCell<Value>=const{RefCell::new(Value::Null)};
 static LEVEL:RefCell<Option<NativeLevel>>=const{RefCell::new(None)};
 static FLAGS_READY:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};
}
fn vec(v: V) -> [f64; 3] {
    [v.x, v.y, v.z]
}
fn flags(s: &NativeState) -> Flags {
    let mut bits = 0;
    for (bit, class) in [
        (0, "CactusBlock"),
        (1, "SweetBerryBushBlock"),
        (2, "FireBlock"),
        (3, "WebBlock"),
        (5, "BasePressurePlateBlock"),
        (8, "WitherRoseBlock"),
        (9, "PowderSnowBlock"),
    ] {
        if s.has_class(class) {
            bits |= 1 << bit;
        }
    }
    if s.facts().fluid == Fluid::Lava {
        bits |= 1 << 7;
    }
    Flags {
        hazards: bits,
        ladder: s.has_class("LadderBlock"),
        open_trapdoor: s.has_class("TrapDoorBlock") && s.property("open") == Some("true"),
        facing: match s.property("facing") {
            Some("north") => 1,
            Some("south") => 2,
            Some("west") => 3,
            Some("east") => 4,
            _ => 0,
        },
        magma: s.has_class("MagmaBlock"),
    }
}
fn key(k: [bool; 7]) -> Input {
    Input {
        forward: k[0],
        backward: k[1],
        left: k[2],
        right: k[3],
        jump: k[4],
        shift: k[5],
        sprint: k[6],
    }
}
fn bounds(player: &NativePlayer) -> NavBox {
    player.read(|p| super::auto_walk_player::box_(p.bounds()))
}
fn ladder(player: &NativePlayer) -> bool {
    let p = Pos::containing(player.position());
    let level = &player.inventory.level;
    hazards::ladder(
        flags(&level.block_state(p)),
        flags(&level.block_state(Pos::new(p.x, p.y - 1, p.z))),
    )
}
fn intersects(box_: NavBox, p: Pos, shape: &[[f64; 6]]) -> bool {
    shape.iter().any(|s| {
        NavBox::new(
            V::new(p.x as f64 + s[0], p.y as f64 + s[1], p.z as f64 + s[2]),
            V::new(p.x as f64 + s[3], p.y as f64 + s[4], p.z as f64 + s[5]),
        )
        .intersects(box_)
    })
}
fn hazard_collision(b: NavBox, level: &NativeLevel, mask: u32) -> bool {
    if mask & (1 << 6) != 0 {
        let lower = b.min.y - 0.2;
        let expanded = NavBox::new(V::new(b.min.x, lower, b.min.z), b.max);
        let y = lower.floor() as i32;
        for z in (b.min.z.floor() as i32)..=(b.max.z.floor() as i32) {
            for x in (b.min.x.floor() as i32)..=(b.max.x.floor() as i32) {
                let p = Pos::new(x, y, z);
                let s = level.block_state(p);
                if s.has_class("MagmaBlock")
                    && s.collision_shape(level, p).aabbs().iter().any(|shape| {
                        NavBox::new(
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
                        .intersects(expanded)
                    })
                {
                    return true;
                }
            }
        }
    }
    for z in (b.min.z.floor() as i32)..=(b.max.z.floor() as i32) {
        for y in (b.min.y.floor() as i32)..=(b.max.y.floor() as i32) {
            for x in (b.min.x.floor() as i32)..=(b.max.x.floor() as i32) {
                let p = Pos::new(x, y, z);
                let s = level.block_state(p);
                let f = flags(&s);
                let selected = f.hazards & mask;
                if selected & (1 << 7) != 0 {
                    let above = level.block_state(Pos::new(x, y + 1, z));
                    let height = if above.facts().fluid == Fluid::Lava {
                        1.
                    } else {
                        f64::from(f32::from(s.facts().fluid_level) / 9.)
                    };
                    if intersects(b, p, &[[0., 0., 0., 1., height, 1.]]) {
                        return true;
                    }
                }
                if selected & 1 != 0
                    && intersects(
                        NavBox::new(
                            V::new(b.min.x - 0.001, b.min.y, b.min.z - 0.001),
                            V::new(b.max.x + 0.001, b.max.y, b.max.z + 0.001),
                        ),
                        p,
                        &[[0., 0., 0., 1., 1., 1.]],
                    )
                {
                    return true;
                }
                let normal = (selected & !((1 << 7) | 1)) != 0
                    || mask & (1 << 4) != 0
                        && hazards::ladder(f, flags(&level.block_state(Pos::new(x, y - 1, z))));
                if normal && intersects(b, p, &s.facts().outline) {
                    return true;
                }
            }
        }
    }
    false
}
fn safe(player: &NativePlayer, k: Input, yaw: f32, mask: u32) -> bool {
    let mut input = SimulatedInput::new(k);
    input.keys.sprint = player.read(|p| p.sprinting);
    let mut simulation = NativeSimulation::new(player, input, yaw);
    let mut previous = bounds(player);
    let started = mask & (1 << 4) != 0 && ladder(player);
    for _ in 0..2 {
        simulation.tick();
        let current = bounds(simulation.player());
        let swept = NavBox::new(
            V::new(
                previous.min.x.min(current.min.x),
                previous.min.y.min(current.min.y),
                previous.min.z.min(current.min.z),
            ),
            V::new(
                previous.max.x.max(current.max.x),
                previous.max.y.max(current.max.y),
                previous.max.z.max(current.max.z),
            ),
        );
        if mask & (1 << 4) != 0 && !started && ladder(simulation.player())
            || hazard_collision(current, &player.inventory.level, mask)
            || hazard_collision(swept, &player.inventory.level, mask)
        {
            return false;
        }
        previous = current;
    }
    true
}
fn level(epoch: u64) -> NativeLevel {
    LEVEL.with(|l| {
        let mut l = l.borrow_mut();
        if l.as_ref().is_none_or(|l| l.epoch != epoch) {
            *l = Some(NativeLevel::new(
                world::read(NavigationRegistry::capture),
                epoch,
            ));
        }
        l.as_ref().expect("guard level").clone()
    })
}
pub(super) fn clear() {
    LEVEL.with(|l| *l.borrow_mut() = None);
    horizon_client::anchor_bridge::set(None);
    horizon_world::avoid_hazards::set_mask(0);
}
impl Game<'_> {
    pub(super) fn movement_guard_prepare(&mut self, value: &str) {
        let Ok(config) = serde_json::from_str::<Value>(value) else {
            return;
        };
        CONFIG.with(|c| *c.borrow_mut() = config.clone());
        let eligible = self.native_window()
            && self.lan_acting.is_none()
            && self.lan_acting_uuid.is_none()
            && self
                .player
                .as_ref()
                .is_some_and(|p| p.native_client_queries_allowed());
        if !eligible {
            clear();
            return;
        }
        let epoch = world::current_epoch(self);
        world::with_scope(self, || {
            let level = level(epoch);
            let a = &config["anchor"];
            let h = &config["hazards"];
            if !FLAGS_READY.get() {
                let flags = (0..level.registry.registry.len())
                    .map(|id| {
                        let f = flags(&level.registry.state(BlockStateId(id as u32)));
                        horizon_world::avoid_hazards_model::Flags {
                            hazards: f.hazards,
                            ladder: f.ladder,
                            open_trapdoor: f.open_trapdoor,
                            facing: f.facing,
                            magma: f.magma,
                        }
                    })
                    .collect();
                horizon_world::avoid_hazards::initialize(flags);
                FLAGS_READY.set(true);
            }
            let mask = if h["running"] == true && h["mode"] == "Shape" {
                h["avoid"].as_u64().unwrap_or(1023) as u32
            } else {
                0
            };
            horizon_world::avoid_hazards::set_mask(mask);
            let player = NativePlayer::new(level.clone());
            let allowed = a["running"] == true
                && player.read(|p| !p.fall_flying && !p.spectator && !p.swimming);
            let snapshot = if allowed {
                let settings = Anchor {
                    max_distance: a["distance"].as_f64().unwrap_or(4.) as f32,
                    horizontal_speed: a["horizontal"].as_f64().unwrap_or(0.3) as f32,
                    vertical_speed: a["vertical"].as_f64().unwrap_or(0.1) as f32,
                };
                let p = Pos::containing(player.position());
                let r = settings.max_distance.ceil() as i32;
                let holes = anchor::scan(
                    [p.x - r, (p.y - r).max(level.min_y()), p.z - r],
                    [p.x + r, p.y + r, p.z + r],
                    |p| {
                        let s = level.block_state(Pos::new(p[0], p[1], p[2]));
                        if !level.is_known(Pos::new(p[0], p[1], p[2])) {
                            anchor::Block::Other
                        } else if matches!(
                            s.name(),
                            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                        ) {
                            anchor::Block::Air
                        } else if s.facts().explosion_resistance >= 600. {
                            anchor::Block::Resistant
                        } else {
                            anchor::Block::Other
                        }
                    },
                );
                let b = bounds(&player);
                let bounds = [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z];
                settings
                    .goal(&holes, vec(player.position()), bounds)
                    .map(|goal| horizon_client::anchor_bridge::Snapshot { settings, goal })
            } else {
                None
            };
            horizon_client::anchor_bridge::set(snapshot);
        });
    }
    pub(super) fn movement_guard_input(&mut self, mut raw: [bool; 7]) -> [bool; 7] {
        if !self.native_window()
            || self.lan_acting.is_some()
            || self.lan_acting_uuid.is_some()
            || !self
                .player
                .as_ref()
                .is_some_and(|p| p.native_client_queries_allowed())
        {
            return raw;
        }
        let config = CONFIG.with(|c| c.borrow().clone());
        let h = &config["hazards"];
        let mask = h["avoid"].as_u64().unwrap_or(1023) as u32;
        if h["running"] != true || h["mode"] != "Input" || mask == 0 {
            return raw;
        }
        let epoch = world::current_epoch(self);
        world::with_scope(self, || {
            let player = NativePlayer::new(level(epoch));
            let yaw = config["movementYaw"]
                .as_f64()
                .map_or_else(|| world::read(|g| g.yaw), |y| y as f32);
            let original = key(raw);
            let chosen = hazards::choose([raw[0], raw[1], raw[2], raw[3]], |d| {
                safe(
                    &player,
                    Input {
                        forward: d[0],
                        backward: d[1],
                        left: d[2],
                        right: d[3],
                        ..original
                    },
                    yaw,
                    mask,
                )
            });
            raw[..4].copy_from_slice(&chosen);
            raw
        })
    }
}
