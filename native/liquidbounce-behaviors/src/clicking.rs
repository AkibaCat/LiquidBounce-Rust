// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396, Copyright (c) 2015-2026 CCBlueX.
//! Millisecond click planning, cooldown gates and same-mapping coordination.
//! Call `tick` at FIRST_PRIORITY (1000), with a monotonic millisecond clock.

use std::{
    collections::hash_map::RandomState,
    hash::BuildHasher,
    sync::atomic::{AtomicU64, Ordering},
};

pub const TICK_MS: i64 = 50;
pub const TICKS_AHEAD: i32 = 20;
const HORIZON_MS: i64 = 1000;
const IDLE_MS: i64 = 250;
const HISTORY: usize = 32;

/// Runtime supplies HotSpot Math.log/exp and StrictMath.log separately.
#[derive(Clone, Copy, Debug)]
pub struct Math {
    pub log: fn(f64) -> f64,
    pub exp: fn(f64) -> f64,
    pub strict_log: fn(f64) -> f64,
}
impl Default for Math {
    fn default() -> Self {
        Self {
            log: f64::ln,
            exp: f64::exp,
            strict_log: f64::ln,
        }
    }
}

/// java.util.Random: 48-bit LCG and the cached polar Gaussian pair.
#[derive(Clone, Debug)]
pub struct JavaRandom {
    seed: u64,
    gaussian: Option<f64>,
}
impl JavaRandom {
    pub fn seeded(seed: u64) -> Self {
        Self {
            seed: (seed ^ 0x5deece66d) & ((1 << 48) - 1),
            gaussian: None,
        }
    }
    fn next(&mut self, bits: u32) -> u64 {
        self.seed = self.seed.wrapping_mul(0x5deece66d).wrapping_add(11) & ((1 << 48) - 1);
        self.seed >> (48 - bits)
    }
    pub fn double(&mut self) -> f64 {
        ((self.next(26) << 27) + self.next(27)) as f64 / 9_007_199_254_740_992.0
    }
    pub fn gaussian(&mut self, strict_log: fn(f64) -> f64) -> f64 {
        if let Some(value) = self.gaussian.take() {
            return value;
        }
        loop {
            let v1 = 2.0 * self.double() - 1.0;
            let v2 = 2.0 * self.double() - 1.0;
            let s = v1 * v1 + v2 * v2;
            if s >= 1.0 || s == 0.0 {
                continue;
            }
            let multiplier = (-2.0 * strict_log(s) / s).sqrt();
            self.gaussian = Some(v2 * multiplier);
            return v1 * multiplier;
        }
    }
}
impl Default for JavaRandom {
    fn default() -> Self {
        static IDS: AtomicU64 = AtomicU64::new(1);
        Self::seeded(RandomState::new().hash_one(IDS.fetch_add(1, Ordering::Relaxed)))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Technique {
    #[default]
    Human,
    Constant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub technique: Technique,
    pub cps: [i32; 2],
    pub max_per_tick: i32,
    /// None for use clickers; Some([1, 1]) for attack clickers.
    pub item_cooldown: Option<[f32; 2]>,
    pub miss_cooldown: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            technique: Technique::Human,
            cps: [11, 14],
            max_per_tick: 2,
            item_cooldown: Some([1.0, 1.0]),
            miss_cooldown: true,
        }
    }
}
impl Settings {
    pub fn use_key() -> Self {
        Self {
            item_cooldown: None,
            miss_cooldown: false,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HumanTiming {
    combo_cps: f64,
    range: Option<[i32; 2]>,
}
impl HumanTiming {
    fn interval(
        &mut self,
        combo_ms: i64,
        cps: [i32; 2],
        random: &mut JavaRandom,
        math: Math,
    ) -> i64 {
        if combo_ms == 0 || self.range != Some(cps) {
            self.combo_cps = f64::from(cps[0]) + random.double() * f64::from(cps[1] - cps[0]);
            self.range = Some(cps);
        }
        let mu = (math.log)(1000.0 / self.combo_cps) - 0.45 * 0.45 / 2.0;
        // Kotlin roundToLong uses ties toward positive infinity.
        ((math.exp)(mu + 0.45 * random.gaussian(math.strict_log)) + 0.5)
            .floor()
            .clamp(10.0, 1000.0) as i64
    }
}

#[derive(Clone, Debug)]
pub struct ClickPlan {
    cps: [i32; 2],
    max_per_tick: i32,
    pub tick_time: i64,
    times: Vec<i64>,
    intervals: Vec<i64>,
    next: usize,
    combo_start: i64,
    due: i32,
    consumed: i32,
    idle_since: Option<i64>,
    human: HumanTiming,
    random: JavaRandom,
    math: Math,
}
impl Default for ClickPlan {
    fn default() -> Self {
        Self::new(JavaRandom::default(), Math::default())
    }
}
impl ClickPlan {
    pub fn new(random: JavaRandom, math: Math) -> Self {
        Self {
            cps: [11, 14],
            max_per_tick: 2,
            tick_time: 0,
            times: Vec::new(),
            intervals: Vec::new(),
            next: 0,
            combo_start: 0,
            due: 0,
            consumed: 0,
            idle_since: None,
            human: HumanTiming::default(),
            random,
            math,
        }
    }
    pub fn combo_ms(&self) -> i64 {
        self.tick_time.wrapping_sub(self.combo_start)
    }
    pub fn set_math(&mut self, math: Math) {
        self.math = math;
    }
    pub fn reset(&mut self, now: i64) {
        self.times.clear();
        self.intervals.clear();
        self.next = 0;
        self.due = 0;
        self.consumed = 0;
        self.idle_since = None;
        self.combo_start = now;
        self.tick_time = now.wrapping_sub(TICK_MS);
        self.times.push(now);
    }
    pub fn tick(&mut self, now: i64, settings: Settings) {
        // Technique changes keep the existing horizon, as in the source.
        if self.cps != settings.cps {
            self.cps = settings.cps;
            self.replan();
        }
        if self.max_per_tick != settings.max_per_tick {
            self.max_per_tick = settings.max_per_tick;
            self.replan();
        }
        if self.consumed > 0 {
            self.idle_since = None;
        } else if self.due > 0 && self.idle_since.is_none() {
            self.idle_since = Some(self.tick_time);
        }
        if self.times.is_empty()
            || now.wrapping_sub(self.tick_time) > IDLE_MS
            || self
                .idle_since
                .is_some_and(|idle| now.wrapping_sub(idle) > IDLE_MS)
        {
            self.reset(now);
        }
        let expected = self.tick_time.wrapping_add(TICK_MS);
        self.tick_time = if now.wrapping_sub(expected).wrapping_abs() > TICK_MS {
            now
        } else {
            expected
        };
        self.plan_until(self.tick_time.wrapping_add(HORIZON_MS), settings.technique);
        let mut count = 0;
        while self.next < self.times.len() && self.times[self.next] <= self.tick_time {
            self.next += 1;
            count += 1;
        }
        self.due = count.min(self.max_per_tick);
        self.consumed = 0;
        let excess = self.next.saturating_sub(HISTORY);
        if excess > 0 {
            self.times.drain(..excess);
            self.next -= excess;
        }
    }
    pub fn clicks_at(&mut self, tick: i32, technique: Technique, enforced: bool) -> i32 {
        let planned = if tick <= 0 {
            self.due
        } else {
            let from = self.tick_time.wrapping_add(i64::from(tick - 1) * TICK_MS);
            let to = from.wrapping_add(TICK_MS);
            self.plan_until(to, technique);
            self.times[self.next..]
                .iter()
                .take_while(|&&time| time <= to)
                .filter(|&&time| time > from)
                .count() as i32
        }
        .min(self.max_per_tick);
        if planned == 0 && enforced { 1 } else { planned }
    }
    pub fn take(&mut self, technique: Technique, enforced: bool) -> bool {
        if self.consumed >= self.clicks_at(0, technique, enforced) {
            return false;
        }
        self.consumed += 1;
        true
    }
    pub fn pressed_by_player(&mut self) {
        if self.next >= self.times.len() {
            return;
        }
        let remaining = self.times.len() - self.next;
        if let Some(index) = self.intervals.len().checked_sub(remaining) {
            self.intervals.remove(index);
        }
        self.times.remove(self.next);
    }
    pub fn pressed_by(&mut self, leader: &Self, time: i64) {
        if self.times.is_empty() {
            return;
        }
        if self.consumed < self.due {
            self.consumed += 1;
        }
        self.replan();
        if time >= *self.times.last().expect("nonempty plan") {
            self.append(time);
        }
        self.next = self.times.len();
        for &planned in &leader.times[leader.next..] {
            if planned > time {
                self.append(planned);
            }
        }
    }
    fn replan(&mut self) {
        let keep = self.next.max(1);
        if self.times.len() <= keep {
            return;
        }
        let dropped = self.times.len() - keep;
        self.times.truncate(keep);
        self.intervals
            .truncate(self.intervals.len().saturating_sub(dropped));
    }
    fn plan_until(&mut self, time: i64, technique: Technique) {
        // Before its first tick, ClickPlan is not queried by an active listener.
        if self.times.is_empty() {
            return;
        }
        while *self.times.last().expect("nonempty plan") <= time {
            let last = *self.times.last().expect("nonempty plan");
            let interval = match technique {
                Technique::Human => self.human.interval(
                    last.wrapping_sub(self.combo_start),
                    self.cps,
                    &mut self.random,
                    self.math,
                ),
                Technique::Constant => (1000.0 / f64::from(self.cps[1]) + 0.5).floor() as i64,
            };
            let mut next = last.wrapping_add(interval);
            if self.times.len() >= self.max_per_tick as usize {
                next = next.max(
                    self.times[self.times.len() - self.max_per_tick as usize].wrapping_add(TICK_MS),
                );
            }
            self.append(next);
        }
    }
    fn append(&mut self, time: i64) {
        self.intervals
            .push(time.wrapping_sub(*self.times.last().expect("nonempty plan")));
        if self.intervals.len() > HISTORY {
            self.intervals.remove(0);
        }
        self.times.push(time);
    }
}

/// Captured server-counted cooldown, already drained from the real packet observer.
#[derive(Clone, Copy, Debug)]
pub struct Cooldown {
    pub real_ticks: f64,
    pub reset_ticks: f64,
    pub server_tick_speed: f64,
    pub attack_delay: f32,
    pub has_cooldown: bool,
}
impl Cooldown {
    pub fn ticks(self, ahead: i32) -> i32 {
        ((self.real_ticks - self.reset_ticks + f64::from(ahead)) * self.server_tick_speed + 0.5)
            .floor() as i32
    }
    pub fn progress(self, ahead: i32) -> f32 {
        (self.ticks(ahead) as f32 + 0.5) / self.attack_delay
    }
}

#[derive(Clone, Debug)]
pub struct Clicker {
    pub settings: Settings,
    pub plan: ClickPlan,
    pub click_amount: Option<i32>,
    pub ticks_since_last_click: i32,
    pub next_cooldown: Option<f32>,
    pub simulate_attack_key_down: bool,
}
impl Default for Clicker {
    fn default() -> Self {
        Self::new(
            Settings::default(),
            true,
            JavaRandom::default(),
            Math::default(),
        )
    }
}
impl Clicker {
    pub fn new(
        settings: Settings,
        simulate_attack_key_down: bool,
        random: JavaRandom,
        math: Math,
    ) -> Self {
        Self {
            settings,
            plan: ClickPlan::new(random, math),
            click_amount: None,
            ticks_since_last_click: 0,
            next_cooldown: settings.item_cooldown.map(|range| range[0]),
            simulate_attack_key_down,
        }
    }
    pub fn tick(&mut self, now: i64) {
        self.ticks_since_last_click = self.ticks_since_last_click.wrapping_add(1);
        self.click_amount = None;
        self.plan.tick(now, self.settings);
    }
    fn cooldown_passed(&self, cooldown: Cooldown, ahead: i32) -> bool {
        self.next_cooldown
            .is_none_or(|minimum| cooldown.progress(ahead) >= minimum)
    }
    fn enforced(&self, cooldown: Cooldown, ahead: i32) -> bool {
        cooldown.has_cooldown
            && self.next_cooldown.is_some()
            && self.cooldown_passed(cooldown, ahead)
            && !self.cooldown_passed(cooldown, ahead - 1)
    }
    pub fn amount_at(&mut self, tick: i32, cooldown: Cooldown) -> i32 {
        self.plan.clicks_at(
            tick,
            self.settings.technique,
            self.enforced(cooldown, tick.max(0)),
        )
    }
    pub fn ticks_until_click(&mut self, cooldown: Cooldown) -> i32 {
        (0..TICKS_AHEAD)
            .find(|&tick| self.amount_at(tick, cooldown) > 0)
            .unwrap_or(TICKS_AHEAD)
    }
    pub fn can_execute(&mut self, cooldown: Cooldown, miss_time: i32) -> bool {
        self.amount_at(0, cooldown) > 0
            && (!self.settings.miss_cooldown || miss_time <= 0)
            && self.cooldown_passed(cooldown, 0)
    }
    /// Split phases let a runtime call native hooks without holding module state.
    pub fn take_attempt(&mut self, cooldown: Cooldown) -> bool {
        self.plan
            .take(self.settings.technique, self.enforced(cooldown, 0))
    }
    pub fn passes_gate(&self, cooldown: Cooldown, miss_time: i32) -> bool {
        (!self.settings.miss_cooldown || miss_time <= 0) && self.cooldown_passed(cooldown, 0)
    }
    pub fn record_success(&mut self, sampled_cooldown: Option<f32>) {
        self.next_cooldown = sampled_cooldown;
        self.ticks_since_last_click = 0;
    }
    pub fn finish_click(&mut self, clicks: i32) {
        self.click_amount = Some(self.click_amount.unwrap_or(0).wrapping_add(clicks));
    }
    /// Consume an attempt even when its gate rejects it. Query live cooldown
    /// before each attempt; run `success` immediately after a successful click,
    /// to notify other running clickers before another attempt can occur.
    pub fn click(
        &mut self,
        mut live: impl FnMut() -> (Cooldown, i32),
        mut block: impl FnMut() -> bool,
        mut sample_cooldown: impl FnMut([f32; 2]) -> f32,
        mut success: impl FnMut(&ClickPlan),
    ) -> i32 {
        let mut clicks: i32 = 0;
        loop {
            let (cooldown, miss_time) = live();
            if !self
                .plan
                .take(self.settings.technique, self.enforced(cooldown, 0))
            {
                break;
            }
            if (!self.settings.miss_cooldown || miss_time <= 0)
                && self.cooldown_passed(cooldown, 0)
                && block()
            {
                self.next_cooldown = self.settings.item_cooldown.map(&mut sample_cooldown);
                self.ticks_since_last_click = 0;
                success(&self.plan);
                clicks = clicks.wrapping_add(1);
            }
        }
        self.click_amount = Some(self.click_amount.unwrap_or(0).wrapping_add(clicks));
        clicks
    }
    pub fn attack_key_down(&self, original: bool) -> bool {
        if self.simulate_attack_key_down {
            self.click_amount.map_or(original, |count| count > 0)
        } else {
            original
        }
    }
}
