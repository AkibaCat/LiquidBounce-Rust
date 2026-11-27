// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent BlockGrid. Shapes are local to their block corner.
use super::super::{geometry::BlockPos, voxel_shape::VoxelShape, walk::WalkState};

pub trait ShapeGrid {
    fn shape(&mut self, pos: BlockPos) -> VoxelShape;
}
pub trait SimulationState: WalkState {
    fn is_fence(&self) -> bool;
    fn is_wall(&self) -> bool;
    fn is_fence_gate(&self) -> bool;
    fn is_cobweb(&self) -> bool;
    /// Exact Blocks.AIR or Blocks.VOID_AIR identity used by 1.8 step effects.
    fn is_step_air(&self) -> bool;
    fn fall_distance_reduction(&self) -> f32;
}
pub trait BlockGrid: ShapeGrid {
    type State: SimulationState;
    fn state(&mut self, pos: BlockPos) -> Self::State;
}

impl<B: SimulationState, L: super::super::terrain::TerrainLevel<B>> ShapeGrid
    for super::super::terrain::Terrain<B, L>
{
    fn shape(&mut self, pos: BlockPos) -> VoxelShape {
        super::super::terrain::Terrain::shape(self, pos)
    }
}
impl<B: SimulationState, L: super::super::terrain::TerrainLevel<B>> BlockGrid
    for super::super::terrain::Terrain<B, L>
{
    type State = B;
    fn state(&mut self, pos: BlockPos) -> B {
        super::super::terrain::Terrain::state(self, pos)
    }
}
