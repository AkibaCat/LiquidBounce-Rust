// SPDX-License-Identifier: GPL-3.0-or-later
//! Original Cruise column sampling, rocket budget and Landing handover.
use super::super::geometry::{Vec3, java_max};
use super::{Controls, Steering, elytra_motion, landing::Landing, rockets::Rockets};
pub trait ColumnHeights {
    fn top(&mut self, x: i32, z: i32) -> Option<i32>;
}
pub struct Cruise<H, F> {
    goal: Vec3,
    heights: H,
    rockets: Rockets,
    ceiling: i32,
    fallback: F,
    altitude: f64,
    ticks: i32,
    landing: Option<Landing>,
}
impl<H: ColumnHeights, F: FnMut(Vec3) -> Option<Vec3>> Cruise<H, F> {
    pub fn new(goal: Vec3, heights: H, rockets: Rockets, ceiling: i32, fallback: F) -> Self {
        Self {
            goal,
            heights,
            rockets,
            ceiling,
            fallback,
            altitude: 0.0,
            ticks: 0,
            landing: None,
        }
    }
    pub fn rockets(&self) -> &Rockets {
        self.landing
            .as_ref()
            .map_or(&self.rockets, |landing| &landing.rockets)
    }
    fn land(&mut self, at: Vec3, position: Vec3, motion: Vec3) -> Controls {
        let mut landing = Landing::new(at, self.rockets, 0.0);
        let controls = landing.steer(position, motion);
        self.landing = Some(landing);
        controls
    }
}
impl<H: ColumnHeights, F: FnMut(Vec3) -> Option<Vec3>> Steering for Cruise<H, F> {
    fn target(&self) -> Vec3 {
        self.landing.as_ref().map_or(self.goal, Steering::target)
    }
    fn steer(&mut self, position: Vec3, motion: Vec3) -> Controls {
        if let Some(landing) = &mut self.landing {
            return landing.steer(position, motion);
        }
        let dx = self.goal.x - position.x;
        let dz = self.goal.z - position.z;
        let distance = (dx * dx + dz * dz).sqrt();
        if distance <= 64.0 {
            return self.land(self.goal, position, motion);
        }
        if self.rockets.left() == 0 && self.rockets.boost_ticks() == 0 {
            let at = (self.fallback)(position).unwrap_or(self.goal);
            return self.land(at, position, motion);
        }
        let prior = self.ticks;
        self.ticks = self.ticks.wrapping_add(1);
        if prior % 20 == 0 {
            self.altitude = cruise_altitude(position, self.goal, &mut self.heights)
                .min(f64::from(self.ceiling.wrapping_sub(4)));
        }
        let below = self.altitude - position.y;
        let fire = self.rockets.can_fire()
            && ((motion.x * motion.x + motion.z * motion.z).sqrt() < 1.2 || below > 3.0);
        if fire {
            self.rockets.fire();
        }
        let boosted = self.rockets.tick();
        Controls {
            y_rot: elytra_motion::y_rot_towards(dx, dz),
            x_rot: elytra_motion::x_rot_towards(below, 24.0).clamp(-40.0, 15.0),
            should_fire_rocket: fire,
            is_boosted: boosted,
        }
    }
}
pub fn cruise_altitude(from: Vec3, goal: Vec3, heights: &mut impl ColumnHeights) -> f64 {
    let mut top = java_max(from.y, goal.y);
    let mut unknown = false;
    let dx = goal.x - from.x;
    let dz = goal.z - from.z;
    let samples = ((dx * dx + dz * dz).sqrt() / 4.0).ceil() as i32;
    for i in 0..=samples {
        let t = if samples == 0 {
            0.0
        } else {
            f64::from(i) / f64::from(samples)
        };
        if let Some(column) = heights.top(
            (from.x + dx * t).floor() as i32,
            (from.z + dz * t).floor() as i32,
        ) {
            top = java_max(top, f64::from(column));
        } else {
            unknown = true;
        }
    }
    top + 12.0 + if unknown { 24.0 } else { 0.0 }
}
pub fn rockets_needed(distance: f64, climb: f64, rocket_ticks: i32) -> i32 {
    if rocket_ticks <= 0 {
        return i32::MAX;
    }
    let per = f64::from(rocket_ticks) * 1.5 * 0.7;
    let climb_per = f64::from(rocket_ticks) * 1.5 * 0.6;
    ((distance / per).ceil() as i32)
        .wrapping_add((java_max(climb, 0.0) / climb_per).ceil() as i32)
        .wrapping_add(3)
}
