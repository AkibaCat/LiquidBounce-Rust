// SPDX-License-Identifier: GPL-3.0-or-later
//! Owning airway searches using the same snapshot as their staged destination.
use super::super::{
    geometry::Vec3,
    replanner::SearchGoal,
    route_storage::{SharedDestination, shared_destination},
    search::Limits,
    staged_search::SharedLevel,
    terrain::{Terrain, TerrainLevel, TerrainState},
};
use super::airway_search::AirwaySearch;

pub fn airway_search<B: TerrainState, L: TerrainLevel<B> + Send + Sync + 'static>(
    terrain: Terrain<B, SharedLevel<L>>,
    start: Vec3,
    descriptor: SearchGoal,
    clearance: f64,
    limits: Limits,
) -> AirwaySearch<B, SharedLevel<L>, SharedDestination> {
    let goal = shared_destination::<B, L>(descriptor, terrain.level().clone());
    AirwaySearch::new(
        terrain,
        start,
        goal,
        clearance,
        descriptor.stage_range.is_some(),
        limits,
    )
}
