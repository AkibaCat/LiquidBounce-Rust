// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleESP and all four selectable modes, LiquidBounce ae0eb396.
use crate::{
    breadcrumbs::{ColoredPoint, Effect, StyleSettings, rainbow, rgba},
    tracers::{ColorMode, Tracers},
};
pub const MODES: [&str; 4] = ["Box", "2D", "Legacy2D", "Glow"];
pub const OBJECTS: [&str; 3] = ["EndCrystals", "Projectiles", "Vehicles"];
pub const FILLS: [f32; 9] = [0.12, 0., 0.08, 0.1, 0.08, 0.18, 0., 0.08, 0.1];
pub type Point = [f64; 3];
pub type Bounds = [f64; 6];
#[derive(Clone, Debug, PartialEq)]
pub struct Prepared {
    pub identity: u64,
    pub bounds: Bounds,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Esp {
    pub mode: usize,
    pub color: Tracers,
    pub invisible: i32,
    pub friends: i32,
    pub maximum_distance: f32,
    pub objects: u8,
    pub object_color: i32,
    pub expand: [f32; 2],
    pub outline: bool,
    pub merge_intersecting: bool,
    pub fill: bool,
    pub outline_2d: bool,
    pub outline_thickness: f32,
    pub corner: bool,
    pub gap: f32,
    pub border: bool,
    pub border_thickness: f32,
    pub health_bar: bool,
    pub health_spacing: f32,
    pub scale: f32,
    pub y_offset: f32,
    pub background_alpha: i32,
    pub style: usize,
    pub styles: [StyleSettings; 9],
    pub selected: std::collections::BTreeSet<u64>,
    pub prepared: Vec<Prepared>,
    pub perspective: Option<i32>,
    pub true_sight: std::collections::BTreeSet<i32>,
}
impl Default for Esp {
    fn default() -> Self {
        let mut color = Tracers::default();
        color.color = 0x64ffffff;
        Self {
            mode: 3,
            color,
            invisible: 0xffffa500u32 as i32,
            friends: 0xff00ff00u32 as i32,
            maximum_distance: 128.,
            objects: 0,
            object_color: 0x64ffffff,
            expand: [0.05; 2],
            outline: true,
            merge_intersecting: false,
            fill: true,
            outline_2d: true,
            outline_thickness: 1.,
            corner: false,
            gap: 50.,
            border: true,
            border_thickness: 1.,
            health_bar: true,
            health_spacing: 2.,
            scale: 0.1,
            y_offset: 0.,
            background_alpha: 150,
            style: 1,
            styles: std::array::from_fn(|i| {
                let mut s = StyleSettings::new(i);
                s.values.insert("Fill".into(), FILLS[i]);
                s
            }),
            selected: Default::default(),
            prepared: vec![],
            perspective: None,
            true_sight: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Entity {
    pub identity: u64,
    pub id: i32,
    pub position: Point,
    pub interpolated: Point,
    pub width: f32,
    pub height: f32,
    pub bounding_height: f32,
    pub living: bool,
    pub player: bool,
    pub hurt: bool,
    pub invisible: bool,
    pub friend: bool,
    pub tag: Option<u32>,
    pub health: f32,
    pub max_health: f32,
    pub object: u8,
    pub own_vehicle: bool,
}
#[derive(Default)]
pub struct Mesh {
    pub quads: Vec<[ColoredPoint; 4]>,
    pub lines: Vec<[ColoredPoint; 2]>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub bounds: [f32; 4],
    pub color: u32,
}
pub fn distance_squared(a: Point, b: Point) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}
fn argb(c: [f32; 4]) -> u32 {
    let p = c.map(|x| (x * 255.) as u32);
    (p[3] << 24) | (p[0] << 16) | (p[1] << 8) | p[2]
}
fn hsb(h: f32, s: f32, b: f32, a: f32) -> u32 {
    let mut rgb = [b; 3];
    if s != 0. {
        let h = (h - h.floor()) * 6.;
        let f = h - h.floor();
        let p = b * (1. - s);
        let q = b * (1. - s * f);
        let t = b * (1. - s * (1. - f));
        rgb = match h as i32 {
            0 => [b, t, p],
            1 => [q, b, p],
            2 => [p, b, t],
            3 => [p, q, b],
            4 => [t, p, b],
            _ => [b, p, q],
        };
    }
    let [r, g, b] = rgb.map(|v| (v * 255. + 0.5) as u32);
    (((a * 255.) as u32) << 24) | (r << 16) | (g << 8) | b
}
impl Esp {
    pub fn clear(&mut self) {
        self.selected.clear();
        self.prepared.clear();
        self.perspective = None;
        self.true_sight.clear();
    }
    pub fn is_object(&self, e: &Entity) -> bool {
        !e.own_vehicle && self.objects & e.object != 0
    }
    pub fn within(&self, e: &Entity, camera: Point) -> bool {
        distance_squared(e.position, camera) < f64::from(self.maximum_distance).powi(2)
    }
    pub fn color(&self, e: &Entity, camera: Point, nanos: f64) -> u32 {
        if !e.living {
            return self.object_color as u32;
        }
        if e.hurt {
            return 0xffff0000;
        }
        if e.invisible {
            return self.invisible as u32;
        }
        if e.player {
            if e.friend && (self.friends as u32) >> 24 > 0 {
                return self.friends as u32;
            }
            if let Some(c) = e.tag {
                return c;
            }
        }
        let c = &self.color;
        match c.color_mode {
            ColorMode::Static => c.color as u32,
            ColorMode::Rainbow => {
                let mut c = rainbow(nanos);
                c[3] = 50. / 255.;
                argb(c)
            }
            ColorMode::Health => {
                let part = e.health.min(e.max_health) / e.max_health;
                let r = ((255. * (1. - part)) as i32).clamp(0, 255) as u32;
                let g = ((255. * part) as i32).clamp(0, 255) as u32;
                ((c.health_alpha as u32) << 24) | (r << 16) | (g << 8)
            }
            ColorMode::Distance => hsb(
                crate::tracers::curve::sample(
                    &c.hue,
                    distance_squared(e.position, camera).sqrt() as f32,
                ) / 360.,
                c.saturation,
                c.brightness,
                c.alpha,
            ),
        }
    }
    pub fn prepare(&mut self, entities: &[Entity], camera: Point) {
        self.prepared.clear();
        if self.mode > 1 {
            return;
        }
        let expand = f64::from(self.expand[self.mode]);
        for e in entities {
            if self.selected.contains(&e.identity) && self.within(e, camera) {
                let half = f64::from(e.width) / 2.;
                self.prepared.push(Prepared {
                    identity: e.identity,
                    bounds: [
                        -half - expand,
                        -expand,
                        -half - expand,
                        half + expand,
                        f64::from(e.height) + expand,
                        half + expand,
                    ],
                });
            }
        }
    }
    pub fn boxes(&self, entities: &[Entity], camera: Point, nanos: f64) -> Vec<(Bounds, u32)> {
        let mut boxes = vec![];
        for p in &self.prepared {
            if let Some(e) = entities.iter().find(|e| e.identity == p.identity) {
                boxes.push((
                    std::array::from_fn(|i| p.bounds[i] + e.interpolated[i % 3]),
                    self.color(e, camera, nanos),
                ));
            }
        }
        if self.mode == 0 && self.merge_intersecting {
            merge(boxes)
        } else {
            boxes
        }
    }
    pub fn effect(&self) -> Option<Effect> {
        if self.mode != 3 {
            return None;
        }
        Some(if self.style == 0 {
            Effect {
                style: 9,
                params: [0.; 8],
                accent: [1.; 4],
                range: 1.,
                opacity: 1.,
            }
        } else {
            self.styles[self.style - 1].effect(self.style - 1)
        })
    }
    pub fn box_mesh(&self, boxes: &[(Bounds, u32)]) -> Mesh {
        let mut mesh = Mesh::default();
        for (b, c) in boxes {
            let p = std::array::from_fn::<_, 8, _>(|i| {
                [
                    b[if i & 1 == 0 { 0 } else { 3 }],
                    b[if i & 2 == 0 { 1 } else { 4 }],
                    b[if i & 4 == 0 { 2 } else { 5 }],
                ]
            });
            let fill = rgba((c & 0x00ffffff) | 0x32000000);
            for face in [
                [0, 1, 3, 2],
                [4, 6, 7, 5],
                [0, 4, 5, 1],
                [2, 3, 7, 6],
                [0, 2, 6, 4],
                [1, 5, 7, 3],
            ] {
                mesh.quads.push(face.map(|i| ColoredPoint {
                    position: p[i],
                    color: fill,
                }));
            }
            if self.outline {
                let color = rgba((c & 0x00ffffff) | 0x64000000);
                for (a, b) in [
                    (0, 1),
                    (0, 2),
                    (0, 4),
                    (1, 3),
                    (1, 5),
                    (2, 3),
                    (2, 6),
                    (3, 7),
                    (4, 5),
                    (4, 6),
                    (5, 7),
                    (6, 7),
                ] {
                    mesh.lines.push([a, b].map(|i| ColoredPoint {
                        position: p[i],
                        color,
                    }));
                }
            }
        }
        mesh
    }
    pub fn legacy_mesh(&self, entities: &[Entity], camera: Point, yaw: f32, nanos: f64) -> Mesh {
        let mut mesh = Mesh::default();
        let radians = (-yaw).to_radians();
        let (sin, cos) = radians.sin_cos();
        let scale = self.scale;
        for e in entities {
            if !self.selected.contains(&e.identity) || !self.within(e, camera) {
                continue;
            }
            let color = rgba(self.color(e, camera, nanos));
            let bg = rgba((self.background_alpha as u32) << 24);
            for (bottom, background, rects) in [
                (false, false, &TOP_FG),
                (false, true, &TOP_BG),
                (true, false, &BOTTOM_FG),
                (true, true, &BOTTOM_BG),
            ] {
                for r in rects {
                    let points = [[r[2], r[1]], [r[0], r[1]], [r[0], r[3]], [r[2], r[3]]];
                    mesh.quads.push(points.map(|[x, y]| {
                        let x = -x * scale;
                        let y = -(y + if bottom {
                            21. - e.bounding_height * 12.
                        } else {
                            0.
                        }) * scale;
                        ColoredPoint {
                            position: [
                                e.interpolated[0] + f64::from(x * cos),
                                e.interpolated[1] + f64::from(self.y_offset + y),
                                e.interpolated[2] + f64::from(-x * sin),
                            ],
                            color: if background { bg } else { color },
                        }
                    }));
                }
            }
        }
        mesh
    }
    pub fn overlay(
        &self,
        rect: [f32; 4],
        color: u32,
        health: Option<(f32, f32)>,
        gui_scale: f32,
    ) -> Vec<Rect> {
        let [x, y, right, bottom] = rect;
        let w = right - x;
        let h = bottom - y;
        let o = self.outline_thickness / gui_scale;
        let b = self.border_thickness / gui_scale;
        let mut out = vec![];
        let push = |out: &mut Vec<Rect>, r: [f32; 4], color| {
            out.push(Rect {
                bounds: [r[0] + x, r[1] + y, r[2] + x, r[3] + y],
                color,
            })
        };
        if self.fill {
            push(&mut out, [0., 0., w, h], (color & 0xffffff) | 0x32000000);
        }
        if self.outline_2d {
            let mut segments = vec![];
            if self.corner && self.gap > 0. {
                let cw = w * (1. - (self.gap / 100.).clamp(0., 1.)) / 2.;
                let ch = h * (1. - (self.gap / 100.).clamp(0., 1.)) / 2.;
                segments.extend([
                    (true, [-o / 2., cw, -o / 2.]),
                    (false, [-o / 2., -o / 2., ch]),
                    (true, [w - cw, w + o / 2., -o / 2.]),
                    (false, [w - o / 2., -o / 2., ch]),
                    (true, [-o / 2., cw, h - o / 2.]),
                    (false, [-o / 2., h - ch, h + o / 2.]),
                    (true, [w - cw, w + o / 2., h - o / 2.]),
                    (false, [w - o / 2., h - ch, h + o / 2.]),
                ]);
            } else {
                segments.extend([
                    (true, [-o / 2., w + o / 2., -o / 2.]),
                    (true, [-o / 2., w + o / 2., h - o / 2.]),
                    (false, [-o / 2., -o / 2., h + o / 2.]),
                    (false, [w - o / 2., -o / 2., h + o / 2.]),
                ]);
            }
            if self.border {
                for (horizontal, s) in &segments {
                    let r = if *horizontal {
                        [s[0] - b, s[2] - b, s[1] + b, s[2] + o + b]
                    } else {
                        [s[0] - b, s[1] - b, s[0] + o + b, s[2] + b]
                    };
                    push(&mut out, r, 0xff000000);
                }
            }
            for (horizontal, s) in segments {
                let r = if horizontal {
                    [s[0], s[2], s[1], s[2] + o]
                } else {
                    [s[0], s[1], s[0] + o, s[2]]
                };
                push(&mut out, r, (color & 0xffffff) | 0xff000000);
            }
        }
        let shift = if self.outline_2d && self.border {
            -2. * b
        } else {
            0.
        } - self.health_spacing / gui_scale
            - o;
        if self.health_bar {
            if let Some((health, max_health)) = health {
                let percentage = (health / max_health.max(1.)).clamp(0., 1.);
                if self.border {
                    push(
                        &mut out,
                        [
                            shift - o / 2. - b,
                            -o / 2. - b,
                            shift + o / 2. + b,
                            h + o / 2. + b,
                        ],
                        0xff000000,
                    );
                }
                let r = percentage.mul_add(-255., 255.) as u32;
                let g = percentage.mul_add(255., 0.) as u32;
                push(
                    &mut out,
                    [
                        shift - o / 2.,
                        h - h * percentage - o / 2.,
                        shift + o / 2.,
                        h + o / 2.,
                    ],
                    0xff000000 | (r << 16) | (g << 8),
                );
            }
        }
        out
    }
}
/// Connected components use strict original AABB intersection and full ARGB keys.
/// Testing the current union would incorrectly merge a box in a component's gap.
pub fn merge(boxes: Vec<(Bounds, u32)>) -> Vec<(Bounds, u32)> {
    let n = boxes.len();
    let mut roots = (0..n).collect::<Vec<_>>();
    fn root(p: &mut [usize], i: usize) -> usize {
        if p[i] != i {
            p[i] = root(p, p[i]);
        }
        p[i]
    }
    for i in 0..n {
        for j in i + 1..n {
            let (a, c) = boxes[i];
            let (b, d) = boxes[j];
            if c == d && (0..3).all(|k| a[k] < b[k + 3] && a[k + 3] > b[k]) {
                let ri = root(&mut roots, i);
                let rj = root(&mut roots, j);
                roots[rj] = ri;
            }
        }
    }
    let mut out: Vec<(usize, Bounds, u32)> = vec![];
    for (i, (b, c)) in boxes.into_iter().enumerate() {
        let r = root(&mut roots, i);
        if let Some((_, a, _)) = out.iter_mut().find(|v| v.0 == r) {
            for k in 0..3 {
                a[k] = a[k].min(b[k]);
                a[k + 3] = a[k + 3].max(b[k + 3]);
            }
        } else {
            out.push((r, b, c));
        }
    }
    out.into_iter().map(|(_, b, c)| (b, c)).collect()
}
const TOP_FG: [[f32; 4]; 4] = [
    [-7., 2., -4., 3.],
    [4., 2., 7., 3.],
    [-7., 0.5, -6., 3.],
    [6., 0.5, 7., 3.],
];
const TOP_BG: [[f32; 4]; 4] = [
    [-7., 3., -4., 3.3],
    [4., 3., 7., 3.3],
    [-7.3, 0.5, -7., 3.3],
    [7., 0.5, 7.3, 3.3],
];
const BOTTOM_FG: [[f32; 4]; 4] = [
    [4., -20., 7., -19.],
    [-7., -20., -4., -19.],
    [6., -20., 7., -17.5],
    [-7., -20., -6., -17.5],
];
const BOTTOM_BG: [[f32; 4]; 4] = [
    [7., -20., 7.3, -17.5],
    [-7.3, -20., -7., -17.5],
    [4., -20.3, 7.3, -20.],
    [-7.3, -20.3, -4., -20.],
];
