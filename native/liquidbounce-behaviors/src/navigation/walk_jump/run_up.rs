// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent RunUp.kt. The optional path is scoped to one takeOff call.
use super::airtime::Airtime;
use crate::navigation::{
    geometry::Vec3,
    simulation::{
        block_grid::BlockGrid,
        player_motion::{PlayerMotion, RESTING},
    },
    walk::course::Course,
};
#[derive(Default, Debug)]
pub struct RunUp {
    pub path: Option<Vec<Vec3>>,
}
impl RunUp {
    pub fn tick(
        &mut self,
        grid: &mut impl BlockGrid,
        course: &mut Course,
        motion: &mut PlayerMotion,
        can_sprint: bool,
        y_rot: f32,
        jump: bool,
    ) {
        motion.tick(grid, y_rot, 1, 0, jump, can_sprint, false);
        Airtime::note_obstruction(course, motion);
        if let Some(path) = &mut self.path {
            path.push(motion.position());
        }
    }
    pub fn y_rot_at(
        course: &Course,
        motion: &PlayerMotion,
        tick: i32,
        first_y_rot: Option<f32>,
    ) -> f32 {
        if tick == 0
            && let Some(y) = first_y_rot
        {
            return y;
        }
        Airtime::y_rot_to(motion, course.aim_x, course.aim_z)
    }
    pub fn hop_travel(
        &mut self,
        grid: &mut impl BlockGrid,
        course: &mut Course,
        motion: &mut PlayerMotion,
        can_sprint: bool,
        runway: f64,
        hops: i32,
    ) -> f64 {
        motion.reset(grid, course.behind(runway), RESTING, true, false, 0);
        let sx = motion.x;
        let sz = motion.z;
        if self.hop(grid, course, motion, can_sprint, hops) < 0 {
            return f64::NAN;
        }
        (motion.x - sx) * course.dir_x + (motion.z - sz) * course.dir_z
    }
    pub fn hop(
        &mut self,
        grid: &mut impl BlockGrid,
        course: &mut Course,
        motion: &mut PlayerMotion,
        can_sprint: bool,
        hops: i32,
    ) -> i32 {
        let mut ticks = 0;
        for _ in 0..hops {
            let mut air = 0;
            loop {
                self.tick(
                    grid,
                    course,
                    motion,
                    can_sprint,
                    Airtime::y_rot_to_target(course, motion),
                    air == 0,
                );
                air += 1;
                if motion.on_ground || air >= Course::MAX_AIR_TICKS {
                    break;
                }
            }
            let progress = (motion.x - course.from_x) * course.dir_x
                + (motion.z - course.from_z) * course.dir_z;
            if !motion.on_ground
                || (motion.y - course.from_y).abs() > Course::LANDING_PROBE
                || progress > course.edge
            {
                return -1;
            }
            ticks += air;
        }
        ticks
    }
}
