// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleNoRotateSet and noncombat RotationsValueGroup at LiquidBounce ae0eb396.
//! Settings stay live on each mode object after a target captures that mode.

use crate::rotations::{DefaultRotationTarget, MovementCorrection, Rotation, wrap_degrees};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoRotateMode {
    #[default]
    SilentAccept,
    ResetRotation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SmoothMode {
    #[default]
    Linear,
    Sigmoid,
    Acceleration,
}

impl SmoothMode {
    /// RotationManager processor 0 remains the default Sprint processor.
    pub const fn processor_id(self) -> u64 {
        match self {
            Self::Linear => 1,
            Self::Sigmoid => 2,
            Self::Acceleration => 3,
        }
    }
    pub const fn from_processor_id(id: u64) -> Option<Self> {
        match id {
            1 => Some(Self::Linear),
            2 => Some(Self::Sigmoid),
            3 => Some(Self::Acceleration),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatRange(pub f32, pub f32);

impl FloatRange {
    /// ArrayExtensions.random: reversed/equal ranges return start without a
    /// random draw; the nontrivial case uses nextFloat(start, end.nextUp()).
    pub fn sample(self, random: &mut impl FnMut() -> f32) -> Result<f32, &'static str> {
        if self.0 >= self.1 {
            return Ok(self.0);
        }
        let bound = self.1.next_up();
        if !self.0.is_finite() || !bound.is_finite() || self.0 >= bound {
            return Err("random float range must be finite and nonempty");
        }
        Ok(crate::random::bounded_f32(random(), self.0, bound))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearSettings {
    pub horizontal_turn_speed: FloatRange,
    pub vertical_turn_speed: FloatRange,
}
impl Default for LinearSettings {
    fn default() -> Self {
        Self {
            horizontal_turn_speed: FloatRange(180.0, 180.0),
            vertical_turn_speed: FloatRange(180.0, 180.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SigmoidSettings {
    pub speeds: LinearSettings,
    pub steepness: f32,
    pub midpoint: f32,
}
impl Default for SigmoidSettings {
    fn default() -> Self {
        Self {
            speeds: Default::default(),
            steepness: 10.0,
            midpoint: 0.3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccelerationSettings {
    pub yaw_acceleration: FloatRange,
    pub pitch_acceleration: FloatRange,
    pub dynamic_accel: bool,
    pub coef_distance: f32,
    pub yaw_crosshair_accel: FloatRange,
    pub pitch_crosshair_accel: FloatRange,
    pub acceleration_error: bool,
    pub yaw_accel_error: f32,
    pub pitch_accel_error: f32,
    pub constant_error: bool,
    pub yaw_constant_error: f32,
    pub pitch_constant_error: f32,
    pub sigmoid_deceleration: bool,
    pub steepness: f32,
    pub midpoint: f32,
}
impl Default for AccelerationSettings {
    fn default() -> Self {
        Self {
            yaw_acceleration: FloatRange(20.0, 25.0),
            pitch_acceleration: FloatRange(20.0, 25.0),
            dynamic_accel: false,
            coef_distance: -1.393,
            yaw_crosshair_accel: FloatRange(17.0, 20.0),
            pitch_crosshair_accel: FloatRange(17.0, 20.0),
            acceleration_error: true,
            yaw_accel_error: 0.1,
            pitch_accel_error: 0.1,
            constant_error: true,
            yaw_constant_error: 0.1,
            pitch_constant_error: 0.1,
            sigmoid_deceleration: false,
            steepness: 10.0,
            midpoint: 0.3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RotationsSettings {
    pub active: SmoothMode,
    pub linear: LinearSettings,
    pub sigmoid: SigmoidSettings,
    pub acceleration: AccelerationSettings,
    pub movement_correction: MovementCorrection,
    pub reset_threshold: f32,
    pub ticks_until_reset: i32,
}
impl Default for RotationsSettings {
    fn default() -> Self {
        Self {
            active: Default::default(),
            linear: Default::default(),
            sigmoid: Default::default(),
            acceleration: Default::default(),
            movement_correction: MovementCorrection::Silent,
            reset_threshold: 2.0,
            ticks_until_reset: 5,
        }
    }
}

/// The target captures a processor identity, while the processor's settings
/// are read live when processing or resetting. Switching the selected mode
/// does not replace a processor already referenced by an existing target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoRotateTarget {
    pub target: DefaultRotationTarget,
    pub processor: SmoothMode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Correction {
    pub restored: Rotation,
    pub target: Option<NoRotateTarget>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NoRotateSet {
    pub mode: NoRotateMode,
    pub rotations: RotationsSettings,
}

impl NoRotateSet {
    /// Invoke after the actual accepted vanilla position/rotation handler,
    /// including its acknowledgement packets. Loading screens bypass it.
    pub fn after_correction(
        &self,
        running: bool,
        loading_screen: bool,
        previous: Rotation,
        corrected: Rotation,
        allowed_to_update: bool,
    ) -> Option<Correction> {
        if !running || loading_screen {
            return None;
        }
        let target = if self.mode == NoRotateMode::ResetRotation && allowed_to_update {
            Some(NoRotateTarget {
                target: DefaultRotationTarget {
                    rotation: Rotation {
                        is_normalized: true,
                        ..corrected
                    },
                    // These fixed fields serve only the original Sprint path;
                    // NoRotateSet processing uses this captured processor instead.
                    horizontal_turn_speed: 180.0,
                    vertical_turn_speed: 180.0,
                    ticks_until_reset: self.rotations.ticks_until_reset,
                    reset_threshold: self.rotations.reset_threshold,
                    consider_inventory: true,
                    movement_correction: self.rotations.movement_correction,
                },
                processor: self.rotations.active,
            })
        } else {
            None
        };
        Some(Correction {
            restored: Rotation {
                yaw: previous.yaw + 0.000001f32,
                pitch: previous.pitch + 0.000001f32,
                is_normalized: true,
            },
            target,
        })
    }
}

fn length(delta: [f32; 2]) -> f32 {
    // Pinned RotationDelta.length -> Mth.length(float,float).
    f64::from(delta[0] * delta[0] + delta[1] * delta[1]).sqrt() as f32
}

fn sigmoid(difference: f32, steepness: f32, midpoint: f32) -> f32 {
    let scaled = difference / 120.0;
    let exponent = -steepness * (scaled - midpoint);
    (1.0f64 / (1.0 + f64::from(exponent).exp())) as f32
}

fn coerce(value: f32, first: f32, last: f32) -> Result<f32, &'static str> {
    if first > last {
        return Err("cannot coerce into an empty acceleration range");
    }
    // Float comparisons retain NaN, unlike Rust's panic-on-NaN clamp bounds.
    Ok(if value < first {
        first
    } else if value > last {
        last
    } else {
        value
    })
}

impl RotationsSettings {
    /// NoRotateSet's RotationTarget always has entity=null, including while
    /// resetting. DynamicAccel's crosshair branch is therefore never selected;
    /// CoefDistance is still multiplied by distance=0, retaining NaN behavior.
    pub fn process(
        &self,
        processor: SmoothMode,
        current: Rotation,
        target: Rotation,
        last_managed: Option<Rotation>,
        player_last: Rotation,
        random: &mut impl FnMut() -> f32,
    ) -> Result<Rotation, &'static str> {
        match processor {
            SmoothMode::Linear | SmoothMode::Sigmoid => {
                let speeds = if processor == SmoothMode::Linear {
                    self.linear
                } else {
                    self.sigmoid.speeds
                };
                let horizontal = speeds.horizontal_turn_speed.sample(random)?;
                let vertical = speeds.vertical_turn_speed.sample(random)?;
                let (horizontal, vertical) = if processor == SmoothMode::Sigmoid {
                    let difference = length(current.delta_to(target));
                    let difference = if difference > 180.0 {
                        180.0
                    } else {
                        difference
                    };
                    // SigmoidAngleSmooth computes sigmoid as Double, multiplies
                    // by turnSpeed before converting to Float. Recompute that
                    // expression instead of multiplying this rounded factor.
                    let scaled = difference / 120.0;
                    let exponent = -self.sigmoid.steepness * (scaled - self.sigmoid.midpoint);
                    let factor64 = 1.0 / (1.0 + f64::from(exponent).exp());
                    (
                        coerce((factor64 * f64::from(horizontal)) as f32, 0.0, 180.0)?,
                        coerce((factor64 * f64::from(vertical)) as f32, 0.0, 180.0)?,
                    )
                } else {
                    (horizontal, vertical)
                };
                // Kotlin Float.coerceIn(min,max) rejects negative finite
                // factors when they invert the computed component limits.
                let delta = current.delta_to(target);
                let size = length(delta);
                let yaw_limit = (delta[0] / size).abs() * horizontal;
                let pitch_limit = (delta[1] / size).abs() * vertical;
                Ok(Rotation::new(
                    current.yaw + coerce(delta[0], -yaw_limit, yaw_limit)?,
                    current.pitch + coerce(delta[1], -pitch_limit, pitch_limit)?,
                ))
            }
            SmoothMode::Acceleration => {
                let settings = self.acceleration;
                let previous = last_managed.unwrap_or(player_last).delta_to(current);
                let delta = current.delta_to(target);
                // The source computes this even while the group is disabled.
                let deceleration = coerce(
                    sigmoid(length(delta), settings.steepness, settings.midpoint),
                    0.0,
                    180.0,
                )?;
                let deceleration = if settings.sigmoid_deceleration {
                    deceleration
                } else {
                    1.0
                };
                let distance_factor = (f64::from(settings.coef_distance) * 0.0) as f32;
                // Four draws precede both error providers, even for ranges
                // whose lower and upper bound are independently sampled.
                let yaw_range = [
                    -settings.yaw_acceleration.sample(random)? + distance_factor,
                    settings.yaw_acceleration.sample(random)? + distance_factor,
                ];
                let pitch_range = [
                    -settings.pitch_acceleration.sample(random)? + distance_factor,
                    settings.pitch_acceleration.sample(random)? + distance_factor,
                ];
                // This source overload takes ClosedFloatingPointRange; its
                // isEmpty() treats NaN bounds as empty, unlike primitive
                // coerceIn(min,max), which compares only min > max.
                if yaw_range
                    .iter()
                    .chain(&pitch_range)
                    .any(|bound| bound.is_nan())
                {
                    return Err("cannot coerce into an empty acceleration range");
                }
                let yaw_accel = coerce(
                    wrap_degrees(delta[0] - previous[0]),
                    yaw_range[0],
                    yaw_range[1],
                )? * deceleration;
                let pitch_accel = coerce(
                    wrap_degrees(delta[1] - previous[1]),
                    pitch_range[0],
                    pitch_range[1],
                )? * deceleration;
                let mut error = |acceleration: f32,
                                 acceleration_error: f32,
                                 constant_error: f32|
                 -> Result<f32, &'static str> {
                    let acceleration_error = if settings.acceleration_error {
                        acceleration_error
                    } else {
                        0.0
                    };
                    let constant_error = if settings.constant_error {
                        constant_error
                    } else {
                        0.0
                    };
                    let acceleration_error =
                        FloatRange(-acceleration_error, acceleration_error).sample(random)?;
                    let constant_error =
                        FloatRange(-constant_error, constant_error).sample(random)?;
                    Ok(acceleration * acceleration_error + constant_error)
                };
                let yaw_error = error(
                    yaw_accel,
                    settings.yaw_accel_error,
                    settings.yaw_constant_error,
                )?;
                let pitch_error = error(
                    pitch_accel,
                    settings.pitch_accel_error,
                    settings.pitch_constant_error,
                )?;
                Ok(Rotation::new(
                    current.yaw + (previous[0] + yaw_accel + yaw_error),
                    current.pitch + (previous[1] + pitch_accel + pitch_error),
                ))
            }
        }
    }
}
