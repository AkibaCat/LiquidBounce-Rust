// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleNoSlow and all eleven child trees, LiquidBounce ae0eb396.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    None,
    Reuse,
    Switch,
    Blink,
    Interact,
    Grim2360,
    Grim2364,
    Grim2371,
    InvalidHand,
    Intave14,
    Jump,
    Release,
    Aac5,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Reuse => "Reuse",
            Self::Switch => "Switch",
            Self::Blink => "Blink",
            Self::Interact => "Interact",
            Self::Grim2360 => "Grim-2.3.60",
            Self::Grim2364 => "Grim-2.3.64-MC1.8",
            Self::Grim2371 => "Grim-2.3.71",
            Self::InvalidHand => "InvalidHand",
            Self::Intave14 => "Intave-14.8.4",
            Self::Jump => "Jump",
            Self::Release => "Release",
            Self::Aac5 => "AAC-5",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "None" => Self::None,
            "Reuse" => Self::Reuse,
            "Switch" => Self::Switch,
            "Blink" => Self::Blink,
            "Interact" => Self::Interact,
            "Grim-2.3.60" | "Grim2360" => Self::Grim2360,
            "Grim-2.3.64-MC1.8" | "Grim2364-1.8" => Self::Grim2364,
            "Grim-2.3.71" | "Grim2371" => Self::Grim2371,
            "InvalidHand" => Self::InvalidHand,
            "Intave-14.8.4" | "Intave14" => Self::Intave14,
            "Jump" => Self::Jump,
            "Release" => Self::Release,
            "AAC-5" | "AAC5" => Self::Aac5,
            _ => return None,
        })
    }
}
pub const BLOCKING_MODES: [Mode; 10] = [
    Mode::None,
    Mode::Reuse,
    Mode::Switch,
    Mode::Blink,
    Mode::Interact,
    Mode::Grim2360,
    Mode::Grim2364,
    Mode::Grim2371,
    Mode::InvalidHand,
    Mode::Intave14,
];
pub const CONSUME_MODES: [Mode; 8] = [
    Mode::None,
    Mode::Grim2360,
    Mode::Grim2364,
    Mode::InvalidHand,
    Mode::Grim2371,
    Mode::Jump,
    Mode::Intave14,
    Mode::Release,
];
pub const BOW_MODES: [Mode; 5] = [
    Mode::None,
    Mode::Grim2360,
    Mode::Grim2364,
    Mode::Grim2371,
    Mode::InvalidHand,
];
pub const SNEAK_MODES: [Mode; 3] = [Mode::None, Mode::Switch, Mode::Aac5];
pub const TREES: [&str; 11] = [
    "Blocking",
    "Consume",
    "Bow",
    "Bundle",
    "Sneaking",
    "Soulsand",
    "SlimeBlock",
    "HoneyBlock",
    "PowderSnow",
    "Fluid",
    "Slowness",
];
pub fn modes(group: usize) -> &'static [Mode] {
    match group {
        0 => &BLOCKING_MODES,
        1 => &CONSUME_MODES,
        2 => &BOW_MODES,
        4 => &SNEAK_MODES,
        _ => &[],
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Timing {
    #[default]
    PreAndPost,
    Pre,
    Post,
}
impl Timing {
    pub fn tag(self) -> &'static str {
        match self {
            Self::PreAndPost => "PreAndPost",
            Self::Pre => "Pre",
            Self::Post => "Post",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "PreAndPost" => Some(Self::PreAndPost),
            "Pre" => Some(Self::Pre),
            "Post" => Some(Self::Post),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UseGroup {
    pub enabled: bool,
    pub forward: f32,
    pub sideways: f32,
    pub mode: Mode,
}
impl Default for UseGroup {
    fn default() -> Self {
        Self {
            enabled: true,
            forward: 1.,
            sideways: 1.,
            mode: Mode::None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub uses: [UseGroup; 4],
    pub enabled: [bool; 7],
    pub multipliers: [f32; 5],
    pub fluid_collision: bool,
    pub only_slow_server: bool,
    pub no_block_interact: [bool; 2],
    pub reuse_timing: Timing,
    pub switch_timing: Timing,
    pub sneak_mode: Mode,
    pub sneak_switch_timing: Timing,
    pub sneak_aac_timing: Timing,
    pub intave_new: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            uses: [UseGroup::default(); 4],
            enabled: [true; 7],
            multipliers: [1., 1., 1., 1., 1.],
            fluid_collision: true,
            only_slow_server: false,
            no_block_interact: [true; 2],
            reuse_timing: Timing::default(),
            switch_timing: Timing::default(),
            sneak_mode: Mode::None,
            sneak_switch_timing: Timing::default(),
            sneak_aac_timing: Timing::default(),
            intave_new: false,
        }
    }
}
// Slowness has a separate multiplier: its default is zero, unlike block factors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoSlow {
    pub settings: Settings,
    pub slowness: f32,
    pub prevent: bool,
}
impl Default for NoSlow {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            slowness: 0.,
            prevent: false,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Action {
    #[default]
    None,
    Eat,
    Drink,
    Block,
    Spyglass,
    Horn,
    Brush,
    Bow,
    Trident,
    Crossbow,
    Bundle,
    Spear,
}
impl Action {
    pub fn group(self) -> Option<usize> {
        match self {
            Self::Eat | Self::Drink => Some(1),
            Self::Block | Self::Spyglass | Self::Horn | Self::Brush => Some(0),
            Self::Bow | Self::Trident | Self::Crossbow => Some(2),
            Self::Bundle => Some(3),
            _ => None,
        }
    }
    pub fn from_id(id: u8) -> Self {
        match id {
            1 => Self::Eat,
            2 => Self::Drink,
            3 => Self::Block,
            4 => Self::Spyglass,
            5 => Self::Horn,
            6 => Self::Brush,
            7 => Self::Bow,
            8 => Self::Trident,
            9 => Self::Crossbow,
            10 => Self::Bundle,
            11 => Self::Spear,
            _ => Self::None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Context {
    pub using: bool,
    pub consumable: bool,
    pub action: Action,
    pub hand: i32,
    pub ticks: i32,
    pub remaining: i32,
    pub blocking: bool,
    pub tracked_hand: Option<i32>,
    pub moving: bool,
    pub selected: i32,
    pub sneaking: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Carried(i32),
    Use { hand: i32, predict: bool },
    Release { at_player: bool },
    StopUsing,
    LegacySneak(bool),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Send {
    pub command: Command,
    pub untracked: bool,
}
impl NoSlow {
    pub fn group_running(&self, running: bool, c: Context, g: usize) -> bool {
        running
            && match g {
                0 => self.settings.uses[0].enabled && (c.blocking || c.tracked_hand.is_some()),
                1 => self.settings.uses[1].enabled && c.using && c.consumable,
                2 => {
                    self.settings.uses[2].enabled
                        && c.using
                        && matches!(c.action, Action::Bow | Action::Trident | Action::Crossbow)
                }
                4 => self.settings.enabled[0] && c.sneaking,
                _ => false,
            }
    }
    pub fn use_multiplier(&self, running: bool, c: Context, original: [f32; 2]) -> [f32; 2] {
        if !running {
            return original;
        }
        if c.action == Action::Spear && !c.blocking {
            return [1., 1.];
        }
        let group = if c.blocking {
            Some(0)
        } else {
            c.action.group()
        };
        let Some(g) = group else {
            return original;
        };
        let s = self.settings.uses[g];
        if !s.enabled
            || self.prevent
            || (g == 0 && self.settings.only_slow_server && c.tracked_hand.is_some())
        {
            original
        } else {
            [s.forward, s.sideways]
        }
    }
    pub fn network(&self, running: bool, c: Context, pre: bool) -> Vec<Send> {
        let mut v = Vec::new();
        let mut send = |command, untracked| v.push(Send { command, untracked });
        for g in 0..3 {
            if !self.group_running(running, c, g) {
                continue;
            }
            let mode = self.settings.uses[g].mode;
            match mode {
                Mode::Grim2360 if pre && c.using => {
                    if c.hand == 0 {
                        send(
                            Command::Use {
                                hand: 1,
                                predict: false,
                            },
                            true,
                        );
                    } else {
                        for slot in [c.selected % 8 + 1, c.selected % 7 + 2, c.selected] {
                            send(Command::Carried(slot), true);
                        }
                    }
                }
                Mode::Grim2364 if pre && c.using => {
                    for slot in [c.selected % 8 + 1, c.selected % 7 + 2, c.selected] {
                        send(Command::Carried(slot), true);
                    }
                }
                Mode::Reuse | Mode::Switch if g == 0 => {
                    if let Some(hand) = c.tracked_hand {
                        let timing = if mode == Mode::Reuse {
                            self.settings.reuse_timing
                        } else {
                            self.settings.switch_timing
                        };
                        let applies =
                            timing == Timing::PreAndPost || pre == (timing == Timing::Pre);
                        if applies {
                            if mode == Mode::Reuse {
                                if pre || timing != Timing::PreAndPost {
                                    send(Command::Release { at_player: false }, true);
                                }
                                if !pre || timing != Timing::PreAndPost {
                                    send(
                                        Command::Use {
                                            hand,
                                            predict: true,
                                        },
                                        true,
                                    );
                                }
                            } else {
                                if pre || timing != Timing::PreAndPost {
                                    send(Command::Carried((c.selected + 1) % 8), true);
                                }
                                if !pre || timing != Timing::PreAndPost {
                                    send(Command::Carried(c.selected), true);
                                    if timing == Timing::PreAndPost {
                                        send(
                                            Command::Use {
                                                hand,
                                                predict: true,
                                            },
                                            true,
                                        );
                                    }
                                }
                            }
                        }
                        // The original Reuse listener has this second, unconditional POST use.
                        if mode == Mode::Reuse && !pre {
                            send(
                                Command::Use {
                                    hand,
                                    predict: true,
                                },
                                true,
                            );
                        }
                    }
                }
                Mode::Interact if g == 0 && !pre => {
                    if let Some(hand) = c.tracked_hand {
                        send(Command::Carried(c.selected), true);
                        send(
                            Command::Use {
                                hand,
                                predict: true,
                            },
                            true,
                        );
                    }
                }
                Mode::Release if g == 1 && pre && c.using && c.moving => {
                    send(Command::Release { at_player: true }, false)
                }
                Mode::Intave14 if g == 1 && pre => {
                    if self.settings.intave_new {
                        if c.ticks <= 2 || c.remaining == 0 {
                            send(Command::Release { at_player: true }, false);
                        }
                    } else {
                        if c.using {
                            send(Command::Release { at_player: true }, false);
                        }
                        if c.ticks == 3 {
                            send(Command::StopUsing, false);
                            send(Command::Release { at_player: true }, false);
                        }
                    }
                }
                _ => {}
            }
        }
        if self.group_running(running, c, 4) {
            let mode = self.settings.sneak_mode;
            let timing = if mode == Mode::Aac5 {
                self.settings.sneak_aac_timing
            } else {
                self.settings.sneak_switch_timing
            };
            let applies = timing == Timing::PreAndPost || pre == (timing == Timing::Pre);
            if applies {
                if mode == Mode::Aac5 {
                    send(Command::LegacySneak(true), false);
                } else if mode == Mode::Switch {
                    if timing == Timing::PreAndPost {
                        send(Command::LegacySneak(pre), false);
                    } else {
                        send(Command::LegacySneak(true), false);
                        send(Command::LegacySneak(false), false);
                    }
                }
            }
        }
        v
    }
    pub fn jump(&self, running: bool, c: Context) -> bool {
        self.group_running(running, c, 1) && self.settings.uses[1].mode == Mode::Jump && c.moving
    }
    /// Stable client-library bridge values; negative values mean vanilla.
    pub fn policy(&self, running: bool, c: Context) -> [f64; 12] {
        let s = self.settings;
        let mut p = [-1.; 12];
        p[0] = f64::from(u8::from(running));
        if !running {
            return p;
        }
        for i in 0..5 {
            if s.enabled[i] {
                p[i + 1] = f64::from(s.multipliers[i]);
            }
        }
        p[6] = f64::from(u8::from(s.enabled[5]));
        p[7] = f64::from(u8::from(s.fluid_collision));
        p[8] = if s.enabled[6] {
            f64::from(self.slowness)
        } else {
            0.15
        };
        let m = self.use_multiplier(true, c, [-1., -1.]);
        p[9] = f64::from(m[0]);
        p[10] = f64::from(m[1]);
        p[11] = f64::from(u8::from(self.jump(true, c)));
        p
    }
}
