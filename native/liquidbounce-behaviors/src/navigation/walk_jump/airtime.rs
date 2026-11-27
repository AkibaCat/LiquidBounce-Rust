// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Airtime.kt; references are passed explicitly in Rust.
use super::{jump::Jump, yaw};
use crate::navigation::{
    geometry::{Vec3, java_min},
    simulation::{
        block_grid::BlockGrid,
        player_motion::{HALF_WIDTH, PlayerMotion},
    },
    walk::course::Course,
};
#[derive(Clone, Debug)]
pub struct Airtime {
    max_fall: f64,
    holds_y_rot: bool,
    pub air_ticks: i32,
}
impl Airtime {
    pub fn new(safe_fall: i32, risky_jumps: bool, holds_y_rot: bool) -> Self {
        Self {
            max_fall: f64::from(safe_fall) + if risky_jumps { 4.5 } else { 0.0 },
            holds_y_rot,
            air_ticks: 0,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn jump(
        &mut self,
        grid: &mut impl BlockGrid,
        course: &mut Course,
        motion: &mut PlayerMotion,
        can_sprint: bool,
        run_up: Vec3,
        takeoff: i32,
        jumps: bool,
        hops: i32,
    ) -> Option<Jump> {
        let margin = self.land(grid, course, motion, can_sprint);
        if margin.is_nan() {
            return None;
        }
        Some(Jump {
            run_up,
            takeoff,
            jumps,
            air_ticks: self.air_ticks,
            margin,
            hops,
            aim: Vec3::new(course.aim_x, course.target_height, course.aim_z),
        })
    }
    pub fn land(
        &mut self,
        grid: &mut impl BlockGrid,
        course: &mut Course,
        motion: &mut PlayerMotion,
        can_sprint: bool,
    ) -> f64 {
        let floor = java_min(course.target_height, course.from_y) - Course::MAX_BELOW;
        let catches = course.catches;
        self.air_ticks = 1;
        while !motion.on_ground && motion.y > floor && self.air_ticks < Course::MAX_AIR_TICKS {
            if catches && course.is_in_target(motion.x, motion.y, motion.z) {
                return course.margin(motion.x, motion.z);
            }
            let back = !catches && Self::overshoots(course, motion);
            let pulls_back = self.holds_y_rot
                && motion.delta_x * (course.target_x - motion.x)
                    + motion.delta_z * (course.target_z - motion.z)
                    < 0.0;
            let sprint = can_sprint && !back && !pulls_back;
            motion.tick(
                grid,
                Self::y_rot_to_target(course, motion),
                if back { -1 } else { 1 },
                0,
                false,
                sprint,
                false,
            );
            Self::note_obstruction(course, motion);
            self.air_ticks += 1;
        }
        if catches || !motion.on_ground || motion.landing_fall > self.max_fall {
            return f64::NAN;
        }
        let touchdown = Self::footing(grid, course, motion);
        for _ in 0..3 {
            motion.tick(
                grid,
                Self::y_rot_to_target(course, motion),
                1,
                0,
                false,
                false,
                true,
            );
        }
        let settled = Self::footing(grid, course, motion);
        if touchdown.is_nan() || settled > touchdown {
            settled
        } else {
            touchdown
        }
    }
    pub fn note_obstruction(course: &mut Course, motion: &PlayerMotion) {
        let reach = Course::HALF_BLOCK + HALF_WIDTH + 0.05;
        if motion.horizontal_collision
            && ((motion.x - course.target_x).abs() > reach
                || (motion.z - course.target_z).abs() > reach)
        {
            course.obstructed = true;
        }
    }
    pub fn y_rot_to_target(course: &Course, motion: &PlayerMotion) -> f32 {
        Self::y_rot_to(motion, course.target_x, course.target_z)
    }
    pub fn y_rot_to(motion: &PlayerMotion, x: f64, z: f64) -> f32 {
        yaw(x - motion.x, z - motion.z)
    }
    fn footing(grid: &mut impl BlockGrid, course: &Course, motion: &PlayerMotion) -> f64 {
        let target = course.target;
        if !motion.on_ground || (motion.y - course.target_height).abs() > 1.0e-3 {
            f64::NAN
        } else if motion.stands_on(target.x, target.y, target.z) {
            motion.footing(grid, target)
        } else if motion.stands_on(target.x, target.y.wrapping_sub(1), target.z) {
            motion.footing(grid, target.offset(0, -1, 0))
        } else {
            f64::NAN
        }
    }
    fn overshoots(course: &Course, motion: &PlayerMotion) -> bool {
        let tx = course.target_x - motion.x;
        let tz = course.target_z - motion.z;
        let remaining = (tx * tx + tz * tz).sqrt();
        if remaining < 1.0e-4 {
            return false;
        }
        let speed = (motion.delta_x * tx + motion.delta_z * tz) / remaining;
        course.overshoots(remaining, speed, motion.y, motion.delta_y)
    }
}
