// SPDX-License-Identifier: GPL-3.0-or-later
//! Owned staged goal and shared immutable snapshot level, without a self reference.
use super::{
    aabb::Axis,
    destination::{Destination, NearDestination},
    geometry::{BlockPos, Vec3},
    replanner::SearchGoal,
    terrain::{TerrainLevel, TerrainState},
};
use std::sync::Arc;

pub struct SharedLevel<L>(Arc<L>);
impl<L> SharedLevel<L> {
    pub fn new(level: L) -> Self {
        Self(Arc::new(level))
    }
}
impl<L> Clone for SharedLevel<L> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl<B: TerrainState, L: TerrainLevel<B>> TerrainLevel<B> for SharedLevel<L> {
    fn block_state(&self, pos: BlockPos) -> B {
        self.0.block_state(pos)
    }
    fn void_air(&self) -> B {
        self.0.void_air()
    }
    fn is_known(&self, pos: BlockPos) -> bool {
        self.0.is_known(pos)
    }
    fn is_inside_border(&self, x: f64, z: f64) -> bool {
        self.0.is_inside_border(x, z)
    }
    fn min_y(&self) -> i32 {
        self.0.min_y()
    }
    fn height(&self) -> i32 {
        self.0.height()
    }
    fn max_y(&self) -> i32 {
        self.0.max_y()
    }
    fn axis_step_order(&self, movement: Vec3) -> [Axis; 3] {
        self.0.axis_step_order(movement)
    }
    fn collision_shape_for(&self, state: &B, pos: BlockPos) -> Option<super::voxel_shape::VoxelShape> {
        self.0.collision_shape_for(state, pos)
    }
}

pub struct SearchDestination<K> {
    goal: NearDestination,
    start: Vec3,
    range: Option<f64>,
    known: K,
    max_stage_distance: f64,
}
impl<K: Fn(BlockPos) -> bool> SearchDestination<K> {
    pub fn new(descriptor: SearchGoal, known: K) -> Self {
        Self {
            goal: descriptor.goal,
            start: descriptor.start,
            range: descriptor.stage_range,
            known,
            max_stage_distance: descriptor
                .goal
                .distance(BlockPos::containing(descriptor.start))
                - 2.0,
        }
    }
    fn is_stage(&self, pos: BlockPos, range: f64) -> bool {
        let dx = f64::from(pos.x) + 0.5 - self.start.x;
        let dz = f64::from(pos.z) + 0.5 - self.start.z;
        dx * dx + dz * dz >= range * range
            || !(self.known)(pos.offset(1, 0, 0))
            || !(self.known)(pos.offset(-1, 0, 0))
            || !(self.known)(pos.offset(0, 0, 1))
            || !(self.known)(pos.offset(0, 0, -1))
    }
}
impl<K: Fn(BlockPos) -> bool> Destination for SearchDestination<K> {
    fn is_goal(&self, pos: BlockPos) -> bool {
        self.goal.is_goal(pos)
            || self.range.is_some_and(|range| {
                self.is_stage(pos, range) && self.goal.distance(pos) <= self.max_stage_distance
            })
    }
    fn distance(&self, pos: BlockPos) -> f64 {
        self.goal.distance(pos)
    }
    fn ends(&self) -> Vec<Vec3> {
        self.goal.ends()
    }
}
