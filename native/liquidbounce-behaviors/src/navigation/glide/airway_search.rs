// SPDX-License-Identifier: GPL-3.0-or-later
//! Original AirwaySearch through clearance-aware open space.
use super::super::{
    aabb::Aabb,
    astar::{AStar, InvalidEdgeCost, WeightedEdge},
    destination::Destination,
    flight::FlyRoute,
    geometry::{BlockPos, Vec3, java_min},
    owned_search::OwnedTerrainSearch,
    search::{AStarSearch, Begin, Failure, Limits, Outcome, SearchProblem, closest_nodes},
    terrain::{Terrain, TerrainLevel, TerrainState},
};
use std::collections::HashMap;
pub const BODY_SIZE: f64 = 0.6;
pub fn body_box(feet: Vec3, margin: f64) -> Aabb {
    let half = BODY_SIZE / 2.0 + margin;
    Aabb::new(
        Vec3::new(feet.x - half, feet.y - margin, feet.z - half),
        Vec3::new(feet.x + half, feet.y + BODY_SIZE + margin, feet.z + half),
    )
}
pub struct AirwaySearch<B: TerrainState, L: TerrainLevel<B>, D: Destination> {
    pub engine: AStarSearch<BlockPos, FlyRoute>,
    pub problem: AirwayProblem<B, L, D>,
}
impl<B: TerrainState, L: TerrainLevel<B>, D: Destination> AirwaySearch<B, L, D> {
    pub fn new(
        terrain: Terrain<B, L>,
        start: Vec3,
        goal: D,
        clearance: f64,
        staged: bool,
        limits: Limits,
    ) -> Self {
        Self {
            engine: AStarSearch::new(limits),
            problem: AirwayProblem::new(terrain, start, goal, clearance, staged),
        }
    }
    pub fn advance(&mut self) -> Result<Outcome<FlyRoute>, InvalidEdgeCost> {
        self.engine.advance(&mut self.problem)
    }
    pub fn advance_with_clock(
        &mut self,
        clock: impl FnMut() -> i64,
    ) -> Result<Outcome<FlyRoute>, InvalidEdgeCost> {
        self.engine.advance_with_clock(&mut self.problem, clock)
    }
    pub fn preview(&mut self) -> Option<Vec<Vec3>> {
        Some(
            self.engine
                .search
                .as_ref()?
                .closest_path()?
                .nodes
                .into_iter()
                .map(|node| self.problem.point(node))
                .collect(),
        )
    }
}
impl<B: TerrainState, L: TerrainLevel<B>, D: Destination> OwnedTerrainSearch<Terrain<B, L>>
    for AirwaySearch<B, L, D>
{
    type Route = FlyRoute;
    fn terrain_mut(&mut self) -> &mut Terrain<B, L> {
        &mut self.problem.terrain
    }
    fn advance(&mut self) -> Outcome<FlyRoute> {
        AirwaySearch::advance(self).expect("airway search edge cost is valid")
    }
    fn preview(&mut self) -> Option<Vec<Vec3>> {
        AirwaySearch::preview(self)
    }
}
pub struct AirwayProblem<B: TerrainState, L: TerrainLevel<B>, D: Destination> {
    pub terrain: Terrain<B, L>,
    start: Vec3,
    goal: D,
    clearance: f64,
    staged: bool,
    start_node: BlockPos,
    open: HashMap<i64, bool>,
    roomy: HashMap<i64, bool>,
    stage_distance: f64,
    edge_reach: i32,
}
impl<B: TerrainState, L: TerrainLevel<B>, D: Destination> AirwayProblem<B, L, D> {
    fn new(terrain: Terrain<B, L>, start: Vec3, goal: D, clearance: f64, staged: bool) -> Self {
        let start_node = BlockPos::containing(Vec3::new(start.x, start.y - clearance, start.z));
        let stage_distance = goal.distance(BlockPos::new(
            start_node.x,
            (f64::from(start_node.y) + clearance).floor() as i32,
            start_node.z,
        )) - 2.0;
        Self {
            terrain,
            start,
            goal,
            clearance,
            staged,
            start_node,
            open: HashMap::new(),
            roomy: HashMap::new(),
            stage_distance,
            edge_reach: (clearance.ceil() as i32).wrapping_add(2),
        }
    }
    fn node(&self, feet: Vec3) -> BlockPos {
        BlockPos::containing(Vec3::new(feet.x, feet.y - self.clearance, feet.z))
    }
    fn feet_block(&self, node: BlockPos) -> BlockPos {
        BlockPos::new(
            node.x,
            (f64::from(node.y) + self.clearance).floor() as i32,
            node.z,
        )
    }
    fn point(&self, node: BlockPos) -> Vec3 {
        Vec3::new(
            f64::from(node.x) + 0.5,
            f64::from(node.y) + self.clearance,
            f64::from(node.z) + 0.5,
        )
    }
    fn node_box(&self, x: i32, y: i32, z: i32, margin: f64) -> Aabb {
        body_box(
            Vec3::new(
                f64::from(x) + 0.5,
                f64::from(y) + self.clearance,
                f64::from(z) + 0.5,
            ),
            margin,
        )
    }
    fn is_clear(&mut self, body: Aabb) -> bool {
        self.terrain.is_clear(body) && !self.terrain.contains_any_liquid(body)
    }
    pub fn is_clear_line(&mut self, from: Vec3, to: Vec3, margin: f64) -> bool {
        let delta = Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z);
        let samples = 1.max(
            ((delta.x * delta.x + delta.y * delta.y + delta.z * delta.z).sqrt() / 0.5).ceil()
                as i32,
        );
        for sample in 0..=samples {
            if !self.is_clear(body_box(
                from.fma(f64::from(sample) / f64::from(samples), delta),
                margin,
            )) {
                return false;
            }
        }
        true
    }
    pub fn is_open(&mut self, x: i32, y: i32, z: i32) -> bool {
        let key = BlockPos::new(x, y, z).packed();
        if let Some(&open) = self.open.get(&key) {
            return open;
        }
        let distance = x
            .wrapping_sub(self.start_node.x)
            .wrapping_abs()
            .max(y.wrapping_sub(self.start_node.y).wrapping_abs())
            .max(z.wrapping_sub(self.start_node.z).wrapping_abs());
        let margin = java_min(
            self.clearance,
            f64::from(distance.wrapping_sub(1).max(0)) * 0.25,
        );
        let open = self.is_clear(self.node_box(x, y, z, margin));
        self.open.insert(key, open);
        open
    }
    pub fn is_roomy(&mut self, x: i32, y: i32, z: i32) -> bool {
        let key = BlockPos::new(x, y, z).packed();
        if let Some(&roomy) = self.roomy.get(&key) {
            return roomy;
        }
        let roomy = self.is_clear(self.node_box(x, y, z, self.clearance + 1.5));
        self.roomy.insert(key, roomy);
        roomy
    }
    fn can_move(&mut self, node: BlockPos, dx: i32, dy: i32, dz: i32) -> bool {
        for x in 0.min(dx)..=0.max(dx) {
            for y in 0.min(dy)..=0.max(dy) {
                for z in 0.min(dz)..=0.max(dz) {
                    if (x != 0 || y != 0 || z != 0)
                        && !self.is_open(
                            node.x.wrapping_add(x),
                            node.y.wrapping_add(y),
                            node.z.wrapping_add(z),
                        )
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn next_to_unknown(&self, node: BlockPos) -> bool {
        let y = self.feet_block(node).y;
        !self.terrain.is_known(BlockPos::new(
            node.x.wrapping_add(self.edge_reach),
            y,
            node.z,
        )) || !self.terrain.is_known(BlockPos::new(
            node.x.wrapping_sub(self.edge_reach),
            y,
            node.z,
        )) || !self.terrain.is_known(BlockPos::new(
            node.x,
            y,
            node.z.wrapping_add(self.edge_reach),
        )) || !self.terrain.is_known(BlockPos::new(
            node.x,
            y,
            node.z.wrapping_sub(self.edge_reach),
        ))
    }
    fn path(&mut self, nodes: Vec<BlockPos>, can_reach: bool) -> FlyRoute {
        let mut points = vec![self.start];
        for node in nodes.into_iter().skip(1) {
            points.push(self.point(node));
        }
        let mut corners = vec![points[0]];
        let mut anchor = 0;
        while anchor < points.len() - 1 {
            let mut next = anchor + 1;
            while next < points.len() - 1
                && self.is_clear_line(points[anchor], points[next + 1], self.clearance + 0.5)
            {
                next += 1;
            }
            corners.push(points[next]);
            anchor = next;
        }
        FlyRoute {
            start: self.start,
            points: corners,
            can_reach,
        }
    }
}
impl<B: TerrainState, L: TerrainLevel<B>, D: Destination> SearchProblem<BlockPos, FlyRoute>
    for AirwayProblem<B, L, D>
{
    fn begin(&mut self) -> Begin<BlockPos, FlyRoute> {
        let node = self.node(self.start);
        if self.goal.is_goal(self.feet_block(node)) {
            return Begin::Outcome(Outcome::AlreadyThere);
        }
        for end in self.goal.ends() {
            if self.goal.is_goal(BlockPos::containing(end))
                && self.is_clear_line(self.start, end, self.clearance)
            {
                return Begin::Outcome(Outcome::Found(FlyRoute {
                    start: self.start,
                    points: vec![self.start, end],
                    can_reach: true,
                }));
            }
        }
        Begin::from_start(node)
    }
    fn is_goal(&mut self, node: &BlockPos) -> bool {
        let feet = self.feet_block(*node);
        self.goal.is_goal(feet)
            || self.staged
                && self.next_to_unknown(*node)
                && self.goal.distance(feet) <= self.stage_distance
    }
    fn neighbors(&mut self, node: &BlockPos) -> Vec<WeightedEdge<BlockPos>> {
        let mut edges = Vec::with_capacity(26);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 || !self.can_move(*node, dx, dy, dz) {
                        continue;
                    }
                    let to = node.offset(dx, dy, dz);
                    let mut cost = f64::from(dx * dx + dy * dy + dz * dz).sqrt();
                    if dy > 0 {
                        cost *= 4.0;
                    }
                    if !self.is_roomy(to.x, to.y, to.z) {
                        cost += 1.0;
                    }
                    edges.push(WeightedEdge { node: to, cost });
                }
            }
        }
        edges
    }
    fn heuristic(&mut self, node: &BlockPos) -> f64 {
        self.goal.distance(self.feet_block(*node)) * 1.5
    }
    fn route(&mut self, nodes: Vec<BlockPos>) -> Outcome<FlyRoute> {
        Outcome::Found(self.path(nodes, true))
    }
    fn closest_or(
        &mut self,
        failure: Failure,
        search: Option<&AStar<BlockPos>>,
    ) -> Outcome<FlyRoute> {
        match closest_nodes(search, |node| self.goal.distance(self.feet_block(*node))) {
            Some(nodes) => Outcome::Found(self.path(nodes, false)),
            None => Outcome::Failed(failure),
        }
    }
}
