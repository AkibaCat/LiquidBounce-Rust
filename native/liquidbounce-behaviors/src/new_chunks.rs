// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleNewChunks, LiquidBounce ae0eb396, CCBlueX 2015-2026.
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq)]
pub struct NewChunks {
    pub render_distance: i32,
    pub render_y: f32,
    pub auto_y: bool,
    pub smooth: bool,
    pub persist: bool,
    pub new_color: i32,
    pub old_color: i32,
    pub chunks: BTreeMap<[i32; 2], bool>,
}
impl Default for NewChunks {
    fn default() -> Self {
        Self {
            render_distance: 32,
            render_y: 0.,
            auto_y: false,
            smooth: true,
            persist: true,
            new_color: 0x5000ff00,
            old_color: 0x50ff0000,
            chunks: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub corners: [[f64; 3]; 4],
    pub color: u32,
    pub outline: u32,
}
fn channels(color: u32) -> [i32; 4] {
    [24, 16, 8, 0].map(|s| ((color >> s) & 255) as i32)
}
fn pack(color: [i32; 4]) -> u32 {
    color.into_iter().fold(0, |v, c| (v << 8) | c as u32)
}
fn interpolate(a: u32, b: u32, t: f32) -> u32 {
    let a = channels(a);
    let b = channels(b);
    pack(
        std::array::from_fn(|i| t.mul_add((b[i] - a[i]) as f32, a[i] as f32) as i32)
            .map(|v| v.clamp(0, 255)),
    )
}
fn darker(color: u32) -> u32 {
    let mut c = channels(color);
    for v in &mut c[1..] {
        *v = (*v as f32 * 0.7) as i32;
    }
    pack(c)
}
impl NewChunks {
    pub fn clear(&mut self) {
        self.chunks.clear();
    }
    pub fn chunk(&mut self, position: [i32; 2]) {
        self.chunks.entry(position).or_insert(false);
    }
    pub fn forget(&mut self, position: [i32; 2]) {
        if !self.persist {
            self.chunks.remove(&position);
        }
    }
    pub fn flowing(&mut self, block: [i32; 3]) {
        self.chunks.insert([block[0] >> 4, block[2] >> 4], true);
    }
    pub fn render(&self, player: [f64; 3]) -> Vec<Plane> {
        let max = f64::from(self.render_distance) * 16.;
        let y = if self.auto_y {
            player[1] - 100.
        } else {
            f64::from(self.render_y)
        };
        let mut out = vec![];
        for (&[x, z], &new) in &self.chunks {
            let bx = f64::from(x.wrapping_mul(16));
            let bz = f64::from(z.wrapping_mul(16));
            let dx = player[0] - (bx + 8.);
            let dz = player[2] - (bz + 8.);
            if dx * dx + dz * dz > max * max {
                continue;
            }
            let mut color = if new { self.new_color } else { self.old_color } as u32;
            if self.smooth && !new {
                let mut total = 0f32;
                for dx in -2i32..=2 {
                    for dz in -2i32..=2 {
                        if dx == 0 && dz == 0 {
                            continue;
                        }
                        if self.chunks.get(&[x.wrapping_add(dx), z.wrapping_add(dz)]) == Some(&true)
                        {
                            total += (3. - dx.abs().max(dz.abs()) as f32) / 2.;
                        }
                    }
                }
                if total > 0. {
                    color = interpolate(
                        self.old_color as u32,
                        self.new_color as u32,
                        (total / 12.).min(1.),
                    );
                }
            }
            out.push(Plane {
                corners: [
                    [bx, y, bz],
                    [bx, y, bz + 16.],
                    [bx + 16., y, bz + 16.],
                    [bx + 16., y, bz],
                ],
                color,
                outline: darker(color),
            });
        }
        out
    }
}
