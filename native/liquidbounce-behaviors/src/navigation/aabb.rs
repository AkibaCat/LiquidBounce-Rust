// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac math/AABB.kt, including its ordered slab ray clip.
use super::geometry::{BlockPos, Vec3, java_max, java_min};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}
impl Axis {
    pub const ALL: [Self; 3] = [Self::X, Self::Y, Self::Z];
    pub const fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
    pub fn choose<T: Copy>(self, values: [T; 3]) -> T {
        values[self.index()]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}
impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min: Vec3::new(
                java_min(min.x, max.x),
                java_min(min.y, max.y),
                java_min(min.z, max.z),
            ),
            max: Vec3::new(
                java_max(min.x, max.x),
                java_max(min.y, max.y),
                java_max(min.z, max.z),
            ),
        }
    }
    pub fn from_block(pos: BlockPos) -> Self {
        Self::new(
            Vec3::new(f64::from(pos.x), f64::from(pos.y), f64::from(pos.z)),
            Vec3::new(
                f64::from(pos.x) + 1.0,
                f64::from(pos.y) + 1.0,
                f64::from(pos.z) + 1.0,
            ),
        )
    }
    pub fn min(self, axis: Axis) -> f64 {
        axis.choose([self.min.x, self.min.y, self.min.z])
    }
    pub fn max(self, axis: Axis) -> f64 {
        axis.choose([self.max.x, self.max.y, self.max.z])
    }
    pub fn size(self) -> Vec3 {
        Vec3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
    pub fn center(self) -> Vec3 {
        Vec3::new(
            lerp(self.min.x, self.max.x),
            lerp(self.min.y, self.max.y),
            lerp(self.min.z, self.max.z),
        )
    }
    pub fn bottom_center(self) -> Vec3 {
        Vec3::new(
            lerp(self.min.x, self.max.x),
            self.min.y,
            lerp(self.min.z, self.max.z),
        )
    }
    pub fn with_min_y(self, value: f64) -> Self {
        Self::new(Vec3::new(self.min.x, value, self.min.z), self.max)
    }
    pub fn with_max_y(self, value: f64) -> Self {
        Self::new(self.min, Vec3::new(self.max.x, value, self.max.z))
    }
    pub fn moved(self, offset: Vec3) -> Self {
        Self::new(add(self.min, offset), add(self.max, offset))
    }
    pub fn moved_block(self, pos: BlockPos) -> Self {
        self.moved(Vec3::new(
            f64::from(pos.x),
            f64::from(pos.y),
            f64::from(pos.z),
        ))
    }
    pub fn inflate(self, amount: Vec3) -> Self {
        Self::new(
            Vec3::new(
                self.min.x - amount.x,
                self.min.y - amount.y,
                self.min.z - amount.z,
            ),
            add(self.max, amount),
        )
    }
    pub fn inflate_uniform(self, amount: f64) -> Self {
        self.inflate(Vec3::new(amount, amount, amount))
    }
    pub fn deflate(self, amount: f64) -> Self {
        self.inflate_uniform(-amount)
    }
    pub fn expand_towards(self, movement: Vec3) -> Self {
        Self::new(
            Vec3::new(
                if movement.x < 0.0 {
                    self.min.x + movement.x
                } else {
                    self.min.x
                },
                if movement.y < 0.0 {
                    self.min.y + movement.y
                } else {
                    self.min.y
                },
                if movement.z < 0.0 {
                    self.min.z + movement.z
                } else {
                    self.min.z
                },
            ),
            Vec3::new(
                if movement.x > 0.0 {
                    self.max.x + movement.x
                } else {
                    self.max.x
                },
                if movement.y > 0.0 {
                    self.max.y + movement.y
                } else {
                    self.max.y
                },
                if movement.z > 0.0 {
                    self.max.z + movement.z
                } else {
                    self.max.z
                },
            ),
        )
    }
    pub fn minmax(self, other: Self) -> Self {
        Self::new(
            Vec3::new(
                java_min(self.min.x, other.min.x),
                java_min(self.min.y, other.min.y),
                java_min(self.min.z, other.min.z),
            ),
            Vec3::new(
                java_max(self.max.x, other.max.x),
                java_max(self.max.y, other.max.y),
                java_max(self.max.z, other.max.z),
            ),
        )
    }
    pub fn intersect(self, other: Self) -> Self {
        Self::new(
            Vec3::new(
                java_max(self.min.x, other.min.x),
                java_max(self.min.y, other.min.y),
                java_max(self.min.z, other.min.z),
            ),
            Vec3::new(
                java_min(self.max.x, other.max.x),
                java_min(self.max.y, other.max.y),
                java_min(self.max.z, other.max.z),
            ),
        )
    }
    pub fn intersects(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }
    pub fn contains(self, pos: Vec3) -> bool {
        pos.x >= self.min.x
            && pos.x < self.max.x
            && pos.y >= self.min.y
            && pos.y < self.max.y
            && pos.z >= self.min.z
            && pos.z < self.max.z
    }
    pub fn distance_sqr(self, pos: Vec3) -> f64 {
        let dx = java_max(java_max(self.min.x - pos.x, pos.x - self.max.x), 0.0);
        let dy = java_max(java_max(self.min.y - pos.y, pos.y - self.max.y), 0.0);
        let dz = java_max(java_max(self.min.z - pos.z, pos.z - self.max.z), 0.0);
        dx * dx + dy * dy + dz * dz
    }
    pub fn clip(self, from: Vec3, to: Vec3) -> Option<Vec3> {
        let mut hit = Clip::new(from, to);
        hit.include(self);
        hit.face.map(|_| hit.location())
    }
    pub fn of_size(center: Vec3, size: Vec3) -> Self {
        Self::new(
            Vec3::new(
                center.x - size.x / 2.0,
                center.y - size.y / 2.0,
                center.z - size.z / 2.0,
            ),
            Vec3::new(
                center.x + size.x / 2.0,
                center.y + size.y / 2.0,
                center.z + size.z / 2.0,
            ),
        )
    }
    pub fn unit_cube_from_lower_corner(pos: Vec3) -> Self {
        Self::new(pos, add(pos, Vec3::new(1.0, 1.0, 1.0)))
    }
}
fn lerp(min: f64, max: f64) -> f64 {
    min + 0.5 * (max - min)
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}

