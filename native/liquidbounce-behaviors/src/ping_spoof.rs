// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned ModulePingSpoof: wall-clock delay and independent incoming snapshots.
use std::collections::VecDeque;
#[derive(Debug)]
pub struct Snapshot<P> {
    pub session: u64,
    pub received: i64,
    pub packet: P,
}
#[derive(Debug)]
pub struct DelayQueue<P> {
    queue: VecDeque<Snapshot<P>>,
}
impl<P> Default for DelayQueue<P> {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}
impl<P> DelayQueue<P> {
    pub fn push(&mut self, session: u64, received: i64, packet: P) {
        self.queue.push_back(Snapshot {
            session,
            received,
            packet,
        });
    }
    pub fn drain(&mut self, session: u64, now: i64, delay: i32, running: bool) -> Vec<P> {
        let mut ready = Vec::new();
        self.queue.retain(|s| s.session == session && session != 0);
        let mut pending = VecDeque::new();
        while let Some(s) = self.queue.pop_front() {
            if !running || now.saturating_sub(s.received) >= i64::from(delay) {
                ready.push(s.packet);
            } else {
                pending.push_back(s);
            }
        }
        self.queue = pending;
        ready
    }
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delay_disabled_flush_and_stale_session_smoke() {
        let mut q = DelayQueue::default();
        q.push(2, 100, 11);
        q.push(2, 200, 22);
        q.push(1, 0, 99);
        assert!(q.drain(2, 599, 500, true).is_empty());
        assert_eq!(q.drain(2, 600, 500, true), [11]);
        assert_eq!(q.drain(2, 600, 500, false), [22]);
        q.push(2, 700, 33);
        assert!(q.drain(3, 5000, 0, true).is_empty());
        q.push(3, 0, 44);
        q.clear();
        assert!(q.drain(3, 10, 0, false).is_empty());
    }
}
