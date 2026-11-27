// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoPearl and its 240-tick trajectory continuation, LiquidBounce ae0eb396.
use crate::{
    auto_bow_math::{B, V},
    rotations::Rotation,
};
use std::collections::VecDeque;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Trigger,
    Target,
}
impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Trigger => "Trigger",
            Self::Target => "Target",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Trigger" => Some(Self::Trigger),
            "Target" => Some(Self::Target),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub limits: bool,
    pub angle: i32,
    pub min_distance: f32,
    pub destination_distance: f32,
    pub rotate: bool,
    pub combat_pause: i32,
    pub slot_reset: [i32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Trigger,
            limits: true,
            angle: 180,
            min_distance: 8.,
            destination_distance: 8.,
            rotate: true,
            combat_pause: 0,
            slot_reset: [0, 0],
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Spawn {
    pub owner: Option<i32>,
    pub owner_self: bool,
    pub owner_attackable: bool,
    pub aura_target: Option<i32>,
    pub crosshair_cos: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pending {
    pub rotation: Rotation,
    pub waited: i32,
}
#[derive(Default, Debug)]
pub struct AutoPearl {
    pub settings: Settings,
    pub queue: VecDeque<Rotation>,
    pub pending: Option<Pending>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Rotate(Rotation),
    Throw { slot: i32, rotation: Rotation },
}
impl AutoPearl {
    pub fn can_trigger(&self, e: Spawn) -> bool {
        if self.settings.limits
            && (f64::from(self.settings.angle) * std::f64::consts::PI / 180.).cos()
                > e.crosshair_cos
        {
            return false;
        }
        if e.owner.is_none() {
            return self.settings.mode == Mode::Trigger;
        }
        !e.owner_self
            && match self.settings.mode {
                Mode::Trigger => e.owner_attackable,
                Mode::Target => e.owner == e.aura_target,
            }
    }
    pub fn destination_allowed(&self, player: V, destination: V) -> bool {
        !self.settings.limits
            || destination.sub(player).len2() >= f64::from(self.settings.min_distance).powi(2)
    }
    pub fn throw_allowed(&self, desired: V, simulated: Option<V>) -> bool {
        simulated.is_some_and(|actual| {
            !self.settings.limits
                || desired.sub(actual).len2()
                    < f64::from(self.settings.destination_distance).powi(2)
        })
    }
    pub fn enqueue(&mut self, rotation: Rotation) {
        if self.queue.is_empty() {
            self.queue.push_back(rotation);
        }
    }
    pub fn rotation_update(&self) -> Option<Rotation> {
        self.queue.front().copied()
    }
    pub fn begin_tick(&mut self, slot: Option<i32>) -> Option<Effect> {
        if self.pending.is_some() {
            return None;
        }
        let rotation = self.queue.pop_front()?;
        let slot = slot?;
        if self.settings.rotate {
            self.pending = Some(Pending {
                rotation,
                waited: 0,
            });
            None
        } else {
            Some(Effect::Throw { slot, rotation })
        }
    }
    pub fn resume(&mut self, slot: Option<i32>, direction_angle: f32) -> (Option<Effect>, bool) {
        let Some(mut p) = self.pending else {
            return (None, false);
        };
        let Some(slot) = slot else {
            self.pending = None;
            return (None, false);
        };
        p.waited = p.waited.wrapping_add(1);
        let sufficient = direction_angle <= 1.;
        if sufficient || p.waited >= 20 {
            self.pending = None;
            return (
                if sufficient {
                    Some(Effect::Throw {
                        slot,
                        rotation: p.rotation,
                    })
                } else {
                    None
                },
                false,
            );
        }
        self.pending = Some(p);
        (Some(Effect::Rotate(p.rotation)), true)
    }
    pub fn disabled(&mut self) {
        self.queue.clear();
        self.pending = None;
    }
}
/// Real and hypothetical EnderPearl both apply the velocity-only first tick.
/// Block collisions have priority; living-owner entity margins are exactly zero.
pub fn simulate(
    mut pos: V,
    mut velocity: V,
    min_y: f64,
    entities: &[B],
    mut block: impl FnMut(V, V) -> Option<(bool, V)>,
    mut fluid: impl FnMut(V) -> bool,
) -> Option<V> {
    let physics = |pos: V, velocity: V, fluid: &mut dyn FnMut(V) -> bool| {
        velocity
            .scale(if fluid(pos) { 0.8 } else { 0.99 })
            .sub(V(0., 0.03, 0.))
    };
    velocity = physics(pos, velocity, &mut fluid);
    for _ in 1..240 {
        if pos.1 < min_y {
            return None;
        }
        let before = pos;
        pos = pos.add(velocity);
        let (miss, hit) = block(before, pos)?;
        if !miss {
            return Some(hit);
        }
        let broad = B {
            lo: V(before.0 - 0.25, before.1 - 0.25, before.2 - 0.25),
            hi: V(before.0 + 0.25, before.1 + 0.25, before.2 + 0.25),
        };
        let expanded = B {
            lo: V(
                broad.lo.0 + velocity.0.min(0.),
                broad.lo.1 + velocity.1.min(0.),
                broad.lo.2 + velocity.2.min(0.),
            ),
            hi: V(
                broad.hi.0 + velocity.0.max(0.),
                broad.hi.1 + velocity.1.max(0.),
                broad.hi.2 + velocity.2.max(0.),
            ),
        }
        .inflate(1.);
        let mut closest = before.sub(pos).len2();
        let mut entity_hit = None;
        for b in entities {
            if !b.intersects(expanded) {
                continue;
            }
            let at = if b.contains(before) {
                Some(before)
            } else {
                b.clip(before, pos)
            };
            if let Some(at) = at {
                let d = before.sub(at).len2();
                if d < closest || closest == 0. {
                    closest = d;
                    entity_hit = Some(at);
                }
            }
        }
        if entity_hit.is_some() {
            return entity_hit;
        }
        velocity = physics(pos, velocity, &mut fluid);
    }
    None
}
