// SPDX-License-Identifier: GPL-3.0-or-later
use super::super::{
    astar::{AStar, InvalidEdgeCost, WeightedEdge},
    destination::Destination,
    geometry::{BlockPos, Vec3, java_min},
    owned_search::OwnedTerrainSearch,
    search::{AStarSearch, Begin, Failure, Limits, Outcome, SearchProblem, closest_nodes},
    terrain::{Terrain, TerrainLevel},
};
use super::{
    moves::{Moves, RideState},
    vehicle::{Vehicle, VehicleKind},
};
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub node: BlockPos,
    pub position: Vec3,
    pub jump: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    pub start: Vec3,
    pub points: Vec<Point>,
    pub can_reach: bool,
    pub exhausted: bool,
    pub positions: Vec<Vec3>,
}
impl Route {
    pub fn new(start: Vec3, points: Vec<Point>, can_reach: bool, exhausted: bool) -> Self {
        let positions = std::iter::once(start)
            .chain(points.iter().map(|p| p.position))
            .collect();
        Self {
            start,
            points,
            can_reach,
            exhausted,
            positions,
        }
    }
    pub fn end(&self) -> Vec3 {
        *self.positions.last().expect("route contains its start")
    }
}
pub struct RideProblem<B: RideState, L: TerrainLevel<B>, D: Destination> {
    pub moves: Moves<B, L>,
    start: Vec3,
    goal: D,
}
impl<B: RideState, L: TerrainLevel<B>, D: Destination> RideProblem<B, L, D> {
    fn distance(&self, node: BlockPos) -> f64 {
        let distance = self.goal.distance(node);
        if self.moves.vehicle.kind == VehicleKind::Boat {
            java_min(distance, self.goal.distance(node.above()))
        } else {
            distance
        }
    }
    fn path(&mut self, nodes: Vec<BlockPos>, can_reach: bool, exhausted: bool) -> Outcome<Route> {
        let mut points = Vec::new();
        for pair in nodes.windows(2) {
            let Some(from) = self.moves.spot(pair[0]) else {
                return Outcome::Failed(Failure::NoPath);
            };
            let Some(to) = self.moves.spot(pair[1]) else {
                return Outcome::Failed(Failure::NoPath);
            };
            points.push(Point {
                node: pair[1],
                position: to.position,
                jump: self.moves.is_jump(from, to),
            });
        }
        Outcome::Found(Route::new(self.start, points, can_reach, exhausted))
    }
}
impl<B: RideState, L: TerrainLevel<B>, D: Destination> SearchProblem<BlockPos, Route>
    for RideProblem<B, L, D>
{
    fn begin(&mut self) -> Begin<BlockPos, Route> {
        let node = BlockPos::containing(self.start);
        let start = if self.moves.spot(node).is_some() {
            node
        } else if self.moves.spot(node.above()).is_some() {
            node.above()
        } else if self.moves.spot(node.offset(0, -1, 0)).is_some() {
            node.offset(0, -1, 0)
        } else {
            node
        };
        if self.is_goal(&start) {
            Begin::Outcome(Outcome::AlreadyThere)
        } else if self.moves.spot(start).is_none() {
            Begin::Outcome(Outcome::Failed(Failure::InvalidDestination))
        } else {
            Begin::from_start(start)
        }
    }
    fn is_goal(&mut self, node: &BlockPos) -> bool {
        self.goal.is_goal(*node)
            || self.moves.vehicle.kind == VehicleKind::Boat && self.goal.is_goal(node.above())
    }
    fn neighbors(&mut self, node: &BlockPos) -> Vec<WeightedEdge<BlockPos>> {
        self.moves.neighbors(*node)
    }
    fn heuristic(&mut self, node: &BlockPos) -> f64 {
        self.distance(*node) / self.moves.vehicle.speed
    }
    fn route(&mut self, nodes: Vec<BlockPos>) -> Outcome<Route> {
        self.path(nodes, true, false)
    }
    fn closest_or(&mut self, failure: Failure, search: Option<&AStar<BlockPos>>) -> Outcome<Route> {
        let Some(nodes) = closest_nodes(search, |node| self.distance(*node)) else {
            return Outcome::Failed(failure);
        };
        self.path(nodes, false, failure == Failure::NoPath)
    }
}
pub struct RouteSearch<B: RideState, L: TerrainLevel<B>, D: Destination> {
    pub engine: AStarSearch<BlockPos, Route>,
    pub problem: RideProblem<B, L, D>,
}
impl<B: RideState, L: TerrainLevel<B>, D: Destination> RouteSearch<B, L, D> {
    pub fn new(
        terrain: Terrain<B, L>,
        start: Vec3,
        goal: D,
        vehicle: Vehicle,
        limits: Limits,
    ) -> Self {
        Self {
            engine: AStarSearch::new(limits),
            problem: RideProblem {
                moves: Moves::new(terrain, vehicle),
                start,
                goal,
            },
        }
    }
    pub fn advance(&mut self) -> Result<Outcome<Route>, InvalidEdgeCost> {
        self.engine.advance(&mut self.problem)
    }
    pub fn advance_with_clock(
        &mut self,
        clock: impl FnMut() -> i64,
    ) -> Result<Outcome<Route>, InvalidEdgeCost> {
        self.engine.advance_with_clock(&mut self.problem, clock)
    }
    pub fn preview(&mut self) -> Option<Vec<Vec3>> {
        let nodes = self.engine.search.as_ref()?.closest_path()?.nodes;
        Some(
            nodes
                .into_iter()
                .filter_map(|n| self.problem.moves.spot(n).map(|s| s.position))
                .collect(),
        )
    }
}
impl<B: RideState, L: TerrainLevel<B>, D: Destination> OwnedTerrainSearch<Terrain<B, L>>
    for RouteSearch<B, L, D>
{
    type Route = Route;
    fn terrain_mut(&mut self) -> &mut Terrain<B, L> {
        &mut self.problem.moves.terrain
    }
    fn advance(&mut self) -> Outcome<Route> {
        RouteSearch::advance(self).expect("ride search edge cost is valid")
    }
    fn preview(&mut self) -> Option<Vec<Vec3>> {
        RouteSearch::preview(self)
    }
}
