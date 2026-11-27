// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Leaping.kt: measured pace and silent-yaw input correction.
use super::{
    body::{WalkBody, WalkPlayer},
    directional_input_for_y_rot,
    step_progress::Movement,
    stride::Stride,
};
use crate::navigation::{
    input::MovementKeys,
    terrain::Terrain,
    walk::{abilities::WalkAbilities, route::Step},
    walk_jump::{jump::Jump, leap::Leap},
};
#[allow(clippy::too_many_arguments)]
pub fn leap<B: WalkBody>(
    body: &B,
    keys: &mut MovementKeys,
    frame_y_rot: f32,
    step: &Step,
    jump: &Jump,
    abilities: &WalkAbilities<<B::Player as WalkPlayer>::State>,
    leap: &mut Leap,
    stride: &Stride,
) -> Movement {
    let mut terrain = Terrain::player(body.live_level());
    let player = body.player();
    let movement_y_rot = body.movement_y_rot();
    let measured = stride.abilities(abilities, player);
    let input = leap.tick(player, &mut terrain, movement_y_rot, step, jump, &measured);
    let silent = body.movement_y_rot();
    let silent = (silent != frame_y_rot).then_some(silent);
    keys.directional_input =
        directional_input_for_y_rot(input.forward, input.strafe, frame_y_rot, silent);
    keys.jump = input.jump;
    keys.sneak = input.shift;
    Movement {
        target: leap.target,
        sprint: leap.sprint,
        ..Movement::default()
    }
}
