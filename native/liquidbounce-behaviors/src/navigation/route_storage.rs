// SPDX-License-Identifier: GPL-3.0-or-later
//! Real owning route searches used by the unchanged background search lifecycle.
use super::{
    destination::Destination,
    flight::{FlyAbilities, FlyRoute, FlyRouteSearch},
    geometry::{BlockPos, Vec3},
    owned_search::OwnedTerrainSearch,
    replanner::SearchGoal,
    search::{Limits, Outcome},
    simulation::block_grid::SimulationState,
    staged_search::{SearchDestination, SharedLevel},
    terrain::{Terrain, TerrainLevel, TerrainState},
    walk::{route::Route, route_search::WalkRouteSearch},
};
pub type SharedDestination = SearchDestination<Box<dyn Fn(BlockPos) -> bool + Send + Sync>>;
pub fn shared_destination<B: TerrainState, L: TerrainLevel<B> + Send + Sync + 'static>(
    descriptor: SearchGoal,
    level: SharedLevel<L>,
) -> SharedDestination {
    SearchDestination::new(
        descriptor,
        Box::new(move |pos| {
            <SharedLevel<L> as TerrainLevel<B>>::is_known(&level, pos)
                && <SharedLevel<L> as TerrainLevel<B>>::is_inside_border(
                    &level,
                    f64::from(pos.x) + 0.5,
                    f64::from(pos.z) + 0.5,
                )
        }),
    )
}
pub fn flight_search<B: TerrainState, L: TerrainLevel<B> + Send + Sync + 'static>(
    terrain: Terrain<B, SharedLevel<L>>,
    start: Vec3,
    descriptor: SearchGoal,
    abilities: FlyAbilities,
    limits: Limits,
) -> FlyRouteSearch<B, SharedLevel<L>, SharedDestination> {
    let goal = shared_destination::<B, L>(descriptor, terrain.level().clone());
    FlyRouteSearch::new(terrain, start, goal, abilities, limits)
}
impl<B: TerrainState, L: TerrainLevel<B>, D: Destination> OwnedTerrainSearch<Terrain<B, L>>
    for FlyRouteSearch<B, L, D>
{
    type Route = FlyRoute;
    fn terrain_mut(&mut self) -> &mut Terrain<B, L> {
        self.problem.moves.terrain_mut()
    }
    fn advance(&mut self) -> Outcome<FlyRoute> {
        FlyRouteSearch::advance(self).expect("flight search edge cost is valid")
    }
    fn preview(&mut self) -> Option<Vec<Vec3>> {
        FlyRouteSearch::preview(self)
    }
}
impl<B: SimulationState, L: TerrainLevel<B>, D: Destination> OwnedTerrainSearch<Terrain<B, L>>
    for WalkRouteSearch<B, L, D>
{
    type Route = Route<B>;
    fn terrain_mut(&mut self) -> &mut Terrain<B, L> {
        &mut self.problem.terrain
    }
    fn advance(&mut self) -> Outcome<Route<B>> {
        WalkRouteSearch::advance(self).expect("walk search edge cost is valid")
    }
    fn preview(&mut self) -> Option<Vec<Vec3>> {
        WalkRouteSearch::preview(self)
    }
}
