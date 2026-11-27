// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Minecraft Agent BlockColliders, including modern and 1.8 offsets.
use super::super::{
    aabb::{Aabb, Axis},
    geometry::{BlockPos, Vec3, java_max, java_min},
};
use super::block_grid::ShapeGrid;
use std::collections::HashMap;

const EPSILON: f64 = 1.0e-7;
#[derive(Default, Debug)]
pub struct BlockColliders {
    blocks: HashMap<i64, Vec<[f64; 6]>>,
    colliders: Vec<[f64; 6]>,
    moving: [f64; 6],
    pub moved_x: f64,
    pub moved_y: f64,
    pub moved_z: f64,
}
impl BlockColliders {
    fn boxes(&mut self, grid: &mut impl ShapeGrid, pos: BlockPos) -> &[[f64; 6]] {
        self.blocks.entry(pos.packed()).or_insert_with(|| {
            grid.shape(pos)
                .aabbs()
                .iter()
                .map(|b| {
                    [
                        b.min.x + f64::from(pos.x),
                        b.min.y + f64::from(pos.y),
                        b.min.z + f64::from(pos.z),
                        b.max.x + f64::from(pos.x),
                        b.max.y + f64::from(pos.y),
                        b.max.z + f64::from(pos.z),
                    ]
                })
                .collect()
        })
    }
    pub fn intersects(&mut self, grid: &mut impl ShapeGrid, pos: BlockPos, body: Aabb) -> bool {
        self.boxes(grid, pos).iter().any(|b| {
            b[0] < body.max.x
                && b[3] > body.min.x
                && b[1] < body.max.y
                && b[4] > body.min.y
                && b[2] < body.max.z
                && b[5] > body.min.z
        })
    }
    pub fn footing(
        &mut self,
        grid: &mut impl ShapeGrid,
        pos: BlockPos,
        x: f64,
        z: f64,
        half_width: f64,
    ) -> f64 {
        self.boxes(grid, pos).iter().fold(0.0, |best, b| {
            let along_x = java_min(x + half_width - b[0], b[3] - (x - half_width));
            let along_z = java_min(z + half_width - b[2], b[5] - (z - half_width));
            java_max(best, java_min(along_x, along_z))
        })
    }
    pub fn collect(&mut self, grid: &mut impl ShapeGrid, body: Aabb) {
        self.colliders.clear();
        for x in ((body.min.x - EPSILON).floor() as i32)..=((body.max.x + EPSILON).floor() as i32) {
            for z in
                ((body.min.z - EPSILON).floor() as i32)..=((body.max.z + EPSILON).floor() as i32)
            {
                for y in ((body.min.y - EPSILON).floor() as i32).wrapping_sub(1)
                    ..=((body.max.y + EPSILON).floor() as i32)
                {
                    let boxes = self.boxes(grid, BlockPos::new(x, y, z)).to_vec();
                    self.colliders.extend(boxes);
                }
            }
        }
    }
    pub fn collide(&mut self, delta: Vec3, body: Aabb) {
        if self.colliders.is_empty() {
            self.moved_x = delta.x;
            self.moved_y = delta.y;
            self.moved_z = delta.z;
            return;
        }
        self.set_moving(body);
        self.moved_y = self.clip(1, delta.y);
        if delta.x.abs() < delta.z.abs() {
            self.moved_z = self.clip(2, delta.z);
            self.moved_x = self.clip(0, delta.x);
        } else {
            self.moved_x = self.clip(0, delta.x);
            self.moved_z = self.clip(2, delta.z);
        }
    }
    pub fn offset(&mut self, axis: Axis, body: Aabb, movement: f64) -> f64 {
        self.set_moving(body);
        let axis = axis.index();
        let a = (axis + 1) % 3;
        let b = (axis + 2) % 3;
        let mut moved = movement;
        for collider in &self.colliders {
            if collider[3 + a] <= self.moving[a]
                || collider[a] >= self.moving[3 + a]
                || collider[3 + b] <= self.moving[b]
                || collider[b] >= self.moving[3 + b]
            {
                continue;
            }
            if moved > 0.0 && self.moving[3 + axis] <= collider[axis] {
                moved = java_min(moved, collider[axis] - self.moving[3 + axis]);
            } else if moved < 0.0 && self.moving[axis] >= collider[3 + axis] {
                moved = java_max(moved, collider[3 + axis] - self.moving[axis]);
            }
        }
        moved
    }
    pub fn step_heights(&self, body: Aabb, max_step: f32, skip: f32) -> Vec<f32> {
        let mut candidates: Vec<f32> = Vec::with_capacity(4);
        for collider in &self.colliders {
            for face in [1, 4] {
                let relative = (collider[face] - body.min.y) as f32;
                if relative >= 0.0
                    && relative != skip
                    && relative <= max_step
                    && !candidates.iter().any(|v| v.to_bits() == relative.to_bits())
                {
                    candidates.push(relative);
                }
            }
        }
        candidates.sort_unstable_by(f32::total_cmp);
        candidates
    }
    fn set_moving(&mut self, body: Aabb) {
        self.moving = [
            body.min.x, body.min.y, body.min.z, body.max.x, body.max.y, body.max.z,
        ];
    }
    fn clip(&mut self, axis: usize, movement: f64) -> f64 {
        if movement.abs() < EPSILON {
            return 0.0;
        }
        let mut moved = movement;
        for collider in &self.colliders {
            if self.overlaps(collider, (axis + 1) % 3) && self.overlaps(collider, (axis + 2) % 3) {
                if moved > 0.0 {
                    let gap = collider[axis] - self.moving[3 + axis];
                    if gap > -EPSILON {
                        moved = java_min(moved, gap);
                    }
                } else {
                    let gap = collider[3 + axis] - self.moving[axis];
                    if gap < EPSILON {
                        moved = java_max(moved, gap);
                    }
                }
            }
            if moved.abs() < EPSILON {
                moved = 0.0;
                break;
            }
        }
        self.moving[axis] += moved;
        self.moving[3 + axis] += moved;
        moved
    }
    fn overlaps(&self, collider: &[f64; 6], axis: usize) -> bool {
        collider[3 + axis] - EPSILON > self.moving[axis]
            && collider[axis] + EPSILON < self.moving[3 + axis]
    }
}
