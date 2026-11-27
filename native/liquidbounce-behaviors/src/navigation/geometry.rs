// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac math coordinates and search block regions.
use std::hash::{Hash, Hasher};

fn double_bits(value: f64) -> u64 {
    if value.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        value.to_bits()
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        double_bits(self.x) == double_bits(other.x)
            && double_bits(self.y) == double_bits(other.y)
            && double_bits(self.z) == double_bits(other.z)
    }
}
impl Eq for Vec3 {}
impl Hash for Vec3 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        double_bits(self.x).hash(state);
        double_bits(self.y).hash(state);
        double_bits(self.z).hash(state);
    }
}
impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub fn distance_sqr(self, other: Self) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        dx * dx + dy * dy + dz * dz
    }
    pub fn distance(self, other: Self) -> f64 {
        self.distance_sqr(other).sqrt()
    }
    pub fn bottom_center(pos: BlockPos) -> Self {
        Self::new(
            f64::from(pos.x) + 0.5,
            f64::from(pos.y),
            f64::from(pos.z) + 0.5,
        )
    }
    pub fn fma(self, scale: f64, other: Self) -> Self {
        Self::new(
            scale.mul_add(other.x, self.x),
            scale.mul_add(other.y, self.y),
            scale.mul_add(other.z, self.z),
        )
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    pub fn containing(pos: Vec3) -> Self {
        Self::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        )
    }
    pub fn above(self) -> Self {
        Self::new(self.x, self.y.wrapping_add(1), self.z)
    }
    pub fn offset(self, x: i32, y: i32, z: i32) -> Self {
        Self::new(
            self.x.wrapping_add(x),
            self.y.wrapping_add(y),
            self.z.wrapping_add(z),
        )
    }
    pub fn packed(self) -> i64 {
        ((i64::from(self.x) & 0x3ff_ffff) << 38)
            | (i64::from(self.y) & 0xfff)
            | ((i64::from(self.z) & 0x3ff_ffff) << 12)
    }
    pub fn from_packed(packed: i64) -> Self {
        Self::new(
            (packed >> 38) as i32,
            ((packed << 52) >> 52) as i32,
            ((packed << 26) >> 38) as i32,
        )
    }
    pub fn distance_sqr_long(self, other: Self) -> i64 {
        let dx = i64::from(self.x) - i64::from(other.x);
        let dy = i64::from(self.y) - i64::from(other.y);
        let dz = i64::from(self.z) - i64::from(other.z);
        dx.wrapping_mul(dx)
            .wrapping_add(dy.wrapping_mul(dy))
            .wrapping_add(dz.wrapping_mul(dz))
    }
}
// Kotlin math min/max propagate NaN and retain the JVM signed-zero rule.
pub fn java_min(left: f64, right: f64) -> f64 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == 0.0 && right == 0.0 {
        f64::from_bits(left.to_bits() | right.to_bits())
    } else {
        left.min(right)
    }
}
pub fn java_max(left: f64, right: f64) -> f64 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == 0.0 && right == 0.0 {
        f64::from_bits(left.to_bits() & right.to_bits())
    } else {
        left.max(right)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockRegion {
    pub min: BlockPos,
    pub max: BlockPos,
}
impl BlockRegion {
    pub fn contains(self, pos: BlockPos) -> bool {
        (self.min.x..=self.max.x).contains(&pos.x)
            && (self.min.y..=self.max.y).contains(&pos.y)
            && (self.min.z..=self.max.z).contains(&pos.z)
    }
    pub fn around(start: Vec3, ends: &[Vec3], max_length: f64, vertical_margin: i32) -> Self {
        let mut min = start;
        let mut max = start;
        let mut closest = f64::MAX;
        for end in ends {
            min = Vec3::new(
                java_min(min.x, end.x),
                java_min(min.y, end.y),
                java_min(min.z, end.z),
            );
            max = Vec3::new(
                java_max(max.x, end.x),
                java_max(max.y, end.y),
                java_max(max.z, end.z),
            );
            closest = java_min(closest, start.distance(*end));
        }
        let detour = if closest == f64::MAX {
            max_length
        } else {
            java_max(0.0, (max_length - closest) / 2.0)
        };
        let horizontal = detour + 3.0;
        let vertical = java_min(detour, f64::from(vertical_margin)) + 3.0;
        Self {
            min: BlockPos::new(
                ((min.x - horizontal) as i32).wrapping_sub(1),
                ((min.y - vertical) as i32).wrapping_sub(1),
                ((min.z - horizontal) as i32).wrapping_sub(1),
            ),
            max: BlockPos::new(
                ((max.x + horizontal) as i32).wrapping_add(1),
                ((max.y + vertical) as i32).wrapping_add(1),
                ((max.z + horizontal) as i32).wrapping_add(1),
            ),
        }
    }
}
