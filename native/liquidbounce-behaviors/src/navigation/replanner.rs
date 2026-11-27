// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac route/Replanner.kt.
use super::{
    background::{BackgroundLevel, BackgroundSearch, Executor, SearchFactory, SliceSearch},
    destination::{Destination, NearDestination},
    geometry::{BlockPos, BlockRegion, Vec3},
    search::Outcome,
};
use std::sync::Arc;

pub trait ReplannerLevel<T>: BackgroundLevel<T> {
    /// LiveLevel.isKnown: loaded chunk and dimension height, without the border test.
    fn is_known(&mut self, pos: BlockPos) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct SearchGoal {
    pub goal: NearDestination,
    pub start: Vec3,
    pub stage_range: Option<f64>,
}
#[derive(Clone, Debug)]
pub struct SearchKey<V> {
    pub start: BlockPos,
    pub goal: NearDestination,
    pub staged: bool,
    pub variant: V,
}
impl<V: PartialEq> PartialEq for SearchKey<V> {
    fn eq(&self, other: &Self) -> bool {
        fn bits(value: f64) -> u64 {
            if value.is_nan() {
                f64::NAN.to_bits()
            } else {
                value.to_bits()
            }
        }
        self.start == other.start
            && self.goal.center == other.goal.center
            && bits(self.goal.radius) == bits(other.goal.radius)
            && self.staged == other.staged
            && self.variant == other.variant
    }
}
#[derive(Clone, Copy)]
struct Request {
    goal: Vec3,
    stage: Option<NearDestination>,
}
impl Request {
    fn is_for(request: Option<Self>, goal: Vec3) -> bool {
        request.is_some_and(|request| request.goal.distance_sqr(goal) < 1.5 * 1.5)
    }
}

pub type RouteFactory<T, S> = Arc<dyn Fn(&mut T, SearchGoal) -> S + Send + Sync>;
pub struct Replanner<V, T, S, R> {
    background: BackgroundSearch<SearchKey<V>, T, S, R>,
    vertical_margin: i32,
    ticks: i32,
    last_request: i32,
    retry_at: i32,
    requested: Option<Request>,
    routed: Option<Request>,
    failed: bool,
    ends_at_stage: bool,
}
impl<
        V: PartialEq + Clone + Send + Sync + 'static,
        T: Send + 'static,
        S: SliceSearch<T, R> + Send + 'static,
        R: Clone + Send + 'static,
    > Replanner<V, T, S, R>
{
    pub const STAGE_LEAD: f64 = 12.0;
    pub fn new(vertical_margin: i32, executor: Arc<dyn Executor>) -> Self {
        Self {
            background: BackgroundSearch::new(executor),
            vertical_margin,
            ticks: 0,
            last_request: i32::MIN / 2,
            retry_at: 0,
            requested: None,
            routed: None,
            failed: false,
            ends_at_stage: false,
        }
    }
    pub fn ticks(&self) -> i32 {
        self.ticks
    }
    pub fn failed(&self) -> bool {
        self.failed
    }
    pub fn ends_at_stage(&self) -> bool {
        self.ends_at_stage
    }
    pub fn is_searching(&self) -> bool {
        !self.background.is_idle()
    }
    pub fn preview(&self) -> Option<&[Vec3]> {
        self.background.preview()
    }
    pub fn tick(&mut self) {
        self.ticks = self.ticks.wrapping_add(1);
    }
    pub fn poll(&mut self, level: &mut impl BackgroundLevel<T>) -> Outcome<R> {
        let result = self.background.poll(level);
        match result {
            Outcome::Searching => return result,
            Outcome::Failed(_) => {
                self.failed = true;
                self.retry_at = self.ticks.wrapping_add(20);
            }
            _ => {
                self.failed = false;
                self.ends_at_stage = false;
                self.routed = self.requested;
            }
        }
        self.background.cancel();
        result
    }
    pub fn route_ends(&mut self, end: BlockPos) {
        self.ends_at_stage = self
            .routed
            .and_then(|request| request.stage)
            .is_some_and(|stage| !stage.is_goal(end));
    }
    pub fn forget(&mut self) {
        self.routed = None;
    }
    pub fn has_goal_moved(&self, goal: Vec3) -> bool {
        !Request::is_for(self.routed, goal)
    }
    pub fn is_new_goal(&self, goal: Vec3) -> bool {
        !Request::is_for(self.requested, goal)
    }
    pub fn is_due(&self, urgent: bool) -> bool {
        self.ticks >= self.retry_at
            && (urgent || !self.is_searching() && self.ticks.wrapping_sub(self.last_request) >= 5)
    }

    #[allow(clippy::too_many_arguments)] // Source request plus the explicit main-thread level borrow.
    pub fn request(
        &mut self,
        level: &mut impl ReplannerLevel<T>,
        start: Vec3,
        goal: NearDestination,
        max_length: f64,
        variant: V,
        stage_range: f64,
        search: RouteFactory<T, S>,
    ) {
        if !level.has_level() {
            return;
        }
        let staged = start.distance_sqr(goal.center) >= (max_length / 2.0) * (max_length / 2.0)
            || !level.is_known(BlockPos::containing(goal.center));
        let region = if staged {
            BlockRegion::around(start, &[], stage_range, self.vertical_margin)
        } else {
            goal.region(start, max_length, self.vertical_margin)
        };
        let descriptor = SearchGoal {
            goal,
            start,
            stage_range: staged.then_some(stage_range),
        };
        let new_search: SearchFactory<T, S> = Arc::new(move |terrain| search(terrain, descriptor));
        self.background.request(
            level,
            SearchKey {
                start: BlockPos::containing(start),
                goal,
                staged,
                variant,
            },
            region,
            new_search,
        );
        if self.is_new_goal(goal.center) {
            self.failed = false;
        }
        self.requested = Some(Request {
            goal: goal.center,
            stage: staged.then_some(goal),
        });
        self.last_request = self.ticks;
    }
    pub fn reset(&mut self) {
        self.background.cancel();
        self.requested = None;
        self.routed = None;
        self.failed = false;
        self.ends_at_stage = false;
    }
    pub fn block_changed(&mut self, pos: BlockPos) {
        self.background.block_changed(pos);
    }
    pub fn level_changed(&mut self) {
        self.background.level_changed();
    }
}
