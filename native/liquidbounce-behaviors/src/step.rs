// SPDX-License-Identifier: GPL-3.0-or-later
//! Step's five modes, original wait phases, and Instant jump-order packets.
pub const MODES: [&str; 5] = [
    "Instant",
    "Legit",
    "Vulcan-2.8.6",
    "BlocksMC",
    "Hypixel-2025-03-11",
];
pub const JUMP_ORDER: [f64; 9] = [
    0.,
    0.41999998688698,
    0.7531999805212,
    1.00133597911215,
    1.166109260938214,
    1.24918707874468,
    1.25220334025373,
    1.17675927506424,
    1.024424088213685,
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Instant,
    Legit,
    Vulcan,
    BlocksMc,
    Hypixel,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        MODES[self as usize]
    }
    pub fn parse(v: &str) -> Option<Self> {
        match v {
            "Instant" => Some(Self::Instant),
            "Legit" => Some(Self::Legit),
            "Vulcan-2.8.6" | "Vulcan286" => Some(Self::Vulcan),
            "BlocksMC" => Some(Self::BlocksMc),
            "Hypixel-2025-03-11" | "Hypixel" => Some(Self::Hypixel),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub height: f32,
    pub trim: bool,
    pub simulate_jump_order: [i32; 2],
    pub wait: [i32; 2],
    pub full_packet: bool,
    pub base_timer: f32,
    pub recovery_timer: f32,
    pub alternate_bypass: bool,
    pub spoof: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Instant,
            height: 1.,
            trim: false,
            simulate_jump_order: [0, 2],
            wait: [0, 0],
            full_packet: true,
            base_timer: 3.,
            recovery_timer: 0.6,
            alternate_bypass: false,
            spoof: false,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Effects {
    pub jump: bool,
    pub set_y: Option<f64>,
    pub add_y: f64,
    pub strafe: Option<f64>,
    pub subtract_y_fraction: bool,
    pub ground: bool,
    pub timer: Option<(f32, i32)>,
    pub wait: Option<u64>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Success {
    pub destinations: Vec<f64>,
    pub award_jump: bool,
    pub full_packet: bool,
}
#[derive(Clone, Copy, Debug)]
struct Wait {
    id: u64,
    phase: u8,
    elapsed: i32,
    ticks: i32,
    height: f64,
}
#[derive(Debug, Default)]
pub struct Step {
    pub settings: Settings,
    pub ticks_wait: i32,
    pub stepping: bool,
    pub step_counter: i32,
    next: u64,
    waiter: Option<Wait>,
}
impl Step {
    pub fn set_mode(&mut self, mode: Mode) {
        if self.settings.mode != mode {
            self.disabled();
            self.settings.mode = mode;
        }
    }
    pub fn disabled(&mut self) {
        self.stepping = false;
        self.waiter = None;
        if self.settings.mode == Mode::Vulcan {
            self.step_counter = 0;
        }
    }
    pub fn game_tick(&mut self, running: bool) {
        if running && self.settings.mode == Mode::Instant && self.ticks_wait > 0 {
            self.ticks_wait -= 1;
        }
    }
    pub fn height(&self, running: bool, original: f32) -> f32 {
        if running && self.settings.mode == Mode::Instant && self.ticks_wait <= 0 {
            self.settings.height
        } else {
            original
        }
    }
    pub fn success(
        &mut self,
        running: bool,
        adjusted_y: f64,
        player_y: f64,
        random: impl FnOnce([i32; 2]) -> i32,
    ) -> Success {
        if !running
            || self.settings.mode != Mode::Instant
            || adjusted_y <= 0.5
            || self.settings.simulate_jump_order == [0, 0]
        {
            return Success::default();
        }
        let s = self.settings;
        let from = s.simulate_jump_order[0].clamp(0, 8) as usize;
        let to = s.simulate_jump_order[1].clamp(0, 8) as usize;
        let destinations = if from <= to {
            JUMP_ORDER[from..=to]
                .iter()
                .filter(|&&y| y != 0.)
                .map(|y| {
                    if s.trim {
                        (player_y + y).min(player_y + adjusted_y)
                    } else {
                        player_y + y
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        self.ticks_wait = random(s.wait);
        Success {
            destinations,
            award_jump: true,
            full_packet: s.full_packet,
        }
    }
    fn wait(&mut self, phase: u8, ticks: i32, height: f64) -> u64 {
        self.next = self.next.checked_add(1).expect("Step waiter exhausted");
        self.waiter = Some(Wait {
            id: self.next,
            phase,
            elapsed: 0,
            ticks,
            height,
        });
        self.next
    }
    pub fn input(&mut self, running: bool, speed_running: bool, can_step: [bool; 3]) -> Effects {
        if !running || self.stepping {
            return Effects::default();
        }
        match self.settings.mode {
            Mode::Vulcan if can_step[0] => {
                self.stepping = true;
                self.step_counter = self.step_counter.wrapping_add(1);
                Effects {
                    jump: true,
                    wait: Some(self.wait(0, 2, 1.)),
                    ..Default::default()
                }
            }
            Mode::BlocksMc if !speed_running && can_step[0] => {
                self.stepping = true;
                Effects {
                    jump: true,
                    set_y: Some(0.42),
                    timer: Some((self.settings.base_timer, 3)),
                    wait: Some(self.wait(0, 1, 1.)),
                    ..Default::default()
                }
            }
            Mode::Hypixel if can_step[2] => {
                self.stepping = true;
                let height = if can_step[0] {
                    1.
                } else if can_step[1] {
                    1.25
                } else {
                    1.5
                };
                Effects {
                    jump: true,
                    set_y: Some(0.42),
                    wait: Some(self.wait(0, 1, height)),
                    ..Default::default()
                }
            }
            _ => Effects::default(),
        }
    }
    pub fn callback(&mut self, id: u64, running: bool, speed_running: bool) -> (bool, Effects) {
        let Some(wait) = self.waiter.as_mut().filter(|w| w.id == id) else {
            return (false, Effects::default());
        };
        if !running || (self.settings.mode == Mode::BlocksMc && speed_running) {
            self.disabled();
            return (false, Effects::default());
        }
        wait.elapsed += 1;
        if wait.elapsed < wait.ticks {
            return (true, Effects::default());
        }
        let w = *wait;
        self.waiter = None;
        let mut e = Effects::default();
        match self.settings.mode {
            Mode::Vulcan => match w.phase {
                0 => {
                    if self.step_counter % 2 == 0 {
                        e.set_y = Some(0.24680001947880004);
                        e.strafe = Some(0.2);
                    }
                    e.wait = Some(self.wait(1, 1, w.height));
                }
                1 => {
                    if self.step_counter % 2 == 0 {
                        e.set_y = Some(0.);
                    }
                    e.wait = Some(self.wait(2, 1, w.height));
                }
                _ => {
                    e.set_y = Some(-0.17);
                    self.stepping = false;
                }
            },
            Mode::BlocksMc => match w.phase {
                0 => {
                    e.set_y = Some(0.33);
                    e.wait = Some(self.wait(1, 1, w.height));
                }
                1 => {
                    e.set_y = Some(0.25);
                    e.wait = Some(self.wait(2, 2, w.height));
                }
                _ => {
                    e.strafe = Some(0.281);
                    e.subtract_y_fraction = true;
                    e.timer = Some((self.settings.recovery_timer, 2));
                    self.stepping = false;
                }
            },
            Mode::Hypixel => match w.phase {
                0 => {
                    if w.height > 1. {
                        e.add_y = 0.061;
                    }
                    e.wait = Some(self.wait(1, 2, w.height));
                }
                1 => {
                    if w.height == 1. {
                        e.add_y = -0.14;
                    } else {
                        e.add_y = -0.095;
                    }
                    if w.height > 1.25 {
                        e.wait = Some(self.wait(2, 5, w.height));
                    } else {
                        self.stepping = false;
                        e.strafe = Some(0.1838601407459074);
                    }
                }
                _ => {
                    if self.settings.alternate_bypass {
                        e.ground = true;
                    } else {
                        e.set_y = Some(0.42);
                    }
                    self.stepping = false;
                    e.strafe = Some(0.1838601407459074);
                }
            },
            _ => self.stepping = false,
        }
        (false, e)
    }
    pub fn queues_outgoing(&self, running: bool, speed_running: bool) -> bool {
        running && !speed_running && self.settings.mode == Mode::BlocksMc && self.stepping
    }
}
