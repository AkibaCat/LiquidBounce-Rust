// SPDX-License-Identifier: GPL-3.0-or-later
// LiquidBounce ae0eb396 ModuleBlink and BlinkManager queue operations.
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub directions: u32,
    pub dummy: bool,
    pub ambush: bool,
    pub auto_disable: bool,
    pub auto_reset: bool,
    pub reset_after: i32,
    pub reset_action: usize,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            directions: 2,
            dummy: false,
            ambush: false,
            auto_disable: true,
            auto_reset: false,
            reset_after: 100,
            reset_action: 0,
        }
    }
}
#[derive(Default)]
pub struct Blink {
    pub settings: Settings,
    pub ticks: i32,
}
impl Blink {
    pub fn enabled(&mut self) {
        self.ticks = 0;
    }
    pub fn movement(&mut self) -> Option<usize> {
        self.ticks = self.ticks.wrapping_add(1);
        if self.settings.auto_reset && self.ticks > self.settings.reset_after {
            self.ticks = 0;
            Some(self.settings.reset_action)
        } else {
            None
        }
    }
    pub fn queues(&self, running: bool, outgoing: bool) -> bool {
        running && self.settings.directions & if outgoing { 2 } else { 1 } != 0
    }
}

/// Shared packet metadata is deliberately independent of the host protocol.
pub struct Snapshot<P> {
    pub packet: P,
    pub outgoing: bool,
    pub millis: i64,
    pub position: Option<[f64; 3]>,
    pub movement: bool,
}
pub struct Queue<P>(pub VecDeque<Snapshot<P>>);
impl<P> Default for Queue<P> {
    fn default() -> Self {
        Self(VecDeque::new())
    }
}
impl<P> Queue<P> {
    pub fn positions(&self) -> Vec<[f64; 3]> {
        self.0.iter().filter_map(|s| s.position).collect()
    }
    pub fn take_origin(&mut self, outgoing: bool) -> Vec<Snapshot<P>> {
        self.take_if(|s| s.outgoing == outgoing)
    }
    pub fn take_if(&mut self, mut predicate: impl FnMut(&Snapshot<P>) -> bool) -> Vec<Snapshot<P>> {
        let mut keep = VecDeque::new();
        let mut ready = Vec::new();
        for s in self.0.drain(..) {
            if predicate(&s) {
                ready.push(s);
            } else {
                keep.push_back(s);
            }
        }
        self.0 = keep;
        ready
    }
    pub fn flush_count(&mut self, count: i32) -> Vec<Snapshot<P>> {
        let mut n = 0i32;
        let mut ready = Vec::new();
        while let Some(s) = self.0.pop_front() {
            if s.position.is_some() {
                n = n.wrapping_add(1);
            }
            ready.push(s);
            if n >= count {
                break;
            }
        }
        ready
    }
    pub fn cancel(&mut self) -> (Option<[f64; 3]>, Vec<Snapshot<P>>) {
        let at = self.0.iter().find_map(|s| s.position);
        let ready = self.take_if(|s| !s.movement);
        self.0.clear();
        (at, ready)
    }
    pub fn is_above_time(&self, now: i64, delay: i64) -> bool {
        self.0
            .front()
            .is_some_and(|s| now.wrapping_sub(s.millis) >= delay)
    }
}
