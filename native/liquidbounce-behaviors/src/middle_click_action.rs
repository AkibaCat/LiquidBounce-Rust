// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleMiddleClickAction at LiquidBounce ae0eb396.
#[path = "navigation/java_case.rs"]
mod java_case;
pub const MODES: [&str; 2] = ["FriendClicker", "Pearl"];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    FriendClicker,
    Pearl,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        MODES[self as usize]
    }
    pub fn parse(s: &str) -> Option<Self> {
        [Self::FriendClicker, Self::Pearl]
            .into_iter()
            .find(|mode| java_case::equals_ignore_case(s, mode.tag()))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Offhand,
    Hotbar(u8),
}
impl Slot {
    pub fn hand(self) -> i32 {
        if self == Self::Offhand { 1 } else { 0 }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Select(Slot, i32),
    Use(Slot, i32),
    ToggleFriend,
}
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub screen_open: bool,
    pub pitch: f32,
    pub pickup: bool,
    pub pearl: Option<Slot>,
    pub candidate: bool,
    pub visible: bool,
}
#[derive(Clone, Debug)]
pub struct MiddleClickAction {
    pub mode: Mode,
    pub pick_up_range: f32,
    pub slot_reset_delay: i32,
    pub stop_on_submit: [f32; 2],
    was_pressed: bool,
    clicked: bool,
}
impl Default for MiddleClickAction {
    fn default() -> Self {
        Self {
            mode: Mode::FriendClicker,
            pick_up_range: 3.0,
            slot_reset_delay: 1,
            stop_on_submit: [85.0, 90.0],
            was_pressed: false,
            clicked: false,
        }
    }
}
impl MiddleClickAction {
    pub fn set_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            if self.mode == Mode::Pearl {
                self.pearl_disable();
            }
            self.mode = mode;
        }
    }
    pub fn pearl_disable(&mut self) {
        self.was_pressed = false;
    }
    pub fn world_change(&mut self) {
        if self.mode == Mode::Pearl {
            self.was_pressed = false;
        }
    }
    pub fn cancel_pick(&self, running: bool, pearl: Option<Slot>) -> bool {
        running && self.mode == Mode::Pearl && pearl.is_some()
    }
    pub fn tick(&mut self, running: bool, c: Context) -> Option<Action> {
        if !running {
            return None;
        }
        match self.mode {
            Mode::Pearl => {
                if c.screen_open
                    || (c.pitch >= self.stop_on_submit[0] && c.pitch <= self.stop_on_submit[1])
                {
                    self.was_pressed = false;
                    return None;
                }
                if c.pickup {
                    let slot = c.pearl?;
                    self.was_pressed = true;
                    Some(Action::Select(slot, self.slot_reset_delay))
                } else if self.was_pressed {
                    self.was_pressed = false;
                    c.pearl.map(|slot| Action::Use(slot, self.slot_reset_delay))
                } else {
                    None
                }
            }
            Mode::FriendClicker => {
                // The source returns before updating clicked when the ray misses.
                if !c.candidate {
                    return None;
                }
                let toggle = c.visible && c.pickup && !self.clicked;
                self.clicked = c.pickup;
                toggle.then_some(Action::ToggleFriend)
            }
        }
    }
}
