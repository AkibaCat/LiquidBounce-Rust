// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Bridge.kt.
use super::path_points::StepControl;
use crate::navigation::{
    geometry::BlockPos,
    walk::route::Step,
    walk_jump::{vector_yaw, wrap_degrees},
};
#[derive(Clone, Debug)]
pub struct Bridge {
    pub start: usize,
    pub end: usize,
    pub to: BlockPos,
    pub rises: bool,
    pub is_straight: bool,
    pub y_rot: f32,
    pub turn_after: i32,
}
fn direction(step: &Step) -> BlockPos {
    BlockPos::new(
        step.to.x.wrapping_sub(step.from.x),
        0,
        step.to.z.wrapping_sub(step.from.z),
    )
}
impl Bridge {
    pub fn at(steps: &[Step], index: usize) -> Option<Self> {
        if !steps[index].is_bridge() && !steps[index].is_rising_bridge() {
            return None;
        }
        let mut start = index;
        while start > 0 && (steps[start - 1].is_bridge() || steps[start - 1].is_rising_bridge()) {
            start -= 1;
        }
        let mut end = index + 1;
        while end < steps.len() && (steps[end].is_bridge() || steps[end].is_rising_bridge()) {
            end += 1;
        }
        Some(Self::new(steps, start, end))
    }
    pub fn new(steps: &[Step], start: usize, end: usize) -> Self {
        let to = steps[end - 1].to;
        let y_rot = vector_yaw(
            f64::from(to.x) - f64::from(steps[start].from.x),
            f64::from(to.z) - f64::from(steps[start].from.z),
        );
        let turn_after = steps.get(end).map_or(0, |next| {
            let way = direction(next);
            let turn = wrap_degrees(vector_yaw(f64::from(way.x), f64::from(way.z)) - y_rot);
            if turn.abs() < 30.0 {
                0
            } else {
                turn.signum() as i32
            }
        });
        Self {
            start,
            end,
            to,
            rises: (start..end).any(|i| steps[i].is_rising_bridge()),
            is_straight: (start..end).all(|i| direction(&steps[i]) == direction(&steps[start])),
            y_rot,
            turn_after,
        }
    }
    pub fn length(&self) -> usize {
        self.end - self.start
    }
}
