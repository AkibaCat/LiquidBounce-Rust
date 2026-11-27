// SPDX-License-Identifier: GPL-3.0-or-later
use crate::hole_tracker::Hole;
use serde_json::{Value, json};
pub const MODES: [&str; 2] = ["Box", "GlowingPlane"];
#[derive(Clone, Debug, PartialEq)]
pub struct HoleEsp {
    pub mode: usize,
    pub box_outline: bool,
    pub plane_outline: bool,
    pub glow_height: f32,
    pub horizontal: i32,
    pub vertical: i32,
    pub distance_fade: f32,
    pub colors: [i32; 4],
}
impl Default for HoleEsp {
    fn default() -> Self {
        Self {
            mode: 1,
            box_outline: true,
            plane_outline: true,
            glow_height: 0.7,
            horizontal: 32,
            vertical: 8,
            distance_fade: 0.3,
            colors: [
                0xff19c15cu32 as i32,
                0xfff7381bu32 as i32,
                0xff35baccu32 as i32,
                0xfff7cf1bu32 as i32,
            ],
        }
    }
}
fn color(rgb: i32, alpha: i32, fade: f32) -> [f32; 4] {
    [
        ((rgb >> 16) & 255) as f32 / 255.,
        ((rgb >> 8) & 255) as f32 / 255.,
        (rgb & 255) as f32 / 255.,
        (alpha as f32 * fade) as i32 as f32 / 255.,
    ]
}
fn point(position: [f64; 3], color: [f32; 4]) -> Value {
    json!({"position":position,"color":color})
}
impl HoleEsp {
    pub fn fade(&self, p: [f64; 3], h: &Hole) -> f32 {
        if self.distance_fade == 0. {
            return 1.;
        }
        let vertical = (p[1] - f64::from(h.position[1])) / f64::from(self.vertical);
        let dx = p[0] - f64::from(h.position[0]);
        let dz = p[2] - f64::from(h.position[2]);
        ((1. - vertical.max((dx * dx + dz * dz).sqrt() / f64::from(self.horizontal)))
            / f64::from(self.distance_fade))
        .clamp(0., 1.) as f32
    }
    pub fn geometry(&self, p: [f64; 3], holes: &[Hole]) -> Value {
        let mut quads = vec![];
        let mut lines = vec![];
        let block = p.map(|n| n.floor() as i32);
        for h in holes {
            if (block[1] - h.position[1]).abs() > self.vertical
                || (block[0] - h.position[0]).abs() > self.horizontal
                || (block[2] - h.position[2]).abs() > self.horizontal
            {
                continue;
            }
            let fade = self.fade(p, h);
            let rgb = self.colors[usize::from(h.kind.min(3))];
            let base = color(rgb, 50, fade);
            let outline = color(rgb, 100, fade);
            let at = |x: f64, y: f64, z: f64| {
                [
                    f64::from(h.position[0]) + x,
                    f64::from(h.position[1]) + y,
                    f64::from(h.position[2]) + z,
                ]
            };
            let (x, z) = (f64::from(h.size[0]), f64::from(h.size[1]));
            let corners: [[_; 3]; 8] = std::array::from_fn(|i| {
                at(
                    if i & 1 == 0 { 0. } else { x },
                    if i & 2 == 0 { 0. } else { 1. },
                    if i & 4 == 0 { 0. } else { z },
                )
            });
            let mut quad = |ids: [usize; 4]| {
                if base[3] != 0. {
                    quads.push(json!(ids.map(|i| point(corners[i], base))));
                }
            };
            if self.mode == 0 {
                for face in [
                    [0, 1, 3, 2],
                    [4, 6, 7, 5],
                    [0, 4, 5, 1],
                    [2, 3, 7, 6],
                    [0, 2, 6, 4],
                    [1, 5, 7, 3],
                ] {
                    quad(face);
                }
                if self.box_outline && outline[3] != 0. {
                    for i in 0..8 {
                        for axis in [1, 2, 4] {
                            if i & axis == 0 {
                                lines.push(json!([
                                    point(corners[i], outline),
                                    point(corners[i | axis], outline)
                                ]));
                            }
                        }
                    }
                }
            } else {
                quad([0, 4, 5, 1]);
                if self.plane_outline && outline[3] != 0. {
                    for (a, b) in [(0, 4), (4, 5), (5, 1), (1, 0)] {
                        lines.push(json!([
                            point(corners[a], outline),
                            point(corners[b], outline)
                        ]));
                    }
                }
                if self.glow_height != 0. {
                    let transparent = [base[0], base[1], base[2], 0.];
                    for (a, b) in [(0, 4), (4, 5), (5, 1), (1, 0)] {
                        let mut top_a = corners[a];
                        let mut top_b = corners[b];
                        top_a[1] += f64::from(self.glow_height);
                        top_b[1] += f64::from(self.glow_height);
                        quads.push(json!([
                            point(corners[a], base),
                            point(corners[b], base),
                            point(top_b, transparent),
                            point(top_a, transparent)
                        ]));
                    }
                }
            }
        }
        json!({"quads":quads,"lines":lines,"effect":null})
    }
}
