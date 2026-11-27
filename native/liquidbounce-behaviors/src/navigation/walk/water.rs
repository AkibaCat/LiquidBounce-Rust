// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Water.kt.
use super::costs::Costs;
use crate::navigation::{
    geometry::BlockPos,
    terrain::{Terrain, TerrainLevel, TerrainState},
};

#[derive(Clone, Copy, Debug)]
pub struct Water {
    can_sprint: bool,
}
impl Water {
    pub const MAX_AIR: i32 = 300;
    pub const SWIMMING_HEIGHT: f64 = 0.6_f32 as f64;
    pub fn new(can_sprint: bool) -> Self {
        Self { can_sprint }
    }
    pub fn fits<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
    ) -> bool {
        t.is_water(p)
            && (!self.is_low(t, p)
                || self.can_sprint
                    && self.is_submerged(t, p)
                    && t.has_room(p, Self::SWIMMING_HEIGHT))
    }
    pub fn is_low<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
    ) -> bool {
        !t.has_room(p, t.height())
    }
    pub fn is_submerged<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
    ) -> bool {
        t.is_water(p.above()) || !t.is_passable(p.above())
    }
    pub fn moves_underwater(air: i32) -> i32 {
        (f64::from(air / 2) / Costs::SWIM) as i32
    }
}
