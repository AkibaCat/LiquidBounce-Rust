// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleExtinguish state and placement ordering from LiquidBounce ae0eb396.
use crate::{clock::Chronometer, timed_pickup_tracker::TimedPickupTracker};

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub cooldown: f32,
    pub not_during_combat: bool,
    pub pickup: bool,
    pub pickup_span: [f32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            cooldown: 1.,
            not_during_combat: true,
            pickup: true,
            pickup_span: [0.1, 10.],
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    pub now: i64,
    pub water_evaporates: bool,
    pub fire_resistant: bool,
    pub in_combat: bool,
    pub on_fire: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Pickup([i32; 3]),
    Extinguish,
}
pub struct Extinguish<T> {
    pub settings: Settings,
    pub current_target: Option<T>,
    pub cooldown: Chronometer,
    pub pickup_tracker: TimedPickupTracker,
}
impl<T> Default for Extinguish<T> {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            current_target: None,
            cooldown: Chronometer::default(),
            pickup_tracker: TimedPickupTracker::new(1),
        }
    }
}
impl<T> Extinguish<T> {
    pub fn enabled(&mut self) {
        self.current_target = None;
        self.pickup_tracker.clear();
    }
    pub fn world_changed(&mut self) {
        self.current_target = None;
    }
    /// A Nether early return deliberately retains the old plan, matching the source.
    /// Disabling releases only the silent slot at the runtime boundary.
    pub fn rotation_update(&mut self, c: Context, mut plan: impl FnMut(Action) -> Option<T>) {
        if c.water_evaporates {
            return;
        }
        self.current_target = None;
        self.pickup_tracker
            .prune((self.settings.pickup_span[1] * 1000.) as i64, c.now, |_| {
                true
            });
        if c.fire_resistant || (self.settings.not_during_combat && c.in_combat) {
            return;
        }
        if self.settings.pickup {
            if let Some(position) = self.pickup_tracker.first_eligible(
                (self.settings.pickup_span[0] * 1000.) as i64,
                c.now,
                |_| true,
            ) {
                self.current_target = plan(Action::Pickup(position));
                if self.current_target.is_some() {
                    return;
                }
            }
        }
        if c.on_fire && self.cooldown.has_elapsed(0, &mut || c.now) {
            self.current_target = plan(Action::Extinguish);
        }
    }
    pub fn succeeded(&mut self, placed: [i32; 3], now: i64) {
        self.cooldown
            .wait_for_at_least((self.settings.cooldown * 1000.) as i64, &mut || now);
        self.pickup_tracker.record(placed, now);
    }
}
