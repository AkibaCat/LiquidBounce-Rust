// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent Terrain movement/stand queries and StandHeight.kt.
use super::{
    Aabb, BlockPos, DestinationCheck, Terrain, TerrainLevel, TerrainState, Vec3, VoxelShape,
    intersects, visit_blocks,
};
use crate::navigation::{aabb::Axis, geometry::java_min};
impl<B: TerrainState, L: TerrainLevel<B>> Terrain<B, L> {
    pub fn stand_height(&mut self, pos: BlockPos) -> f64 {
        let key = pos.packed();
        if let Some(&height) = self.stands.get(&key) {
            return height;
        }
        let height = self.compute_stand_height(pos, self.height);
        self.stands.insert(key, height);
        height
    }
    pub fn crouch_height(&mut self, pos: BlockPos) -> f64 {
        let key = pos.packed();
        if let Some(&height) = self.crouch_stands.get(&key) {
            return height;
        }
        let height = self.compute_stand_height(pos, java_min(self.height, Self::CROUCH_HEIGHT));
        self.crouch_stands.insert(key, height);
        height
    }
    pub fn is_standable(&mut self, pos: BlockPos) -> bool {
        !self.stand_height(pos).is_nan()
    }
    pub fn feet(&mut self, pos: BlockPos) -> Option<Vec3> {
        let height = self.stand_height(pos);
        (!height.is_nan())
            .then(|| Vec3::new(f64::from(pos.x) + 0.5, height, f64::from(pos.z) + 0.5))
    }
    pub fn check(&mut self, feet: BlockPos) -> DestinationCheck {
        if feet.y < self.min_y() || feet.y > self.max_y() {
            DestinationCheck::OutsideWorld
        } else if !self
            .level
            .is_inside_border(f64::from(feet.x) + 0.5, f64::from(feet.z) + 0.5)
        {
            DestinationCheck::OutsideBorder
        } else if !self.level.is_known(feet) {
            DestinationCheck::NotLoaded
        } else if !self.is_standable(feet) {
            DestinationCheck::NotStandable
        } else {
            DestinationCheck::Valid
        }
    }
    pub fn is_clear(&mut self, body: Aabb) -> bool {
        visit_blocks(body, |pos| {
            let shape = self.shape(pos);
            shape.is_empty() || !intersects(&shape, pos, body)
        })
    }
    pub fn contains_any_liquid(&mut self, body: Aabb) -> bool {
        !visit_blocks(body, |pos| !self.state(pos).has_liquid())
    }
    pub fn sweep(&mut self, from: Vec3, to: Vec3) -> bool {
        let movement = Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z);
        let body = self.body_box(from);
        let colliders = self.colliders(body.expand_towards(movement));
        if colliders.is_empty() {
            return true;
        }
        let mut moved = Vec3::ZERO;
        for axis in self.level.axis_step_order(movement) {
            let distance = axis.choose([movement.x, movement.y, movement.z]);
            if distance == 0.0 {
                continue;
            }
            let collision = VoxelShape::collide_all(axis, body.moved(moved), &colliders, distance);
            if (collision - distance).abs() > 1.0e-7 {
                return false;
            }
            moved = match axis {
                Axis::X => Vec3::new(collision, moved.y, moved.z),
                Axis::Y => Vec3::new(moved.x, collision, moved.z),
                Axis::Z => Vec3::new(moved.x, moved.y, collision),
            };
        }
        true
    }
    pub fn is_supported(&mut self, feet: Vec3) -> bool {
        let body = self.body_box(feet);
        !self.is_clear(Aabb::new(
            Vec3::new(body.min.x, feet.y - 1.0e-3, body.min.z),
            Vec3::new(body.max.x, feet.y, body.max.z),
        ))
    }
    pub fn has_room(&mut self, pos: BlockPos, height: f64) -> bool {
        let body = self.body_box_height(Vec3::bottom_center(pos), height);
        self.is_clear(body)
    }
    pub fn is_open(&mut self, pos: BlockPos) -> bool {
        self.is_passable(pos) && self.is_passable(pos.above())
    }
    pub fn is_bowl(&mut self, pos: BlockPos) -> bool {
        let at = self.shape(pos);
        if at.is_empty() || at.same_instance(&VoxelShape::block()) {
            return false;
        }
        let top = self.stand_height(pos);
        top > f64::from(pos.y)
            && at
                .aabbs()
                .iter()
                .any(|b| f64::from(pos.y) + b.max.y > top + Self::STEP_HEIGHT)
    }
    fn colliders(&mut self, region: Aabb) -> Vec<VoxelShape> {
        let mut colliders = Vec::new();
        visit_blocks(region, |pos| {
            let shape = self.shape(pos);
            if !shape.is_empty() {
                colliders.push(shape.moved(Vec3::new(
                    f64::from(pos.x),
                    f64::from(pos.y),
                    f64::from(pos.z),
                )));
            }
            true
        });
        colliders
    }
    fn compute_stand_height(&mut self, pos: BlockPos, body_height: f64) -> f64 {
        let below_pos = pos.offset(0, -1, 0);
        if pos.y > self.max_y() || !self.is_known(pos) || !self.is_known(below_pos) {
            return f64::NAN;
        }
        let below = self.shape(below_pos);
        let at = self.shape(pos);
        if at.same_instance(&VoxelShape::block())
            || below.is_empty() && at.is_empty()
            || self.is_hazard(below_pos)
            || self.is_hazard(pos)
        {
            return f64::NAN;
        }
        let top = if below.same_instance(&VoxelShape::block()) && at.is_empty() {
            f64::from(pos.y)
        } else {
            partial_top(&below, &at, pos.y, self.width / 2.0)
        };
        if top.is_nan() {
            return f64::NAN;
        }
        let body = self.body_box_height(
            Vec3::new(f64::from(pos.x) + 0.5, top, f64::from(pos.z) + 0.5),
            body_height,
        );
        if self.is_clear(body) { top } else { f64::NAN }
    }
}
fn footprint(body: Aabb, half_width: f64) -> bool {
    body.min.x < 0.5 + half_width
        && body.max.x > 0.5 - half_width
        && body.min.z < 0.5 + half_width
        && body.max.z > 0.5 - half_width
}
// The source's !(boxTop <= top) intentionally accepts unordered comparisons;
// top starts as NaN, so replacing it with boxTop > top would discard every floor.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn partial_top(below: &VoxelShape, at: &VoxelShape, y: i32, half_width: f64) -> f64 {
    let mut top = f64::NAN;
    for &body in below.aabbs() {
        let box_top = f64::from(y.wrapping_sub(1)) + body.max.y;
        if footprint(body, half_width)
            && box_top >= f64::from(y)
            && box_top < f64::from(y.wrapping_add(1))
            && !(box_top <= top)
        {
            top = box_top;
        }
    }
    for &body in at.aabbs() {
        if !footprint(body, half_width) {
            continue;
        }
        let box_top = f64::from(y) + body.max.y;
        if box_top >= f64::from(y.wrapping_add(1)) {
            return f64::NAN;
        }
        if !(box_top <= top) {
            top = box_top;
        }
    }
    top
}
