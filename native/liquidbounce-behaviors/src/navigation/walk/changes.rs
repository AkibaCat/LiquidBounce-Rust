// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Changes.kt with explicit Terrain/Nodes dependencies.
use super::{
    WalkState,
    abilities::WalkAbilities,
    costs::Costs,
    nodes::{DX, DZ, Nodes},
};
use crate::navigation::{
    geometry::BlockPos,
    terrain::{Terrain, TerrainLevel},
};
#[derive(Default)]
pub struct Changes {
    pub breaks: Vec<BlockPos>,
    pub places: Vec<BlockPos>,
    pub break_cost: f64,
    pub from: BlockPos,
}
impl Changes {
    pub fn clear(&mut self) {
        self.breaks.clear();
        self.places.clear();
        self.break_cost = 0.0;
    }
    pub fn pass_or_break<B: WalkState, L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        nodes: &Nodes<B>,
        abilities: &WalkAbilities<B>,
        p: BlockPos,
    ) -> bool {
        if t.is_passable(p) {
            return true;
        }
        let ticks = self.break_ticks(t, nodes, abilities, p, false);
        if ticks.is_nan() {
            return false;
        }
        self.breaks.push(p);
        self.break_cost += ticks + Costs::BREAK_OVERHEAD;
        true
    }
    pub fn break_ticks<B: WalkState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        nodes: &Nodes<B>,
        abilities: &WalkAbilities<B>,
        p: BlockPos,
        floor: bool,
    ) -> f64 {
        let stood_on =
            p.x == self.from.x && p.z == self.from.z && p.y == self.from.y.wrapping_sub(1);
        if !abilities.can_break
            || !nodes.is_harmless(t, p)
            || stood_on && !floor
            || !nodes.is_dry(t, p)
            || abilities.unbreakable.contains(&p.packed())
        {
            return f64::NAN;
        }
        let state = t.state(p);
        if state.is_air() || t.state(p.above()).is_falling() {
            return f64::NAN;
        }
        let ticks = (abilities.break_ticks)(&state);
        if ticks.is_finite() { ticks } else { f64::NAN }
    }
    pub fn can_place_against<B: WalkState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        nodes: &Nodes<B>,
        p: BlockPos,
        on_floor: bool,
    ) -> bool {
        if nodes.is_solid(t, p.offset(0, -1, 0)) {
            return true;
        }
        for (dx, dz) in DX.into_iter().zip(DZ) {
            let side = p.offset(dx, 0, dz);
            if on_floor
                && side.x == self.from.x
                && side.z == self.from.z
                && p.y == self.from.y.wrapping_sub(1)
                || nodes.is_solid(t, side)
            {
                return true;
            }
        }
        false
    }
}
