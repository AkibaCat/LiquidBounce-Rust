// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Climbing.kt.
use super::{body::WalkPlayer, center_input, step_progress::Movement};
use crate::navigation::{
    aabb::Direction,
    geometry::{BlockPos, Vec3},
    input::MovementKeys,
    terrain::{Terrain, TerrainLevel, TerrainState},
    walk::route::Step,
};
pub fn climb_up<B: TerrainState, L: TerrainLevel<B>>(
    player: &impl WalkPlayer,
    terrain: &mut Terrain<B, L>,
    keys: &mut MovementKeys,
    frame_y_rot: f32,
    step: &Step,
) -> Movement {
    let position = player.position();
    let feet = BlockPos::containing(position);
    let cell = if terrain.is_climbable(feet) {
        feet
    } else {
        step.from
    };
    let side = terrain.climb_side(cell);
    let aim = if let Some(side) = side {
        let (x, z) = match side {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
            Direction::East => (1, 0),
            _ => (0, 0),
        };
        Vec3::new(
            step.feet.x + f64::from(x),
            step.feet.y,
            step.feet.z + f64::from(z),
        )
    } else {
        step.feet
    };
    keys.directional_input = center_input(position, aim, frame_y_rot);
    keys.jump = true;
    keys.sneak = false;
    Movement {
        target: Some(aim),
        ..Movement::default()
    }
}
