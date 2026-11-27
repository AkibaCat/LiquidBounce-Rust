// SPDX-License-Identifier: GPL-3.0-or-later
//! All six ModuleHats modes, LiquidBounce ae0eb396.
use crate::breadcrumbs::{Breadcrumbs, ColoredPoint, Effect, rgba};
#[path = "tracers_curve.rs"]
mod curve;
pub const MODES: [&str; 6] = ["Cone", "Halo", "Orbs", "Flower", "Star", "Image"];
const TAU: f32 = std::f32::consts::TAU;
const TABLE: &[u8; 262144] = include_bytes!("../vendor/minecraft-math/sine-table.bin");
fn fast_sin(x: f64) -> f32 {
    let i = ((x * 10430.378350470453) as i64 & 65535) as usize * 4;
    f32::from_le_bytes(TABLE[i..i + 4].try_into().unwrap())
}
fn fast_cos(x: f32) -> f32 {
    let i = ((f64::from(x) * 10430.378350470453 + 16384.) as i64 & 65535) as usize * 4;
    f32::from_le_bytes(TABLE[i..i + 4].try_into().unwrap())
}
fn angle(i: usize, n: usize) -> f32 {
    i as f32 * TAU / n as f32
}
fn sin(x: f32) -> f32 {
    f64::from(x).sin() as f32
}
fn cos(x: f32) -> f32 {
    f64::from(x).cos() as f32
}
pub fn rotation_angle(ms: u64, speed: f32) -> f32 {
    if speed.abs() < 0.00001 {
        0.
    } else {
        (ms % 360000) as f32 * 0.001 * speed
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Colors {
    pub sync: bool,
    pub first: i32,
    pub second: i32,
    pub spin: f32,
}
impl Default for Colors {
    fn default() -> Self {
        Self {
            sync: true,
            first: 0x7d4677ff,
            second: 0x7da2bbff,
            spin: 1.,
        }
    }
}
impl Colors {
    pub fn at(&self, a: f32, hurt: bool, ms: u64) -> [f32; 4] {
        if hurt {
            return rgba(self.first as u32 & 0xff000000 | 0xff0000);
        }
        if self.sync {
            return rgba(self.first as u32);
        }
        let offset = ((ms as f64 / 10000. * f64::from(self.spin)) % 1.) * f64::from(TAU);
        let t = fast_sin(f64::from(a) + offset) * 0.5 + 0.5;
        let a = self.first as u32;
        let b = self.second as u32;
        let channel = |shift: u32| {
            let x = ((a >> shift) & 255u32) as f32;
            let y = ((b >> shift) & 255u32) as f32;
            (t.mul_add(y - x, x) as i32).clamp(0, 255) as u32
        };
        rgba((channel(24) << 24) | (channel(16) << 16) | (channel(8) << 8) | channel(0))
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ModeSettings {
    pub follow_rotation: bool,
    pub armor_offset: f32,
    pub show_damage: bool,
    pub view_on_friend: bool,
    pub distance: i32,
    pub first_person: bool,
    pub radius: f32,
    pub peak: f32,
    pub thickness: f32,
    pub sharpness: f32,
    pub count: i32,
    pub speed: f32,
    pub size: f32,
    pub spin: f32,
    pub wave: bool,
    pub wave_height: f32,
    pub wave_speed: f32,
    pub colors: Colors,
    pub color: i32,
    pub image: String,
    pub scale: [f32; 2],
}
impl ModeSettings {
    pub fn for_mode(i: usize) -> Self {
        Self {
            follow_rotation: false,
            armor_offset: 0.1,
            show_damage: true,
            view_on_friend: true,
            distance: 64,
            first_person: true,
            radius: match i {
                0 => 0.6,
                2 => 0.5,
                _ => 0.3,
            },
            peak: 0.3,
            thickness: 0.05,
            sharpness: 0.6,
            count: if i == 2 { 6 } else { 5 },
            speed: 0.5,
            size: 0.1,
            spin: if i == 2 { 2. } else { 1. },
            wave: true,
            wave_height: 0.1,
            wave_speed: 2.,
            colors: Colors::default(),
            color: if i == 5 { -1 } else { 0x7d4677ff },
            image: String::new(),
            scale: [1., 1.],
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Hats {
    pub mode: usize,
    pub modes: [ModeSettings; 6],
    pub height: Vec<[f32; 2]>,
    pub period: i32,
    pub symmetric: bool,
    pub render: Breadcrumbs,
}
impl Default for Hats {
    fn default() -> Self {
        Self {
            mode: 0,
            modes: std::array::from_fn(ModeSettings::for_mode),
            height: vec![[0., 0.2], [1., 0.2]],
            period: 1000,
            symmetric: true,
            render: Breadcrumbs::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Entity {
    pub local: bool,
    pub friend: bool,
    pub position: [f64; 3],
    pub interpolated: [f64; 3],
    pub rotation: [f32; 2],
    pub height: f32,
    pub eye_height: f32,
    pub armor: bool,
    pub hurt: bool,
}
#[derive(Default)]
pub struct Mesh {
    pub quads: Vec<[ColoredPoint; 4]>,
    pub image_quads: Vec<[[f64; 3]; 4]>,
}
impl Hats {
    pub fn height(&self, elapsed_ms: u64) -> f32 {
        let p = self.period.max(1) as u64;
        let mut t = if self.symmetric {
            (elapsed_ms % (p * 2)) as f32 / p as f32
        } else {
            (elapsed_ms % p) as f32 / p as f32
        };
        if t > 1. {
            t = 2. - t;
        }
        curve::sample(&self.height, t)
    }
    pub fn effect(&self) -> Option<Effect> {
        if self.mode == 5 {
            None
        } else {
            self.render.effect()
        }
    }
    pub fn wears(&self, e: &Entity, local: [f64; 3], first_person: bool, free_look: bool) -> bool {
        let s = &self.modes[self.mode];
        if e.local {
            !first_person || s.first_person || free_look
        } else {
            s.view_on_friend
                && e.friend
                && e.position
                    .iter()
                    .zip(local)
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f64>()
                    < (s.distance as f64).powi(2)
        }
    }
    pub fn mesh(
        &self,
        entities: &[Entity],
        local: [f64; 3],
        first_person: bool,
        free_look: bool,
        elapsed_ms: u64,
        epoch_ms: u64,
    ) -> Mesh {
        let s = &self.modes[self.mode];
        let mut out = Mesh::default();
        let height = self.height(elapsed_ms);
        for e in entities
            .iter()
            .filter(|e| self.wears(e, local, first_person, free_look))
        {
            let offset =
                e.height - e.eye_height + height + if e.armor { s.armor_offset } else { 0. };
            let world = |p: [f32; 3]| {
                let mut p = [p[0], p[1] + offset, p[2]];
                if s.follow_rotation {
                    let y = std::f32::consts::PI - e.rotation[0].to_radians();
                    let x = -e.rotation[1].to_radians();
                    let q = [
                        p[0],
                        cos(x) * p[1] - sin(x) * p[2],
                        sin(x) * p[1] + cos(x) * p[2],
                    ];
                    p = [
                        cos(y) * q[0] + sin(y) * q[2],
                        q[1],
                        -sin(y) * q[0] + cos(y) * q[2],
                    ];
                }
                [
                    e.interpolated[0] + f64::from(p[0]),
                    e.interpolated[1] + f64::from(e.eye_height) + f64::from(p[1]),
                    e.interpolated[2] + f64::from(p[2]),
                ]
            };
            let spin = rotation_angle(epoch_ms, s.spin);
            let hurt = e.hurt && s.show_damage;
            let rotate = |p: [f32; 3]| {
                if spin.abs() < 0.00001 {
                    p
                } else {
                    [
                        cos(spin) * p[0] + sin(spin) * p[2],
                        p[1],
                        -sin(spin) * p[0] + cos(spin) * p[2],
                    ]
                }
            };
            let mut quad = |p: [[f32; 3]; 4], mut color: [f32; 4]| {
                if self.render.shader {
                    color[3] = 1.;
                }
                out.quads.push(p.map(|p| ColoredPoint {
                    position: world(p),
                    color,
                }))
            };
            match self.mode {
                0 => {
                    for i in 0..128 {
                        let a = angle(i, 128);
                        let b = angle(i + 1, 128);
                        quad(
                            [
                                [cos(a) * s.radius, 0., sin(a) * s.radius],
                                [cos(b) * s.radius, 0., sin(b) * s.radius],
                                [0., s.peak, 0.],
                                [0., s.peak, 0.],
                            ],
                            s.colors.at(a, hurt, epoch_ms),
                        );
                    }
                }
                1 | 3 | 4 => {
                    let segments = if self.mode == 1 {
                        144
                    } else {
                        s.count.max(0) as usize * 32
                    };
                    let radius = |a: f32| match self.mode {
                        3 => {
                            let inner = s.radius * s.sharpness;
                            let f = std::f32::consts::PI / s.count as f32;
                            let r = ((a % (f * 2.)) - f).abs() / f;
                            inner + (s.radius - inner) * (1. - r)
                        }
                        4 => {
                            let section = TAU / s.count as f32;
                            let m = (a % section) / section;
                            let linear = 1. - (m * 2. - 1.).abs();
                            s.radius * (1. - s.sharpness + s.sharpness * linear.powf(1.75))
                        }
                        _ => s.radius,
                    };
                    for i in 0..segments {
                        let a = angle(i, segments);
                        let b = angle(i + 1, segments);
                        let r = radius(a);
                        let n = radius(b);
                        let color = s.colors.at(a, hurt, epoch_ms);
                        for j in 0..12 {
                            let u = angle(j, 12);
                            let v = angle(j + 1, 12);
                            let y = s.thickness * fast_sin(f64::from(u));
                            let ny = s.thickness * fast_sin(f64::from(v));
                            let off = s.thickness * fast_cos(u);
                            let noff = s.thickness * fast_cos(v);
                            let p = [
                                [
                                    (n + off) * fast_sin(f64::from(b)),
                                    y,
                                    (n + off) * fast_cos(b),
                                ],
                                [
                                    (r + off) * fast_sin(f64::from(a)),
                                    y,
                                    (r + off) * fast_cos(a),
                                ],
                                [
                                    (r + noff) * fast_sin(f64::from(a)),
                                    ny,
                                    (r + noff) * fast_cos(a),
                                ],
                                [
                                    (n + noff) * fast_sin(f64::from(b)),
                                    ny,
                                    (n + noff) * fast_cos(b),
                                ],
                            ];
                            quad(if self.mode == 1 { p } else { p.map(rotate) }, color);
                        }
                    }
                }
                2 => {
                    let time = (epoch_ms % 1000000) as f32 / 1000. * s.speed;
                    let color = if hurt {
                        rgba(s.color as u32 & 0xff000000 | 0xff0000)
                    } else {
                        rgba(s.color as u32)
                    };
                    let a = fast_sin(f64::from(spin)) * s.size;
                    let b = fast_cos(spin) * s.size;
                    for i in 0..s.count.max(0) as usize {
                        let an = angle(i, s.count as usize) + time;
                        let x = sin(an) * s.radius;
                        let z = cos(an) * s.radius;
                        let y = if s.wave {
                            sin(time * s.wave_speed + i as f32) * s.wave_height
                        } else {
                            0.
                        };
                        let ring = [
                            [x + a, y, z + b],
                            [x + b, y, z - a],
                            [x - a, y, z - b],
                            [x - b, y, z + a],
                        ];
                        for h in [y + s.size, y - s.size] {
                            for (a, b) in [(3, 0), (0, 1), (1, 2), (2, 3)] {
                                quad([[x, h, z], ring[a], ring[b], ring[b]], color);
                            }
                        }
                    }
                }
                5 => {
                    let a = rotation_angle(epoch_ms, s.spin);
                    let rx = std::f32::consts::FRAC_PI_2;
                    out.image_quads
                        .push(
                            [[-0.5, -0.5], [-0.5, 0.5], [0.5, 0.5], [0.5, -0.5]].map(|p| {
                                let x = p[0] * s.scale[0];
                                let y = p[1] * s.scale[1];
                                let u = cos(a) * x - sin(a) * y;
                                let v = sin(a) * x + cos(a) * y;
                                world([u, cos(rx) * v, sin(rx) * v])
                            }),
                        );
                }
                _ => {}
            }
        }
        out
    }
}
