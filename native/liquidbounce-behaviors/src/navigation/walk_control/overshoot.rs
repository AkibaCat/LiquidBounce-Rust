// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Overshoot.kt.
use super::{body::WalkPlayer, path_points::StepControl};
use crate::navigation::{
    geometry::{Vec3, java_max},
    walk::route::Step,
};
#[derive(Default)]
pub struct Overshoot {
    count: i32,
    reversing: bool,
}
impl Overshoot {
    pub fn count(&self) -> i32 {
        self.count
    }
    pub fn forward_aim(
        &mut self,
        player: &impl WalkPlayer,
        position: Vec3,
        aim: Vec3,
        step: &Step,
        mut point_ahead: impl FnMut(Vec3, f64) -> Option<Vec3>,
    ) -> Vec3 {
        self.forward_aim_tagged(player, position, (aim, ()), (), step, |p, d| {
            point_ahead(p, d).map(|point| (point, ()))
        })
        .0
    }
    /// Carries source object provenance through the same decisions. Follower
    /// needs Kotlin reference identity, which coordinate equality cannot model.
    #[allow(clippy::too_many_arguments)]
    pub fn forward_aim_tagged<T>(
        &mut self,
        player: &impl WalkPlayer,
        position: Vec3,
        aim: (Vec3, T),
        position_tag: T,
        step: &Step,
        mut point_ahead: impl FnMut(Vec3, f64) -> Option<(Vec3, T)>,
    ) -> (Vec3, T) {
        let motion = player.delta_movement();
        let speed = (motion.x * motion.x + motion.z * motion.z).sqrt();
        let ahead = (aim.0.x - position.x) * motion.x + (aim.0.z - position.z) * motion.z;
        let reverses = speed > 0.3 && ahead < 0.0;
        if reverses && !self.reversing {
            self.count = self.count.wrapping_add(1);
        }
        self.reversing = reverses;
        if !reverses || !step.is_plain() {
            return aim;
        }
        let point = point_ahead(position, java_max(speed * 3.0, 1.0));
        if let Some(point) = point
            .filter(|p| (p.0.x - position.x) * motion.x + (p.0.z - position.z) * motion.z >= 0.0)
        {
            point
        } else if player.on_ground() {
            aim
        } else {
            (position, position_tag)
        }
    }
}
