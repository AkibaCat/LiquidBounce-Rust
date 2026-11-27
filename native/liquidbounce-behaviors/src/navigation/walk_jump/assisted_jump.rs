// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent AssistedJump.kt, with reference identity for retained plans.
use super::{
    jump::Jump,
    jump_targets::JumpTargets,
    jumps::Jumps,
    landings::{Landings, Missed, angle},
    leap::{Leap, LeapInput},
    vector_yaw,
};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    simulation::{
        block_grid::SimulationState,
        player_motion::{HALF_WIDTH, PlayerMotion, RESTING},
    },
    terrain::{Terrain, TerrainLevel},
    walk::route::{Move, Step},
};
use std::{collections::HashMap, sync::Arc};
pub type LandingRef = Arc<Landing>;
#[derive(Clone, Debug)]
pub struct Landing {
    pub pos: BlockPos,
    pub feet: f64,
    pub jump: Jump,
    pub step: Option<Step>,
    pub bounces: bool,
    pub takes_off: bool,
    pub settles: bool,
}
impl Landing {
    pub fn aim(&self) -> Vec3 {
        self.jump.aim
    }
    pub fn is_landed(&self, position: Vec3, on_ground: bool) -> bool {
        on_ground && (position.y - self.feet).abs() < 1.0e-3 && self.is_over(position)
    }
    fn is_over(&self, position: Vec3) -> bool {
        (position.x - f64::from(self.pos.x) - 0.5).abs() < 0.5 + HALF_WIDTH
            && (position.z - f64::from(self.pos.z) - 0.5).abs() < 0.5 + HALF_WIDTH
    }
    pub fn aim_y_rot(
        &self,
        position: Vec3,
        motion: Vec3,
        on_ground: bool,
        look: Option<Vec3>,
    ) -> f32 {
        let look = look.unwrap_or_else(|| self.aim());
        let dx = look.x - position.x;
        let dz = look.z - position.z;
        let passed = !on_ground && dx * motion.x + dz * motion.z < 0.0;
        let near = dx * dx + dz * dz < 0.75 * 0.75;
        let bounces_on = self.takes_off && self.is_over(position);
        if near || passed || bounces_on || self.is_landed(position, on_ground) {
            f32::NAN
        } else {
            vector_yaw(dx, dz)
        }
    }
    pub fn steer<B: SimulationState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        jumps: &mut Jumps<B>,
    ) -> Option<LeapInput> {
        let air = jumps.air_keys(t, state, y_rot, self.pos, self.feet, self.takes_off)?;
        let sprint = air.forward > 0;
        let lands = if self.settles {
            let mut probe = state.copy();
            probe.tick(t, y_rot, air.forward, air.strafe, false, sprint, false);
            probe.on_ground
        } else {
            false
        };
        Some(LeapInput::new(
            air.forward,
            air.strafe,
            false,
            sprint && !lands,
            lands,
        ))
    }
}
#[derive(Clone, Debug)]
pub struct JumpArc {
    pub target: LandingRef,
    pub path: Vec<Vec3>,
    pub takeoff: i32,
}
impl JumpArc {
    pub fn new(target: LandingRef) -> Self {
        Self {
            target,
            path: Vec::new(),
            takeoff: -1,
        }
    }
    pub fn follow<B: SimulationState, L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &mut PlayerMotion,
        y_rot: Option<f32>,
        jumps: &mut Jumps<B>,
    ) -> f32 {
        self.path.push(state.position());
        let bounces = state.delta_y > 0.0 && !self.target.jump.jumps;
        if state.on_ground && !bounces {
            if !jumps.take_off(
                t,
                state,
                y_rot,
                self.target.pos,
                self.target.feet,
                &self.target.jump,
                &mut self.path,
            ) {
                return f32::NAN;
            }
            self.takeoff = self.path.len() as i32 - 2;
        }
        let mut turn = y_rot.unwrap_or_else(|| {
            let a = self.target.aim();
            vector_yaw(a.x - state.x, a.z - state.z)
        });
        let mut ticks = 0;
        loop {
            let next = self.target.aim_y_rot(
                state.position(),
                state.delta_movement(),
                state.on_ground,
                None,
            );
            if !next.is_nan() {
                turn = next;
            }
            let Some(keys) = self.target.steer(t, state, turn, jumps) else {
                break;
            };
            state.tick(
                t,
                turn,
                keys.forward,
                keys.strafe,
                keys.jump,
                keys.sprint,
                keys.shift,
            );
            self.path.push(state.position());
            if state.on_ground {
                break;
            }
            ticks += 1;
            if ticks >= 60 {
                break;
            }
        }
        turn
    }
}
#[derive(Clone, Debug)]
pub struct AssistedJump {
    pub target: Option<LandingRef>,
    pub arcs: Vec<JumpArc>,
    leap: Leap,
    planned: HashMap<BlockPos, Option<LandingRef>>,
    planned_from: Option<BlockPos>,
    airborne: bool,
    ticks_since_landing: i32,
}
impl Default for AssistedJump {
    fn default() -> Self {
        Self {
            target: None,
            arcs: Vec::new(),
            leap: Leap::default(),
            planned: HashMap::new(),
            planned_from: None,
            airborne: false,
            ticks_since_landing: 10,
        }
    }
}
impl AssistedJump {
    pub fn aim_y_rot(&self, position: Vec3, motion: Vec3, on_ground: bool) -> f32 {
        let Some(target) = &self.target else {
            return f32::NAN;
        };
        let look = if target.step.is_some() {
            self.leap.target
        } else {
            None
        };
        target.aim_y_rot(position, motion, on_ground, look)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn tick<B: SimulationState, L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        heading: f32,
        keys: LeapInput,
        max_angle: f32,
        jumps: &mut Jumps<B>,
        targets: &mut JumpTargets,
    ) -> Option<LeapInput> {
        if !state.on_ground {
            return self
                .target
                .as_ref()
                .and_then(|target| target.steer(t, state, y_rot, jumps));
        }
        if self.airborne {
            self.airborne = false;
            self.ticks_since_landing = 0;
        } else if self.ticks_since_landing < 10 {
            self.ticks_since_landing += 1;
        }
        let target = self
            .run_up_kept(state, heading, max_angle)
            .or_else(|| self.pick(t, targets, jumps, state, y_rot, heading, max_angle));
        if let Some(next) = &target
            && next.step.is_some()
            && self
                .target
                .as_ref()
                .is_none_or(|current| !Arc::ptr_eq(next, current))
        {
            self.leap = Leap::default();
        }
        self.target = target.clone();
        let Some(target) = target else {
            return self.untargeted(t, state, y_rot, keys);
        };
        if let Some(step) = &target.step {
            let input = self.leap.decide(t, state, y_rot, step, &target.jump, jumps);
            self.airborne = input.jump;
            return Some(input);
        }
        if state.delta_y > 0.0 && !target.jump.jumps {
            self.airborne = true;
            return target.steer(t, state, y_rot, jumps);
        }
        let jump_now = target.jump.jumps && target.jump.takeoff == 0;
        self.airborne = jump_now;
        Some(LeapInput::new(1, 0, jump_now, true, false))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn trace<B: SimulationState, L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        heading: f32,
        max_angle: f32,
        jumps: &mut Jumps<B>,
        targets: &mut JumpTargets,
    ) {
        self.arcs.clear();
        let Some(target) = self.target.clone() else {
            return;
        };
        let mut probe = state.copy();
        let run_up = target.step.is_some();
        if run_up {
            probe.reset(t, target.jump.run_up, RESTING, true, false, 0);
        }
        self.arcs.push(JumpArc::new(target.clone()));
        let turn = self.arcs[0].follow(
            t,
            &mut probe,
            if run_up { None } else { Some(y_rot) },
            jumps,
        );
        if !target.takes_off
            || probe.delta_y <= 0.0
            || !target.is_landed(probe.position(), probe.on_ground)
        {
            return;
        }
        let Some(next) = self.pick(t, targets, jumps, &probe, turn, heading, max_angle) else {
            return;
        };
        let mut arc = JumpArc::new(next);
        arc.follow(t, &mut probe, Some(turn), jumps);
        self.arcs.push(arc);
    }
    pub fn release(&mut self) {
        self.target = None;
        self.arcs.clear();
    }
    pub fn reset(&mut self) {
        self.release();
        self.airborne = false;
        self.ticks_since_landing = 10;
        self.planned.clear();
        self.planned_from = None;
    }
    fn run_up_kept(
        &mut self,
        state: &PlayerMotion,
        heading: f32,
        max_angle: f32,
    ) -> Option<LandingRef> {
        let target = self.target.clone()?;
        let step = target.step.as_ref()?;
        if self.leap.hopeless {
            self.planned.insert(step.to, None);
            return None;
        }
        let stays = state.block_position() == step.from;
        if stays && angle(state.position(), target.aim(), heading) <= max_angle {
            Some(target)
        } else {
            None
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn pick<B: SimulationState, L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        targets: &mut JumpTargets,
        jumps: &mut Jumps<B>,
        state: &PlayerMotion,
        y_rot: f32,
        heading: f32,
        max_angle: f32,
    ) -> Option<LandingRef> {
        let position = state.position();
        let feet = state.block_position();
        let standing = t.stand_height(feet);
        let height = if standing.is_nan() {
            position.y
        } else {
            standing
        };
        let mut found = Landings::find(
            t, targets, jumps, state, y_rot, heading, max_angle, feet, height,
        );
        if state.delta_y > 0.0 {
            self.best(t, targets, jumps, state, y_rot, heading, max_angle, &found)
                .or_else(|| {
                    self.settle(
                        t,
                        state,
                        y_rot,
                        jumps,
                        if standing.is_nan() { None } else { Some(feet) },
                        height,
                    )
                })
        } else if standing.is_nan() {
            self.best(t, targets, jumps, state, y_rot, heading, max_angle, &found)
        } else {
            found
                .run_up(|missed| self.run_up_to(t, missed, jumps, feet, height))
                .or_else(|| self.best(t, targets, jumps, state, y_rot, heading, max_angle, &found))
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn best<B: SimulationState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        targets: &mut JumpTargets,
        jumps: &mut Jumps<B>,
        state: &PlayerMotion,
        y_rot: f32,
        heading: f32,
        max_angle: f32,
        found: &Landings,
    ) -> Option<LandingRef> {
        let best = found.best.clone()?;
        if !best.bounces || best.takes_off {
            return Some(best);
        }
        let Some(jump) = jumps.last_from_here(t, state, y_rot, best.pos, best.feet) else {
            return Some(best);
        };
        let chained = Arc::new(Landing {
            pos: best.pos,
            feet: best.feet,
            jump,
            step: None,
            bounces: true,
            takes_off: true,
            settles: false,
        });
        let mut landing = state.copy();
        let turn = JumpArc::new(chained.clone()).follow(t, &mut landing, Some(y_rot), jumps);
        let leads_on = chained.is_landed(landing.position(), landing.on_ground)
            && landing.delta_y > 0.0
            && Landings::find(
                t, targets, jumps, &landing, turn, heading, max_angle, best.pos, best.feet,
            )
            .best
            .is_some();
        Some(if leads_on { chained } else { best })
    }
    fn run_up_to<B: SimulationState, L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        missed: &Missed,
        jumps: &mut Jumps<B>,
        feet: BlockPos,
        height: f64,
    ) -> Option<LandingRef> {
        if Some(feet) != self.planned_from {
            self.planned.clear();
            self.planned_from = Some(feet);
        }
        if let Some(target) = self.planned.get(&missed.pos) {
            return target.clone();
        }
        let target = jumps
            .plan(t, feet, height, missed.pos, missed.top, true)
            .map(|jump| {
                let landing = Vec3::new(
                    f64::from(missed.pos.x) + 0.5,
                    missed.top,
                    f64::from(missed.pos.z) + 0.5,
                );
                let step = Step {
                    from: feet,
                    to: missed.pos,
                    movement: Move::Parkour,
                    feet: landing,
                    breaks: Vec::new(),
                    places: Vec::new(),
                    opens: Vec::new(),
                    cost: 0.0,
                    shift: false,
                    edge: false,
                    is_under_water: false,
                    jump: Some(jump.clone()),
                };
                Arc::new(Landing {
                    pos: missed.pos,
                    feet: missed.top,
                    jump,
                    step: Some(step),
                    bounces: false,
                    takes_off: false,
                    settles: false,
                })
            });
        self.planned.insert(missed.pos, target.clone());
        target
    }
    fn settle<B: SimulationState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        jumps: &mut Jumps<B>,
        feet: Option<BlockPos>,
        height: f64,
    ) -> Option<LandingRef> {
        let from = self
            .target
            .as_ref()
            .filter(|it| it.is_landed(state.position(), state.on_ground));
        let pos = from.map(|it| it.pos).or(feet)?;
        let top = from.map_or(height, |it| it.feet);
        let keys = jumps.air_keys(t, state, y_rot, pos, top, false)?;
        let middle = Vec3::new(f64::from(pos.x) + 0.5, top, f64::from(pos.z) + 0.5);
        let jump = Jump {
            run_up: state.position(),
            takeoff: 0,
            jumps: false,
            air_ticks: 0,
            margin: keys.margin,
            hops: 0,
            aim: middle,
        };
        Some(Arc::new(Landing {
            pos,
            feet: top,
            jump,
            step: None,
            bounces: false,
            takes_off: false,
            settles: true,
        }))
    }
    fn untargeted<B: SimulationState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        keys: LeapInput,
    ) -> Option<LeapInput> {
        let mut probe = state.copy();
        let mut leaves = -1;
        let mut lands = false;
        for tick in 0..5 {
            probe.tick(
                t,
                y_rot,
                keys.forward,
                keys.strafe,
                false,
                state.is_sprinting,
                false,
            );
            if !probe.on_ground && leaves < 0 {
                leaves = tick;
            } else if probe.on_ground && leaves >= 0 {
                lands = true;
                break;
            }
        }
        let brakes = self.ticks_since_landing < 10
            && (0..3).contains(&leaves)
            && state.horizontal_speed() > 0.1;
        let drop = state.y - probe.y;
        if brakes {
            Some(LeapInput::new(-1, 0, keys.jump, false, false))
        } else if leaves == 0 && lands && state.delta_y <= 0.0 && drop > 0.0 && drop <= 0.6 {
            Some(LeapInput::new(
                keys.forward,
                keys.strafe,
                false,
                keys.sprint,
                false,
            ))
        } else {
            None
        }
    }
}
