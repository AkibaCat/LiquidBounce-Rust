// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoLeave / tickConditional, including its threshold-tick precedence.
#[path = "navigation/java_case.rs"]
mod java_case;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KickMode {
    #[default]
    Quit,
    InvalidPacket,
    SelfHurt,
    IllegalChat,
    IllegalSwitchitem,
    IllegalInteract,
}
impl KickMode {
    pub const ALL: [Self; 6] = [
        Self::Quit,
        Self::InvalidPacket,
        Self::SelfHurt,
        Self::IllegalChat,
        Self::IllegalSwitchitem,
        Self::IllegalInteract,
    ];
    pub fn tag(self) -> &'static str {
        match self {
            Self::Quit => "Quit",
            Self::InvalidPacket => "InvalidPacket",
            Self::SelfHurt => "SelfHurt",
            Self::IllegalChat => "IllegalChat",
            Self::IllegalSwitchitem => "IllegalSwitchitem",
            Self::IllegalInteract => "IllegalInteract",
        }
    }
    pub fn parse(tag: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|mode| java_case::equals_ignore_case(mode.tag(), tag))
    }
    pub fn code(self) -> i32 {
        Self::ALL.into_iter().position(|mode| mode == self).unwrap() as i32
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub health: f32,
    pub delay: [i32; 2],
    pub mode: KickMode,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            health: 8.0,
            delay: [0, 0],
            mode: KickMode::Quit,
        }
    }
}
#[derive(Debug, Default)]
pub struct AutoLeave {
    pub settings: Settings,
    next: u64,
    wait: Option<(u64, i32, i32)>,
}
impl AutoLeave {
    pub fn cancel_sequences(&mut self) {
        self.wait = None;
    }
    pub fn busy(&self) -> bool {
        self.wait.is_some()
    }
    pub fn waiting(&self, id: u64) -> bool {
        self.wait.is_some_and(|wait| wait.0 == id)
    }
    /// Start one DROP handler. Positive and negative delays suspend without
    /// querying the player; zero invokes the predicate synchronously.
    pub fn start(
        &mut self,
        running: bool,
        delay: i32,
        break_loop: impl FnOnce() -> bool,
    ) -> (Option<u64>, bool) {
        if !running || self.wait.is_some() {
            return (None, false);
        }
        if delay == 0 {
            return (None, !break_loop());
        }
        self.next = self.next.wrapping_add(1);
        self.wait = Some((self.next, delay, 0));
        (Some(self.next), false)
    }
    /// tickConditional calls breakLoop before comparing elapsed. Its Boolean
    /// result is elapsed >= ticks, even if breakLoop is true on that last tick.
    pub fn advance(
        &mut self,
        id: u64,
        running: bool,
        break_loop: impl FnOnce() -> bool,
    ) -> (bool, bool) {
        let Some((ticket, ticks, elapsed)) = self.wait.filter(|wait| wait.0 == id) else {
            return (false, false);
        };
        if !running {
            self.wait = None;
            return (false, false);
        }
        let elapsed = elapsed.wrapping_add(1);
        if break_loop() || elapsed >= ticks {
            self.wait = None;
            (false, elapsed >= ticks)
        } else {
            self.wait = Some((ticket, ticks, elapsed));
            (true, false)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn short_delay_cancel_and_threshold_guard_smoke() {
        let mut module = AutoLeave::default();
        assert_eq!(module.start(true, 0, || false), (None, true));
        assert_eq!(module.start(true, 0, || true), (None, false));
        let id = module
            .start(true, 2, || panic!("predicate must wait"))
            .0
            .unwrap();
        assert_eq!(module.advance(id, true, || false), (true, false));
        assert_eq!(module.advance(id, true, || true), (false, true));
        let id = module.start(true, 3, || false).0.unwrap();
        assert_eq!(module.advance(id, true, || true), (false, false));
        let id = module.start(true, 3, || false).0.unwrap();
        module.cancel_sequences();
        assert_eq!(
            module.advance(id, true, || panic!("cancelled")),
            (false, false)
        );
        for mode in KickMode::ALL {
            assert_eq!(KickMode::parse(mode.tag()), Some(mode));
        }
    }
}
