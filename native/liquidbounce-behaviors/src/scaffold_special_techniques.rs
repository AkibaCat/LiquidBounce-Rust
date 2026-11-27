// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned GodBridge and Breezily decisions; the native adapter supplies world facts.
pub type Input = [bool; 4];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cardinal {
    South,
    West,
    North,
    East,
}
pub fn cardinal(yaw: f64) -> Cardinal {
    match ((yaw / 90.0 + 0.5).floor() as i32) & 3 {
        0 => Cardinal::South,
        1 => Cardinal::West,
        2 => Cardinal::North,
        _ => Cardinal::East,
    }
}
pub fn cardinal_offset(yaw: f64) -> [i32; 3] {
    match cardinal(yaw) {
        Cardinal::South => [0, 0, 1],
        Cardinal::West => [-1, 0, 0],
        Cardinal::North => [0, 0, -1],
        Cardinal::East => [1, 0, 0],
    }
}
pub fn moving_yaw(movement_angle: f32) -> f32 {
    ((movement_angle + 180.0) / 45.0).round_ties_even() * 45.0
}
pub fn no_input_rotation(target: Option<[f32; 2]>) -> Option<[f32; 2]> {
    target.map(|r| [(r[0] / 90.0).floor() * 90.0 + 45.0, 75.0])
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GodBridge {
    pub on_right_side: bool,
}
impl GodBridge {
    pub fn rotation(
        &mut self,
        moving_yaw: f32,
        on_ground: bool,
        position: [f64; 3],
        cos_sin: [f32; 2],
        leaning_off: bool,
        next_air: bool,
    ) -> [f32; 2] {
        if moving_yaw % 90.0 != 0.0 {
            return [moving_yaw, 75.6];
        }
        if on_ground {
            self.on_right_side = (position[0] + f64::from(cos_sin[0]) * 0.5).floor()
                != position[0].floor()
                || (position[2] + f64::from(cos_sin[1]) * 0.5).floor() != position[2].floor();
            if leaning_off && next_air {
                self.on_right_side = !self.on_right_side;
            }
        }
        [
            moving_yaw + if self.on_right_side { 45.0 } else { -45.0 },
            75.7,
        ]
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Breezily {
    pub last_sideways: f32,
    pub last_air_time: i64,
    pub current_edge_distance: f64,
}
impl Default for Breezily {
    fn default() -> Self {
        Self {
            last_sideways: 0.0,
            last_air_time: 0,
            current_edge_distance: 0.45,
        }
    }
}
impl Breezily {
    pub fn rotation(moving_yaw: f32) -> [f32; 2] {
        [
            moving_yaw,
            if moving_yaw % 90.0 == 0.0 { 80.0 } else { 75.6 },
        ]
    }
    pub fn input(
        &mut self,
        input: Input,
        shift_down: bool,
        below_air: bool,
        now: i64,
        position: [f64; 3],
        yaw: f32,
        mut random_distance: impl FnMut() -> f32,
    ) -> Input {
        if !input[0] || shift_down {
            return input;
        }
        if below_air {
            self.last_air_time = now;
        } else if now.wrapping_sub(self.last_air_time) > 500 {
            return input;
        }
        let x = position[0] - position[0].floor();
        let z = position[2] - position[2].floor();
        let maximum = 1.0 - self.current_edge_distance;
        let mut sideways = 0.0;
        match cardinal(f64::from(yaw)) {
            Cardinal::South => {
                if x > maximum {
                    sideways = 1.0;
                }
                if x < self.current_edge_distance {
                    sideways = -1.0;
                }
            }
            Cardinal::North => {
                if x > maximum {
                    sideways = -1.0;
                }
                if x < self.current_edge_distance {
                    sideways = 1.0;
                }
            }
            Cardinal::East => {
                if z > maximum {
                    sideways = -1.0;
                }
                if z < self.current_edge_distance {
                    sideways = 1.0;
                }
            }
            Cardinal::West => {
                if z > maximum {
                    sideways = 1.0;
                }
                if z < self.current_edge_distance {
                    sideways = -1.0;
                }
            }
        }
        if self.last_sideways != sideways && sideways != 0.0 {
            self.last_sideways = sideways;
            self.current_edge_distance = f64::from(random_distance());
        }
        [
            input[0],
            input[1],
            self.last_sideways == -1.0,
            self.last_sideways == 1.0,
        ]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgeMode {
    Jump,
    Sneak,
    StopInput,
    Backwards,
}
impl LedgeMode {
    pub const ALL: [Self; 4] = [Self::Jump, Self::Sneak, Self::StopInput, Self::Backwards];
    pub fn tag(self) -> &'static str {
        match self {
            Self::Jump => "Jump",
            Self::Sneak => "Sneak",
            Self::StopInput => "StopInput",
            Self::Backwards => "Backwards",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LedgeAction {
    pub jump: bool,
    pub sneak_time: i32,
    pub stop_input: bool,
    pub step_back: bool,
}
pub fn jump_apex(jump_power: f32) -> f64 {
    let mut motion = f64::from(jump_power);
    let mut height = 0.0;
    while motion > 0.0 {
        height += motion;
        motion = (motion - 0.08) * f64::from(0.98f32);
    }
    height
}
pub fn action(mode: LedgeMode, mut sneak_time: impl FnMut() -> i32) -> LedgeAction {
    match mode {
        LedgeMode::Jump => LedgeAction {
            jump: true,
            ..Default::default()
        },
        LedgeMode::Sneak => LedgeAction {
            sneak_time: sneak_time(),
            ..Default::default()
        },
        LedgeMode::StopInput => LedgeAction {
            stop_input: true,
            ..Default::default()
        },
        LedgeMode::Backwards => LedgeAction {
            step_back: true,
            ..Default::default()
        },
    }
}
/// Selection consumes randomness only on the source branches that choose a mode.
pub fn ledge_mode(
    block_count: i32,
    force_sneak_below: i32,
    modes: &[LedgeMode],
    jump_power: f32,
    mut choose: impl FnMut(usize) -> usize,
) -> LedgeMode {
    let mode = if block_count < force_sneak_below {
        LedgeMode::Sneak
    } else {
        modes[choose(modes.len())]
    };
    if mode == LedgeMode::Jump && jump_apex(jump_power) >= 2.0 {
        let remaining = modes
            .iter()
            .copied()
            .filter(|m| *m != LedgeMode::Jump)
            .collect::<Vec<_>>();
        if remaining.is_empty() {
            LedgeMode::Sneak
        } else {
            remaining[choose(remaining.len())]
        }
    } else {
        mode
    }
}
