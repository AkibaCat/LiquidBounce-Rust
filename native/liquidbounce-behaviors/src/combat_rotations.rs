// SPDX-License-Identifier: GPL-3.0-or-later
//! Combat RotationsValueGroup at LiquidBounce ae0eb396. ProjectilePuncher
//! passes entity=null; the shared noncombat acceleration math is exact here.
use crate::{
    no_rotate_set::{FloatRange, RotationsSettings, SmoothMode as Basic},
    rotations::Rotation,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SmoothMode {
    #[default]
    Linear,
    Sigmoid,
    Interpolation,
    Acceleration,
    Ai,
}
impl SmoothMode {
    pub const ALL: [Self; 5] = [
        Self::Linear,
        Self::Sigmoid,
        Self::Interpolation,
        Self::Acceleration,
        Self::Ai,
    ];
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Sigmoid => "Sigmoid",
            Self::Interpolation => "Interpolation",
            Self::Acceleration => "Acceleration",
            Self::Ai => "AI",
        }
    }
    fn id(self) -> u64 {
        match self {
            Self::Linear => 1,
            Self::Sigmoid => 2,
            Self::Interpolation => 3,
            Self::Acceleration => 4,
            Self::Ai => 5,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Interpolation {
    pub horizontal: [i32; 2],
    pub vertical: [i32; 2],
    pub direction_change: [i32; 2],
    pub midpoint: f32,
}
impl Default for Interpolation {
    fn default() -> Self {
        Self {
            horizontal: [80, 85],
            vertical: [20, 25],
            direction_change: [95, 100],
            midpoint: 0.35,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Correction {
    #[default]
    Interpolation,
    Linear,
    None,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Ai {
    pub model: String,
    pub correction: Correction,
    pub interpolation: Interpolation,
    pub linear: [FloatRange; 2],
    pub yaw_multiplier: f32,
    pub pitch_multiplier: f32,
}
impl Default for Ai {
    fn default() -> Self {
        Self {
            model: "21KC11KP".into(),
            correction: Default::default(),
            interpolation: Interpolation {
                horizontal: [2, 5],
                vertical: [2, 5],
                ..Default::default()
            },
            linear: [FloatRange(5., 5.); 2],
            yaw_multiplier: 1.5,
            pitch_multiplier: 1.,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ShortStop {
    pub enabled: bool,
    pub rate: i32,
    pub duration: [i32; 2],
    pub elapsed: i32,
    pub current_duration: i32,
}
impl Default for ShortStop {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: 3,
            duration: [1, 2],
            elapsed: 0,
            current_duration: crate::random::random_i32_inclusive(1, 2),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Fail {
    pub enabled: bool,
    pub rate: i32,
    pub factor: f32,
    pub horizontal: FloatRange,
    pub vertical: FloatRange,
    pub duration: [i32; 2],
    pub elapsed: i32,
    pub current_duration: i32,
    pub shift: Rotation,
}
impl Default for Fail {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: 3,
            factor: 0.04,
            horizontal: FloatRange(5., 10.),
            vertical: FloatRange(0., 2.),
            duration: [1, 4],
            elapsed: 0,
            current_duration: crate::random::random_i32_inclusive(1, 4),
            shift: Rotation::ZERO,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub active: SmoothMode,
    pub basic: RotationsSettings,
    pub interpolation: Interpolation,
    pub ai: Ai,
    pub short_stop: ShortStop,
    pub fail: Fail,
}

fn int(r: [i32; 2], _random: &mut impl FnMut() -> f32) -> Result<i32, &'static str> {
    if r[0] > r[1] {
        return Err("empty combat rotation integer range");
    }
    Ok(crate::random::random_i32_inclusive(r[0], r[1]))
}
impl Interpolation {
    pub fn process(
        &self,
        from: Rotation,
        to: Rotation,
        previous_target: Option<Rotation>,
        random: &mut impl FnMut() -> f32,
    ) -> Result<Rotation, &'static str> {
        let change = if let Some(previous) = previous_target {
            (f64::from(previous.delta_length_squared_to(to)).sqrt() as f32 / 180.).clamp(0., 1.)
                * (int(self.direction_change, random)? as f32 / 100.)
        } else {
            0.
        };
        let h = (int(self.horizontal, random)? as f32 / 100.).clamp(0., 1.);
        let v = (int(self.vertical, random)? as f32 / 100.).clamp(0., 1.);
        let [yaw, pitch] = from.delta_to(to);
        let factor = |diff: f32, speed: f32| {
            let t = (diff.abs() / 180.).clamp(0., 1.);
            if t > self.midpoint {
                let t = 1. - t;
                ((1. - t) * (1. - t) * 0.05 + 2. * (1. - t) * t + t * t) * speed
            } else {
                1. / (1. + (-0.5 * (t - 0.3)).exp()) * (speed + change).clamp(0., 1.)
            }
        };
        Ok(from.towards_linear(
            to,
            factor(yaw, h) * yaw.abs(),
            factor(pitch, v) * pitch.abs(),
        ))
    }
}
impl Settings {
    /// Active smoothing and optional processors are captured by RotationTarget.
    /// Their values and running guards remain live throughout resetting.
    pub fn processor_id(&self) -> u64 {
        self.active.id()
            | if self.fail.enabled { 8 } else { 0 }
            | if self.short_stop.enabled { 16 } else { 0 }
    }
    pub fn decode(id: u64) -> Option<(SmoothMode, bool, bool)> {
        if id > 31 {
            return None;
        }
        Some((
            match id & 7 {
                1 => SmoothMode::Linear,
                2 => SmoothMode::Sigmoid,
                3 => SmoothMode::Interpolation,
                4 => SmoothMode::Acceleration,
                5 => SmoothMode::Ai,
                _ => return None,
            },
            id & 8 != 0,
            id & 16 != 0,
        ))
    }
    pub fn first_tick(
        &mut self,
        module_running: bool,
        random: &mut impl FnMut() -> f32,
    ) -> Result<(), &'static str> {
        let f = &mut self.fail;
        if !module_running || !f.enabled {
            return Ok(());
        }
        if f.rate as f32 > FloatRange(0., 100.).sample(random)? {
            f.current_duration = int(f.duration, random)?;
            let yaw_positive = crate::random::with_thread_random(|r| r.next_bool());
            let yaw = f.horizontal.sample(random)?;
            let pitch_positive = crate::random::with_thread_random(|r| r.next_bool());
            let pitch = f.vertical.sample(random)?;
            f.shift = Rotation::new(
                if yaw_positive { yaw } else { -yaw },
                if pitch_positive { pitch } else { -pitch },
            );
            f.elapsed = 0;
        } else {
            f.elapsed = f.elapsed.wrapping_add(1);
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        id: u64,
        module_running: bool,
        from: Rotation,
        to: Rotation,
        last: Option<Rotation>,
        player_last: Rotation,
        previous_target: Option<Rotation>,
        server: Rotation,
        random: &mut impl FnMut() -> f32,
    ) -> Result<Rotation, &'static str> {
        let (mode, fail, short) = Self::decode(id).ok_or("unknown combat rotation processor")?;
        let mut result = match mode {
            SmoothMode::Linear => {
                self.basic
                    .process(Basic::Linear, from, to, last, player_last, random)?
            }
            SmoothMode::Sigmoid => {
                self.basic
                    .process(Basic::Sigmoid, from, to, last, player_last, random)?
            }
            SmoothMode::Acceleration => {
                self.basic
                    .process(Basic::Acceleration, from, to, last, player_last, random)?
            }
            // DeepLearningEngine is unavailable in this native runtime, exactly the
            // source's unsupported-engine branch. AI keeps all imported settings.
            SmoothMode::Interpolation | SmoothMode::Ai => {
                self.interpolation
                    .process(from, to, previous_target, random)?
            }
        };
        let f = &self.fail;
        if fail
            && module_running
            && f.enabled
            && f.elapsed < f.current_duration
            && let Some(last) = last
        {
            result = Rotation::new(
                result.yaw + (last.yaw - server.yaw) * f.factor + f.shift.yaw,
                result.pitch + (last.pitch - server.pitch) * f.factor + f.shift.pitch,
            );
        }
        let s = &mut self.short_stop;
        if short && module_running && s.enabled {
            let triggers = match s.rate {
                0 => false,
                100 => true,
                _ => random() * 100. < s.rate as f32,
            };
            if triggers {
                s.current_duration = int(s.duration, random)?;
                s.elapsed = 0;
            }
            if s.elapsed < s.current_duration {
                s.elapsed = s.elapsed.wrapping_add(1);
                result = from.towards_linear(
                    result,
                    FloatRange(0., 0.1).sample(random)?,
                    FloatRange(0., 0.1).sample(random)?,
                );
            }
        }
        Ok(result)
    }
}
