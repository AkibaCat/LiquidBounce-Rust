// SPDX-License-Identifier: GPL-3.0-or-later
//! Owning walk search using the same terrain snapshot as its staged destination.
use crate::navigation::{
    geometry::Vec3,
    replanner::SearchGoal,
    route_storage::{SharedDestination, shared_destination},
    search::Limits,
    simulation::block_grid::SimulationState,
    staged_search::SharedLevel,
    terrain::{Terrain, TerrainLevel},
    walk::{abilities::WalkAbilities, route_search::WalkRouteSearch},
};
pub fn walk_search<B: SimulationState, L: TerrainLevel<B> + Send + Sync + 'static>(
    terrain: Terrain<B, SharedLevel<L>>,
    start: Vec3,
    descriptor: SearchGoal,
    abilities: WalkAbilities<B>,
    limits: Limits,
) -> WalkRouteSearch<B, SharedLevel<L>, SharedDestination> {
    let goal = shared_destination::<B, L>(descriptor, terrain.level().clone());
    WalkRouteSearch::new(terrain, start, goal, abilities, limits)
}
