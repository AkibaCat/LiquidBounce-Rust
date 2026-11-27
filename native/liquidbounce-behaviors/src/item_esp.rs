// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleItemESP and Legacy2DMarkerRenderer, LiquidBounce ae0eb396.
use crate::breadcrumbs::{Effect, StyleSettings, rainbow};
use crate::storage_esp::{Bounds, Point};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Glow,
    Box,
    Legacy2D,
}
impl Mode {
    pub const ALL: [Self; 3] = [Self::Glow, Self::Box, Self::Legacy2D];
    pub fn tag(self) -> &'static str {
        match self {
            Self::Glow => "Glow",
            Self::Box => "Box",
            Self::Legacy2D => "Legacy2D",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ItemEsp {
    pub whitelist: bool,
    pub items: BTreeSet<String>,
    pub maximum_distance: f32,
    pub tracers: bool,
    pub arrows: bool,
    pub regular_arrows: bool,
    pub spectral_arrows: bool,
    pub arrows_with_effects: bool,
    pub tridents: bool,
    pub mode: Mode,
    pub merge_intersecting: bool,
    pub scale: f32,
    pub y_offset: f32,
    pub background_alpha: i32,
    pub rainbow: bool,
    pub color: i32,
    pub style: usize,
    pub styles: [StyleSettings; 9],
    /// Entity objects captured at GameTick; native identity survives removal.
    pub selected: BTreeSet<u64>,
}
impl Default for ItemEsp {
    fn default() -> Self {
        Self {
            whitelist: false,
            items: BTreeSet::new(),
            maximum_distance: 128.,
            tracers: false,
            arrows: true,
            regular_arrows: true,
            spectral_arrows: true,
            arrows_with_effects: true,
            tridents: true,
            mode: Mode::Glow,
            merge_intersecting: false,
            scale: 0.1,
            y_offset: 0.,
            background_alpha: 150,
            rainbow: false,
            color: 0xffffb348u32 as i32,
            style: 1,
            styles: std::array::from_fn(|i| {
                let mut s = StyleSettings::new(i);
                s.values.insert("Fill".into(), crate::storage_esp::FILLS[i]);
                s
            }),
            selected: BTreeSet::new(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind<'a> {
    Item(&'a str),
    Arrow { allowed: bool, effect_color: i32 },
    Spectral { allowed: bool },
    Trident,
}
impl ItemEsp {
    pub fn clear(&mut self) {
        self.selected.clear();
    }
    pub fn should_render(&self, kind: Kind<'_>, eye: Point, camera: Point) -> bool {
        let distance = (0..3).map(|i| (eye[i] - camera[i]).powi(2)).sum::<f64>();
        if distance > f64::from(self.maximum_distance * self.maximum_distance) {
            return false;
        }
        match kind {
            Kind::Item(item) => self.items.contains(item) == self.whitelist,
            Kind::Trident => self.tridents,
            Kind::Arrow {
                allowed,
                effect_color,
            } => {
                self.arrows
                    && allowed
                    && if effect_color == -1 {
                        self.regular_arrows
                    } else {
                        self.arrows_with_effects
                    }
            }
            Kind::Spectral { allowed } => self.arrows && allowed && self.spectral_arrows,
        }
    }
    pub fn color_at(&self, nanos: f64) -> [f32; 4] {
        if self.rainbow {
            let mut color = rainbow(nanos);
            color[3] = 50. / 255.;
            color
        } else {
            crate::breadcrumbs::rgba(self.color as u32)
        }
    }
    pub fn effect(&self) -> Effect {
        if self.style == 0 {
            Effect {
                style: 9,
                params: [0.; 8],
                accent: [1.; 4],
                range: 1.,
                opacity: 1.,
            }
        } else {
            self.styles[self.style - 1].effect(self.style - 1)
        }
    }
}
pub fn item_box(p: Point) -> Bounds {
    [
        p[0] - 0.125,
        p[1] + 0.125,
        p[2] - 0.125,
        p[0] + 0.125,
        p[1] + 0.375,
        p[2] + 0.125,
    ]
}
fn intersects(a: Bounds, b: Bounds) -> bool {
    (0..3).all(|i| a[i] < b[i + 3] && a[i + 3] > b[i])
}
fn java_compare(a: f64, b: f64) -> std::cmp::Ordering {
    // java.lang.Double.compare orders -0 before +0, and all NaNs last.
    if a < b {
        std::cmp::Ordering::Less
    } else if a > b {
        std::cmp::Ordering::Greater
    } else {
        let bits = |v: f64| {
            if v.is_nan() {
                0x7ff8000000000000u64 as i64
            } else {
                v.to_bits() as i64
            }
        };
        bits(a).cmp(&bits(b))
    }
}
/// Strict intersections of ORIGINAL boxes only, then union the connected sets.
pub fn merge_boxes(items: &[Bounds]) -> Vec<Bounds> {
    fn find(parent: &mut [usize], mut n: usize) -> usize {
        while n != parent[n] {
            parent[n] = parent[parent[n]];
            n = parent[n];
        }
        n
    }
    let mut parent = (0..items.len()).collect::<Vec<_>>();
    let mut rank = vec![0u32; items.len()];
    let mut order = (0..items.len()).collect::<Vec<_>>();
    order.sort_by(|&a, &b| java_compare(items[a][0], items[b][0]).then(a.cmp(&b)));
    let mut active: Vec<usize> = vec![];
    for i in order {
        active.retain(|&j| items[j][3] > items[i][0]);
        for &j in &active {
            if !intersects(items[i], items[j]) {
                continue;
            }
            let (mut a, mut b) = (find(&mut parent, i), find(&mut parent, j));
            if a == b {
                continue;
            }
            if rank[a] < rank[b] {
                std::mem::swap(&mut a, &mut b);
            }
            parent[b] = a;
            if rank[a] == rank[b] {
                rank[a] += 1;
            }
        }
        active.push(i);
    }
    let mut roots = vec![None; items.len()];
    let mut output: Vec<Bounds> = vec![];
    for (i, &b) in items.iter().enumerate() {
        let root = find(&mut parent, i);
        if let Some(j) = roots[root] {
            let merged: &mut Bounds = &mut output[j];
            for a in 0..3 {
                merged[a] = merged[a].min(b[a]);
                merged[a + 3] = merged[a + 3].max(b[a + 3]);
            }
        } else {
            roots[root] = Some(output.len());
            output.push(b);
        }
    }
    output
}
pub const TOP_FOREGROUND: [[f32; 4]; 4] = [
    [-7., 2., -4., 3.],
    [4., 2., 7., 3.],
    [-7., 0.5, -6., 3.],
    [6., 0.5, 7., 3.],
];
pub const TOP_BACKGROUND: [[f32; 4]; 4] = [
    [-7., 3., -4., 3.3],
    [4., 3., 7., 3.3],
    [-7.3, 0.5, -7., 3.3],
    [7., 0.5, 7.3, 3.3],
];
pub const BOTTOM_FOREGROUND: [[f32; 4]; 4] = [
    [4., -20., 7., -19.],
    [-7., -20., -4., -19.],
    [6., -20., 7., -17.5],
    [-7., -20., -6., -17.5],
];
pub const BOTTOM_BACKGROUND: [[f32; 4]; 4] = [
    [7., -20., 7.3, -17.5],
    [-7.3, -20., -7., -17.5],
    [4., -20.3, 7.3, -20.],
    [-7.3, -20.3, -4., -20.],
];
/// PoseStack's float mesh transforms: translate, rotate Y(-yaw), scale(-s,-s,s).
pub fn legacy_marker(
    position: Point,
    camera: Point,
    yaw: f32,
    height: f64,
    scale: f32,
) -> Vec<([Point; 4], bool)> {
    let angle = (-yaw).to_radians();
    let (sin, cos) = angle.sin_cos();
    let relative: [f32; 3] = std::array::from_fn(|i| (position[i] - camera[i]) as f32);
    let offset = (21. - height * 12.) as f32;
    let mut out = vec![];
    for (rects, bottom, background) in [
        (&TOP_FOREGROUND, false, false),
        (&TOP_BACKGROUND, false, true),
        (&BOTTOM_FOREGROUND, true, false),
        (&BOTTOM_BACKGROUND, true, true),
    ] {
        for &[x1, y1, x2, y2] in rects {
            let points = [(x2, y1), (x1, y1), (x1, y2), (x2, y2)].map(|(x, y)| {
                let x = x * (-scale);
                let y = (y + if bottom { offset } else { 0. }) * (-scale);
                [
                    camera[0] + f64::from(relative[0] + cos * x),
                    camera[1] + f64::from(relative[1] + y),
                    camera[2] + f64::from(relative[2] - sin * x),
                ]
            });
            out.push((points, background));
        }
    }
    out
}
