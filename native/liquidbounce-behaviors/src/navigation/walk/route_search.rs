// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent walk RouteSearch.kt using the real current AStar context.
use super::{abilities::WalkAbilities, costs::Costs, moves::Moves, route::Route, water::Water};
use crate::navigation::{
    astar::{AStar, InvalidEdgeCost, WeightedEdge},
    destination::Destination,
    geometry::{BlockPos, Vec3},
    search::{AStarSearch, Begin, Failure, Limits, Outcome, SearchProblem, closest_nodes},
    simulation::block_grid::SimulationState,
    terrain::{Terrain, TerrainLevel},
};
use std::collections::HashMap;
pub struct WalkProblem<B: SimulationState, L: TerrainLevel<B>, D: Destination> {
    pub terrain: Terrain<B, L>,
    pub start: Vec3,
    pub goal: D,
    abilities: WalkAbilities<B>,
    moves: Moves<B>,
    speed: f64,
    start_node: BlockPos,
    dives: HashMap<i64, i32>,
    max_dive: i32,
    placed: HashMap<i64, i32>,
    placed_after: Vec<i32>,
    offered: HashMap<i64, f64>,
}
impl<B: SimulationState, L: TerrainLevel<B>, D: Destination> WalkProblem<B, L, D> {
    pub fn new(terrain: Terrain<B, L>, start: Vec3, goal: D, abilities: WalkAbilities<B>) -> Self {
        Self {
            moves: Moves::new(terrain.width(), abilities.clone()),
            terrain,
            start,
            goal,
            speed: if abilities.can_sprint {
                Costs::SPRINT
            } else {
                Costs::WALK
            },
            start_node: BlockPos::containing(start),
            abilities,
            dives: HashMap::new(),
            max_dive: Water::moves_underwater(Water::MAX_AIR),
            placed: HashMap::new(),
            placed_after: Vec::new(),
            offered: HashMap::new(),
        }
    }
    fn find_start_node(&mut self) -> BlockPos {
        let node = BlockPos::containing(self.start);
        let nodes = self.moves.nodes();
        let t = &mut self.terrain;
        if nodes.is_node(t, node) {
            node
        } else if nodes.is_node(t, node.above()) {
            node.above()
        } else if nodes.is_node(t, node.offset(0, -1, 0)) {
            node.offset(0, -1, 0)
        } else {
            self.held_by(node).unwrap_or(node)
        }
    }
    fn held_by(&mut self, node: BlockPos) -> Option<BlockPos> {
        let mut best = None;
        let mut best_distance = f64::MAX;
        let half = f64::from(0.6_f32) / 2.0;
        for x in ((self.start.x - half).floor() as i32)..=((self.start.x + half).floor() as i32) {
            for z in ((self.start.z - half).floor() as i32)..=((self.start.z + half).floor() as i32)
            {
                let dx = f64::from(x) + 0.5 - self.start.x;
                let dz = f64::from(z) + 0.5 - self.start.z;
                let distance = dx * dx + dz * dz;
                let p = BlockPos::new(x, node.y, z);
                if (x != node.x || z != node.z)
                    && distance < best_distance
                    && self.moves.nodes().is_node(&mut self.terrain, p)
                {
                    best = Some(p);
                    best_distance = distance;
                }
            }
        }
        best
    }
    fn dive_length(&mut self, node: BlockPos, search: Option<&AStar<BlockPos>>) -> i32 {
        let dive = if !self.moves.nodes().is_underwater(&mut self.terrain, node) {
            0
        } else if node == self.start_node {
            self.max_dive
                .wrapping_sub(Water::moves_underwater(self.abilities.air))
        } else {
            search
                .and_then(|s| s.previous(&node))
                .and_then(|previous| self.dives.get(&previous.packed()))
                .copied()
                .unwrap_or(0)
                .wrapping_add(1)
        };
        self.dives.insert(node.packed(), dive);
        dive
    }
    fn edges(
        &mut self,
        node: BlockPos,
        search: Option<&AStar<BlockPos>>,
    ) -> Vec<WeightedEdge<BlockPos>> {
        let dive = self.dive_length(node, search);
        let placed_before = self.placed.get(&node.packed()).copied().unwrap_or(0);
        let mut edges = Vec::new();
        self.placed_after.clear();
        let reachable = search.map_or(0.0, |s| s.cost(&node)) + 6.0;
        let max_dive = self.max_dive;
        let max_blocks = self.abilities.placeable_blocks;
        let avoid = &self.abilities.avoid;
        let placed_after = &mut self.placed_after;
        self.moves.for_each(
            &mut self.terrain,
            node,
            &mut |t, nodes, offer| {
                let to = offer.to;
                let upwards = to.x == node.x && to.z == node.z && to.y > node.y;
                if dive >= max_dive && !upwards && nodes.is_underwater(t, to) {
                    return;
                }
                let placed_total = placed_before.wrapping_add(offer.places.len() as i32);
                if placed_total > max_blocks {
                    return;
                }
                let penalty = if avoid.contains(&to.packed()) {
                    Costs::AVOID
                } else {
                    0.0
                };
                edges.push(WeightedEdge {
                    node: to,
                    cost: offer.cost + penalty,
                });
                placed_after.push(placed_total);
            },
            &mut |to| search.map_or(f64::INFINITY, |s| s.cost(&to)) > reachable,
        );
        if self.abilities.can_place {
            self.record_placed(node, &edges, search);
        }
        edges
    }
    fn record_placed(
        &mut self,
        node: BlockPos,
        edges: &[WeightedEdge<BlockPos>],
        search: Option<&AStar<BlockPos>>,
    ) {
        let Some(search) = search else {
            return;
        };
        let from = search.cost(&node);
        self.offered.clear();
        for (i, edge) in edges.iter().enumerate() {
            let key = edge.node.packed();
            let cost = from + edge.cost;
            let best = self
                .offered
                .get(&key)
                .copied()
                .unwrap_or_else(|| search.cost(&edge.node));
            if cost < best {
                self.offered.insert(key, cost);
                self.placed.insert(key, self.placed_after[i]);
            }
        }
    }
    fn end_on_ground(&mut self, mut nodes: Vec<BlockPos>) -> Option<Vec<BlockPos>> {
        let placed = self
            .placed
            .get(&nodes.last()?.packed())
            .copied()
            .unwrap_or(0);
        if !self.abilities.can_place || placed < self.abilities.placeable_blocks {
            return Some(nodes);
        }
        let end = nodes
            .iter()
            .rposition(|&node| self.moves.nodes().is_node(&mut self.terrain, node))?;
        if end >= 1 {
            nodes.truncate(end + 1);
            Some(nodes)
        } else {
            None
        }
    }
    fn path(&mut self, nodes: Vec<BlockPos>, can_reach: bool) -> Outcome<Route<B>> {
        let mut steps = Vec::with_capacity(nodes.len().saturating_sub(1));
        for pair in nodes.windows(2) {
            let Some(step) = self.moves.step(&mut self.terrain, pair[0], pair[1]) else {
                return Outcome::Failed(Failure::NoPath);
            };
            steps.push(step);
        }
        Outcome::Found(Route::new(
            self.start,
            steps,
            can_reach,
            self.abilities.clone(),
        ))
    }
}
impl<B: SimulationState, L: TerrainLevel<B>, D: Destination> SearchProblem<BlockPos, Route<B>>
    for WalkProblem<B, L, D>
{
    fn begin(&mut self) -> Begin<BlockPos, Route<B>> {
        self.start_node = self.find_start_node();
        if self.goal.is_goal(self.start_node) {
            Begin::Outcome(Outcome::AlreadyThere)
        } else {
            Begin::from_start(self.start_node)
        }
    }
    fn is_goal(&mut self, node: &BlockPos) -> bool {
        self.goal.is_goal(*node)
    }
    fn heuristic(&mut self, node: &BlockPos) -> f64 {
        self.goal.distance(*node) * self.speed
    }
    fn neighbors(&mut self, node: &BlockPos) -> Vec<WeightedEdge<BlockPos>> {
        self.edges(*node, None)
    }
    fn neighbors_with_context(
        &mut self,
        node: &BlockPos,
        search: &AStar<BlockPos>,
    ) -> Vec<WeightedEdge<BlockPos>> {
        self.edges(*node, Some(search))
    }
    fn route(&mut self, nodes: Vec<BlockPos>) -> Outcome<Route<B>> {
        self.path(nodes, true)
    }
    fn closest_or(
        &mut self,
        failure: Failure,
        search: Option<&AStar<BlockPos>>,
    ) -> Outcome<Route<B>> {
        match closest_nodes(search, |node| self.goal.distance(*node))
            .and_then(|nodes| self.end_on_ground(nodes))
        {
            Some(nodes) => self.path(nodes, false),
            None => Outcome::Failed(failure),
        }
    }
}
pub struct WalkRouteSearch<B: SimulationState, L: TerrainLevel<B>, D: Destination> {
    pub engine: AStarSearch<BlockPos, Route<B>>,
    pub problem: WalkProblem<B, L, D>,
}
impl<B: SimulationState, L: TerrainLevel<B>, D: Destination> WalkRouteSearch<B, L, D> {
    pub fn new(
        terrain: Terrain<B, L>,
        start: Vec3,
        goal: D,
        abilities: WalkAbilities<B>,
        limits: Limits,
    ) -> Self {
        Self {
            engine: AStarSearch::new(limits),
            problem: WalkProblem::new(terrain, start, goal, abilities),
        }
    }
    pub fn advance(&mut self) -> Result<Outcome<Route<B>>, InvalidEdgeCost> {
        self.engine.advance(&mut self.problem)
    }
    pub fn advance_with_clock(
        &mut self,
        clock: impl FnMut() -> i64,
    ) -> Result<Outcome<Route<B>>, InvalidEdgeCost> {
        self.engine.advance_with_clock(&mut self.problem, clock)
    }
    pub fn preview(&self) -> Option<Vec<Vec3>> {
        Some(
            self.engine
                .search
                .as_ref()?
                .closest_path()?
                .nodes
                .into_iter()
                .map(Vec3::bottom_center)
                .collect(),
        )
    }
}
