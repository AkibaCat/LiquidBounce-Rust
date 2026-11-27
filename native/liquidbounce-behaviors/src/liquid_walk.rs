// SPDX-License-Identifier: GPL-3.0-or-later
//! LiquidWalk's four original event policies and independent packet toggles.
pub const MODES: [&str; 4] = ["Vanilla", "NCP-3.16.1", "Verus-b3901", "Vulcan-2.9.1"];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Vanilla,
    Ncp,
    Verus,
    Vulcan,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        MODES[self as usize]
    }
    pub fn code(self) -> u8 {
        self as u8 + 1
    }
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Vanilla),
            2 => Some(Self::Ncp),
            3 => Some(Self::Verus),
            4 => Some(Self::Vulcan),
            _ => None,
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Vanilla" => Some(Self::Vanilla),
            "NCP-3.16.1" | "NoCheatPlus" => Some(Self::Ncp),
            "Verus-b3901" | "VerusB3901" => Some(Self::Verus),
            "Vulcan-2.9.1" | "Vulcan291" => Some(Self::Vulcan),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub motion: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Vanilla,
            motion: 0.8,
        }
    }
}
/// inside liquid at the body's bottom row, lower .01 row, other collisions,
/// and actual water/bubble-column state, respectively.
pub type Context = [bool; 4];
pub fn shape_allowed(mode: Option<Mode>, shift: bool, fall: f64, fire: bool, inside: bool) -> bool {
    matches!(mode, Some(Mode::Vanilla | Mode::Ncp | Mode::Verus))
        && !shift
        && fall <= 3.
        && !fire
        && !inside
}
pub fn input_jump(mode: Option<Mode>, sneak: bool, inside: bool) -> bool {
    mode == Some(Mode::Vanilla) && !sneak && inside
}
pub fn cancels_jump(mode: Option<Mode>, standing: bool) -> bool {
    mode == Some(Mode::Ncp) && standing
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tick {
    pub velocity_y: Option<f64>,
    pub timer: Option<f32>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Packet {
    pub subtract_y: bool,
    pub ground: Option<bool>,
}
#[derive(Debug)]
pub struct LiquidWalk {
    pub settings: Settings,
    shift_down: bool,
    spoof: bool,
}
impl Default for LiquidWalk {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            shift_down: false,
            spoof: true,
        }
    }
}
impl LiquidWalk {
    pub fn tick(&self, running: bool, shift: bool, context: Context) -> Tick {
        if !running {
            return Tick::default();
        }
        match self.settings.mode {
            Mode::Ncp if context[0] && !shift => Tick {
                velocity_y: Some(0.08),
                timer: None,
            },
            Mode::Vulcan => Tick {
                velocity_y: context[3].then_some(f64::from(self.settings.motion)),
                timer: Some(if context[3] { 1.125 } else { 1. }),
            },
            _ => Tick::default(),
        }
    }
    pub fn packet(
        &mut self,
        running: bool,
        shift: bool,
        in_water: bool,
        context: Context,
    ) -> Packet {
        if !running {
            return Packet::default();
        }
        let allowed = !shift && !in_water && context[1] && !context[2];
        match self.settings.mode {
            Mode::Ncp if allowed => {
                let subtract = self.shift_down;
                self.shift_down = !self.shift_down;
                Packet {
                    subtract_y: subtract,
                    ground: None,
                }
            }
            Mode::Verus => {
                if allowed {
                    let spoof = self.spoof;
                    self.spoof = !self.spoof;
                    Packet {
                        subtract_y: false,
                        ground: Some(spoof),
                    }
                } else {
                    self.spoof = true;
                    Packet::default()
                }
            }
            _ => Packet::default(),
        }
    }
}
