// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete PathPoints.kt floor-click prediction with the original exact body.
use super::{
    body::{SimulatedInput, WalkBody, WalkSimulation, simulate_tick},
    path_points::StepControl,
};
use crate::navigation::{
    input::{Input, MovementKeys},
    simulation::player_motion::MotionPlayer,
    walk::route::Step,
};
pub fn can_click_floor(
    step: &Step,
    body: &impl WalkBody,
    keys: &mut MovementKeys,
    frame_y_rot: f32,
    sneaks: bool,
) -> bool {
    if stays_over_floor(step, body, keys.to_input(false), frame_y_rot, sneaks) {
        return true;
    }
    if !keys.sneak {
        return false;
    }
    keys.directional_input = Input::EMPTY;
    stays_over_floor(step, body, keys.to_input(false), frame_y_rot, sneaks)
}
fn stays_over_floor(
    step: &Step,
    body: &impl WalkBody,
    input: Input,
    frame_y_rot: f32,
    sneaks: bool,
) -> bool {
    let mut simulated = simulate_tick(body, input);
    for _ in 0..4 {
        if step.past_edge(simulated.player().position()) >= 0.3 {
            return false;
        }
        if step.outward(simulated.player().delta_movement()) <= 0.0 {
            return true;
        }
        let brake = step
            .hold_back(
                simulated.player().position(),
                simulated.player().delta_movement(),
                frame_y_rot,
            )
            .with_keys(false, sneaks, false);
        simulated.set_input(SimulatedInput {
            keys: brake,
            force_stay_on_ground_surface: false,
            ignores_edge_back_off: true,
        });
        simulated.tick();
    }
    step.past_edge(simulated.player().position()) < 0.3
}
