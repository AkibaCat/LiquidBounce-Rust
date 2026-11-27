// SPDX-License-Identifier: GPL-3.0-or-later
//! Exact live player/block facts read by the original walking controller.
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    simulation::{
        Mechanics, block_grid::SimulationState, motion_traits::MotionTraits,
        player_motion::MotionPlayer,
    },
};
pub trait ControlState: SimulationState {
    fn is_farmland(&self) -> bool;
    fn is_bottom_stair(&self) -> bool;
}
pub trait WalkPlayer: MotionPlayer {
    type State: ControlState;
    type Level: crate::navigation::terrain::TerrainLevel<Self::State>;
    fn level(&self) -> &Self::Level;
    fn y(&self) -> f64;
    fn movement_speed(&self) -> f64;
    fn jump_strength(&self) -> f64;
    fn jump_boost_power(&self) -> f32;
    fn mechanics(&self) -> Mechanics;
    fn tick_count(&self) -> i32;
    fn horizontal_collision(&self) -> bool;
    fn is_in_water(&self) -> bool;
    fn on_climbable(&self) -> bool;
    fn is_using_item(&self) -> bool;
    fn block_below_movement(&self) -> BlockPos;
    fn food_level(&self) -> i32;
    fn eye_position(&self) -> Vec3;
    fn is_eye_in_water(&self) -> bool;
    fn air_supply(&self) -> i32;
    fn max_air_supply(&self) -> i32;
}
pub fn capture_motion(player: &impl WalkPlayer) -> MotionTraits {
    let speed = player.movement_speed() as f32;
    let speed = if player.is_sprinting() {
        speed / 1.3_f32
    } else {
        speed
    };
    MotionTraits::new(
        speed,
        player.jump_strength() as f32,
        player.jump_boost_power(),
        player.mechanics(),
    )
}
pub fn is_stable(player: &impl WalkPlayer) -> bool {
    player.on_ground() || player.is_in_water() || player.on_climbable()
}

/// The original Body's exact simulation contract. The game adapter owns a real
/// cloned 26.3 body; the common land model never implements this interface.
#[derive(Clone, Copy, Debug)]
pub struct SimulatedInput {
    pub keys: crate::navigation::input::Input,
    pub force_stay_on_ground_surface: bool,
    pub ignores_edge_back_off: bool,
}
impl SimulatedInput {
    pub fn new(keys: crate::navigation::input::Input) -> Self {
        Self {
            keys,
            force_stay_on_ground_surface: false,
            ignores_edge_back_off: false,
        }
    }
}
pub trait WalkSimulation {
    type Player: WalkPlayer;
    fn player(&self) -> &Self::Player;
    fn predicted_position(&self) -> Vec3;
    fn set_input(&mut self, input: SimulatedInput);
    fn tick(&mut self);
    fn has_backed_off_edge(&self) -> bool;
}
pub trait WalkBody {
    type Player: WalkPlayer;
    type LiveLevel: crate::navigation::terrain::TerrainLevel<<Self::Player as WalkPlayer>::State>;
    type Simulation: WalkSimulation;
    fn player(&self) -> &Self::Player;
    fn live_level(&self) -> Self::LiveLevel;
    fn exact_simulation(&self) -> bool;
    fn input_y_rot(&self) -> f32;
    fn movement_y_rot(&self) -> f32;
    fn simulate(&self, input: SimulatedInput, y_rot: f32) -> Self::Simulation;
}
pub fn simulate_tick(
    body: &impl WalkBody,
    input: crate::navigation::input::Input,
) -> impl WalkSimulation {
    let mut simulated = body.simulate(SimulatedInput::new(input), body.input_y_rot());
    simulated.tick();
    simulated
}
