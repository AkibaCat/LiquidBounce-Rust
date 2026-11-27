// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent Support: ordered closest-block selection under the feet.
use super::super::{
    aabb::Aabb,
    geometry::{BlockPos, Vec3},
};
use super::{block_colliders::BlockColliders, block_grid::ShapeGrid};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Support {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub exists: bool,
}
impl Support {
    pub fn set(&mut self, x: i32, y: i32, z: i32) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.exists = true;
    }
    pub fn clear(&mut self) {
        self.exists = false;
    }
    pub fn find(
        &mut self,
        colliders: &mut BlockColliders,
        grid: &mut impl ShapeGrid,
        feet: Vec3,
        back_x: f64,
        back_z: f64,
    ) {
        let half_width = f64::from(0.6_f32) / 2.0;
        let min_x = feet.x - back_x - half_width;
        let min_z = feet.z - back_z - half_width;
        let probe = Aabb::new(
            Vec3::new(min_x, feet.y - 1.0e-6, min_z),
            Vec3::new(min_x + 2.0 * half_width, feet.y, min_z + 2.0 * half_width),
        );
        let mut best = f64::MAX;
        self.exists = false;
        for bx in (probe.min.x.floor() as i32)..=(probe.max.x.floor() as i32) {
            for bz in (probe.min.z.floor() as i32)..=(probe.max.z.floor() as i32) {
                for by in
                    (probe.min.y.floor() as i32).wrapping_sub(1)..=(probe.max.y.floor() as i32)
                {
                    if !colliders.intersects(grid, BlockPos::new(bx, by, bz), probe) {
                        continue;
                    }
                    let dx = f64::from(bx) + 0.5 - feet.x;
                    let dy = f64::from(by) + 0.5 - feet.y;
                    let dz = f64::from(bz) + 0.5 - feet.z;
                    let distance = dx * dx + dy * dy + dz * dz;
                    if distance < best {
                        best = distance;
                        self.set(bx, by, bz);
                    }
                }
            }
        }
    }
}
