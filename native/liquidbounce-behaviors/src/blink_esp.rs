// SPDX-License-Identifier: GPL-3.0-or-later
//! BlinkManager global ESP, LiquidBounce ae0eb396 (not a gameplay module).
use crate::breadcrumbs::{Breadcrumbs, ColoredPoint, Geometry as Mesh};
pub const MODES: [&str; 5] = ["Box", "Model", "Wireframe", "Ring", "None"];
#[derive(Clone, Debug)]
pub struct BlinkEsp {
    pub mode: usize,
    pub box_color: i32,
    pub box_outline: i32,
    pub model_outline: i32,
    pub light_percent: i32,
    pub wire_color: i32,
    pub wire_outline: i32,
    pub radius: f32,
    pub ring_color: i32,
    pub ring_outline: i32,
    pub line: i32,
    pub render: Breadcrumbs,
}
impl Default for BlinkEsp {
    fn default() -> Self {
        let mut render = Breadcrumbs::default();
        for style in &mut render.styles {
            style.values.insert("Fill".into(), 1.);
        }
        Self {
            mode: 2,
            box_color: 0x57242093,
            box_outline: 0xff242093u32 as i32,
            model_outline: 0x00242093,
            light_percent: 60,
            wire_color: 0x57242093,
            wire_outline: 0xff242093u32 as i32,
            radius: 0.5,
            ring_color: 0x57242093,
            ring_outline: 0xff242093u32 as i32,
            line: 0xff4677ffu32 as i32,
            render,
        }
    }
}
pub fn rgba(argb: i32) -> [f32; 4] {
    let v = argb as u32;
    [
        ((v >> 16) & 255) as f32 / 255.,
        ((v >> 8) & 255) as f32 / 255.,
        (v & 255) as f32 / 255.,
        (v >> 24) as f32 / 255.,
    ]
}
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub first_person: bool,
    pub width: f32,
    pub height: f32,
    pub pose: i32,
    pub swim: f32,
}
#[derive(Default)]
pub struct Visual {
    pub body: Mesh,
    pub line: Mesh,
    pub ring: Option<([f64; 3], f32, [f32; 4], [f32; 4])>,
    pub model: Option<[f64; 3]>,
}
fn point(position: [f64; 3], color: i32) -> ColoredPoint {
    ColoredPoint {
        position,
        color: rgba(color),
    }
}
const FACES: [[usize; 4]; 6] = [
    [0, 1, 3, 2],
    [4, 6, 7, 5],
    [0, 4, 5, 1],
    [2, 3, 7, 6],
    [0, 2, 6, 4],
    [1, 5, 7, 3],
];
const EDGES: [[usize; 2]; 12] = [
    [0, 1],
    [0, 2],
    [0, 4],
    [1, 3],
    [1, 5],
    [2, 3],
    [2, 6],
    [3, 7],
    [4, 5],
    [4, 6],
    [5, 7],
    [6, 7],
];
fn cuboid(
    mesh: &mut Mesh,
    min: [f64; 3],
    max: [f64; 3],
    color: i32,
    outline: i32,
    transform: impl Fn([f64; 3]) -> [f64; 3],
) {
    let corners: [[f64; 3]; 8] = std::array::from_fn(|n| {
        transform(std::array::from_fn(|i| {
            if n & (1 << i) == 0 { min[i] } else { max[i] }
        }))
    });
    if color as u32 >> 24 != 0 {
        for face in FACES {
            mesh.quads.push(face.map(|i| point(corners[i], color)));
        }
    }
    if outline as u32 >> 24 != 0 {
        for edge in EDGES {
            mesh.lines.push(edge.map(|i| point(corners[i], outline)));
        }
    }
}
fn rotate(mut p: [f64; 3], pivot: [f64; 3], angle: f32, axis: usize) -> [f64; 3] {
    let (a, b) = match axis {
        0 => (1, 2),
        1 => (2, 0),
        _ => (0, 1),
    };
    let (s, c) = f64::from(angle).to_radians().sin_cos();
    let x = p[a] - pivot[a];
    let y = p[b] - pivot[b];
    p[a] = pivot[a] + c * x - s * y;
    p[b] = pivot[b] + s * x + c * y;
    p
}
impl BlinkEsp {
    pub fn visual(&self, positions: &[[f64; 3]], rotation: [f32; 2], p: Player) -> Visual {
        let mut out = Visual::default();
        if self.line as u32 >> 24 != 0 {
            for segment in positions.windows(2) {
                out.line
                    .lines
                    .push([point(segment[0], self.line), point(segment[1], self.line)]);
            }
        }
        let Some(&base) = positions.first() else {
            return out;
        };
        if self.mode == 3 {
            if self.radius > 0.
                && (self.ring_color as u32 >> 24 != 0 || self.ring_outline as u32 >> 24 != 0)
            {
                out.ring = Some((
                    base,
                    self.radius,
                    rgba(self.ring_color),
                    rgba(self.ring_outline),
                ));
            }
            return out;
        }
        if p.first_person {
            return out;
        }
        match self.mode {
            0 => {
                let d = f64::from(p.width) / 2.;
                cuboid(
                    &mut out.body,
                    [-d - 0.05, -0.05, -d - 0.05],
                    [d + 0.05, f64::from(p.height) + 0.05, d + 0.05],
                    self.box_color,
                    self.box_outline,
                    |q| std::array::from_fn(|i| base[i] + q[i]),
                );
            }
            1 => out.model = Some(base),
            2 => {
                let parts = [
                    ([-0.125, 0., 0.], [0., 0.375, 0.125]),
                    ([0., 0., 0.], [0.125, 0.375, 0.125]),
                    ([-0.125, 0.375, 0.], [0.125, 0.75, 0.125]),
                    ([-0.25, 0.375, 0.], [-0.125, 0.75, 0.125]),
                    ([0.125, 0.375, 0.], [0.25, 0.75, 0.125]),
                    ([-0.125, 0.75, -0.0625], [0.125, 1., 0.1875]),
                ];
                for (i, (mut lo, mut hi)) in parts.into_iter().enumerate() {
                    if p.pose == 5 {
                        let delta = match i {
                            0 | 1 => [0., 0., 0.125],
                            2 => [0., -0.12, 0.05],
                            3 | 4 => [0., -0.12, 0.03],
                            _ => [0., -0.18, 0.1],
                        };
                        for j in 0..3 {
                            lo[j] += delta[j];
                            hi[j] += delta[j];
                        }
                    }
                    let center = std::array::from_fn(|j| (lo[j] + hi[j]) / 2.);
                    let bottom = [center[0], lo[1], center[2]];
                    let mut pitch = if i == 5 { rotation[1] } else { 0. };
                    let mut roll = 0.;
                    let mut pivot = bottom;
                    if p.pose == 5 {
                        pitch = match i {
                            2 => 28.64789,
                            3 | 4 => 22.918312,
                            _ => pitch,
                        };
                    }
                    if p.pose == 3 {
                        pivot = if i == 5 { bottom } else { center };
                        roll = match i {
                            0 => -6.,
                            1 => 6.,
                            3 => -15.,
                            4 => 15.,
                            _ => 0.,
                        };
                        if i == 5 {
                            let progress = if p.swim > 0. { p.swim } else { 1. };
                            pitch = rotation[1] + progress * (-45. - rotation[1]);
                        }
                    }
                    cuboid(
                        &mut out.body,
                        lo,
                        hi,
                        self.wire_color,
                        self.wire_outline,
                        |q| {
                            let mut q = rotate(q, pivot, pitch, 0);
                            q = rotate(q, pivot, roll, 2);
                            if p.pose == 3 {
                                q = rotate(q, [0., 0.5625, 0.0625], 90., 0);
                                q[1] -= 0.4375;
                            }
                            q = q.map(|v| v * 1.9);
                            q = rotate(q, [0.; 3], -crate::rotations::wrap_degrees(rotation[0]), 1);
                            std::array::from_fn(|j| base[j] + q[j])
                        },
                    );
                }
            }
            _ => {}
        }
        out
    }
}
