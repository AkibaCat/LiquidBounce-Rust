// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent walk/Planning.kt over the real owning Replanner lifecycle.
use super::{
    ability_capture::{CaptureHost, capture_walk},
    body::{ControlState, WalkPlayer},
    break_times::BreakState,
    follower::{Follower, FollowerHost},
    path_points::RouteControl,
    stuck_spots::StuckSpots,
};
use crate::navigation::{
    background::Executor,
    destination::NearDestination,
    geometry::{BlockPos, Vec3},
    owned_search::{OwnedTerrainSearch, SearchSlot, StoredLevel, StoredWorker},
    replanner::{Replanner, ReplannerLevel, RouteFactory, SearchGoal},
    search::{Limits, Outcome},
    simulation::player_motion::MotionPlayer,
    walk::{abilities::WalkAbilities, route::Route},
};
use std::{collections::HashSet, sync::Arc};

pub trait PlanningHost: CaptureHost + FollowerHost {
    /// options().maxLength, including its separate live options read.
    fn max_length(&self) -> f64;
    fn following_path_debug(&mut self, steps: usize, end: Vec3);
    fn route_found(&mut self, path: &Arc<Route<<Self::Player as WalkPlayer>::State>>);
    fn route_searching(&mut self, start: Vec3, goal: Vec3, radius: f64);
}
pub type WalkSearchFactory<B, T, S> =
    Arc<dyn Fn(T, Vec3, SearchGoal, WalkAbilities<B>, Limits) -> S + Send + Sync>;
