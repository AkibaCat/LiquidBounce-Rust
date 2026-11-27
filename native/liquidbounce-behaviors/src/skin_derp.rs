// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleSkinDerp at LiquidBounce ae0eb396. Real stored model-part
//! options change; normal Minecraft client-information broadcasting is separate.

pub const PARTS: [&str; 7] = [
    "Hat",
    "Jacket",
    "LeftPants",
    "RightPants",
    "LeftSleeve",
    "RightSleeve",
    "Cape",
];
/// DerpParts EnumSet iteration order differs from PlayerModelPart bit order.
pub const PART_MASKS: [u8; 7] = [64, 2, 16, 32, 4, 8, 1];

pub fn part_index(tag: &str) -> Option<usize> {
    // Tagged.makeLookupTable uses Java's case-insensitive equality.
    fn fold(c: char) -> char {
        let mut upper = c.to_uppercase();
        let first = upper.next().unwrap_or(c);
        let upper = if upper.next().is_none() { first } else { c };
        upper.to_lowercase().next().unwrap_or(upper)
    }
    PARTS
        .iter()
        .position(|part| part.chars().map(fold).eq(tag.chars().map(fold)))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub sync: bool,
    pub delay: i32,
    /// Selection bits in DerpParts ordinal order, not protocol mask order.
    pub parts: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            sync: false,
            delay: 0,
            parts: 127,
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Wait {
    remaining: i32,
    callback: u64,
}
#[derive(Clone, Debug, Default)]
pub struct SkinDerp {
    pub settings: Settings,
    previous_parts: u8,
    wait: Option<Wait>,
    callback_counter: u64,
    wait_registrations: Vec<u64>,
}
impl SkinDerp {
    pub const fn previous_parts(&self) -> u8 {
        self.previous_parts
    }
    pub const fn busy(&self) -> bool {
        self.wait.is_some()
    }
    /// Actual in-game onEnabled lifecycle only. Enabled edits outside a world
    /// skip the source callback, leaving the earlier capture (initially empty).
    pub fn on_enabled(&mut self, actual_parts: u8) {
        self.previous_parts = actual_parts & 127;
    }
    /// Source clears every current part and then re-enables the captured set.
    /// setModelPart only mutates an option set, with no events in that loop.
    pub fn on_disabled(&mut self) -> u8 {
        self.cancel_sequences();
        self.previous_parts
    }
    pub fn cancel_sequences(&mut self) {
        self.wait = None;
    }
    pub fn take_wait_registrations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.wait_registrations)
    }
    fn apply_parts(&self, actual_parts: u8, random: &mut impl FnMut() -> bool) -> Option<u8> {
        if self.settings.parts & 127 == 0 {
            return None;
        }
        let mut output = actual_parts & 127;
        for (index, mask) in PART_MASKS.into_iter().enumerate() {
            if self.settings.parts & (1 << index) == 0 {
                continue;
            }
            let enabled = if self.settings.sync {
                output & mask == 0
            } else {
                random()
            };
            if enabled {
                output |= mask;
            } else {
                output &= !mask;
            }
        }
        Some(output)
    }
    /// Normal GameTick exclusive DROP handler. Only Delay is captured before
    /// suspension; Sync, Parts and actual option values are read after resume.
    pub fn start_tick_handler(
        &mut self,
        running: bool,
        actual_parts: u8,
        random: &mut impl FnMut() -> bool,
    ) -> Option<u8> {
        if !running || self.wait.is_some() {
            return None;
        }
        if self.settings.delay == 0 {
            return self.apply_parts(actual_parts, random);
        }
        self.callback_counter = self
            .callback_counter
            .checked_add(1)
            .expect("SkinDerp callback id exhausted");
        self.wait = Some(Wait {
            remaining: self.settings.delay.max(1),
            callback: self.callback_counter,
        });
        self.wait_registrations.push(self.callback_counter);
        None
    }
    /// This callback owns its countdown until completion. Check id before live
    /// guards so a cancelled older FIFO task cannot touch a replacement wait.
    pub fn advance_callback(
        &mut self,
        callback: u64,
        running: bool,
        actual_parts: u8,
        random: &mut impl FnMut() -> bool,
    ) -> (bool, Option<u8>) {
        let Some(wait) = self.wait.as_mut().filter(|wait| wait.callback == callback) else {
            return (false, None);
        };
        if !running {
            self.cancel_sequences();
            return (false, None);
        }
        wait.remaining -= 1;
        if wait.remaining > 0 {
            return (true, None);
        }
        self.wait = None;
        (false, self.apply_parts(actual_parts, random))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_parts_follow_enum_order_and_sync_uses_current_actual_options() {
        let mut state = SkinDerp::default();
        let mut draws = [true, false, true, false, true, false, true].into_iter();
        assert_eq!(
            state.start_tick_handler(true, 127, &mut || draws.next().unwrap()),
            Some(64 | 16 | 4 | 1)
        );
        assert!(draws.next().is_none());
        state.settings.sync = true;
        assert_eq!(
            state.start_tick_handler(true, 64 | 2, &mut || panic!("sync does not sample")),
            Some(127 ^ (64 | 2))
        );
        state.settings.parts = 1;
        assert_eq!(
            state.start_tick_handler(true, 2, &mut || unreachable!()),
            Some(64 | 2)
        );
        state.settings.parts = 0;
        assert_eq!(
            state.start_tick_handler(true, 127, &mut || unreachable!()),
            None
        );
    }
    #[test]
    fn delay_capture_live_settings_raw_values_cancellation_and_identity() {
        let mut state = SkinDerp::default();
        state.settings.delay = 2;
        assert_eq!(
            state.start_tick_handler(true, 127, &mut || unreachable!()),
            None
        );
        let old = state.take_wait_registrations()[0];
        state.settings.delay = 0;
        state.settings.parts = 1;
        state.settings.sync = true;
        assert_eq!(
            state.start_tick_handler(true, 127, &mut || unreachable!()),
            None
        );
        assert_eq!(
            state.advance_callback(old, true, 127, &mut || unreachable!()),
            (true, None)
        );
        assert_eq!(
            state.advance_callback(old, true, 0, &mut || unreachable!()),
            (false, Some(64))
        );
        state.settings.delay = i32::MIN;
        state.start_tick_handler(true, 0, &mut || unreachable!());
        let next = state.take_wait_registrations()[0];
        assert_eq!(
            state.advance_callback(old, false, 0, &mut || unreachable!()),
            (false, None)
        );
        assert!(state.busy());
        assert_eq!(
            state.advance_callback(next, true, 64, &mut || unreachable!()),
            (false, Some(0))
        );
        state.settings.delay = i32::MAX;
        state.start_tick_handler(true, 0, &mut || unreachable!());
        let next = state.take_wait_registrations()[0];
        assert_eq!(
            state.advance_callback(next, false, 0, &mut || unreachable!()),
            (false, None)
        );
        assert!(!state.busy());
    }
    #[test]
    fn captured_lifecycle_set_is_distinct_from_actual_options_and_initially_empty() {
        let mut state = SkinDerp::default();
        assert_eq!(state.on_disabled(), 0);
        state.on_enabled(1 | 64);
        state.settings.sync = true;
        assert_eq!(
            state.start_tick_handler(true, 127, &mut || unreachable!()),
            Some(0)
        );
        assert_eq!(state.on_disabled(), 65);
        assert_eq!(state.on_disabled(), 65);
        state.on_enabled(2);
        assert_eq!(state.on_disabled(), 2);
        assert_eq!(part_index("leftpants"), Some(2));
        assert_eq!(part_index("LEFTSLEEVE"), Some(4));
        assert_eq!(part_index("LeftPantsLeg"), None);
    }
}
