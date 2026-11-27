// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSnapTap, LiquidBounce ae0eb396. No disable/world-change reset exists.
use crate::rotations::DirectionalInput;

pub const LEFT: u8 = 1;
pub const RIGHT: u8 = 2;
pub const BACK: u8 = 4;
pub const FORWARD: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Press,
    Release,
    Repeat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct AxisState {
    holding_negative: bool,
    holding_positive: bool,
    last_negative_press: i64,
    last_positive_press: i64,
}
impl AxisState {
    fn press(&mut self, positive: bool, now: i64) {
        if positive {
            self.holding_positive = true;
            self.last_positive_press = now;
        } else {
            self.holding_negative = true;
            self.last_negative_press = now;
        }
    }
    fn release(&mut self, positive: bool) {
        if positive {
            self.holding_positive = false;
        } else {
            self.holding_negative = false;
        }
    }
    fn preference(self) -> Option<bool> {
        (self.holding_negative && self.holding_positive)
            .then_some(self.last_positive_press >= self.last_negative_press)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnapTap {
    horizontal: AxisState,
    vertical: AxisState,
}
impl SnapTap {
    /// KeyboardKeyEvent fires on every screen. Read the current movement
    /// bindings for each event. Kotlin's when chooses only its first match:
    /// left, right, back, forward. Repeat never updates holding or timestamp.
    pub fn keyboard(
        &mut self,
        running: bool,
        matches: u8,
        action: Action,
        now: impl FnOnce() -> i64,
    ) {
        if !running || action == Action::Repeat {
            return;
        }
        let (axis, positive) = if matches & LEFT != 0 {
            (&mut self.horizontal, false)
        } else if matches & RIGHT != 0 {
            (&mut self.horizontal, true)
        } else if matches & BACK != 0 {
            (&mut self.vertical, false)
        } else if matches & FORWARD != 0 {
            (&mut self.vertical, true)
        } else {
            return;
        };
        match action {
            Action::Press => axis.press(positive, now()),
            Action::Release => axis.release(positive),
            Action::Repeat => unreachable!("repeat returned before matching"),
        }
    }
    /// Axis state, not the current input flags, controls the two-held guard.
    /// This intentionally retains stale state after disabled key releases.
    pub fn movement_input(&self, running: bool, input: &mut DirectionalInput) {
        if !running {
            return;
        }
        if let Some(positive) = self.horizontal.preference() {
            input.left = !positive;
            input.right = positive;
        }
        if let Some(positive) = self.vertical.preference() {
            input.backwards = !positive;
            input.forwards = positive;
        }
    }
}