type Routes<B, T, S> = Replanner<i32, SearchSlot<T, S>, StoredWorker, Route<B>>;
pub struct Planning<B: ControlState, T, S> {
    routes: Routes<B, T, S>,
    create: WalkSearchFactory<B, T, S>,
    stuck_spots: StuckSpots,
    failures: i32,
    stuck: bool,
    new_stuck_spot: bool,
    was_off_path: bool,
    settled_at: Option<i32>,
}
impl<
        B: ControlState + BreakState + 'static,
        T: Send + 'static,
        S: OwnedTerrainSearch<T, Route = Route<B>> + Send + 'static,
    > Planning<B, T, S>
{
    pub fn new(executor: Arc<dyn Executor>, create: WalkSearchFactory<B, T, S>) -> Self {
        Self {
            routes: Replanner::new(16, executor),
            create,
            stuck_spots: StuckSpots::default(),
            failures: 0,
            stuck: false,
            new_stuck_spot: false,
            was_off_path: false,
            settled_at: None,
        }
    }
    pub fn is_searching(&self) -> bool {
        self.routes.is_searching()
    }
    pub fn failed(&self) -> bool {
        self.routes.failed()
    }
    pub fn preview(&self) -> Option<&[Vec3]> {
        self.routes.preview()
    }
    pub fn is_settled(&self) -> bool {
        self.settled_at.is_some()
    }
    pub fn tick(&mut self, goal: Vec3) {
        self.routes.tick();
        self.stuck_spots.tick(goal);
    }
    pub fn stuck_at(&mut self, pos: BlockPos) {
        self.new_stuck_spot = self.stuck_spots.add(pos) || self.new_stuck_spot;
        self.stuck = true;
    }
    pub fn drop_route(&mut self) {
        self.routes.reset();
        self.stuck = false;
        self.new_stuck_spot = false;
    }
    pub fn reset(&mut self) {
        self.routes.reset();
        self.failures = 0;
        self.stuck = false;
        self.new_stuck_spot = false;
        self.was_off_path = false;
        self.settled_at = None;
    }
    pub fn forget_stuck_spots(&mut self) {
        self.stuck_spots.clear();
    }
    pub fn block_changed(&mut self, pos: BlockPos) {
        self.routes.block_changed(pos);
    }
    pub fn level_changed(&mut self) {
        self.routes.level_changed();
    }
    pub fn collect<H: PlanningHost>(
        &mut self,
        host: &mut H,
        follower: &mut Follower<B>,
        level: &mut impl ReplannerLevel<T>,
        position: Vec3,
    ) where
        H::Player: WalkPlayer<State = B>,
    {
        match self.routes.poll(&mut StoredLevel::new(level)) {
            Outcome::Searching => {}
            Outcome::AlreadyThere => {
                follower.clear();
                self.settled_at = Some(self.routes.ticks());
            }
            Outcome::Found(path) => self.accept(host, follower, Arc::new(path), position),
            Outcome::Failed(_) => {
                self.failures = self.failures.wrapping_add(1);
                if follower.is_finished() {
                    follower.clear();
                }
            }
        }
    }
    fn accept<H: PlanningHost>(
        &mut self,
        host: &mut H,
        follower: &mut Follower<B>,
        path: Arc<Route<B>>,
        position: Vec3,
    ) where
        H::Player: WalkPlayer<State = B>,
    {
        self.failures = 0;
        self.stuck = false;
        if path.steps().is_empty() {
            follower.clear();
            self.settled_at = Some(self.routes.ticks());
            return;
        }
        let Some(start_index) = path.join_index(position) else {
            self.routes.forget();
            return;
        };
        self.routes
            .route_ends(path.steps().last().expect("nonempty path").to);
        self.settled_at = None;
        follower.follow(path.clone(), start_index);
        host.following_path_debug(path.steps().len(), path.end());
        host.route_found(&path);
    }
    #[allow(clippy::too_many_arguments)]
    pub fn plan_if_needed<H: PlanningHost>(
        &mut self,
        host: &mut H,
        follower: &Follower<B>,
        level: &mut impl ReplannerLevel<T>,
        unbreakable: &HashSet<i64>,
        goal: Vec3,
        radius: f64,
        position: Vec3,
    ) where
        H::Player: WalkPlayer<State = B>,
    {
        let path = follower.path();
        let off_path = follower.is_off_path(host);
        let left_path = off_path && !self.was_off_path;
        self.was_off_path = off_path;
        let finished_short = path.is_some() && follower.is_finished();
        let near_stage = path.is_some()
            && self.routes.ends_at_stage()
            && host.player().on_ground()
            && super::horizontal(position, path.expect("checked path").end()) < 12.0;
        let settled = self
            .settled_at
            .is_some_and(|at| self.routes.ticks().wrapping_sub(at) < 100);
        let needed = path.is_none() && !settled
            || self.routes.has_goal_moved(goal)
            || off_path
            || finished_short
            || self.stuck
            || near_stage;
        if needed && self.routes.is_due(self.new_stuck_spot || left_path) {
            self.request(host, follower, level, unbreakable, goal, radius, position);
            self.stuck = false;
            self.new_stuck_spot = false;
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn request<H: PlanningHost>(
        &mut self,
        host: &mut H,
        follower: &Follower<B>,
        level: &mut impl ReplannerLevel<T>,
        unbreakable: &HashSet<i64>,
        goal: Vec3,
        radius: f64,
        start: Vec3,
    ) where
        H::Player: WalkPlayer<State = B>,
    {
        if self.routes.is_new_goal(goal) {
            self.failures = 0;
        }
        let max_length = host.max_length() * f64::from(1_i32 << self.failures.min(2));
        let abilities = capture_walk(host, &self.stuck_spots.blocks, unbreakable);
        let limits = Limits {
            visited_nodes_per_slice: 5_000,
            ..if abilities.can_parkour {
                Limits {
                    max_visited_nodes: 40_000,
                    max_compute_nanos: 2_000_000_000,
                    ..Limits::new(max_length)
                }
            } else {
                Limits::new(max_length)
            }
        };
        let create = self.create.clone();
        let search: RouteFactory<SearchSlot<T, S>, StoredWorker> =
            Arc::new(move |slot, descriptor| {
                slot.initialize(|terrain| create(terrain, start, descriptor, abilities.clone(), limits))
            });
        self.routes.request(
            &mut StoredLevel::new(level),
            start,
            NearDestination::new(goal, radius),
            max_length,
            self.stuck_spots.version(),
            max_length * 0.75,
            search,
        );
        if follower.path().is_none() && !self.routes.failed() {
            host.route_searching(start, goal, radius);
        }
    }
    /// Original private orchestration state, useful to source-oracle adapters.
    pub fn diagnostics(&self) -> (i32, bool, bool, bool, Option<i32>) {
        (
            self.failures,
            self.stuck,
            self.new_stuck_spot,
            self.was_off_path,
            self.settled_at,
        )
    }
    pub fn stuck_spots(&self) -> &StuckSpots {
        &self.stuck_spots
    }
}
