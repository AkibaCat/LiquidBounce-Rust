// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Minecraft Agent navigation/flight/Flying.kt orchestration.
use super::{
    background::Executor,
    destination::NearDestination,
    flight::{Acceleration, FlyAbilities, FlyRoute},
    flight_body::FlightBody,
    flight_follower::Follower,
    flight_handling::{FlightPlayer, Handling},
    geometry::{BlockPos, Vec3},
    input::Input,
    owned_search::{OwnedTerrainSearch, SearchSlot, StoredLevel, StoredWorker},
    progress::FollowState,
    replanner::{Replanner, ReplannerLevel, RouteFactory, SearchGoal},
    search::{Limits, Outcome},
};
use std::sync::Arc;

pub trait FlightReport {
    fn following(&mut self, points: &[Vec3], follower: &Follower);
    fn invalid(&mut self);
    fn searching(&mut self, position: Vec3, goal: Vec3, radius: f64, preview: Option<&[Vec3]>);
    fn done(&mut self);
}
pub type FlightSearchFactory<T, S> =
    Arc<dyn Fn(T, Vec3, SearchGoal, FlyAbilities, Limits) -> S + Send + Sync>;
type Routes<T, S> = Replanner<Acceleration, SearchSlot<T, S>, StoredWorker, FlyRoute>;
pub struct Flying<T, S> {
    routes: Routes<T, S>,
    create: FlightSearchFactory<T, S>,
    pub handling: Handling,
    follower: Follower,
    stuck: bool,
    is_active: bool,
}
impl<T: Send + 'static, S: OwnedTerrainSearch<T, Route = FlyRoute> + Send + 'static> Flying<T, S> {
    pub fn new(executor: Arc<dyn Executor>, create: FlightSearchFactory<T, S>) -> Self {
        Self {
            routes: Replanner::new(32, executor),
            create,
            handling: Handling::default(),
            follower: Follower::default(),
            stuck: false,
            is_active: false,
        }
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn target(&self) -> Option<Vec3> {
        self.follower.target()
    }
    pub fn sprint(&self) -> bool {
        self.follower.sprint()
    }
    pub fn follower(&self) -> &Follower {
        &self.follower
    }

    #[allow(clippy::too_many_arguments)] // Context borrows are explicit at the Rust host boundary.
    pub fn navigate(
        &mut self,
        body: &mut impl FlightBody,
        level: &mut impl ReplannerLevel<T>,
        report: &mut impl FlightReport,
        goal: Vec3,
        radius: f64,
        max_length: f64,
        abilities: FlyAbilities,
        frame_y_rot: f32,
    ) {
        self.routes.tick();
        self.is_active = true;
        let position = body.player().position();
        self.collect_route(level, goal, position);
        if self.follower.path().is_some() && !self.follower.is_finished() {
            if self
                .follower
                .tick(body, &mut self.handling, frame_y_rot, abilities)
                == FollowState::Stuck
            {
                self.stuck = true;
            }
            report.following(
                &self
                    .follower
                    .path()
                    .expect("following route remains")
                    .points,
                &self.follower,
            );
        } else {
            self.follower.clear();
            body.press(Input::EMPTY, true);
            if self.routes.failed() {
                report.invalid();
            } else if self.routes.is_searching() {
                report.searching(position, goal, radius, self.routes.preview());
            } else {
                report.done();
            }
        }
        self.plan_if_needed(body, level, goal, radius, max_length, position, abilities);
    }
    pub fn halt(&mut self) {
        self.routes.reset();
        self.follower.clear();
        self.is_active = false;
        self.stuck = false;
    }
    pub fn block_changed(&mut self, pos: BlockPos) {
        self.routes.block_changed(pos);
    }
    pub fn level_changed(&mut self) {
        self.routes.level_changed();
    }
    fn collect_route(&mut self, level: &mut impl ReplannerLevel<T>, goal: Vec3, position: Vec3) {
        match self.routes.poll(&mut StoredLevel::new(level)) {
            Outcome::Searching => {}
            Outcome::AlreadyThere => self.accept(FlyRoute {
                start: position,
                points: vec![position, goal],
                can_reach: true,
            }),
            Outcome::Found(path) => self.accept(path),
            Outcome::Failed(_) if self.follower.is_finished() => self.follower.clear(),
            Outcome::Failed(_) => {}
        }
    }
    fn accept(&mut self, path: FlyRoute) {
        self.stuck = false;
        self.routes.route_ends(BlockPos::containing(path.end()));
        self.follower.follow(path);
    }
    #[allow(clippy::too_many_arguments)]
    fn plan_if_needed(
        &mut self,
        body: &impl FlightBody,
        level: &mut impl ReplannerLevel<T>,
        goal: Vec3,
        radius: f64,
        max_length: f64,
        position: Vec3,
        abilities: FlyAbilities,
    ) {
        let path = self.follower.path();
        let finished = path.is_some() && self.follower.is_finished();
        let near_stage = path.is_some_and(|path| {
            self.routes.ends_at_stage()
                && position.distance(path.end()) < Routes::<T, S>::STAGE_LEAD
        });
        let urgent = self.stuck || self.follower.is_off_path(body, &self.handling);
        let needed =
            path.is_none() || self.routes.has_goal_moved(goal) || finished || near_stage || urgent;
        if !needed || !self.routes.is_due(urgent) {
            return;
        }
        let mut limits = Limits::new(max_length);
        limits.max_visited_nodes = 20_000;
        let create = Arc::clone(&self.create);
        let search: RouteFactory<SearchSlot<T, S>, StoredWorker> =
            Arc::new(move |slot, descriptor| {
                slot.initialize(|terrain| create(terrain, position, descriptor, abilities, limits))
            });
        self.routes.request(
            &mut StoredLevel::new(level),
            position,
            NearDestination::new(goal, radius),
            max_length,
            abilities.control(),
            max_length / 2.0,
            search,
        );
        self.stuck = false;
    }
}
