// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396, Copyright (c) 2015-2026 CCBlueX.
//! Criticals modes and their source state transitions; native effects live outside.
use crate::clicking::Cooldown;
use std::collections::VecDeque;
pub const MODES: [&str; 7] = [
    "None", "Packet", "NoGround", "Jump", "Blink", "Timer", "Rewound",
];
pub const PACKETS: [&str; 7] = [
    "Vanilla",
    "NCP-2024-01-25",
    "Falling",
    "Low",
    "Down",
    "Grim-2.3.59",
    "BlocksMC",
];
pub const PACKET_TYPES: [&str; 4] = [
    "OnGroundOnly",
    "PositionAndOnGround",
    "LookAndOnGround",
    "Full",
];
pub const SPRINT_MODES: [&str; 4] = ["None", "Legit", "OnNetwork", "OnAttack"];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: usize,
    pub packet_mode: usize,
    pub packet_type: usize,
    pub jump_height: f32,
    pub jump_range: f32,
    pub optimize: bool,
    pub check_killaura: bool,
    pub check_auto_clicker: bool,
    pub check_trigger_bot: bool,
    pub can_be_seen: bool,
    pub blink_delay: [i32; 2],
    pub blink_range: f32,
    pub timer_speed: f32,
    pub timer_range: f32,
    pub sprint_enabled: bool,
    pub stop_sprinting: usize,
    pub sprint_range: f32,
    pub visuals: bool,
    pub fake: bool,
    pub critical: i32,
    pub magic: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: 1,
            packet_mode: 1,
            packet_type: 3,
            jump_height: 0.42,
            jump_range: 4.,
            optimize: true,
            check_killaura: false,
            check_auto_clicker: false,
            check_trigger_bot: false,
            can_be_seen: true,
            blink_delay: [300, 600],
            blink_range: 4.,
            timer_speed: 0.8,
            timer_range: 4.,
            sprint_enabled: false,
            stop_sprinting: 1,
            sprint_range: 4.,
            visuals: false,
            fake: false,
            critical: 1,
            magic: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Player {
    pub allows_ignoring_ground: bool,
    pub on_ground: bool,
    pub sprinting: bool,
    pub was_sprinting: bool,
    pub legacy: bool,
    pub fall_distance: f64,
    pub y: f64,
    pub old_y: f64,
    pub tick_count: i32,
}
impl Player {
    pub fn can_crit(self, cooldown: Cooldown, ignore_ground: bool, ignore_sprint: bool) -> bool {
        self.allows_ignoring_ground
            && (ignore_ground || !self.on_ground)
            && cooldown.progress(0) > 0.9
            && (!self.sprinting || ignore_sprint || self.legacy)
    }
    pub fn would_crit(self, cooldown: Cooldown, ignore_sprint: bool) -> bool {
        self.can_crit(cooldown, false, ignore_sprint) && self.fall_distance > 0.0
    }
}
#[derive(Clone, Debug, Default)]
pub struct Criticals {
    pub settings: Settings,
    pub adjust_next_jump: bool,
    pub rewound_ticks: i32,
}
impl Criticals {
    pub fn transition(&mut self, was_running: bool, running: bool, old_mode: usize) {
        if running && (!was_running || old_mode != self.settings.mode) && self.settings.mode == 6 {
            self.rewound_ticks = 0;
        }
    }
    pub fn jump_active(&self, running: bool, modules: [bool; 3]) -> bool {
        let s = self.settings;
        running
            && s.mode == 3
            && ((!s.check_killaura && !s.check_auto_clicker && !s.check_trigger_bot)
                || s.check_killaura && modules[0]
                || s.check_auto_clicker && modules[1]
                || s.check_trigger_bot && modules[2])
    }
    pub fn jump(&mut self, motion: &mut f32) {
        if *motion == 0.42 && self.adjust_next_jump {
            *motion = self.settings.jump_height;
            self.adjust_next_jump = false;
        }
    }
    pub fn jump_input(
        &mut self,
        active: bool,
        p: Player,
        enemy: bool,
        should_wait: bool,
        jump: &mut bool,
    ) {
        if active
            && p.allows_ignoring_ground
            && (!self.settings.optimize || !should_wait)
            && enemy
            && p.on_ground
        {
            *jump = true;
            self.adjust_next_jump = true;
        }
    }
    pub fn sprint_running(&self, running: bool, p: Player, c: Cooldown, enemy: bool) -> bool {
        running && self.settings.sprint_enabled && p.would_crit(c, true) && enemy
    }
    /// Source order: INPUT=0, MOVEMENT_TICK=1, NETWORK=2.
    pub fn stop_sprint(&self, source: u8, when_running: bool) -> bool {
        when_running
            && match self.settings.stop_sprinting {
                1 => source <= 1,
                2 => source == 0 || source == 2,
                _ => false,
            }
    }
    pub fn offsets(
        &self,
        running: bool,
        living: bool,
        p: Player,
        c: Cooldown,
        when_running: bool,
    ) -> Vec<(f64, bool)> {
        if !running
            || self.settings.mode != 1
            || !living
            || !p.can_crit(c, true, when_running && self.settings.stop_sprinting == 0)
        {
            return vec![];
        }
        let heights: &[f64] = match self.settings.packet_mode {
            0 => &[0.2, 0.01],
            1 => &[0.11, 0.1100013579, 0.0000013579],
            2 => &[0.0625, 0.0625013579, 0.0000013579],
            3 => &[1e-9, 0.],
            4 => &[-1e-9],
            5 if !p.on_ground => &[-0.000001],
            6 if p.tick_count % 4 == 0 => &[0.0011, 0.],
            _ => &[],
        };
        heights
            .iter()
            .enumerate()
            .map(|(i, &y)| (y, self.settings.packet_mode == 6 && i == 0))
            .collect()
    }
    pub fn rewound(&mut self, p: Player) -> Option<f64> {
        self.rewound_ticks = if p.on_ground && p.y <= p.old_y {
            self.rewound_ticks.wrapping_add(1)
        } else {
            0
        };
        match self.rewound_ticks {
            0 | 1 => None,
            2 => Some(0.003),
            t => Some((0.001 - 1e-10 * f64::from(t)).max(0.)),
        }
    }
}
pub fn next_crit(c: Cooldown) -> f32 {
    ((c.attack_delay * 0.9 - 0.5 - c.ticks(0) as f32) / c.server_tick_speed as f32).max(0.)
}
pub fn wait_for_jump(c: Cooldown, initial: f32, collision_tick: Option<i32>) -> bool {
    let until_fall = initial / 0.08f32;
    let ground = collision_tick.unwrap_or((until_fall as i32).wrapping_mul(2));
    if ground as f32 + until_fall < next_crit(c) {
        return false;
    }
    until_fall + 1. < next_crit(c)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlinkAction {
    Flush,
    Pass,
    Queue,
}
pub struct Queued<P> {
    pub session: u64,
    pub millis: i64,
    pub packet: P,
}
pub struct Blink<P> {
    pub queue: VecDeque<Queued<P>>,
    pub next_delay: i32,
    pub in_state: bool,
    pub enemy: bool,
}
impl<P> Blink<P> {
    pub fn new(next_delay: i32) -> Self {
        Self {
            queue: VecDeque::new(),
            next_delay,
            in_state: false,
            enemy: false,
        }
    }
    /// Exceeded time intentionally keeps isInState unchanged, matching the
    /// early return in CriticalsBlink's handler (even for packet=null probes).
    pub fn action(
        &mut self,
        running: bool,
        would_crit: bool,
        now: i64,
        pass: bool,
        sample: impl FnOnce() -> i32,
    ) -> BlinkAction {
        if running && !would_crit && self.enemy {
            if self
                .queue
                .front()
                .is_some_and(|p| now.wrapping_sub(p.millis) >= i64::from(self.next_delay))
            {
                self.next_delay = sample();
                return BlinkAction::Flush;
            }
            self.in_state = true;
            if pass {
                BlinkAction::Pass
            } else {
                BlinkAction::Queue
            }
        } else {
            self.in_state = false;
            BlinkAction::Flush
        }
    }
    pub fn enqueue(&mut self, session: u64, now: i64, packet: P) {
        self.queue.push_back(Queued {
            session,
            millis: now,
            packet,
        });
    }
    pub fn take(&mut self) -> Vec<Queued<P>> {
        self.queue.drain(..).collect()
    }
    pub fn disconnect(&mut self) {
        self.queue.clear();
        self.in_state = false;
    }
}
