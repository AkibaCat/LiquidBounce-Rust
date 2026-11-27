// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleResetVL ae0eb396; its exclusive wait uses the shared ticker.

pub const TIMER_PRIORITY: i32 = 60;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub times: i32,
    pub timer: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            times: 10,
            timer: 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    RequestTimer(f32),
    Velocity(f64),
    Disable,
    NotifySuccess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    AfterTimer,
    AfterUp,
    Waiting(u64),
    AfterDown,
    AfterDisable,
    AfterNotification,
}

#[derive(Default, Debug)]
pub struct ResetVl {
    pub settings: Settings,
    /// Original singleton field: do not reset on module/world lifecycle changes.
    pub count: i32,
    next: u64,
    phase: Option<Phase>,
    registrations: Vec<u64>,
}

impl ResetVl {
    pub fn busy(&self) -> bool {
        self.phase.is_some()
    }
    pub fn take_wait_registrations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.registrations)
    }
    /// A real disable cancels suspended work immediately. Code already running
    /// synchronously after enabled=false still emits success and then clears count.
    pub fn cancel_sequences(&mut self) {
        if matches!(self.phase, Some(Phase::Waiting(_))) {
            self.phase = None;
        }
        self.registrations.clear();
    }
    /// tickHandler is Exclusive(DROP): no new run can overlap a suspended one.
    pub fn start_tick_handler(&mut self, running: bool) -> Option<Effect> {
        if !running || self.busy() {
            return None;
        }
        self.phase = Some(Phase::AfterTimer);
        Some(Effect::RequestTimer(self.settings.timer))
    }
    fn finish_check(&mut self) -> Option<Effect> {
        if self.count == self.settings.times {
            self.phase = Some(Phase::AfterDisable);
            Some(Effect::Disable)
        } else {
            self.phase = None;
            None
        }
    }
    /// Continue only after the actual native/Timer/registry effect has run.
    pub fn after_effect(&mut self, effect: Effect, on_ground: bool) -> Option<Effect> {
        match (self.phase, effect) {
            (Some(Phase::AfterTimer), Effect::RequestTimer(_)) if on_ground => {
                self.phase = Some(Phase::AfterUp);
                Some(Effect::Velocity(0.1))
            }
            (Some(Phase::AfterTimer), Effect::RequestTimer(_)) => self.finish_check(),
            (Some(Phase::AfterUp), Effect::Velocity(_)) => {
                self.next = self.next.checked_add(1).expect("ResetVL waiter exhausted");
                self.phase = Some(Phase::Waiting(self.next));
                self.registrations.push(self.next);
                None
            }
            (Some(Phase::AfterDown), Effect::Velocity(_)) => {
                self.count = self.count.wrapping_add(1);
                self.finish_check()
            }
            (Some(Phase::AfterDisable), Effect::Disable) => {
                self.phase = Some(Phase::AfterNotification);
                Some(Effect::NotifySuccess)
            }
            (Some(Phase::AfterNotification), Effect::NotifySuccess) => {
                self.count = 0;
                self.phase = None;
                None
            }
            _ => None,
        }
    }
    /// Wait1 resumes at this callback's FIFO position on the next outer tick.
    pub fn advance_callback(&mut self, id: u64, running: bool) -> (bool, Option<Effect>) {
        if self.phase != Some(Phase::Waiting(id)) {
            return (false, None);
        }
        if !running {
            self.phase = None;
            return (false, None);
        }
        self.phase = Some(Phase::AfterDown);
        (false, Some(Effect::Velocity(-0.1)))
    }
}
