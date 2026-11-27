// SPDX-License-Identifier: GPL-3.0-or-later
//! CombatManager local cooldown and ServerObserver TPS (15 arrival intervals).
use std::collections::VecDeque;
#[derive(Debug)]
pub struct Cooldown {
    pub reset_ticks: f64,
    pub tps: f64,
    intervals: VecDeque<f64>,
    last_millis: i64,
    disconnected: bool,
    pending_tps: Option<f64>,
    pending_respawn: bool,
}
impl Default for Cooldown {
    fn default() -> Self {
        Self {
            reset_ticks: 0.0,
            tps: f64::NAN,
            intervals: VecDeque::new(),
            last_millis: 0,
            disconnected: true,
            pending_tps: None,
            pending_respawn: false,
        }
    }
}
impl Cooldown {
    pub fn outgoing(&mut self, real_ticks: f64) {
        self.reset_ticks = real_ticks;
    }
    pub fn respawn(&mut self) {
        self.pending_respawn = true;
    }
    pub fn set_time(&mut self, millis: i64) {
        if self.disconnected && self.intervals.is_empty() {
            self.disconnected = false;
            self.last_millis = millis;
            return;
        }
        self.intervals
            .push_back(millis.wrapping_sub(self.last_millis) as f64);
        self.last_millis = millis;
        while self.intervals.len() > 15 {
            self.intervals.pop_front();
        }
        let average = self.intervals.iter().sum::<f64>() / self.intervals.len() as f64;
        self.pending_tps = Some(if average > 0.0 && !average.is_nan() {
            (20.0 / (average / 1000.0)).clamp(0.0, 20.0)
        } else {
            f64::NAN
        });
    }
    /// mc.execute work is applied on the next actor operation before its queries.
    pub fn drain(&mut self, real_ticks: f64) {
        if self.pending_respawn {
            self.reset_ticks = real_ticks;
            self.pending_respawn = false;
        }
        if let Some(tps) = self.pending_tps.take() {
            self.tps = tps;
        }
    }
    pub fn world_change(&mut self, real_ticks: f64) {
        self.reset_ticks = real_ticks;
    }
    pub fn disconnect(&mut self, real_ticks: f64) {
        self.world_change(real_ticks);
        self.disconnected = true;
        self.intervals.clear();
        self.tps = f64::NAN;
        self.pending_tps = None;
    }
    pub fn progress(&self, real_ticks: f64, rate: f32, delay: f32) -> f32 {
        let speed = if self.tps.is_nan() {
            1.0
        } else {
            self.tps / f64::from(rate)
        };
        // Kotlin Double.roundToInt rounds ties toward positive infinity.
        let ticks = ((real_ticks - self.reset_ticks) * speed + 0.5).floor() as i32;
        (ticks as f32 + 0.5) / delay
    }
}
