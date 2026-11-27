// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent FollowState.kt and ProgressWatch.kt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FollowState {
    Walking,
    Acting,
    Finished,
    Stuck,
}
#[derive(Clone, Debug)]
pub struct ProgressWatch {
    min_progress: f64,
    stuck_ticks: i32,
    idle_ticks: i32,
    best_distance: f64,
}
impl ProgressWatch {
    pub fn new(min_progress: f64, stuck_ticks: i32) -> Self {
        Self {
            min_progress,
            stuck_ticks,
            idle_ticks: 0,
            best_distance: f64::MAX,
        }
    }
    pub fn idle_ticks(&self) -> i32 {
        self.idle_ticks
    }
    pub fn stalls(&mut self, distance: f64) -> bool {
        if distance < self.best_distance - self.min_progress {
            self.best_distance = distance;
            self.idle_ticks = 0;
            return false;
        }
        self.idle_ticks = self.idle_ticks.wrapping_add(1);
        self.idle_ticks >= self.stuck_ticks
    }
    pub fn excuse(&mut self) {
        self.idle_ticks = 0;
    }
    pub fn reset(&mut self) {
        self.best_distance = f64::MAX;
        self.idle_ticks = 0;
    }
}
