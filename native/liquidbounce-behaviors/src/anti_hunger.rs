// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleAntiHunger policy, LiquidBounce ae0eb396.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub no_sprint: bool,
    pub while_swimming: bool,
    pub keep_floating: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            no_sprint: true,
            while_swimming: false,
            keep_floating: true,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Context {
    pub swimming: bool,
    pub passenger: bool,
    pub destroying: bool,
    pub in_water: bool,
    pub under_water: bool,
    pub fall_distance: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Input,
    Network,
    Movement,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Warning {
    Sprint,
    Break,
}
impl Warning {
    pub const fn message_key(self) -> &'static str {
        match self {
            Self::Sprint => "liquidbounce.module.antiHunger.messages.warnSprint",
            Self::Break => "liquidbounce.module.antiHunger.messages.warnBreak",
        }
    }
    pub const fn jump(self) -> bool {
        matches!(self, Self::Sprint)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Packet {
    PlayerCommand { start_sprinting: bool },
    PlayerAction { start_destroy: bool },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AntiHunger {
    pub settings: Settings,
    /// Original Chronometer starts at epoch zero and survives all lifecycles.
    pub last_notification: i64,
}
impl AntiHunger {
    pub fn no_sprint_running(&self, running: bool, swimming: bool) -> bool {
        running && self.settings.no_sprint && (!swimming || self.settings.while_swimming)
    }
    pub fn sprint(&self, running: bool, swimming: bool, source: Source, sprint: bool) -> bool {
        if self.no_sprint_running(running, swimming)
            && matches!(source, Source::Input | Source::Network)
        {
            false
        } else {
            sprint
        }
    }
    /// Call immediately on the ordinary PacketEvent, including a cancelled one.
    /// Clock evaluation follows the original short-circuit order exactly.
    pub fn warning(
        &self,
        running: bool,
        swimming: bool,
        packet: Packet,
        mut now: impl FnMut() -> i64,
    ) -> Option<Warning> {
        if !running {
            return None;
        }
        match packet {
            Packet::PlayerCommand { start_sprinting } => {
                if !self.no_sprint_running(running, swimming)
                    || self.last_notification.wrapping_add(500) >= now()
                {
                    return None;
                }
                start_sprinting.then_some(Warning::Sprint)
            }
            Packet::PlayerAction { start_destroy } => {
                if self.last_notification.wrapping_add(500) >= now() {
                    return None;
                }
                start_destroy.then_some(Warning::Break)
            }
        }
    }
    /// Original ordering is notification, Chronometer.reset(now), then jump.
    pub fn notification_published(&mut self, now: i64) {
        self.last_notification = now;
    }
    pub fn movement(&self, running: bool, context: Context, on_ground: &mut bool) {
        if running
            && self.settings.keep_floating
            && !context.passenger
            && !context.destroying
            && !context.in_water
            && !context.swimming
            && !context.under_water
            && *on_ground
            && context.fall_distance <= 0.0
        {
            *on_ground = false;
        }
    }
}
