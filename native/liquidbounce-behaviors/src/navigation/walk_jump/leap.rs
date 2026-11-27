// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Leap.kt: live jump, momentum hops and settling approach.
use super::{jump::Jump, jumps::Jumps, vector_yaw, wrap_degrees};
use crate::navigation::{
    geometry::Vec3,
    simulation::{
        block_grid::SimulationState,
        player_motion::{MotionPlayer, PlayerMotion},
    },
    terrain::{Terrain, TerrainLevel},
    walk::{abilities::WalkAbilities, route::Step},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeapInput {
    pub forward: i32,
    pub strafe: i32,
    pub jump: bool,
    pub sprint: bool,
    pub shift: bool,
}
impl LeapInput {
    pub const fn new(forward: i32, strafe: i32, jump: bool, sprint: bool, shift: bool) -> Self {
        Self {
            forward,
            strafe,
            jump,
            sprint,
            shift,
        }
    }
}
#[derive(Default, Clone, Debug)]
pub struct Leap {
    pub target: Option<Vec3>,
    pub sprint: bool,
    pub hopeless: bool,
    hops_made: i32,
    still_ticks: i32,
    retreat: f64,
    running: bool,
    refined: Option<Jump>,
}
impl Leap {
    pub fn tick<B: SimulationState, L: TerrainLevel<B>>(
        &mut self,
        player: &impl MotionPlayer,
        t: &mut Terrain<B, L>,
        movement_y_rot: f32,
        step: &Step,
        jump: &Jump,
        abilities: &WalkAbilities<B>,
    ) -> LeapInput {
        let mut state = PlayerMotion::new(abilities.motion);
        state.reset_player(t, player);
        let mut jumps = Jumps::new(abilities.clone());
        let input = self.decide(t, &state, movement_y_rot, step, jump, &mut jumps);
        self.sprint = input.sprint;
        input
    }
    pub fn decide<B: SimulationState, L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        step: &Step,
        planned: &Jump,
        jumps: &mut Jumps<B>,
    ) -> LeapInput {
        self.target = Some(step.feet);
        if !state.on_ground {
            self.still_ticks = 0;
            self.running = false;
            let keys = jumps.air_keys(t, state, y_rot, step.to, step.feet.y, false);
            let forward = keys.map_or(1, |k| k.forward);
            return LeapInput::new(
                forward,
                keys.map_or(0, |k| k.strafe),
                false,
                forward > 0,
                false,
            );
        }
        let jump = self.refined.as_ref().unwrap_or(planned).clone();
        let ready = self.is_ready(state, self.stop(&jump));
        let across_gap = horizontal_distance_sqr(step.feet, state.x, state.z) > 4.0
            && faces(state, y_rot, step.feet);
        let take_off = if across_gap {
            jumps.taking_off_now(t, state, y_rot, step.to, step.feet.y)
        } else {
            None
        };
        if let Some(take_off) = take_off {
            self.target = Some(take_off.aim);
            return LeapInput::new(1, 0, true, jumps.can_sprint(), false);
        }
        let now = jumps
            .from_here(t, state, y_rot, step.to, step.feet.y, Some(jump.aim))
            .filter(|it| ready || self.running || it.margin >= 0.35);
        if let Some(now) = &now
            && !self.running
            && !faces(state, y_rot, step.feet)
        {
            self.target = Some(now.aim);
            return self.approach(t, state, y_rot, state.position(), now.aim);
        }
        self.running = now.is_some();
        if let Some(now) = now {
            self.target = Some(now.aim);
            return LeapInput::new(
                1,
                0,
                now.jumps && now.takeoff == 0,
                jumps.can_sprint(),
                false,
            );
        }
        if self.hops_made < jump.hops && (self.hops_made > 0 || ready) {
            self.hops_made += 1;
            return LeapInput::new(1, 0, true, jumps.can_sprint(), false);
        }
        self.hops_made = 0;
        if ready && jump.hops == 0 {
            self.move_run_up(t, step, &jump, jumps);
        }
        self.approach(
            t,
            state,
            y_rot,
            self.stop(self.refined.as_ref().unwrap_or(planned)),
            step.feet,
        )
    }
    fn is_ready(&mut self, state: &PlayerMotion, stop: Vec3) -> bool {
        self.still_ticks = if state.horizontal_speed() < 0.03 {
            self.still_ticks.wrapping_add(1)
        } else {
            0
        };
        let at_stop = horizontal_distance_sqr(stop, state.x, state.z)
            < Jump::RUN_UP_TOLERANCE * Jump::RUN_UP_TOLERANCE;
        self.still_ticks > 0 && (at_stop || self.still_ticks >= 5)
    }
    fn move_run_up<B: SimulationState, L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        step: &Step,
        jump: &Jump,
        jumps: &mut Jumps<B>,
    ) {
        if self.refined.is_none() {
            self.refined = jumps
                .refine(t, step.from, jump.run_up.y, step.to, step.feet.y)
                .filter(|it| {
                    it.run_up.distance_sqr(jump.run_up)
                        > Jump::RUN_UP_TOLERANCE * Jump::RUN_UP_TOLERANCE
                });
            if self.refined.is_some() {
                return;
            }
        }
        self.hopeless = self.retreat >= 0.5;
        self.retreat = (self.retreat + 0.1).min(0.5);
    }
    fn stop(&self, jump: &Jump) -> Vec3 {
        let run_up = jump.run_up;
        let distance = 0.05 + self.retreat;
        let dx = jump.aim.x - run_up.x;
        let dz = jump.aim.z - run_up.z;
        let length = (dx * dx + dz * dz).sqrt();
        if length < 1.0e-6 {
            return run_up;
        }
        Vec3::new(
            run_up.x - dx / length * distance,
            run_up.y,
            run_up.z - dz / length * distance,
        )
    }
    fn approach<B: SimulationState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        stop: Vec3,
        look: Vec3,
    ) -> LeapInput {
        let mut probe = state.copy();
        let mut best = LeapInput::new(0, 0, false, false, false);
        let mut best_score = f64::MAX;
        for sneak in [false, true] {
            for forward in -1..=1 {
                for strafe in -1..=1 {
                    probe.copy_from(state);
                    let input = LeapInput::new(forward, strafe, false, false, sneak);
                    let score = Self::rest_distance_sqr(t, &mut probe, y_rot, input, stop, look);
                    if score < best_score - 1.0e-4 {
                        best_score = score;
                        best = input;
                    }
                }
            }
        }
        best
    }
    fn rest_distance_sqr<B: SimulationState, L: TerrainLevel<B>>(
        t: &mut Terrain<B, L>,
        probe: &mut PlayerMotion,
        y_rot: f32,
        input: LeapInput,
        stop: Vec3,
        look: Vec3,
    ) -> f64 {
        for tick in 0..8 {
            let held = tick < 2;
            let tick_y_rot = if tick == 0 {
                y_rot
            } else {
                vector_yaw(look.x - probe.x, look.z - probe.z)
            };
            probe.tick(
                t,
                tick_y_rot,
                if held { input.forward } else { 0 },
                if held { input.strafe } else { 0 },
                false,
                false,
                held && input.shift,
            );
            if !probe.on_ground {
                return 100.0 - f64::from(tick);
            }
        }
        horizontal_distance_sqr(stop, probe.x, probe.z)
    }
}
pub(crate) fn horizontal_distance_sqr(point: Vec3, x: f64, z: f64) -> f64 {
    let dx = point.x - x;
    let dz = point.z - z;
    dx * dx + dz * dz
}
fn faces(state: &PlayerMotion, y_rot: f32, aim: Vec3) -> bool {
    wrap_degrees(vector_yaw(aim.x - state.x, aim.z - state.z) - y_rot).abs() <= 15.0
}