pub struct Clip {
    from: Vec3,
    delta: Vec3,
    scale: f64,
    pub face: Option<Direction>,
}
impl Clip {
    pub fn new(from: Vec3, to: Vec3) -> Self {
        Self {
            from,
            delta: Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z),
            scale: 1.0,
            face: None,
        }
    }
    pub fn location(&self) -> Vec3 {
        add(
            self.from,
            Vec3::new(
                self.scale * self.delta.x,
                self.scale * self.delta.y,
                self.scale * self.delta.z,
            ),
        )
    }
    pub fn include(&mut self, b: Aabb) {
        let d = self.delta;
        let f = self.from;
        if d.x > 1.0e-7 {
            self.plane(
                [d.x, d.y, d.z],
                b.min.x,
                [b.min.y, b.max.y, b.min.z, b.max.z],
                Direction::West,
                [f.x, f.y, f.z],
            );
        } else if d.x < -1.0e-7 {
            self.plane(
                [d.x, d.y, d.z],
                b.max.x,
                [b.min.y, b.max.y, b.min.z, b.max.z],
                Direction::East,
                [f.x, f.y, f.z],
            );
        }
        if d.y > 1.0e-7 {
            self.plane(
                [d.y, d.z, d.x],
                b.min.y,
                [b.min.z, b.max.z, b.min.x, b.max.x],
                Direction::Down,
                [f.y, f.z, f.x],
            );
        } else if d.y < -1.0e-7 {
            self.plane(
                [d.y, d.z, d.x],
                b.max.y,
                [b.min.z, b.max.z, b.min.x, b.max.x],
                Direction::Up,
                [f.y, f.z, f.x],
            );
        }
        if d.z > 1.0e-7 {
            self.plane(
                [d.z, d.x, d.y],
                b.min.z,
                [b.min.x, b.max.x, b.min.y, b.max.y],
                Direction::North,
                [f.z, f.x, f.y],
            );
        } else if d.z < -1.0e-7 {
            self.plane(
                [d.z, d.x, d.y],
                b.max.z,
                [b.min.x, b.max.x, b.min.y, b.max.y],
                Direction::South,
                [f.z, f.x, f.y],
            );
        }
    }
    fn plane(
        &mut self,
        d: [f64; 3],
        point: f64,
        bounds: [f64; 4],
        face: Direction,
        from: [f64; 3],
    ) {
        let scale = (point - from[0]) / d[0];
        let pb = from[1] + scale * d[1];
        let pc = from[2] + scale * d[2];
        if 0.0 < scale
            && scale < self.scale
            && bounds[0] - 1.0e-7 < pb
            && pb < bounds[1] + 1.0e-7
            && bounds[2] - 1.0e-7 < pc
            && pc < bounds[3] + 1.0e-7
        {
            self.scale = scale;
            self.face = Some(face);
        }
    }
}
