// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoWindCharge, including its ten-tick ground aim grace period.
use crate::{
    no_rotate_set::{FloatRange, RotationsSettings, SmoothMode},
    rotations::Rotation,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub rotate: bool,
    pub horizontal_boost: bool,
    pub pitch: FloatRange,
    pub key: String,
    pub combat_pause: i32,
    pub slot_reset: [i32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            rotate: true,
            horizontal_boost: true,
            pitch: FloatRange(70., 70.),
            key: "key.keyboard.left.control".into(),
            combat_pause: 0,
            slot_reset: [0, 0],
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Context {
    pub fall_flying: bool,
    pub swimming: bool,
    pub in_liquid: bool,
    pub jump_key: bool,
    pub slot: Option<i32>,
    pub cooldown: bool,
    pub on_ground: bool,
    pub boost_key: bool,
    pub yaw: f32,
    /// The player's actual directional input, after movement correction.
    pub keys: [bool; 4],
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Effects {
    pub aim: Option<Rotation>,
    pub throw: Option<i32>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AutoWindCharge {
    pub settings: Settings,
    pub hold_jump: bool,
    pub aim_ticks: i32,
    pub horizontal_pitch: Option<f32>,
}
pub fn movement_direction(yaw: f32, [forward, backward, left, right]: [bool; 4]) -> f32 {
    let mut actual = yaw;
    let multiplier = if backward && !forward {
        actual += 180.;
        -0.5
    } else if forward && !backward {
        0.5
    } else {
        1.
    };
    if left && !right {
        actual -= 90. * multiplier;
    }
    if right && !left {
        actual += 90. * multiplier;
    }
    actual
}
impl AutoWindCharge {
    pub fn disabled(&mut self) {
        self.hold_jump = false;
        self.aim_ticks = 0;
        self.horizontal_pitch = None;
    }
    pub fn movement_input(&self, running: bool, jump: &mut bool) {
        if running && self.hold_jump {
            *jump = false;
        }
    }
    /// Preserve the source's early-return state: losing an item or its cooldown
    /// releases jump but retains aim ticks and a sampled horizontal pitch.
    pub fn begin(
        &mut self,
        c: &Context,
        random: &mut impl FnMut() -> f32,
    ) -> Result<Option<Rotation>, &'static str> {
        self.hold_jump = false;
        if c.fall_flying || c.swimming || c.in_liquid || !c.jump_key {
            self.aim_ticks = 0;
            return Ok(None);
        }
        if c.slot.is_none() || c.cooldown {
            return Ok(None);
        }
        Ok(Some(if !self.settings.horizontal_boost || !c.boost_key {
            Rotation::new(c.yaw, 90.)
        } else {
            let pitch = match self.horizontal_pitch {
                Some(p) => p,
                None => {
                    let p = self.settings.pitch.sample(random)?;
                    self.horizontal_pitch = Some(p);
                    p
                }
            };
            Rotation::new(movement_direction(c.yaw, c.keys) - 180., pitch)
        }))
    }
    pub fn finish(
        &mut self,
        c: &Context,
        rotation: Rotation,
        landing: bool,
        direction_angle: f32,
    ) -> Effects {
        if !c.on_ground {
            return Effects {
                aim: (self.settings.rotate && landing).then_some(rotation),
                throw: None,
            };
        }
        let aim = self.settings.rotate.then_some(rotation);
        if self.settings.rotate && direction_angle > 1. {
            self.hold_jump = self.aim_ticks < 10;
            self.aim_ticks = self.aim_ticks.wrapping_add(1);
            return Effects { aim, throw: None };
        }
        self.aim_ticks = 0;
        self.hold_jump = true;
        self.horizontal_pitch = None;
        Effects { aim, throw: c.slot }
    }
}

/// RotationsValueGroup.calculateTicks starts from actualServerRotation. Factor
/// smoothing uses each range's lower bound without consuming random values.
pub fn calculate_ticks(
    settings: &RotationsSettings,
    current: Rotation,
    target: Rotation,
    last: Option<Rotation>,
    player_last: Rotation,
    random: &mut impl FnMut() -> f32,
) -> Result<i32, &'static str> {
    if settings.active == SmoothMode::Acceleration {
        let delta = current.delta_to(target);
        if delta[0].abs() < 1.0e-5 && delta[1].abs() < 1.0e-5 {
            return Ok(0);
        }
        let previous = last.unwrap_or(player_last).delta_to(current);
        // Evaluate around zero so the returned turn speed is not rounded by
        // adding and subtracting the possibly large absolute yaw/pitch.
        let turn = settings.process(
            SmoothMode::Acceleration,
            Rotation::new(0., 0.),
            Rotation::new(delta[0], delta[1]),
            Some(Rotation::new(-previous[0], -previous[1])),
            player_last,
            random,
        )?;
        if (turn.yaw.abs() < 1.0e-5 && turn.pitch.abs() < 1.0e-5)
            || (delta[0].abs() < turn.yaw.abs() && delta[1].abs() < turn.pitch.abs())
        {
            return Ok(0);
        }
        let h = (delta[0].abs() / turn.yaw.abs()).floor();
        let v = (delta[1].abs() / turn.pitch.abs()).floor();
        if h.is_nan() || v.is_nan() {
            Ok(0)
        } else {
            Ok(h.max(v) as i32)
        }
    } else {
        let mut s = settings.clone();
        for speeds in [&mut s.linear, &mut s.sigmoid.speeds] {
            speeds.horizontal_turn_speed.1 = speeds.horizontal_turn_speed.0;
            speeds.vertical_turn_speed.1 = speeds.vertical_turn_speed.0;
        }
        let mut current = current;
        let mut ticks = -1i32;
        loop {
            current = s.process(s.active, current, target, last, player_last, random)?;
            ticks += 1;
            let d = current.delta_to(target);
            if d[0] * d[0] + d[1] * d[1] <= 4. || ticks >= 80 {
                return Ok(ticks);
            }
        }
    }
}
