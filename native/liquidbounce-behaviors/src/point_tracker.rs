// SPDX-License-Identifier: GPL-3.0-or-later
//! PointTracker/AimPoint, including persistent Delay, Lazy and Gaussian state.
use crate::auto_bow_math::{self as math, B, V};

pub trait Samples {
    fn float(&mut self, range: [f32; 2]) -> f32;
    fn int(&mut self, range: [i32; 2]) -> i32;
    fn gaussian(&mut self) -> f64;
    fn chance(&mut self, percent: f32) -> bool;
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub exempt_parts: [bool; 3],
    pub exempt_best: bool,
    pub vertical: f32,
    pub horizontal: f32,
    pub gaussian: bool,
    pub yaw_offset: [f32; 2],
    pub pitch_offset: [f32; 2],
    pub chance: f32,
    pub speed: [f32; 2],
    pub tolerance: f32,
    pub dynamic: bool,
    pub dynamic_hurt: i32,
    pub dynamic_yaw: f32,
    pub dynamic_pitch: f32,
    pub dynamic_speed: [f32; 2],
    pub dynamic_tolerance: f32,
    pub lazy: bool,
    pub threshold: [f32; 2],
    pub delay: bool,
    pub delay_ticks: [i32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            exempt_parts: [false; 3],
            exempt_best: false,
            vertical: 0.2,
            horizontal: 0.1,
            gaussian: false,
            yaw_offset: [0., 0.],
            pitch_offset: [0., 0.],
            chance: 100.,
            speed: [0.1, 0.2],
            tolerance: 0.05,
            dynamic: false,
            dynamic_hurt: 10,
            dynamic_yaw: 0.,
            dynamic_pitch: 0.,
            dynamic_speed: [0.5, 0.75],
            dynamic_tolerance: 0.1,
            lazy: false,
            threshold: [0.1, 0.2],
            delay: false,
            delay_ticks: [2, 4],
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub pos: V,
    pub bounds: B,
}
impl PartialEq for Point {
    fn eq(&self, b: &Self) -> bool {
        let equal = |a: V, b: V| {
            a.array().into_iter().zip(b.array()).all(|(a, b)| {
                let bits = |v: f64| {
                    if v.is_nan() {
                        f64::NAN.to_bits()
                    } else {
                        v.to_bits()
                    }
                };
                bits(a) == bits(b)
            })
        };
        equal(self.pos, b.pos)
            && equal(self.bounds.lo, b.bounds.lo)
            && equal(self.bounds.hi, b.bounds.hi)
    }
}
impl Point {
    pub fn new(pos: V, bounds: B) -> Self {
        Self {
            pos: V(
                pos.0.clamp(bounds.lo.0, bounds.hi.0),
                pos.1.clamp(bounds.lo.1, bounds.hi.1),
                pos.2.clamp(bounds.lo.2, bounds.hi.2),
            ),
            bounds,
        }
    }
    pub fn offset(self, offset: V) -> Self {
        Self::new(self.pos.add(offset), self.bounds.moved(offset))
    }
}
#[derive(Clone, Debug, Default)]
pub struct PointTracker {
    pub settings: Settings,
    pub current_delay: Option<i32>,
    pub current_threshold: Option<f32>,
    pub delay_point: Option<Point>,
    pub lazy_point: Option<Point>,
    pub current_offset: V,
    pub target_offset: V,
}
impl PointTracker {
    pub fn initialize(&mut self, random: &mut impl Samples) {
        // Gaussian, Lazy and Delay are constructed in this order upstream.
        if self.current_threshold.is_none() {
            self.current_threshold = Some(random.float(self.settings.threshold));
        }
        if self.current_delay.is_none() {
            self.current_delay = Some(random.int(self.settings.delay_ticks));
        }
    }
    pub fn threshold_changed(&mut self, random: &mut impl Samples) {
        self.current_threshold = Some(random.float(self.settings.threshold));
    }
    pub fn delay_changed(&mut self, random: &mut impl Samples) {
        self.current_delay = Some(random.int(self.settings.delay_ticks));
    }
    pub fn find(&mut self, eyes: V, bounds: B, random: &mut impl Samples) -> Point {
        self.initialize(random);
        let points = math::projected_points(eyes, bounds);
        let nearest = |a: &&V, b: &&V| {
            a.sub(eyes)
                .len2()
                .partial_cmp(&b.sub(eyes).len2())
                .unwrap_or(std::cmp::Ordering::Equal)
        };
        let best = points
            .iter()
            .min_by(nearest)
            .copied()
            .unwrap_or_else(|| Point::new(eyes, bounds).pos);
        let s = &self.settings;
        let length = (bounds.hi.1 - bounds.lo.1) / 3.;
        let pos = points
            .iter()
            .filter(|p| {
                let part = (s.exempt_parts[0] && p.1 <= bounds.hi.1 && p.1 > bounds.hi.1 - length)
                    || (s.exempt_parts[1]
                        && p.1 <= bounds.hi.1 - length
                        && p.1 >= bounds.lo.1 + length)
                    || (s.exempt_parts[2] && p.1 >= bounds.lo.1 && p.1 < bounds.lo.1 + length);
                let d = p.sub(best);
                let near_best = s.exempt_best
                    && d.0 * d.0 + d.2 * d.2 < f64::from(s.horizontal).powi(2)
                    && d.1.abs() < f64::from(s.vertical);
                !part && !near_best
            })
            .min_by(nearest)
            .copied()
            .unwrap_or(best);
        self.process(Point::new(pos, bounds), random)
    }
    pub fn process(&mut self, mut point: Point, random: &mut impl Samples) -> Point {
        self.initialize(random);
        if self.settings.delay {
            if self.delay_point != Some(point) {
                if let Some(old) = self.delay_point {
                    let delay = self.current_delay.as_mut().unwrap();
                    *delay = delay.wrapping_sub(1);
                    if *delay <= 0 {
                        self.delay_point = Some(point);
                        self.current_delay = Some(random.int(self.settings.delay_ticks));
                    }
                    // The source deliberately returns the old local variable on expiry too.
                    point = old;
                } else {
                    self.delay_point = Some(point);
                }
            }
        }
        if self.settings.lazy {
            if let Some(old) = self.lazy_point {
                let threshold = self.current_threshold.unwrap();
                if point.pos.sub(old.pos).len2() < f64::from(threshold * threshold) {
                    point = old;
                } else {
                    self.lazy_point = Some(point);
                    self.current_threshold = Some(random.float(self.settings.threshold));
                }
            } else {
                self.lazy_point = Some(point);
            }
        }
        if self.settings.gaussian {
            if random.float(self.settings.yaw_offset) > 0.
                && random.float(self.settings.pitch_offset) > 0.
                && self.settings.chance > 0.
            {
                // process() passes a PointInsideBox (not a LivingEntity), so Dynamic's
                // entity check is false in the original. Its settings remain persistent.
                self.update_gaussian(random, None);
            }
            point = point.offset(self.current_offset);
        }
        point
    }
    /// The general source helper also accepts a living entity; PointTracker does not.
    pub fn update_gaussian(&mut self, random: &mut impl Samples, entity: Option<(i32, f64)>) {
        let s = &self.settings;
        let dynamic = s.dynamic && entity.is_some_and(|(hurt, _)| hurt >= s.dynamic_hurt);
        let speed = entity.map_or(0., |(_, speed)| speed);
        let yaw = f64::from(random.float(s.yaw_offset))
            + if dynamic && s.dynamic_yaw > 0. {
                speed * f64::from(s.dynamic_yaw)
            } else {
                0.
            };
        let pitch = f64::from(random.float(s.pitch_offset))
            + if dynamic && s.dynamic_pitch > 0. {
                speed * f64::from(s.dynamic_pitch)
            } else {
                0.
            };
        let d = self.current_offset.sub(self.target_offset);
        let tolerance = f64::from(if dynamic {
            s.dynamic_tolerance
        } else {
            s.tolerance
        });
        if d.0.abs() < tolerance && d.1.abs() < tolerance && d.2.abs() < tolerance {
            if random.chance(s.chance) {
                self.target_offset = V(
                    (0.00942273861037109 + 0.23319837528201348 * random.gaussian()) * yaw,
                    (-0.30075078007595923 + 0.3492437109081718 * random.gaussian()) * pitch,
                    (0.013282929419023442 + 0.24453708645460387 * random.gaussian()) * yaw,
                );
            }
        } else {
            let speed = if dynamic { s.dynamic_speed } else { s.speed };
            self.current_offset = V(
                self.current_offset.0
                    + f64::from(random.float(speed))
                        * (self.target_offset.0 - self.current_offset.0),
                self.current_offset.1
                    + f64::from(random.float(speed))
                        * (self.target_offset.1 - self.current_offset.1),
                self.current_offset.2
                    + f64::from(random.float(speed))
                        * (self.target_offset.2 - self.current_offset.2),
            );
        }
    }
}
