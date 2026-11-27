// SPDX-License-Identifier: GPL-3.0-or-later
// Port of pinned LiquidBounce ModuleProtectionZones (ae0eb396).
#[derive(Clone, Debug, PartialEq)]
pub struct ProtectionZones {
    pub protection_blocks: Vec<String>,
    pub radius_x: i32,
    pub radius_z: i32,
    pub radius_y: i32,
    pub render_limit: i32,
    pub hold_block_to_render: bool,
    pub zone_fill: i32,
    pub zone_outline: i32,
    pub center_zone_outline: i32,
    pub indicator_outline: i32,
    pub indicator_fill: i32,
    pub snap_y: bool,
}
impl Default for ProtectionZones {
    fn default() -> Self {
        Self {
            protection_blocks: vec!["minecraft:emerald_block".into()],
            radius_x: 20,
            radius_z: 20,
            radius_y: 383,
            render_limit: 16,
            hold_block_to_render: false,
            zone_fill: 0x3300ff00,
            zone_outline: 0xff00ff00u32 as i32,
            center_zone_outline: 0xff00ffcfu32 as i32,
            indicator_outline: 0xfffff000u32 as i32,
            indicator_fill: 0x33fff000,
            snap_y: false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draw {
    pub bounds: [f64; 6],
    pub fill: i32,
    pub outline: i32,
}
fn contains(b: [f64; 6], p: [f64; 3]) -> bool {
    (0..3).all(|i| p[i] >= b[i] && p[i] < b[i + 3])
}
fn distance_squared(b: [f64; 6], p: [f64; 3]) -> f64 {
    (0..3)
        .map(|i| (b[i] - p[i]).max((p[i] - b[i + 3]).max(0.)).powi(2))
        .sum()
}
fn block_box(p: [i32; 3]) -> [f64; 6] {
    std::array::from_fn(|i| f64::from(p[i % 3]) + if i < 3 { 0. } else { 1. })
}
// Kotlin roundToInt chooses positive infinity on ties, including negative values.
fn snap(value: i32, origin: i32, step: i32) -> i32 {
    origin.wrapping_add(
        (((f64::from(value) - f64::from(origin)) / f64::from(step) + 0.5).floor() as i32)
            .wrapping_mul(step),
    )
}
impl ProtectionZones {
    pub fn render(
        &self,
        centers: &[[i32; 3]],
        player: [f64; 3],
        min_y: i32,
        max_y: i32,
        holding: bool,
    ) -> Vec<Draw> {
        if self.hold_block_to_render && !holding || self.render_limit <= 0 {
            return vec![];
        }
        let mut centers = centers.to_vec();
        let distance = |p: [i32; 3]| {
            (f64::from(p[0]) + 0.5 - player[0]).powi(2)
                + (f64::from(p[2]) + 0.5 - player[2]).powi(2)
        };
        centers.sort_by(|a, b| distance(*a).total_cmp(&distance(*b)));
        centers.truncate(self.render_limit as usize);
        if centers.is_empty() {
            return vec![];
        }
        let zones = centers
            .iter()
            .map(|c| {
                [
                    f64::from(c[0].wrapping_sub(self.radius_x)),
                    f64::from(c[1].wrapping_sub(self.radius_y).max(min_y)),
                    f64::from(c[2].wrapping_sub(self.radius_z)),
                    f64::from(c[0].wrapping_add(self.radius_x).wrapping_add(1)),
                    f64::from(c[1].wrapping_add(self.radius_y).min(max_y).wrapping_add(1)),
                    f64::from(c[2].wrapping_add(self.radius_z).wrapping_add(1)),
                ]
            })
            .collect::<Vec<_>>();
        // Source returns immediately for the first contained OR nearby zone. Do not
        // turn this into an all-zones containment check: order affects overlaps.
        let mut highlight = None;
        for (i, b) in zones.iter().copied().enumerate() {
            if contains(b, player) {
                break;
            }
            if distance_squared(b, player) <= 25. {
                highlight = Some(i);
                break;
            }
        }
        let mut draws = zones
            .iter()
            .map(|b| Draw {
                bounds: *b,
                fill: 0,
                outline: self.zone_outline,
            })
            .collect::<Vec<_>>();
        draws.extend(centers.iter().map(|p| Draw {
            bounds: block_box(*p),
            fill: 0,
            outline: self.center_zone_outline,
        }));
        if let Some(i) = highlight {
            draws.push(Draw {
                bounds: zones[i],
                fill: self.zone_fill,
                outline: 0,
            });
        }
        if holding {
            let base = centers[0];
            let pos = [
                snap(
                    player[0].floor() as i32,
                    base[0],
                    self.radius_x.wrapping_mul(2).wrapping_add(1),
                ),
                (if self.snap_y {
                    base[1]
                } else {
                    player[1].floor() as i32
                })
                .clamp(min_y, max_y),
                snap(
                    player[2].floor() as i32,
                    base[2],
                    self.radius_z.wrapping_mul(2).wrapping_add(1),
                ),
            ];
            let center = pos.map(|n| f64::from(n) + 0.5);
            if !zones.iter().any(|b| contains(*b, center)) {
                draws.push(Draw {
                    bounds: block_box(pos),
                    fill: self.indicator_fill,
                    outline: self.indicator_outline,
                });
            }
        }
        draws
    }
}
