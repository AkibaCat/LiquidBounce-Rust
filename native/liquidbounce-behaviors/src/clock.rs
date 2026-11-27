// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce Chronometer.kt, Copyright (c) 2015-2026 CCBlueX.

/// Inject a real clock in integration, or a controlled clock in tests. Signed
/// wall-clock milliseconds reproduce System.currentTimeMillis(), including
/// backward clock adjustments and JVM Long overflow.
pub trait MillisClock {
    fn now_ms(&mut self) -> i64;
}

impl<F: FnMut() -> i64> MillisClock for F {
    fn now_ms(&mut self) -> i64 {
        self()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Chronometer {
    last_update: i64,
}

impl Chronometer {
    pub const fn new(last_update: i64) -> Self {
        Self { last_update }
    }

    pub const fn last_update(&self) -> i64 {
        self.last_update
    }

    pub fn elapsed(&self, clock: &mut impl MillisClock) -> i64 {
        self.elapsed_until(clock.now_ms())
    }

    pub fn elapsed_until(&self, time: i64) -> i64 {
        time.wrapping_sub(self.last_update)
    }

    pub fn has_elapsed(&self, ms: i64, clock: &mut impl MillisClock) -> bool {
        self.last_update.wrapping_add(ms) < clock.now_ms()
    }

    pub fn has_at_least_elapsed(&self, ms: i64, clock: &mut impl MillisClock) -> bool {
        self.last_update.wrapping_add(ms) <= clock.now_ms()
    }

    pub fn reset(&mut self, clock: &mut impl MillisClock) {
        self.reset_at(clock.now_ms());
    }

    pub fn reset_at(&mut self, last_update: i64) {
        self.last_update = last_update;
    }

    /// Upstream advances the timestamp; it does not sleep or await here.
    pub fn wait_for_at_least(&mut self, ms: i64, clock: &mut impl MillisClock) {
        self.last_update = self.last_update.max(clock.now_ms().wrapping_add(ms));
    }
}
