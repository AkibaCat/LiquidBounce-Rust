// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete navigation/glide/Airways.kt route lifecycle.
use super::super::{
    background::Executor,
    destination::NearDestination,
    flight::FlyRoute,
    geometry::{BlockPos, Vec3},
    owned_search::{OwnedTerrainSearch, SearchSlot, StoredLevel, StoredWorker},
    replanner::{Replanner, ReplannerLevel, RouteFactory, SearchGoal},
    search::{Limits, Outcome},
};
use super::passage::Passage;
use std::sync::Arc;

#[derive(Clone, Copy)]
struct Clearance(f64);
impl PartialEq for Clearance {
    fn eq(&self, other: &Self) -> bool {
        let bits = |v: f64| {
            if v.is_nan() {
                f64::NAN.to_bits()
            } else {
                v.to_bits()
            }
        };
        bits(self.0) == bits(other.0)
    }
}
pub type AirwayFactory<T, S> = Arc<dyn Fn(T, Vec3, SearchGoal, f64, Limits) -> S + Send + Sync>;
type Routes<T, S> = Replanner<Clearance, SearchSlot<T, S>, StoredWorker, FlyRoute>;
pub struct Airways<T, S> {
    routes: Routes<T, S>,
    create: AirwayFactory<T, S>,
    has_route: bool,
    reaches_goal: bool,
    found_at: i32,
}
impl<T: Send + 'static, S: OwnedTerrainSearch<T, Route = FlyRoute> + Send + 'static> Airways<T, S> {
    pub fn new(executor: Arc<dyn Executor>, create: AirwayFactory<T, S>) -> Self {
        Self {
            routes: Replanner::new(64, executor),
            create,
            has_route: false,
            reaches_goal: false,
            found_at: 0,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        passage: &mut Passage,
        level: &mut impl ReplannerLevel<T>,
        goal: Vec3,
        radius: f64,
        mut position: impl FnMut() -> Vec3,
        clearance: impl FnOnce() -> f64,
    ) {
        self.routes.tick();
        match self.routes.poll(&mut StoredLevel::new(level)) {
            Outcome::Found(route) => {
                self.routes.route_ends(BlockPos::containing(route.end()));
                self.reaches_goal = route.can_reach && !self.routes.ends_at_stage();
                self.accept(passage, &route);
            }
            Outcome::AlreadyThere => {
                self.reaches_goal = true;
                let start = position();
                self.accept(
                    passage,
                    &FlyRoute {
                        start,
                        points: vec![start, goal],
                        can_reach: true,
                    },
                );
            }
            Outcome::Searching | Outcome::Failed(_) => {}
        }
        let urgent = !self.has_route || passage.drift() > 4.0 || self.routes.has_goal_moved(goal);
        let due = urgent
            || !self.reaches_goal && passage.remaining() < 64.0
            || self.routes.ticks().wrapping_sub(self.found_at) > 60;
        if !due || !self.routes.is_due(urgent) {
            return;
        }
        let start = position();
        let clearance = clearance();
        let create = self.create.clone();
        let search: RouteFactory<SearchSlot<T, S>, StoredWorker> =
            Arc::new(move |slot, descriptor| {
                slot.initialize(|terrain| {
                    create(
                        terrain,
                        start,
                        descriptor,
                        clearance,
                        Limits {
                            max_length: 320.0,
                            max_visited_nodes: 120_000,
                            visited_nodes_per_slice: 8_000,
                            max_compute_nanos: 600_000_000,
                            ..Limits::new(320.0)
                        },
                    )
                })
            });
        self.routes.request(
            &mut StoredLevel::new(level),
            start,
            NearDestination::new(goal, radius + clearance + 1.0),
            320.0,
            Clearance(clearance),
            160.0,
            search,
        );
    }
    fn accept(&mut self, passage: &mut Passage, route: &FlyRoute) {
        passage.follow(route, Some(self.reaches_goal));
        self.has_route = true;
        self.found_at = self.routes.ticks();
    }
    pub fn reset(&mut self) {
        self.routes.reset();
        self.has_route = false;
        self.reaches_goal = false;
    }
    pub fn block_changed(&mut self, pos: BlockPos) {
        self.routes.block_changed(pos);
    }
    pub fn level_changed(&mut self) {
        self.routes.level_changed();
    }
}
