// SPDX-License-Identifier: GPL-3.0-or-later
//! AutoRod's cached-slot requirements and cancellable tickConditional/waitTicks run.
use crate::{auto_bow::Priority, point_tracker::Samples};
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub projectile: bool,
    pub range: [f32; 2],
    pub scan_extra: [f32; 2],
    pub max_enemies: i32,
    pub min_health: f32,
    pub min_target_health: f32,
    pub requires: [bool; 5],
    pub ignore: [bool; 3],
    pub holding_ignore: Vec<String>,
    pub priority: Vec<Priority>,
    pub fov: f32,
    pub hurt_time: i32,
    pub aim_threshold: f32,
    pub hide_swing: bool,
    pub hit_timeout: i32,
    pub pull_out_of_range: bool,
    pub slot_reset: [i32; 2],
    pub cooldown: [i32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            projectile: false,
            range: [3.5, 5.],
            scan_extra: [0., 0.],
            max_enemies: 1,
            min_health: 10.,
            min_target_health: 4.,
            requires: [false; 5],
            ignore: [false; 3],
            holding_ignore: ["bow", "crossbow", "ender_pearl", "fire_charge", "trident"]
                .into_iter()
                .map(|s| format!("minecraft:{s}"))
                .collect(),
            priority: vec![Priority::Type, Priority::Distance],
            fov: 180.,
            hurt_time: 10,
            aim_threshold: 5.,
            hide_swing: false,
            hit_timeout: 30,
            pull_out_of_range: true,
            slot_reset: [0, 0],
            cooldown: [4, 8],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hook {
    pub id: i32,
    pub hooked: bool,
    pub stationary: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub selected: u8,
    pub rods: [bool; 10], // hotbar 0..8; offhand index 9
    pub hook: Option<Hook>,
    pub health: f32,
    pub requires: [bool; 5],
    pub ignore: [bool; 3],
    pub main_item: String,
    pub forbidden: bool,
    pub enemies: i32,
}
impl Context {
    pub fn has_rod(&self, slot: u8) -> bool {
        self.rods[if slot == 40 {
            9
        } else {
            usize::from(slot.min(8))
        }]
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Hit { elapsed: i32 },
    Cooldown { remaining: i32 },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Run {
    pub slot: u8,
    pub target: i32,
    pub min_squared: f64,
    pub max_squared: f64,
    pub hit_timeout: i32,
    pub phase: Phase,
}
#[derive(Clone, Debug, Default)]
pub struct AutoRod {
    pub settings: Settings,
    pub target: Option<i32>,
    pub cached_hook: Option<Hook>,
    pub available_slot: Option<u8>,
    pub current_scan_extra: Option<f32>,
    pub run: Option<Run>,
}
impl AutoRod {
    pub fn initialize(&mut self, random: &mut impl Samples) {
        if self.current_scan_extra.is_none() {
            self.scan_changed(random);
        }
    }
    pub fn scan_changed(&mut self, random: &mut impl Samples) {
        self.current_scan_extra = Some(random.float(self.settings.scan_extra));
    }
    pub fn first_tick(&mut self, c: &Context) {
        self.cached_hook = c.hook;
        self.available_slot = self
            .available_slot
            .filter(|&slot| (slot == 40 || slot == c.selected) && c.has_rod(slot))
            .or_else(|| {
                std::iter::once(40)
                    .chain(0..9)
                    .find(|&slot| c.has_rod(slot))
            });
    }
    pub fn requirements_met(&self, c: &Context) -> bool {
        let s = &self.settings;
        s.requires
            .iter()
            .zip(c.requires)
            .all(|(&enabled, met)| !enabled || met)
            && !s
                .ignore
                .iter()
                .zip(c.ignore)
                .any(|(&enabled, met)| enabled && met)
            && c.health > s.min_health
            && self.available_slot.is_some()
            && !s.holding_ignore.contains(&c.main_item)
            && !c.forbidden
            && (s.max_enemies == 0 || c.enemies <= s.max_enemies)
    }
    pub fn ranges(&self) -> (f64, f64) {
        let max = self.settings.range[1]
            + self
                .current_scan_extra
                .unwrap_or(self.settings.scan_extra[0]);
        (
            f64::from(self.settings.range[0] * self.settings.range[0]),
            f64::from(max * max),
        )
    }
    pub fn begin(&mut self, slot: u8, target: i32) {
        let (min_squared, max_squared) = self.ranges();
        self.run = Some(Run {
            slot,
            target,
            min_squared,
            max_squared,
            hit_timeout: self.settings.hit_timeout,
            phase: Phase::Hit { elapsed: 0 },
        });
    }
    /// Return true exactly when the suspended hit waiter resumes to reel.
    pub fn advance_hit(&mut self, slot_is_rod: bool, distance_squared: f64) -> bool {
        let Some(run) = self.run.as_mut() else {
            return false;
        };
        if !slot_is_rod {
            self.run = None;
            return false;
        }
        let Phase::Hit { elapsed } = &mut run.phase else {
            return false;
        };
        *elapsed = elapsed.wrapping_add(1);
        self.cached_hook.is_some_and(|h| h.hooked || h.stationary)
            || (self.settings.pull_out_of_range
                && !(run.min_squared..=run.max_squared).contains(&distance_squared))
            || *elapsed >= run.hit_timeout
    }
    pub fn cooldown(&mut self, ticks: i32) {
        if ticks == 0 {
            self.run = None;
        } else if let Some(run) = self.run.as_mut() {
            run.phase = Phase::Cooldown { remaining: ticks };
        }
    }
    pub fn advance_cooldown(&mut self, slot_is_rod: bool) -> bool {
        if !slot_is_rod {
            self.run = None;
            return false;
        }
        let Some(Run {
            phase: Phase::Cooldown { remaining },
            ..
        }) = self.run.as_mut()
        else {
            return false;
        };
        *remaining = remaining.wrapping_sub(1);
        if *remaining <= 0 {
            self.run = None;
            false
        } else {
            true
        }
    }
    /// Random range and point-processor state intentionally survive disable.
    pub fn clear(&mut self) -> bool {
        let release = self.cached_hook.is_some();
        self.target = None;
        self.cached_hook = None;
        self.available_slot = None;
        self.run = None;
        release
    }
}
#[derive(Clone, Debug, Default)]
pub struct InputTracker {
    pub mouse: [bool; 32],
    pub pressed_at: [i64; 32],
    pub moved_at: i64,
}
impl InputTracker {
    pub fn mouse_event(&mut self, button: i32, pressed: bool, now: i64) {
        if let Some(i) = usize::try_from(button).ok().filter(|&i| i < 32) {
            self.mouse[i] = pressed;
            if pressed {
                self.pressed_at[i] = now;
            }
        }
    }
    pub fn moved(&mut self, x: f64, y: f64, now: i64) {
        if x != 0. || y != 0. {
            self.moved_at = now;
        }
    }
    pub fn mouse_click(&self, button: i32, now: i64) -> bool {
        usize::try_from(button)
            .ok()
            .filter(|&i| i < 32)
            .is_some_and(|i| {
                self.mouse[i]
                    || self.pressed_at[i] > 0 && now.wrapping_sub(self.pressed_at[i]) <= 250
            })
    }
    pub fn mouse_moved(&self, now: i64) -> bool {
        self.moved_at > 0 && now.wrapping_sub(self.moved_at) <= 100
    }
}
