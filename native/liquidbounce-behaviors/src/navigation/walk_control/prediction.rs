// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Prediction.kt: live exact simulation and common land fallback.
use super::{
    body::{ControlState, WalkPlayer, is_stable},
    step_check::wants_jump_at,
};
use crate::navigation::simulation::player_motion::MotionPlayer;
use crate::navigation::{geometry::BlockPos, terrain::TerrainLevel, walk::route::Step};
pub fn leads_onto_stairs<B: ControlState, L: TerrainLevel<B>>(step: &Step, level: &L) -> bool {
    let feet = BlockPos::containing(step.feet);
    level.block_state(feet).is_bottom_stair()
        || level.block_state(feet.offset(0, -1, 0)).is_bottom_stair()
}
pub fn wants_jump(step: &Step, player: &impl WalkPlayer) -> bool {
    !leads_onto_stairs(step, player.level())
        && wants_jump_at(
            step,
            player.position(),
            is_stable(player),
            player.is_in_water(),
        )
}

pub fn gets_ahead(
    body: &impl super::body::WalkBody,
    keys: &crate::navigation::input::MovementKeys,
    frame_y_rot: f32,
    step: &Step,
    movement: super::step_progress::Movement,
) -> bool {
    let Some(aim) = movement.target.filter(|_| !movement.dive) else {
        return true;
    };
    let closer = body.player().position().distance(aim) - super::PROGRESS;
    if body.exact_simulation() {
        simulates_ahead(body, keys, frame_y_rot, step, movement, aim, closer)
    } else {
        moves_ahead(body, keys, frame_y_rot, step, movement, aim, closer)
    }
}
fn simulates_ahead(
    body: &impl super::body::WalkBody,
    keys: &crate::navigation::input::MovementKeys,
    frame_y_rot: f32,
    step: &Step,
    movement: super::step_progress::Movement,
    aim: crate::navigation::geometry::Vec3,
    closer: f64,
) -> bool {
    use super::body::{SimulatedInput, WalkSimulation};
    let sprint = movement.sprint;
    let mut simulated = body.simulate(
        SimulatedInput {
            keys: keys.to_input(sprint),
            force_stay_on_ground_surface: movement.safe_walk,
            ignores_edge_back_off: false,
        },
        frame_y_rot,
    );
    for _ in 0..20 {
        simulated.tick();
        let position = simulated.predicted_position();
        if position.distance(aim) < closer {
            return true;
        }
        let next = super::center_input(position, aim, frame_y_rot).with_keys(
            wants_jump(step, simulated.player()),
            keys.sneak,
            sprint,
        );
        simulated.set_input(SimulatedInput {
            keys: next,
            force_stay_on_ground_surface: movement.safe_walk,
            ignores_edge_back_off: false,
        });
    }
    false
}
fn moves_ahead(
    body: &impl super::body::WalkBody,
    keys: &crate::navigation::input::MovementKeys,
    frame_y_rot: f32,
    step: &Step,
    movement: super::step_progress::Movement,
    aim: crate::navigation::geometry::Vec3,
    closer: f64,
) -> bool {
    use crate::navigation::{simulation::player_motion::PlayerMotion, terrain::Terrain};
    let sprint = movement.sprint;
    let mut terrain = Terrain::player(body.live_level());
    let mut state = PlayerMotion::new(super::body::capture_motion(body.player()));
    state.reset_player(&mut terrain, body.player());
    let mut input = keys.to_input(sprint);
    for _ in 0..20 {
        state.tick(
            &mut terrain,
            frame_y_rot,
            i32::from(input.forward) - i32::from(input.backward),
            i32::from(input.left) - i32::from(input.right),
            input.jump,
            sprint,
            keys.sneak,
        );
        let position = state.position();
        if position.distance(aim) < closer {
            return true;
        }
        let jump = wants_jump_at(step, position, state.on_ground, false);
        input = super::center_input(position, aim, frame_y_rot).with_keys(jump, keys.sneak, sprint);
    }
    false
}
