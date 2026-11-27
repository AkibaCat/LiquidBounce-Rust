// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleItemScroller and its AbstractContainerScreen mixin at ae0eb396.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClickMode {
    #[default]
    QuickMove,
}
impl ClickMode {
    pub const fn tag(self) -> &'static str {
        match self { Self::QuickMove => "QuickMove" }
    }
    pub fn parse(tag: &str) -> Option<Self> {
        // Tagged.makeLookupTable uses Java CASE_INSENSITIVE_ORDER.
        fn fold(c: char) -> char {
            let mut upper = c.to_uppercase();
            let first = upper.next().unwrap_or(c);
            let upper = if upper.next().is_none() { first } else { c };
            upper.to_lowercase().next().unwrap_or(upper)
        }
        tag.chars().map(fold).eq("QuickMove".chars().map(fold)).then_some(Self::QuickMove)
    }
}

#[derive(Clone, Debug)]
pub struct ItemScroller {
    pub click_mode: ClickMode,
    pub delay: [i32; 2],
    /// Chronometer's initial value is zero, and enable/disable does not reset it.
    pub last_update: i64,
}
impl Default for ItemScroller {
    fn default() -> Self {
        Self { click_mode: ClickMode::QuickMove, delay: [2, 3], last_update: 0 }
    }
}

impl ItemScroller {
    pub fn reset_chronometer(&mut self, now_ms: i64) {
        self.last_update = now_ms;
    }

    /// Delay.random is sampled on every eligible query, including mouseClicked.
    /// The time read follows the sample, as in Chronometer.hasAtLeastElapsed.
    pub fn can_perform_scroll(
        &self,
        running: bool,
        shift: bool,
        left_pressed: bool,
        sample: impl FnOnce(i32, i32) -> i32,
        now: impl FnOnce() -> i64,
    ) -> bool {
        if !shift || !running || !left_pressed { return false; }
        let ticks = sample(self.delay[0], self.delay[1]);
        self.last_update.wrapping_add(i64::from(ticks) * 50) <= now()
    }
}
