// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleBreadcrumbs and Rendering/ShaderStyle, LiquidBounce ae0eb396.
use std::collections::{BTreeMap, VecDeque};

pub const STYLES: [&str; 9] = [
    "Smooth",
    "Outline",
    "Neon",
    "Rainbow",
    "Ripple",
    "Hologram",
    "Halftone",
    "Chromatic",
    "Aura",
];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spec {
    pub name: &'static str,
    pub default: f32,
    pub from: f32,
    pub to: f32,
    pub suffix: &'static str,
    pub color: bool,
}
const fn f(name: &'static str, default: f32, from: f32, to: f32, suffix: &'static str) -> Spec {
    Spec {
        name,
        default,
        from,
        to,
        suffix,
        color: false,
    }
}
const fn c(name: &'static str) -> Spec {
    Spec {
        name,
        default: 0.,
        from: 0.,
        to: 0.,
        suffix: "",
        color: true,
    }
}
pub const BASE: [Spec; 2] = [f("Opacity", 1., 0., 1., ""), f("Fill", 0.15, 0., 1., "")];
pub const SPECIFICS: [&[Spec]; 9] = [
    &[
        f("Radius", 6., 1., 32., "px"),
        f("Falloff", 1.6, 0.5, 4., ""),
        f("Intensity", 1., 0., 2., ""),
    ],
    &[
        f("Width", 2., 0.5, 8., "px"),
        f("Softness", 1., 0.1, 4., "px"),
        f("GlowRadius", 6., 0., 24., "px"),
        f("GlowIntensity", 0.35, 0., 1., ""),
    ],
    &[
        c("CoreColor"),
        f("CoreMix", 0.7, 0., 1., ""),
        f("CoreWidth", 1.5, 0.5, 6., "px"),
        f("BloomRadius", 12., 1., 32., "px"),
        f("BloomIntensity", 1.2, 0., 3., ""),
        f("PulseSpeed", 0.5, 0., 4., "Hz"),
        f("PulseAmount", 0.25, 0., 1., ""),
    ],
    &[
        f("Width", 2., 0.5, 8., "px"),
        f("GlowRadius", 6., 0., 24., "px"),
        f("GlowIntensity", 0.5, 0., 1., ""),
        f("Speed", 0.35, -2., 2., ""),
        f("Spread", 400., 50., 2000., "px"),
        f("Saturation", 0.85, 0., 1., ""),
        f("EntityMix", 0., 0., 1., ""),
    ],
    &[
        f("Reach", 28., 4., 64., "px"),
        f("Spacing", 10., 3., 32., "px"),
        f("RingWidth", 2., 0.5, 8., "px"),
        f("Speed", 16., -64., 64., "px/s"),
        f("EdgeWidth", 1.5, 0., 6., "px"),
    ],
    &[
        c("StripeColor"),
        f("Stripes", 0.55, 0., 1., ""),
        f("Spacing", 6., 2., 32., "px"),
        f("Angle", 90., 0., 180., "°"),
        f("Speed", 1., -8., 8., ""),
        f("EdgeWidth", 1.5, 0.5, 6., "px"),
        f("Flicker", 0.15, 0., 1., ""),
    ],
    &[
        f("Reach", 12., 2., 48., "px"),
        f("CellSize", 5., 3., 24., "px"),
        f("DotScale", 0.9, 0.2, 1.5, ""),
        f("EdgeWidth", 1., 0., 4., "px"),
    ],
    &[
        f("Width", 2., 0.5, 8., "px"),
        f("Split", 3., 0., 12., "px"),
        f("Speed", 1.5, -8., 8., "rad/s"),
        f("Tint", 0.3, 0., 1., ""),
        f("GlowRadius", 3., 0., 16., "px"),
    ],
    &[
        f("Radius", 6., 2., 32., "px"),
        f("Height", 16., 0., 48., "px"),
        f("Sway", 3., 0., 16., "px"),
        f("Scale", 10., 4., 64., "px"),
        f("Speed", 1.5, 0., 6., ""),
        f("Intensity", 1.2, 0., 3., ""),
        f("Heat", 0.3, 0., 1., ""),
    ],
];
pub fn specs(style: usize) -> impl Iterator<Item = Spec> {
    BASE.into_iter().chain(SPECIFICS[style].iter().copied())
}
#[derive(Clone, Debug, PartialEq)]
pub struct StyleSettings {
    pub values: BTreeMap<String, f32>,
    pub accent: i32,
}
impl StyleSettings {
    pub fn new(i: usize) -> Self {
        Self {
            values: specs(i)
                .filter(|s| !s.color)
                .map(|s| (s.name.to_owned(), s.default))
                .collect(),
            accent: -1,
        }
    }
    pub fn get(&self, name: &str) -> f32 {
        self.values[name]
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Effect {
    pub style: usize,
    pub params: [f32; 8],
    pub accent: [f32; 4],
    pub range: f32,
    pub opacity: f32,
}
impl StyleSettings {
    pub fn effect(&self, style: usize) -> Effect {
        let g = |n| self.get(n);
        let fill = g("Fill");
        let (values, range): (Vec<f32>, f32) = match style {
            0 => (
                vec![g("Radius"), g("Falloff"), g("Intensity"), fill],
                g("Radius"),
            ),
            1 => (
                vec![
                    g("Width"),
                    g("Softness"),
                    g("GlowRadius"),
                    g("GlowIntensity"),
                    fill,
                ],
                g("Width") + g("GlowRadius") + g("Softness"),
            ),
            2 => (
                vec![
                    g("CoreWidth"),
                    g("BloomRadius"),
                    g("BloomIntensity"),
                    fill,
                    g("CoreMix"),
                    g("PulseSpeed"),
                    g("PulseAmount"),
                ],
                g("CoreWidth").max(g("BloomRadius") * 2.),
            ),
            3 => (
                vec![
                    g("Width"),
                    g("GlowRadius"),
                    g("GlowIntensity"),
                    fill,
                    g("Speed"),
                    g("Spread"),
                    g("Saturation"),
                    g("EntityMix"),
                ],
                g("Width") + g("GlowRadius"),
            ),
            4 => (
                vec![
                    g("Spacing"),
                    g("RingWidth"),
                    g("Speed"),
                    g("EdgeWidth"),
                    fill,
                ],
                g("Reach"),
            ),
            5 => {
                let radians = g("Angle").to_radians();
                (
                    vec![
                        g("Spacing"),
                        radians.cos(),
                        radians.sin(),
                        g("Speed"),
                        g("Stripes"),
                        fill,
                        g("EdgeWidth"),
                        g("Flicker"),
                    ],
                    g("EdgeWidth") + 8.,
                )
            }
            6 => (
                vec![g("CellSize"), g("DotScale"), fill, g("EdgeWidth")],
                g("Reach") + g("CellSize"),
            ),
            7 => (
                vec![
                    g("Width"),
                    g("Split"),
                    g("Speed"),
                    g("Tint"),
                    g("GlowRadius"),
                    fill,
                ],
                g("Width") + g("GlowRadius") + g("Split"),
            ),
            _ => (
                vec![
                    g("Radius"),
                    g("Height"),
                    g("Sway"),
                    g("Scale"),
                    g("Speed"),
                    g("Intensity"),
                    g("Heat"),
                    fill,
                ],
                g("Radius") + g("Height") + g("Sway"),
            ),
        };
        let mut params = [0.; 8];
        params[..values.len()].copy_from_slice(&values);
        Effect {
            style,
            params,
            accent: rgba(self.accent as u32),
            range: range.clamp(1., 64.),
            opacity: g("Opacity"),
        }
    }
}
pub type Point = [f64; 3];
/// Identity is the actual entity insertion sequence (or player lifetime), never a network id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entity {
    pub identity: u64,
    pub own: bool,
    pub alive: bool,
    pub position: Point,
    pub interpolated: Point,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Part {
    position: Point,
    created: i64,
}
#[derive(Clone, Debug, PartialEq)]
struct Trail {
    parts: VecDeque<Part>,
    entity: Entity,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColoredPoint {
    pub position: Point,
    pub color: [f32; 4],
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    pub quads: Vec<[ColoredPoint; 4]>,
    pub lines: Vec<[ColoredPoint; 2]>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Breadcrumbs {
    pub only_own: bool,
    pub color: i32,
    pub rainbow: bool,
    pub height: f32,
    pub temporary: bool,
    pub alive: i32,
    pub fade: bool,
    pub shader: bool,
    pub style: usize,
    pub styles: [StyleSettings; 9],
    trails: BTreeMap<u64, Trail>,
    last_positions: BTreeMap<u64, Point>,
}
impl Default for Breadcrumbs {
    fn default() -> Self {
        Self {
            only_own: true,
            color: 0x784677ff,
            rainbow: false,
            height: 0.5,
            temporary: true,
            alive: 900,
            fade: true,
            shader: false,
            style: 1,
            styles: std::array::from_fn(StyleSettings::new),
            trails: BTreeMap::new(),
            last_positions: BTreeMap::new(),
        }
    }
}
impl Breadcrumbs {
    pub fn clear(&mut self) {
        self.trails.clear();
        self.last_positions.clear();
    }
    pub fn tick(&mut self, now: i64, entities: &[Entity]) {
        self.update_live(entities);
        for entity in entities.iter().filter(|e| !self.only_own || e.own) {
            if self.last_positions.get(&entity.identity) == Some(&entity.position) {
                continue;
            }
            self.last_positions.insert(entity.identity, entity.position);
            self.trails
                .entry(entity.identity)
                .or_insert_with(|| Trail {
                    parts: VecDeque::new(),
                    entity: *entity,
                })
                .parts
                .push_back(Part {
                    position: entity.position,
                    created: now,
                });
        }
        if self.only_own {
            self.trails.retain(|_, t| t.entity.own || !t.entity.alive);
        } else {
            self.trails
                .retain(|id, t| t.entity.alive && entities.iter().any(|e| e.identity == *id));
        }
    }
    fn update_live(&mut self, entities: &[Entity]) {
        for (id, t) in &mut self.trails {
            t.entity.own = false;
            if !entities.iter().any(|e| e.identity == *id) {
                t.entity.alive = false;
            }
        }
        for entity in entities {
            if let Some(t) = self.trails.get_mut(&entity.identity) {
                t.entity = *entity;
            }
        }
    }
    pub fn render(&mut self, now: i64, nanos: f64, entities: &[Entity]) -> Geometry {
        self.update_live(entities);
        let mut geometry = Geometry::default();
        let mut color = if self.rainbow {
            rainbow(nanos)
        } else {
            rgba(self.color as u32)
        };
        if self.shader {
            color[3] = 1.;
        }
        for trail in self.trails.values_mut() {
            if self.temporary {
                let expiry = now - self.alive as i64;
                while trail.parts.front().is_some_and(|p| p.created < expiry) {
                    trail.parts.pop_front();
                }
            }
            let mut points = trail
                .parts
                .iter()
                .map(|p| {
                    let mut c = color;
                    if self.temporary && self.fade {
                        c[3] *= 1. - (now - p.created) as f32 / self.alive as f32;
                    }
                    ColoredPoint {
                        position: p.position,
                        color: c,
                    }
                })
                .collect::<Vec<_>>();
            if let Some(last) = points.last_mut() {
                last.position = trail.entity.interpolated;
            }
            for pair in points.windows(2) {
                let [previous, current] = [pair[0], pair[1]];
                if self.height == 0. {
                    geometry.lines.push([current, previous]);
                } else {
                    let mut p = previous;
                    let mut c = current;
                    p.position[1] += f64::from(self.height);
                    c.position[1] += f64::from(self.height);
                    geometry.quads.push([current, previous, p, c]);
                }
            }
        }
        geometry
    }
    pub fn effect(&self) -> Option<Effect> {
        self.shader
            .then(|| self.styles[self.style].effect(self.style))
    }
}
pub fn rgba(argb: u32) -> [f32; 4] {
    [
        ((argb >> 16) & 255) as f32 / 255.,
        ((argb >> 8) & 255) as f32 / 255.,
        (argb & 255) as f32 / 255.,
        (argb >> 24) as f32 / 255.,
    ]
}
/// java.awt.Color.HSBtoRGB: hue converts to Float before remainder and RGB rounds +0.5.
pub fn rainbow(nanos: f64) -> [f32; 4] {
    let hue = (nanos / 10_000_000_000.) as f32 % 1.;
    let h = (hue - hue.floor()) * 6.;
    let f = h - h.floor();
    let q = 1. - f;
    let rgb = match h as i32 {
        0 => [1., f, 0.],
        1 => [q, 1., 0.],
        2 => [0., 1., f],
        3 => [0., q, 1.],
        4 => [f, 0., 1.],
        _ => [1., 0., q],
    };
    [
        (rgb[0] * 255. + 0.5) as u8 as f32 / 255.,
        (rgb[1] * 255. + 0.5) as u8 as f32 / 255.,
        (rgb[2] * 255. + 0.5) as u8 as f32 / 255.,
        1.,
    ]
}
