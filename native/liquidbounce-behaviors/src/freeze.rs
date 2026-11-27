// SPDX-License-Identifier: GPL-3.0-or-later
//! Freeze mode lifecycle and frame conditions, LiquidBounce ae0eb396.
pub const MODES: [&str; 4] = ["Queue", "Cancel", "Stationary", "TickMovement"];
pub const REASONS: [&str; 6] = [
    "Flag",
    "OnGround",
    "OnMovementInput",
    "InLiquid",
    "Void",
    "OnUseItem",
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    Queue,
    Cancel,
    #[default]
    Stationary,
    TickMovement,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        MODES[self as usize]
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Queue" => Some(Self::Queue),
            "Cancel" => Some(Self::Cancel),
            "Stationary" => Some(Self::Stationary),
            "TickMovement" => Some(Self::TickMovement),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub mode: Mode,
    pub disable_on: u32,
    pub notification: bool,
    pub queue_origins: u32,
    pub cancel_origins: u32,
    pub cancel_c0b: bool,
    pub interval: [i32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Stationary,
            disable_on: 1,
            notification: false,
            queue_origins: 2,
            cancel_origins: 2,
            cancel_c0b: true,
            interval: [20, 20],
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Freeze {
    running: bool,
    mode: Mode,
    ticks: i32,
    pub previous_offsets: [f32; 2],
}
impl Freeze {
    pub fn synchronize(
        &mut self,
        running: bool,
        settings: Settings,
        mut random: impl FnMut([i32; 2]) -> i32,
    ) {
        if running
            && (!self.running || self.mode != settings.mode)
            && settings.mode == Mode::TickMovement
        {
            self.ticks = random(settings.interval);
        }
        if !running || settings.mode != Mode::TickMovement {
            self.ticks = 0;
        }
        self.running = running;
        self.mode = settings.mode;
    }
    pub fn holds(self) -> bool {
        self.running && self.mode != Mode::TickMovement
    }
    pub fn disable_reason(settings: Settings, flag: bool, conditions: [bool; 5]) -> Option<usize> {
        std::iter::once(flag)
            .chain(conditions)
            .enumerate()
            .find_map(|(i, v)| (v && settings.disable_on & (1 << i) != 0).then_some(i))
    }
    pub fn cancel_movement(
        &mut self,
        settings: Settings,
        mut random: impl FnMut([i32; 2]) -> i32,
    ) -> bool {
        if !self.running || self.mode != Mode::TickMovement {
            return false;
        }
        self.ticks -= 1;
        if self.ticks <= 0 {
            self.ticks = random(settings.interval);
            false
        } else {
            true
        }
    }
    pub fn offsets(&mut self, mut random: impl FnMut() -> f64) -> [f32; 2] {
        for previous in &mut self.previous_offsets {
            loop {
                let v = (0.002 + random() * 0.008) as f32;
                if (v - *previous).abs() >= 1e-6 {
                    *previous = v;
                    break;
                }
            }
        }
        self.previous_offsets
    }
}
