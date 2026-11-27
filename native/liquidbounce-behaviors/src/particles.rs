// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned ModuleParticles: attack batches, point-cell collisions and camera billboards.
use serde_json::{Value, json};
pub const TEXTURES: [&str; 11] = [
    "Orbiz",
    "Star",
    "Dollar",
    "Crown",
    "Heart",
    "Lightning",
    "Line",
    "Point",
    "Rhombus",
    "Snowflake",
    "Spark",
];
pub const FILES: [&str; 11] = [
    "glow",
    "star",
    "dollar",
    "crown",
    "heart",
    "lightning",
    "line",
    "point",
    "rhombus",
    "snowflake",
    "spark",
];
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub size: f32,
    pub count: [i32; 2],
    pub rotate: bool,
    pub motion: f32,
    pub bounce: [f32; 3],
    pub drag: f32,
    pub gravity: f32,
    pub color: i32,
    pub textures: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            size: 1.,
            count: [2, 10],
            rotate: true,
            motion: 15.,
            bounce: [0.8, 0.6, 0.8],
            drag: 0.99,
            gravity: 0.8,
            color: 0xff4677ffu32 as i32,
            textures: 2,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Particle {
    pub position: [f64; 3],
    pub previous: [f64; 3],
    pub velocity: [f64; 3],
    pub alpha: f32,
    pub rotation: f32,
    pub texture: usize,
    pub spawned: i64,
    pub collision: Option<i64>,
}
#[derive(Clone, Debug)]
pub struct Particles {
    pub settings: Settings,
    pub particles: Vec<Particle>,
    pub last_attack: i64,
    pub rng: u64,
}
impl Default for Particles {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            particles: vec![],
            last_attack: 0,
            rng: 0x98bb8d112383fa57,
        }
    }
}
impl Particles {
    pub fn clear(&mut self) {
        self.particles.clear();
    }
    fn random(&mut self) -> f64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        (x >> 11) as f64 / 9007199254740992.
    }
    pub fn attack(&mut self, now: i64, eye: [f64; 3], distance: f64, direction: [f64; 3]) -> usize {
        // Chronometer defaults lastUpdate=0; hasElapsed is strictly greater.
        if now <= self.last_attack.saturating_add(230) {
            return 0;
        }
        let choices = (0..TEXTURES.len())
            .filter(|i| self.settings.textures & (1 << i) != 0)
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return 0;
        }
        self.last_attack = now;
        let position = std::array::from_fn(|i| distance.mul_add(direction[i], eye[i]));
        let [lo, hi] = self.settings.count;
        let count = lo + (self.random() * f64::from(hi - lo + 1)) as i32;
        for _ in 0..count {
            let texture = choices[(self.random() * choices.len() as f64) as usize];
            let velocity = [
                -0.01 + self.random() * 0.02,
                0.01 + self.random() * 0.01,
                -0.01 + self.random() * 0.02,
            ];
            let rotation = (self.random() as f32) * 360.;
            self.particles.push(Particle {
                position,
                previous: position,
                velocity,
                alpha: 1.,
                rotation,
                texture,
                spawned: now,
                collision: None,
            });
        }
        count as usize
    }
    pub fn tick(&mut self, now: i64, camera: [f64; 3], mut solid: impl FnMut([i32; 3]) -> bool) {
        let s = &self.settings;
        self.particles.retain_mut(|p| {
            let distance = (0..3)
                .map(|i| (camera[i] - p.position[i]).powi(2))
                .sum::<f64>();
            if p.alpha <= 0. || distance >= 900. {
                return false;
            }
            p.previous = p.position;
            if let Some(collision) = p.collision {
                p.alpha = (1. - (now - collision) as f32 / 3000.).max(0.);
            }
            p.velocity[1] -= f64::from(s.gravity) * 0.03125;
            let step = |v: [f64; 3]| [v[0] * f64::from(s.motion), v[1], v[2] * f64::from(s.motion)];
            let add = |offset: [f64; 3]| std::array::from_fn(|i| p.position[i] + offset[i]);
            let cell = |pos: [f64; 3]| pos.map(|v| v.floor() as i32);
            let mut next = add(step(p.velocity));
            if solid(cell(next)) {
                if p.collision.is_none() {
                    p.collision = Some(now);
                }
                if solid(cell(add([p.velocity[0] * f64::from(s.motion), 0., 0.]))) {
                    p.velocity[0] = -p.velocity[0] * f64::from(s.bounce[0]);
                } else if solid(cell(add([0., p.velocity[1], 0.]))) {
                    p.velocity = [
                        p.velocity[0] * f64::from(s.drag),
                        -p.velocity[1] * f64::from(s.bounce[1]),
                        p.velocity[2] * f64::from(s.drag),
                    ];
                } else if solid(cell(add([0., 0., p.velocity[2] * f64::from(s.motion)]))) {
                    p.velocity[2] = -p.velocity[2] * f64::from(s.bounce[2]);
                }
                next = add(step(p.velocity));
            }
            p.position = next;
            true
        });
    }
    /// Camera rotation basis is supplied by the actual Camera quaternion.
    pub fn geometry(&self, now: i64, partial: f32, right: [f32; 3], up: [f32; 3]) -> Value {
        let s = &self.settings;
        let rgba = (s.color as u32).to_be_bytes();
        let mut images = vec![];
        for p in &self.particles {
            let size = s.size * 0.25 * (1. - (now - p.spawned) as f32 / 12000.);
            let rotation = if s.rotate {
                (p.rotation + 90.) % 360.
            } else {
                90.
            };
            let angle = rotation * std::f32::consts::PI / 180.;
            let (sin, cos) = angle.sin_cos();
            let center: [f64; 3] = std::array::from_fn(|i| {
                p.previous[i] + (p.position[i] - p.previous[i]) * f64::from(partial)
            });
            // T(-size/2,-size/2) * Camera * S(-1,1,-1) * Rz * T(size/2,size/2).
            let corners = [[0., size], [0., 0.], [size, 0.], [size, size]].map(|[x, y]| {
                let x = x + size / 2.;
                let y = y + size / 2.;
                let local_x = -(cos * x - sin * y);
                let local_y = sin * x + cos * y;
                std::array::from_fn::<_, 3, _>(|i| {
                    center[i] + f64::from(right[i] * local_x + up[i] * local_y)
                        - if i < 2 { f64::from(size) / 2. } else { 0. }
                })
            });
            let alpha = (p.alpha * f32::from(rgba[0])) as i32;
            let color = [
                f32::from(rgba[1]) / 255.,
                f32::from(rgba[2]) / 255.,
                f32::from(rgba[3]) / 255.,
                alpha.clamp(0, i32::from(rgba[0])) as f32 / 255.,
            ];
            images.push(json!({"texture":p.texture,"corners":corners,"color":color}));
        }
        json!({"images":images})
    }
}
