// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from ModuleDamageParticles.kt, Copyright (c) 2015-2026 CCBlueX.
use crate::easing::Easing;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrackMode {
    #[default]
    OnTick,
    OnUpdate,
}
impl TrackMode {
    pub fn tag(self) -> &'static str {
        match self {
            Self::OnTick => "OnTick",
            Self::OnUpdate => "OnUpdate",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entity {
    pub identity: u64,
    pub own: bool,
    pub ticked: bool,
    pub dead: bool,
    pub health: f32,
    pub max_health: f32,
    pub center: [f64; 3],
    pub movement: [f64; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub struct Particle {
    pub start: i64,
    pub text: String,
    pub color: i32,
    pub position: [f64; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub struct Draw {
    pub text: String,
    pub color: i32,
    pub position: [f64; 3],
    pub scale: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DamageParticles {
    pub ttl: f32,
    pub scale: f32,
    pub scale_transition: Easing,
    pub displacement: [f64; 3],
    pub displacement_transition: Easing,
    pub track_mode: TrackMode,
    pub damage: i32,
    pub death: i32,
    pub heal: i32,
    pub max_health: i32,
    pub decimal_separator: String,
    particles: VecDeque<Particle>,
    health: BTreeMap<u64, f32>,
}
impl Default for DamageParticles {
    fn default() -> Self {
        Self {
            ttl: 3.,
            scale: 2.,
            scale_transition: Easing::QuadOut,
            displacement: [0., 1.5, 0.],
            displacement_transition: Easing::QuadOut,
            track_mode: TrackMode::OnTick,
            damage: 0xffff0000u32 as i32,
            death: 0xffff0000u32 as i32,
            heal: 0xff00ff00u32 as i32,
            max_health: 0xff00ff00u32 as i32,
            decimal_separator: ".".into(),
            particles: VecDeque::new(),
            health: BTreeMap::new(),
        }
    }
}
/// DecimalFormat("0.#") promotes a Float to its exact double value, with HALF_EVEN rounding.
pub fn number(delta: f32, decimal: &str) -> String {
    if delta.is_infinite() {
        return "∞".into();
    }
    let mut text = format!("{:.1}", f64::from(delta));
    if text.ends_with(".0") {
        text.truncate(text.len() - 2);
    } else if decimal != "." {
        text = text.replace('.', decimal);
    }
    text
}
impl DamageParticles {
    pub fn clear(&mut self) {
        self.particles.clear();
        self.health.clear();
    }
    pub fn particles(&self) -> &VecDeque<Particle> {
        &self.particles
    }
    fn track(&mut self, now: i64, e: Entity, old: f32, new: f32, max: f32) {
        let delta = (old - new).abs();
        if delta <= 0.05 || delta.is_nan() {
            return;
        }
        let color = if new <= 0. {
            self.death
        } else if old > new {
            self.damage
        } else if new < max {
            self.heal
        } else {
            self.max_health
        };
        self.particles.push_back(Particle {
            start: now,
            text: number(delta, &self.decimal_separator),
            color,
            position: std::array::from_fn(|i| e.center[i] + e.movement[i]),
        });
    }
    pub fn update(&mut self, now: i64, e: Entity, old: f32, new: f32, max: f32) {
        if self.track_mode == TrackMode::OnUpdate && !e.own && e.ticked {
            self.track(now, e, old, new, max);
        }
    }
    pub fn tick(&mut self, now: i64, entities: &[Entity]) {
        let earliest = now.wrapping_sub((self.ttl * 1000.) as i64);
        while self.particles.front().is_some_and(|p| p.start < earliest) {
            self.particles.pop_front();
        }
        if self.track_mode != TrackMode::OnTick {
            return;
        }
        for &e in entities {
            if e.own || !e.ticked {
                continue;
            }
            let old = self.health.insert(e.identity, e.health).unwrap_or(0.);
            if old != 0. {
                self.track(now, e, old, e.health, e.max_health);
            }
        }
        self.health
            .retain(|id, _| entities.iter().any(|e| e.identity == *id && !e.dead));
    }
    pub fn render(&self, now: i64) -> Vec<Draw> {
        self.particles
            .iter()
            .map(|p| {
                let progress = now.wrapping_sub(p.start) as f32 / (self.ttl * 1000.);
                let displacement = f64::from(self.displacement_transition.transform(progress));
                Draw {
                    text: p.text.clone(),
                    color: p.color,
                    position: std::array::from_fn(|i| {
                        displacement.mul_add(self.displacement[i], p.position[i])
                    }),
                    scale: self.scale * self.scale_transition.transform(progress),
                }
            })
            .collect()
    }
}
