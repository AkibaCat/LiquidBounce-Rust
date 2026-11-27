// SPDX-License-Identifier: GPL-3.0-or-later
//! Entity-aware Acceleration and source calculateTicks. The common settings
//! codec remains shared with every combat module.
use crate::{
    combat_rotations::{Correction, Settings, SmoothMode},
    no_rotate_set::{AccelerationSettings, FloatRange, SmoothMode as Basic},
    rotations::{Rotation, wrap_degrees},
};
fn length(d: [f32; 2]) -> f32 {
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}
fn coerce(v: f32, lo: f32, hi: f32) -> Result<f32, &'static str> {
    if lo > hi {
        return Err("cannot coerce into an empty range");
    }
    Ok(if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    })
}
fn int(r: [i32; 2], random: &mut impl FnMut() -> f32) -> Result<i32, &'static str> {
    if r[0] > r[1] {
        return Err("empty integer range");
    }
    if r[0] == r[1] {
        return Ok(r[0]);
    }
    let n = i64::from(r[1]) - i64::from(r[0]) + 1;
    Ok(
        (i64::from(r[0]) + (f64::from(random()) * n as f64).floor().min((n - 1) as f64) as i64)
            as i32,
    )
}
pub fn acceleration(
    s: &AccelerationSettings,
    from: Rotation,
    to: Rotation,
    last: Rotation,
    crosshair: bool,
    distance: f64,
    random: &mut impl FnMut() -> f32,
) -> Result<Rotation, &'static str> {
    let previous = last.delta_to(from);
    let diff = from.delta_to(to);
    let exponent = -s.steepness * (length(diff) / 120. - s.midpoint);
    let decel = coerce((1. / (1. + f64::from(exponent).exp())) as f32, 0., 180.)?;
    let decel = if s.sigmoid_deceleration { decel } else { 1. };
    let distance = (f64::from(s.coef_distance) * distance) as f32;
    let (yaw, pitch) = if s.dynamic_accel && crosshair {
        (s.yaw_crosshair_accel, s.pitch_crosshair_accel)
    } else {
        (s.yaw_acceleration, s.pitch_acceleration)
    };
    let y = [
        -yaw.sample(random)? + distance,
        yaw.sample(random)? + distance,
    ];
    let p = [
        -pitch.sample(random)? + distance,
        pitch.sample(random)? + distance,
    ];
    if y.iter().chain(&p).any(|f| f.is_nan()) {
        return Err("cannot coerce into an empty acceleration range");
    }
    let ya = coerce(wrap_degrees(diff[0] - previous[0]), y[0], y[1])? * decel;
    let pa = coerce(wrap_degrees(diff[1] - previous[1]), p[0], p[1])? * decel;
    let mut error = |a: f32, ae: f32, ce: f32| -> Result<f32, &'static str> {
        let ae = if s.acceleration_error { ae } else { 0. };
        let ce = if s.constant_error { ce } else { 0. };
        Ok(a * FloatRange(-ae, ae).sample(random)? + FloatRange(-ce, ce).sample(random)?)
    };
    let ye = error(ya, s.yaw_accel_error, s.yaw_constant_error)?;
    let pe = error(pa, s.pitch_accel_error, s.pitch_constant_error)?;
    Ok(Rotation::new(
        from.yaw + (previous[0] + ya + ye),
        from.pitch + (previous[1] + pa + pe),
    ))
}
pub fn ticks(
    s: &Settings,
    from: Rotation,
    to: Rotation,
    last: Rotation,
    random: &mut impl FnMut() -> f32,
) -> Result<i32, &'static str> {
    if s.active == SmoothMode::Acceleration {
        let diff = from.delta_to(to);
        if diff.iter().all(|v| v.abs() < 1e-5) {
            return Ok(0);
        }
        let next = acceleration(&s.basic.acceleration, from, to, last, false, 0., random)?;
        let d = from.delta_to(next);
        if d.iter().all(|v| v.abs() < 1e-5)
            || diff[0].abs() < d[0].abs() && diff[1].abs() < d[1].abs()
        {
            return Ok(0);
        }
        let h = (diff[0].abs() / d[0].abs()).floor();
        let v = (diff[1].abs() / d[1].abs()).floor();
        return Ok(if h.is_nan() || v.is_nan() {
            0
        } else {
            h.max(v) as i32
        });
    }
    let mode = if s.active == SmoothMode::Ai {
        match s.ai.correction {
            Correction::None => return Ok(0),
            Correction::Linear => SmoothMode::Linear,
            Correction::Interpolation => SmoothMode::Interpolation,
        }
    } else {
        s.active
    };
    let mut base = s.basic.clone();
    let interpolation = if s.active == SmoothMode::Ai {
        &s.ai.interpolation
    } else {
        &s.interpolation
    };
    if s.active == SmoothMode::Ai && mode == SmoothMode::Linear {
        base.linear.horizontal_turn_speed = s.ai.linear[0];
        base.linear.vertical_turn_speed = s.ai.linear[1];
    }
    base.linear.horizontal_turn_speed.1 = base.linear.horizontal_turn_speed.0;
    base.linear.vertical_turn_speed.1 = base.linear.vertical_turn_speed.0;
    base.sigmoid.speeds.horizontal_turn_speed.1 = base.sigmoid.speeds.horizontal_turn_speed.0;
    base.sigmoid.speeds.vertical_turn_speed.1 = base.sigmoid.speeds.vertical_turn_speed.0;
    let mut i = interpolation.clone();
    i.horizontal[1] = i.horizontal[0];
    i.vertical[1] = i.vertical[0];
    let mut current = from;
    let mut n = -1;
    loop {
        current = match mode {
            SmoothMode::Interpolation => i.process(current, to, None, &mut || 0.)?,
            SmoothMode::Sigmoid => {
                base.process(Basic::Sigmoid, current, to, None, last, &mut || 0.)?
            }
            _ => base.process(Basic::Linear, current, to, None, last, &mut || 0.)?,
        };
        n += 1;
        if length(current.delta_to(to)) <= 2. || n >= 80 {
            return Ok(n);
        }
    }
}

impl Settings {
    pub fn kill_aura_process(
        &mut self,
        id: u64,
        module_running: bool,
        from: Rotation,
        to: Rotation,
        last: Option<Rotation>,
        player_last: Rotation,
        previous_target: Option<Rotation>,
        server: Rotation,
        entity: Option<(bool, f64)>,
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
                let (crosshair, distance) = entity.unwrap_or((false, 0.));
                acceleration(
                    &self.basic.acceleration,
                    from,
                    to,
                    last.unwrap_or(player_last),
                    crosshair,
                    distance,
                    random,
                )?
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
