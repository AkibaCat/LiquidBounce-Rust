// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent flight FlyAbilities, Moves, Route and RouteSearch.
use super::{
    astar::{AStar, InvalidEdgeCost, WeightedEdge},
    destination::Destination,
    geometry::{BlockPos, Vec3, java_max, java_min},
    search::{AStarSearch, Begin, Failure, Limits, Outcome, SearchProblem, closest_nodes},
    terrain::{Terrain, TerrainLevel, TerrainState},
};
use std::{cmp::Ordering, collections::HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acceleration {
    Momentum,
    Direct,
}
#[derive(Clone, Copy, Debug)]
pub struct FlyAbilities {
    control: Acceleration,
    horizontal_speed: f64,
    vertical_speed: f64,
    is_flying: bool,
    horizontal_cost: f64,
    vertical_cost: f64,
}
impl FlyAbilities {
    pub const VERTICAL_DRAG: f64 = 0.6;
    pub fn new(
        control: Acceleration,
        horizontal_speed: f64,
        vertical_speed: f64,
        is_flying: bool,
    ) -> Self {
        Self {
            control,
            horizontal_speed,
            vertical_speed,
            is_flying,
            horizontal_cost: 1.0 / horizontal_speed,
            vertical_cost: 1.0 / vertical_speed,
        }
    }
    pub fn creative(flying_speed: f32, is_flying: bool) -> Self {
        let speed = f64::from(flying_speed);
        Self::new(
            Acceleration::Momentum,
            2.0 * speed * 0.98 / (1.0 - f64::from(0.91_f32)),
            speed * 3.0 / (1.0 - Self::VERTICAL_DRAG),
            is_flying,
        )
    }
    pub fn of(may_fly: bool, flying: bool, flying_speed: f32) -> Option<Self> {
        (may_fly || flying).then(|| Self::creative(flying_speed, flying))
    }
    pub fn horizontal_cost(self) -> f64 {
        self.horizontal_cost
    }
    pub fn vertical_cost(self) -> f64 {
        self.vertical_cost
    }
    pub fn control(self) -> Acceleration {
        self.control
    }
    pub fn horizontal_speed(self) -> f64 {
        self.horizontal_speed
    }
    pub fn vertical_speed(self) -> f64 {
        self.vertical_speed
    }
    pub fn is_flying(self) -> bool {
        self.is_flying
    }
}
pub struct FlyMoves<B: TerrainState, L: TerrainLevel<B>, D: Destination> {
    terrain: Terrain<B, L>,
    abilities: FlyAbilities,
    goal: D,
    open: HashMap<i64, bool>,
}
impl<B: TerrainState, L: TerrainLevel<B>, D: Destination> FlyMoves<B, L, D> {
    pub fn new(terrain: Terrain<B, L>, abilities: FlyAbilities, goal: D) -> Self {
        Self {
            terrain,
            abilities,
            goal,
            open: HashMap::new(),
        }
    }
    pub fn terrain(&self) -> &Terrain<B, L> {
        &self.terrain
    }
    pub fn terrain_mut(&mut self) -> &mut Terrain<B, L> {
        &mut self.terrain
    }
    pub fn is_open(&mut self, pos: BlockPos) -> bool {
        if let Some(&open) = self.open.get(&pos.packed()) {
            return open;
        }
        let open = self.terrain.is_known(pos)
            && self.terrain.is_known(pos.above())
            && self
                .terrain
                .is_clear(self.terrain.body_box(Vec3::bottom_center(pos)));
        self.open.insert(pos.packed(), open);
        open
    }
    pub fn point(&mut self, node: BlockPos) -> Vec3 {
        self.terrain
            .feet(node)
            .unwrap_or_else(|| Vec3::bottom_center(node))
    }
    pub fn neighbors(&mut self, node: BlockPos) -> Vec<WeightedEdge<BlockPos>> {
        let mut edges = Vec::with_capacity(26);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 || !self.can_move(node, dx, dy, dz) {
                        continue;
                    }
                    let to = node.offset(dx, dy, dz);
                    let cost = self.cost(to, dx, dy, dz);
                    edges.push(WeightedEdge { node: to, cost });
                }
            }
        }
        edges
    }
    fn can_move(&mut self, node: BlockPos, dx: i32, dy: i32, dz: i32) -> bool {
        for x in 0.min(dx)..=0.max(dx) {
            for y in 0.min(dy)..=0.max(dy) {
                for z in 0.min(dz)..=0.max(dz) {
                    if !self.is_open(node.offset(x, y, z)) {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn cost(&mut self, to: BlockPos, dx: i32, dy: i32, dz: i32) -> f64 {
        let horizontal = (f64::from(dx) * f64::from(dx) + f64::from(dz) * f64::from(dz)).sqrt()
            * self.abilities.horizontal_cost;
        let mut cost = java_max(
            horizontal,
            f64::from(dy.abs()) * self.abilities.vertical_cost,
        );
        if self.terrain.is_water(to) || self.terrain.is_water(to.above()) {
            cost *= 4.0;
        }
        if self.abilities.control == Acceleration::Momentum
            && !self.goal.is_goal(to)
            && !self.terrain.stand_height(to).is_nan()
        {
            cost += 1.0;
        }
        cost
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct FlyRoute {
    pub start: Vec3,
    pub points: Vec<Vec3>,
    pub can_reach: bool,
}
impl FlyRoute {
    pub fn end(&self) -> Vec3 {
        *self.points.last().expect("flight route has a point")
    }
}
pub struct FlyProblem<B: TerrainState, L: TerrainLevel<B>, D: Destination> {
    pub moves: FlyMoves<B, L, D>,
    start: Vec3,
    max_length: f64,
    min_cost: f64,
    goal_points: Option<Vec<Vec3>>,
}
impl<B: TerrainState, L: TerrainLevel<B>, D: Destination> FlyProblem<B, L, D> {
    pub fn new(
        terrain: Terrain<B, L>,
        start: Vec3,
        goal: D,
        abilities: FlyAbilities,
        max_length: f64,
    ) -> Self {
        Self {
            moves: FlyMoves::new(terrain, abilities, goal),
            start,
            max_length,
            min_cost: java_min(abilities.horizontal_cost, abilities.vertical_cost),
            goal_points: None,
        }
    }
    fn goal_points(&mut self) -> Vec<Vec3> {
        if let Some(points) = &self.goal_points {
            return points.clone();
        }
        let reach_sqr = self.max_length * self.max_length;
        let mut points = Vec::new();
        for end in self.moves.goal.ends() {
            if end.distance_sqr(self.start) >= reach_sqr {
                continue;
            }
            let node = BlockPos::containing(end);
            if self.moves.goal.is_goal(node) && self.moves.is_open(node) {
                points.push(self.moves.point(node));
            }
        }
        self.goal_points = Some(points.clone());
        points
    }
    fn straight_route(&mut self) -> Option<Outcome<FlyRoute>> {
        let point = self
            .goal_points()
            .into_iter()
            .find(|&point| is_clear_line(&mut self.moves.terrain, self.start, point))?;
        Some(Outcome::Found(FlyRoute {
            start: self.start,
            points: vec![self.start, point],
            can_reach: true,
        }))
    }
    fn climb_route(&mut self) -> Option<Outcome<FlyRoute>> {
        for point in self.goal_points() {
            let base = java_max(self.start.y, point.y).floor() as i32;
            for height in
                base.wrapping_add(1)..=base.wrapping_add(32).min(self.moves.terrain.max_y())
            {
                let up = Vec3::new(self.start.x, f64::from(height), self.start.z);
                if !is_clear_line(&mut self.moves.terrain, self.start, up) {
                    break;
                }
                let over = Vec3::new(point.x, f64::from(height), point.z);
                if is_clear_line(&mut self.moves.terrain, up, over)
                    && is_clear_line(&mut self.moves.terrain, over, point)
                {
                    let points = self.simplify(&[self.start, up, over, point]);
                    return Some(Outcome::Found(FlyRoute {
                        start: self.start,
                        points,
                        can_reach: true,
                    }));
                }
            }
        }
        None
    }
    fn path(&mut self, nodes: Vec<BlockPos>, can_reach: bool) -> FlyRoute {
        let mut points = Vec::with_capacity(nodes.len());
        points.push(self.start);
        for node in nodes.into_iter().skip(1) {
            points.push(self.moves.point(node));
        }
        FlyRoute {
            start: self.start,
            points: self.simplify(&points),
            can_reach,
        }
    }
    fn simplify(&mut self, points: &[Vec3]) -> Vec<Vec3> {
        let mut corners = vec![points[0]];
        let mut anchor = 0;
        while anchor < points.len() - 1 {
            let mut next = anchor + 1;
            while next < points.len() - 1
                && is_clear_line(&mut self.moves.terrain, points[anchor], points[next + 1])
            {
                next += 1;
            }
            corners.push(points[next]);
            anchor = next;
        }
        corners
    }
}
impl<B: TerrainState, L: TerrainLevel<B>, D: Destination> SearchProblem<BlockPos, FlyRoute>
    for FlyProblem<B, L, D>
{
    fn begin(&mut self) -> Begin<BlockPos, FlyRoute> {
        let feet = BlockPos::containing(self.start);
        let fits_above = !self.moves.is_open(feet) && self.moves.is_open(feet.above());
        let start = if fits_above { feet.above() } else { feet };
        if self.moves.goal.is_goal(start) {
            return Begin::Outcome(Outcome::AlreadyThere);
        }
        self.straight_route()
            .map_or_else(|| Begin::from_start(start), Begin::Outcome)
    }
    fn is_goal(&mut self, node: &BlockPos) -> bool {
        self.moves.goal.is_goal(*node)
    }
    fn neighbors(&mut self, node: &BlockPos) -> Vec<WeightedEdge<BlockPos>> {
        self.moves.neighbors(*node)
    }
    fn heuristic(&mut self, node: &BlockPos) -> f64 {
        let distance = self.moves.goal.distance(*node);
        let center = Vec3::bottom_center(*node);
        let mut nearest = None;
        for end in self.moves.goal.ends() {
            if nearest.is_none_or(|(best, _)| {
                double_compare(center.distance_sqr(end), best) == Ordering::Less
            }) {
                nearest = Some((center.distance_sqr(end), end));
            }
        }
        let Some((_, end)) = nearest else {
            return distance * self.min_cost;
        };
        let delta = Vec3::new(end.x - center.x, end.y - center.y, end.z - center.z);
        let length = (delta.x * delta.x + delta.y * delta.y + delta.z * delta.z).sqrt();
        if length < 1.0e-6 {
            return 0.0;
        }
        let horizontal = distance * (delta.x * delta.x + delta.z * delta.z).sqrt() / length
            * self.moves.abilities.horizontal_cost;
        let vertical = distance * delta.y.abs() / length * self.moves.abilities.vertical_cost;
        java_max(horizontal, vertical) * 1.5
    }
    fn route(&mut self, nodes: Vec<BlockPos>) -> Outcome<FlyRoute> {
        Outcome::Found(self.path(nodes, true))
    }
    fn closest_or(
        &mut self,
        failure: Failure,
        search: Option<&AStar<BlockPos>>,
    ) -> Outcome<FlyRoute> {
        if let Some(route) = self.climb_route() {
            return route;
        }
        match closest_nodes(search, |node| self.moves.goal.distance(*node)) {
            Some(nodes) => Outcome::Found(self.path(nodes, false)),
            None => Outcome::Failed(failure),
        }
    }
}
pub struct FlyRouteSearch<B: TerrainState, L: TerrainLevel<B>, D: Destination> {
    pub engine: AStarSearch<BlockPos, FlyRoute>,
    pub problem: FlyProblem<B, L, D>,
}
impl<B: TerrainState, L: TerrainLevel<B>, D: Destination> FlyRouteSearch<B, L, D> {
    pub fn new(
        terrain: Terrain<B, L>,
        start: Vec3,
        goal: D,
        abilities: FlyAbilities,
        limits: Limits,
    ) -> Self {
        Self {
            engine: AStarSearch::new(limits),
            problem: FlyProblem::new(terrain, start, goal, abilities, limits.max_length),
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
        let nodes = self.engine.search.as_ref()?.closest_path()?.nodes;
        Some(
            nodes
                .into_iter()
                .map(|node| self.problem.moves.point(node))
                .collect(),
        )
    }
}
pub fn is_clear_line<B: TerrainState, L: TerrainLevel<B>>(
    terrain: &mut Terrain<B, L>,
    from: Vec3,
    to: Vec3,
) -> bool {
    let delta = Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z);
    let length = (delta.x * delta.x + delta.y * delta.y + delta.z * delta.z).sqrt();
    let samples = 1.max((length / 0.25).ceil() as i32);
    for sample in 0..=samples {
        let along = length * f64::from(sample) / f64::from(samples);
        let margin = if along < 0.5 || length - along < 0.5 {
            0.0
        } else {
            0.1
        };
        let point = from.fma(f64::from(sample) / f64::from(samples), delta);
        if !terrain.is_clear(
            terrain
                .body_box(point)
                .inflate(Vec3::new(margin, 0.0, margin)),
        ) {
            return false;
        }
    }
    true
}
fn double_compare(left: f64, right: f64) -> Ordering {
    if left < right {
        return Ordering::Less;
    }
    if left > right {
        return Ordering::Greater;
    }
    let bits = |value: f64| {
        if value.is_nan() {
            0x7ff8_0000_0000_0000i64
        } else {
            value.to_bits() as i64
        }
    };
    bits(left).cmp(&bits(right))
}
