// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Swimming.kt; raw world queries intentionally bypass Terrain.
use super::{body::WalkPlayer, center_input, step_progress::Movement};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    input::MovementKeys,
    terrain::{TerrainLevel, TerrainState},
    walk::route::Step,
};
#[derive(Default)]
pub struct Swimming {
    breathing: bool,
}
impl Swimming {
    pub fn is_breathing(&self) -> bool {
        self.breathing
    }
    pub fn swim(
        &self,
        player: &impl WalkPlayer,
        keys: &mut MovementKeys,
        frame_y_rot: f32,
        step: &Step,
    ) -> Movement {
        let position = player.position();
        let target = Vec3::new(step.feet.x, step.feet.y + 0.2, step.feet.z);
        let rise = target.y - position.y;
        keys.directional_input = center_input(position, target, frame_y_rot);
        keys.jump = rise > 0.15;
        keys.sneak = rise < -0.15;
        Movement {
            target: Some(target),
            sprint: true,
            dive: true,
            ..Movement::default()
        }
    }
    pub fn is_short_of_air(&self, player: &impl WalkPlayer) -> bool {
        !player.is_eye_in_water() && player.air_supply() < player.max_air_supply()
    }
    pub fn breathes<B: TerrainState, L: TerrainLevel<B>>(
        &mut self,
        player: &impl WalkPlayer,
        level: &L,
    ) -> bool {
        self.breathing = if !player.is_in_water() || player.air_supply() >= player.max_air_supply()
        {
            false
        } else if f64::from(player.air_supply()) < f64::from(player.max_air_supply()) * 0.3 {
            is_air_above(player, level)
        } else {
            self.breathing
        };
        self.breathing
    }
}
fn is_air_above<B: TerrainState, L: TerrainLevel<B>>(player: &impl WalkPlayer, level: &L) -> bool {
    let head = BlockPos::containing(player.eye_position());
    for dy in 0..=4 {
        let pos = head.offset(0, dy, 0);
        let state = level.block_state(pos);
        if !state.has_liquid() {
            return state.collision_shape(level, pos).is_empty();
        }
    }
    false
}
