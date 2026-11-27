// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAntiVoid and all four modes, pinned LiquidBounce ae0eb396.
use std::collections::BTreeSet;

pub const MODES: [&str; 4] = ["GhostBlock", "Flag", "Blink", "UseItem"];
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub mode: usize,
    pub void_level: i32,
    pub fall_distance: f32,
    pub height: f32,
    pub silent: bool,
    pub y_motion_threshold: f32,
    pub slot_reset: [i32; 2],
    pub items: BTreeSet<String>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: 0,
            void_level: 0,
            fall_distance: 0.5,
            height: 0.42,
            silent: false,
            y_motion_threshold: -0.5,
            slot_reset: [4, 6],
            items: BTreeSet::from(["minecraft:magma_cream".into()]),
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Exempt {
    pub dead: bool,
    pub fly: bool,
    pub clutch_guarding: bool,
    pub scaffold: bool,
}
impl Exempt {
    pub fn mode(self, mode: usize) -> bool {
        self.dead || self.fly || self.clutch_guarding || mode == 2 && self.scaffold
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snapshot {
    pub position: [f64; 3],
    pub fall_distance: f64,
    pub on_ground: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rescue {
    Raise(f64),
    Silent,
    CancelBlink,
    Use { slot: i32, delay: i32 },
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub settings: Settings,
    pub likely_falling: bool,
    pub rescue_position: Option<[f64; 3]>,
    /// EventListener.until registers ordinary listeners, independent of coroutines.
    pub pending_silent: usize,
}
impl State {
    /// Source onEnabled clears only this flag; safe position survives every lifecycle event.
    pub fn enabled(&mut self) {
        self.likely_falling = false;
    }
    pub fn observe(&mut self, likely: bool, position: [f64; 3]) {
        self.likely_falling = likely;
        if !likely {
            self.rescue_position = Some(position);
        }
    }
    pub fn requires_lag(&self, running: bool, exempt: Exempt) -> bool {
        running
            && self.settings.mode == 2
            && self.likely_falling
            && self.rescue_position.is_some()
            && !exempt.mode(2)
    }
    pub fn ghost_floor(&self, running: bool, exempt: Exempt) -> Option<f64> {
        if running && self.settings.mode == 0 && self.likely_falling && !exempt.mode(0) {
            self.rescue_position.map(|p| p[1].floor())
        } else {
            None
        }
    }
    pub fn eligible(&self, running: bool, exempt: Exempt) -> bool {
        running
            && self.likely_falling
            && self.rescue_position.is_some()
            && !exempt.mode(self.settings.mode)
    }
    pub fn rescue(
        &mut self,
        running: bool,
        exempt: Exempt,
        empty_column: bool,
        fall_distance: f64,
        motion_y: f64,
        reaches_ground: bool,
        blink_positions: bool,
        slots: &[(i32, String)],
        selected: i32,
        sample: impl FnOnce([i32; 2]) -> i32,
    ) -> Option<Rescue> {
        if !self.eligible(running, exempt) || !empty_column {
            return None;
        }
        let result = match self.settings.mode {
            0 => return None,
            1 if fall_distance >= f64::from(self.settings.fall_distance) => {
                if self.settings.silent {
                    self.pending_silent += 1;
                    Rescue::Silent
                } else {
                    Rescue::Raise(f64::from(self.settings.height))
                }
            }
            2 if self.requires_lag(running, exempt) && blink_positions => Rescue::CancelBlink,
            3 if !(motion_y >= f64::from(self.settings.y_motion_threshold)) && !reaches_ground => {
                let slot = closest_slot(slots, selected, &self.settings.items)?;
                Rescue::Use {
                    slot,
                    delay: sample(self.settings.slot_reset),
                }
            }
            _ => return None,
        };
        self.rescue_position = None;
        Some(result)
    }
    /// All pending listeners read the current Height, add in registration order,
    /// survive POST, and unregister at PRE or when the Flag mode is not running.
    pub fn network(&mut self, flag_running: bool, pre: bool, y: &mut f64) {
        if !flag_running {
            self.pending_silent = 0;
            return;
        }
        for _ in 0..self.pending_silent {
            *y += f64::from(self.settings.height);
        }
        if pre {
            self.pending_silent = 0;
        }
    }
}
/// The first falling snapshot alone decides prediction; ticks are exactly 0..10.
pub fn predicts_fall(
    snapshots: &[Snapshot],
    mut empty_column: impl FnMut([f64; 3]) -> bool,
) -> bool {
    snapshots
        .iter()
        .take(10)
        .find(|p| p.fall_distance > 0.)
        .is_some_and(|p| empty_column(p.position))
}
/// HotbarItemSlot.entries order is offhand then 0..8; ties retain that order.
pub fn closest_slot(
    slots: &[(i32, String)],
    selected: i32,
    items: &BTreeSet<String>,
) -> Option<i32> {
    let score = |slot: i32| {
        if slot == selected {
            i32::MIN
        } else if slot == 40 {
            i32::MIN + 1
        } else {
            (selected - slot).abs()
        }
    };
    let mut found = None;
    for &(slot, ref item) in slots {
        if items.contains(item) && found.is_none_or(|prior| score(slot) < score(prior)) {
            found = Some(slot);
        }
    }
    found
}
pub fn ghost_shape(empty: bool, block_y: i32, rescue_floor: Option<f64>) -> bool {
    empty && rescue_floor.is_some_and(|floor| f64::from(block_y) < floor)
}
