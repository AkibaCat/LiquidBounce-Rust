// SPDX-License-Identifier: GPL-3.0-or-later
//! LiquidBounce TimedPickupTracker; signed epoch milliseconds and strict windows.
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Tracked {
    position: [i32; 3],
    timestamp: i64,
}

#[derive(Clone, Debug)]
pub struct TimedPickupTracker {
    capacity: usize,
    entries: VecDeque<Tracked>,
}
impl TimedPickupTracker {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be positive.");
        Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn record(&mut self, position: [i32; 3], now: i64) {
        self.entries.push_back(Tracked {
            position,
            timestamp: now,
        });
        while self.entries.len() > self.capacity {
            self.entries.pop_front();
        }
    }
    pub fn prune(&mut self, max_ms: i64, now: i64, mut pickupable: impl FnMut([i32; 3]) -> bool) {
        self.entries
            .retain(|e| now.wrapping_sub(e.timestamp) <= max_ms && pickupable(e.position));
    }
    pub fn first_eligible(
        &self,
        min_ms: i64,
        now: i64,
        mut predicate: impl FnMut([i32; 3]) -> bool,
    ) -> Option<[i32; 3]> {
        self.entries
            .iter()
            .find(|e| now.wrapping_sub(e.timestamp) > min_ms && predicate(e.position))
            .map(|e| e.position)
    }
}
