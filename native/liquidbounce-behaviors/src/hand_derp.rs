// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleHandDerp at LiquidBounce ae0eb396. The real option and
//! server-advertised hand are intentionally independent, including disable.

pub const MODES: [&str; 2] = ["Delay", "Swing"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Delay,
    Swing,
}
impl Mode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Delay => "Delay",
            Self::Swing => "Swing",
        }
    }
    pub fn parse(tag: &str) -> Option<Self> {
        match tag {
            "Delay" => Some(Self::Delay),
            "Swing" => Some(Self::Swing),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainHand {
    Left,
    Right,
}
impl MainHand {
    pub const fn id(self) -> i32 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
    pub const fn from_left(left: bool) -> Self {
        if left { Self::Left } else { Self::Right }
    }
    pub const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub silent: bool,
    pub mode: Mode,
    pub delay: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            silent: false,
            mode: Mode::Delay,
            delay: 1,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Wait {
    remaining: i32,
    ticket: u64,
}
#[derive(Clone, Debug, Default)]
pub struct HandDerp {
    pub settings: Settings,
    original: Option<MainHand>,
    current: Option<MainHand>,
    wait: Option<Wait>,
}
impl HandDerp {
    /// Object initialization, once for the client lifetime. Do not call this
    /// with the first enabled/world snapshot instead of the startup option.
    pub fn initialize(&mut self, initial: MainHand) {
        if self.original.is_none() {
            self.original = Some(initial);
            self.current = Some(initial);
        }
    }
    pub const fn original(&self) -> Option<MainHand> {
        self.original
    }
    pub const fn current(&self) -> Option<MainHand> {
        self.current
    }
    pub const fn busy(&self) -> bool {
        self.wait.is_some()
    }
    pub fn set_mode(&mut self, mode: Mode) {
        if self.settings.mode != mode {
            self.settings.mode = mode;
            self.cancel_sequences();
        }
    }
    pub fn cancel_sequences(&mut self) {
        self.wait = None;
    }
    fn switch(&mut self) -> Option<MainHand> {
        let hand = self.current?.opposite();
        self.current = Some(hand);
        Some(hand)
    }
    /// Global CoroutineTicker phase. Registration tickets are allocated by
    /// the caller across every module, never by this individual behavior.
    pub fn begin_tick(&mut self, running: bool) {
        if !running || self.settings.mode != Mode::Delay {
            self.cancel_sequences();
            return;
        }
        if let Some(wait) = self.wait.as_mut() {
            wait.remaining -= 1;
        }
    }
    pub fn ready_ticket(&self) -> Option<u64> {
        self.wait
            .filter(|wait| wait.remaining <= 0)
            .map(|wait| wait.ticket)
    }
    pub fn resume(&mut self, ticket: u64) -> Option<MainHand> {
        if self.ready_ticket() != Some(ticket) {
            return None;
        }
        self.wait = None;
        self.switch()
    }
    /// A cancelled older global task must never decrement its replacement.
    /// Check identity before applying the live guard or advancing the delay.
    pub fn advance_callback(&mut self, ticket: u64, running: bool) -> (bool, Option<MainHand>) {
        if self.wait.is_none_or(|wait| wait.ticket != ticket) {
            return (false, None);
        }
        self.begin_tick(running);
        if self.ready_ticket() == Some(ticket) {
            return (false, self.resume(ticket));
        }
        (self.wait.is_some(), None)
    }
    /// Normal priority GameTick exclusive DROP handler. Delay is captured
    /// before suspension; zero is immediate and a negative raw value waits
    /// one tick, exactly as waitTicks/tickUntil does in the pinned source.
    pub fn start_delay(
        &mut self,
        running: bool,
        register: impl FnOnce() -> u64,
    ) -> Option<MainHand> {
        if !running || self.settings.mode != Mode::Delay || self.wait.is_some() {
            return None;
        }
        if self.settings.delay == 0 {
            return self.switch();
        }
        self.wait = Some(Wait {
            remaining: self.settings.delay.max(1),
            ticket: register(),
        });
        None
    }
    /// Ordinary priority=1 PacketEvent, even if an earlier listener cancelled
    /// Punch. NoSwing's earlier connection injection prevents the event itself.
    pub fn punch(&mut self, running: bool) -> Option<MainHand> {
        if !running || self.settings.mode != Mode::Swing {
            return None;
        }
        self.switch()
    }
    /// Preserve the source's unusual data.id == options.mainHand.id predicate:
    /// no entity-id check and no replacement with the main-arm metadata index.
    pub fn cancel_metadata(
        &self,
        running: bool,
        actual: MainHand,
        ids: impl IntoIterator<Item = u8>,
    ) -> bool {
        running && self.settings.silent && ids.into_iter().any(|id| i32::from(id) == actual.id())
    }
    pub fn on_disabled(&mut self, actual: MainHand) -> Option<MainHand> {
        self.cancel_sequences();
        if self.original.is_some_and(|original| actual != original) {
            self.switch()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initialization_disable_and_live_option_are_independent() {
        for initial in [MainHand::Left, MainHand::Right] {
            let mut state = HandDerp::default();
            state.initialize(initial);
            state.initialize(initial.opposite());
            state.set_mode(Mode::Swing);
            assert_eq!(state.punch(true), Some(initial.opposite()));
            assert_eq!(state.on_disabled(initial), None);
            assert_eq!(state.current(), Some(initial.opposite()));
            assert_eq!(state.on_disabled(initial.opposite()), Some(initial));
            assert_eq!(
                state.on_disabled(initial.opposite()),
                Some(initial.opposite())
            );
            assert_eq!(state.original(), Some(initial));
        }
    }
    #[test]
    fn delay_capture_zero_negative_and_global_ticket() {
        let mut state = HandDerp::default();
        state.initialize(MainHand::Right);
        state.settings.delay = 2;
        assert_eq!(state.start_delay(true, || 98), None);
        state.settings.delay = 0;
        assert_eq!(state.start_delay(true, || panic!("exclusive DROP")), None);
        state.begin_tick(true);
        assert_eq!(state.ready_ticket(), None);
        state.begin_tick(true);
        assert_eq!(state.ready_ticket(), Some(98));
        assert_eq!(state.resume(99), None);
        assert_eq!(state.resume(98), Some(MainHand::Left));
        assert_eq!(
            state.start_delay(true, || panic!("zero never registers")),
            Some(MainHand::Right)
        );
        state.settings.delay = i32::MIN;
        state.start_delay(true, || 109);
        state.begin_tick(true);
        assert_eq!(state.ready_ticket(), Some(109));
        assert_eq!(state.resume(109), Some(MainHand::Left));
    }
    #[test]
    fn same_mode_retains_and_changed_mode_disable_world_cancel() {
        let mut state = HandDerp::default();
        state.initialize(MainHand::Left);
        state.start_delay(true, || 1);
        state.set_mode(Mode::Delay);
        assert!(state.busy());
        state.set_mode(Mode::Swing);
        assert!(!state.busy());
        assert_eq!(state.punch(false), None);
        assert_eq!(state.punch(true), Some(MainHand::Right));
        state.set_mode(Mode::Delay);
        state.start_delay(true, || 2);
        state.begin_tick(false);
        assert!(!state.busy());
        state.start_delay(true, || 3);
        state.on_disabled(MainHand::Left);
        assert!(!state.busy());
        assert_eq!(state.current(), Some(MainHand::Right));
    }
    #[test]
    fn silent_uses_actual_option_id_and_whole_packet() {
        let mut state = HandDerp::default();
        state.settings.silent = true;
        assert!(state.cancel_metadata(true, MainHand::Left, [9, 0, 17]));
        assert!(!state.cancel_metadata(true, MainHand::Right, [0, 17]));
        assert!(state.cancel_metadata(true, MainHand::Right, [1]));
        assert!(!state.cancel_metadata(false, MainHand::Right, [1]));
        assert!(!state.cancel_metadata(true, MainHand::Left, []));
        state.settings.silent = false;
        assert!(!state.cancel_metadata(true, MainHand::Left, [0]));
    }
}
