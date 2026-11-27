// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAnchor and the supported HoleTracker rectangles (LiquidBounce ae0eb396).
pub type Vec3 = [f64; 3];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub max_distance: f32,
    pub horizontal_speed: f32,
    pub vertical_speed: f32,
}
impl Default for Anchor {
    fn default() -> Self {
        Self {
            max_distance: 4.,
            horizontal_speed: 0.3,
            vertical_speed: 0.1,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    Air,
    Resistant,
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hole {
    pub min: [i32; 3],
    pub size: [i32; 2],
}
impl Hole {
    pub fn center(self) -> Vec3 {
        [
            self.min[0] as f64 + self.size[0] as f64 / 2.,
            self.min[1] as f64,
            self.min[2] as f64 + self.size[1] as f64 / 2.,
        ]
    }
    pub fn contains(self, p: [i32; 3]) -> bool {
        p[1] == self.min[1]
            && p[0] >= self.min[0]
            && p[0] < self.min[0] + self.size[0]
            && p[2] >= self.min[2]
            && p[2] < self.min[2] + self.size[1]
    }
    pub fn intersects(self, b: [f64; 6]) -> bool {
        b[3] > self.min[0] as f64
            && b[0] < (self.min[0] + self.size[0]) as f64
            && b[4] > self.min[1] as f64
            && b[1] < (self.min[1] + 1) as f64
            && b[5] > self.min[2] as f64
            && b[2] < (self.min[2] + self.size[1]) as f64
    }
}
/// Query-time equivalent of the tracker's cached safe columns. The caller
/// supplies only loaded blocks; bedrock and blast resistance >= 600 are safe.
pub fn scan(min: [i32; 3], max: [i32; 3], mut block: impl FnMut([i32; 3]) -> Block) -> Vec<Hole> {
    let mut holes = Vec::<Hole>::new();
    for y in min[1]..=max[1] {
        for z in min[2]..=max[2] {
            for x in min[0]..=max[0] {
                let p = [x, y, z];
                if holes.iter().any(|h| h.contains(p)) {
                    continue;
                }
                for size in [[1, 1], [2, 1], [1, 2], [2, 2]] {
                    let mut valid = true;
                    for dz in 0..size[1] {
                        for dx in 0..size[0] {
                            let a = [x + dx, y, z + dz];
                            if block([a[0], y - 1, a[2]]) != Block::Resistant
                                || (0..3).any(|dy| block([a[0], y + dy, a[2]]) != Block::Air)
                            {
                                valid = false;
                                break;
                            }
                            for [ox, oz] in [[0, -1], [0, 1], [-1, 0], [1, 0]] {
                                let nx = dx + ox;
                                let nz = dz + oz;
                                if (nx < 0 || nx >= size[0] || nz < 0 || nz >= size[1])
                                    && block([x + nx, y, z + nz]) != Block::Resistant
                                {
                                    valid = false;
                                    break;
                                }
                            }
                        }
                    }
                    if valid {
                        holes.push(Hole { min: p, size });
                        break;
                    }
                }
            }
        }
    }
    holes.sort_by_key(|h| (h.min[1], h.min[2], h.min[0]));
    holes
}
fn distance(a: Vec3, b: Vec3) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}
impl Anchor {
    pub fn goal(self, holes: &[Hole], position: Vec3, bounds: [f64; 6]) -> Option<Vec3> {
        if let Some(h) = holes.iter().find(|h| h.intersects(bounds)) {
            return Some(h.center());
        }
        let limit = f64::from(self.max_distance * self.max_distance);
        let mut best = None;
        let mut best_distance = limit;
        for h in holes {
            if f64::from(h.min[1] + 1) > position[1] {
                continue;
            }
            let center = h.center();
            let d = distance(center, position);
            if d <= best_distance && (best.is_none() || d < best_distance) {
                best = Some(center);
                best_distance = d;
            }
        }
        best
    }
    pub fn movement(self, position: Vec3, bounds: [f64; 6], movement: Vec3, goal: Vec3) -> Vec3 {
        let delta = [
            goal[0] - position[0],
            goal[1] - position[1],
            goal[2] - position[2],
        ];
        let length = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
        let unit = if length < 1e-5 {
            [0.; 3]
        } else {
            delta.map(|v| v / length)
        };
        let mut result = movement;
        if self.horizontal_speed == 0. {
            if bounds[0] > goal[0] - 0.5
                && bounds[3] < goal[0] + 0.5
                && bounds[2] > goal[2] - 0.5
                && bounds[5] < goal[2] + 0.5
            {
                result[0] = 0.;
                result[2] = 0.;
            }
        } else {
            let speed = f64::from(self.horizontal_speed)
                .max((movement[0] * movement[0] + movement[2] * movement[2]).sqrt());
            let far = (delta[0] * delta[0] + delta[2] * delta[2]).sqrt() > speed;
            result[0] = if far { unit[0] * speed } else { delta[0] };
            result[2] = if far { unit[2] * speed } else { delta[2] };
        }
        if self.vertical_speed != 0. {
            let limit = f64::from(self.vertical_speed).max(movement[1].abs());
            result[1] = if delta[1].abs() > limit {
                unit[1] * limit
            } else {
                delta[1]
            };
        }
        result
    }
}
