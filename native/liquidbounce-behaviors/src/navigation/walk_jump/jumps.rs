// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Jumps.kt: live and planned jumps, retained simulation pools.
use super::{airtime::Airtime, jump::Jump, run_up::RunUp};
use crate::navigation::{
    geometry::{BlockPos, Vec3, java_min},
    simulation::{
        block_grid::SimulationState,
        player_motion::{PlayerMotion, RESTING},
    },
    terrain::{Terrain, TerrainLevel},
    walk::{
        abilities::WalkAbilities,
        course::{Course, CourseMotion},
    },
};
#[derive(Clone, Copy, Debug)]
pub struct AirKeys {
    pub forward: i32,
    pub strafe: i32,
    pub margin: f64,
}
struct Estimate {
    takeoff: i32,
    margin: f64,
    state: PlayerMotion,
}
pub struct Jumps<B: SimulationState> {
    abilities: WalkAbilities<B>,
    min_margin: f64,
    course: Course,
    motion: PlayerMotion,
    flight: Airtime,
    run: RunUp,
    rest: PlayerMotion,
    ground: Vec<PlayerMotion>,
    pool: Vec<PlayerMotion>,
    estimates: Vec<Estimate>,
}
impl<B: SimulationState> Jumps<B> {
    pub fn new(abilities: WalkAbilities<B>) -> Self {
        Self::new_with_holds_y_rot(abilities, false)
    }
    pub fn new_with_holds_y_rot(abilities: WalkAbilities<B>, holds_y_rot: bool) -> Self {
        let motion = PlayerMotion::new(abilities.motion);
        let rest = motion.copy();
        Self {
            min_margin: if abilities.risky_jumps { 0.15 } else { 0.35 },
            course: Course::new(abilities.safe_fall),
            flight: Airtime::new(abilities.safe_fall, abilities.risky_jumps, holds_y_rot),
            abilities,
            motion,
            rest,
            run: RunUp::default(),
            ground: Vec::new(),
            pool: Vec::new(),
            estimates: Vec::new(),
        }
    }
    pub fn can_sprint(&self) -> bool {
        self.abilities.can_sprint
    }
    pub fn plan<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        from_height: f64,
        to: BlockPos,
        to_height: f64,
        around: bool,
    ) -> Option<Jump> {
        self.course.set(
            t,
            Vec3::new(
                f64::from(from.x) + 0.5,
                from_height,
                f64::from(from.z) + 0.5,
            ),
            to,
            to_height,
        );
        if !self.abilities.risky_jumps && !self.course.is_safe(t) {
            return None;
        }
        let center = self.course.behind(0.0);
        let mut best = self.from_rest(t, center, 0);
        if !has(&best, 0.45) {
            let longest = self.course.run_up_length(t, 1.6);
            let mut back = 0.3;
            while back < longest + 0.3 && !has(&best, 0.45) {
                best = better(
                    best,
                    self.from_rest(t, self.course.behind(java_min(back, longest)), 0),
                );
                back += 0.3;
            }
        }
        if !has(&best, self.min_margin) && self.course.distance <= 6.2 {
            best = better(best, self.hopped(t));
        }
        for offset in [0.3, -0.3] {
            if has(&best, self.min_margin) {
                break;
            }
            self.course.aim_beside(offset);
            best = better(best, self.from_rest(t, center, 0));
        }
        if around && self.course.obstructed && !has(&best, self.min_margin) {
            best = better(best, self.around(t, from));
        }
        best.filter(|j| j.margin >= self.min_margin)
    }
    pub fn refine<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        from_height: f64,
        to: BlockPos,
        to_height: f64,
    ) -> Option<Jump> {
        self.course.set(
            t,
            Vec3::new(
                f64::from(from.x) + 0.5,
                from_height,
                f64::from(from.z) + 0.5,
            ),
            to,
            to_height,
        );
        let longest = self.course.run_up_length(t, 1.6);
        let mut best = None;
        let mut back = 0.0;
        while back <= longest {
            best = better(best, self.from_rest(t, self.course.behind(back), 0));
            back += Course::RUN_UP_STEP;
        }
        best.filter(|j| j.margin >= self.min_margin)
    }
    pub fn from_here<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        to: BlockPos,
        to_height: f64,
        aim: Option<Vec3>,
    ) -> Option<Jump> {
        self.course.set(t, state.position(), to, to_height);
        let here = state.position();
        let mut best = self.attempt(t, state, here, Some(y_rot), 0, i32::MAX);
        for offset in [0.3, -0.3] {
            self.course.aim_beside(offset);
            best = better(best, self.attempt(t, state, here, Some(y_rot), 0, i32::MAX));
        }
        if let Some(aim) = aim {
            self.course.aim_x = aim.x;
            self.course.aim_z = aim.z;
            best = better(best, self.attempt(t, state, here, Some(y_rot), 0, i32::MAX));
        }
        best
    }
    pub fn taking_off_now<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        to: BlockPos,
        to_height: f64,
    ) -> Option<Jump> {
        self.course.set(t, state.position(), to, to_height);
        self.motion.copy_from(state);
        self.run_tick(t, y_rot, true);
        if self.motion.on_ground {
            return None;
        }
        self.flight
            .jump(
                t,
                &mut self.course,
                &mut self.motion,
                self.abilities.can_sprint,
                state.position(),
                0,
                true,
                0,
            )
            .filter(|j| j.margin >= 0.45)
    }
    pub fn last_from_here<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        to: BlockPos,
        to_height: f64,
    ) -> Option<Jump> {
        self.course.set(t, state.position(), to, to_height);
        self.pool.append(&mut self.ground);
        self.motion.copy_from(state);
        self.run_to_edge(t, Some(y_rot));
        for takeoff in (0..self.ground.len()).rev() {
            self.motion.copy_from(&self.ground[takeoff]);
            self.run_tick(
                t,
                RunUp::y_rot_at(&self.course, &self.motion, takeoff as i32, Some(y_rot)),
                true,
            );
            let jump = if self.motion.on_ground {
                None
            } else {
                self.flight.jump(
                    t,
                    &mut self.course,
                    &mut self.motion,
                    self.abilities.can_sprint,
                    state.position(),
                    takeoff as i32,
                    true,
                    0,
                )
            };
            if jump.as_ref().is_some_and(|j| j.margin >= self.min_margin) {
                return jump;
            }
        }
        None
    }
    pub fn air_keys<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        y_rot: f32,
        to: BlockPos,
        to_height: f64,
        keep_speed: bool,
    ) -> Option<AirKeys> {
        self.course.set(t, state.position(), to, to_height);
        let mut best: Option<AirKeys> = None;
        for forward in (-1..=1).rev() {
            for strafe in [0, 1, -1] {
                self.motion.copy_from(state);
                self.motion.tick(
                    t,
                    y_rot,
                    forward,
                    strafe,
                    false,
                    self.abilities.can_sprint,
                    false,
                );
                let margin = self.flight.land(
                    t,
                    &mut self.course,
                    &mut self.motion,
                    self.abilities.can_sprint,
                );
                if keep_speed && margin >= self.min_margin {
                    return Some(AirKeys {
                        forward,
                        strafe,
                        margin,
                    });
                }
                if !margin.is_nan() && best.as_ref().is_none_or(|b| margin > b.margin + 0.02) {
                    best = Some(AirKeys {
                        forward,
                        strafe,
                        margin,
                    });
                }
            }
        }
        best
    }
    #[allow(clippy::too_many_arguments)]
    pub fn take_off<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &mut PlayerMotion,
        y_rot: Option<f32>,
        to: BlockPos,
        to_height: f64,
        jump: &Jump,
        path: &mut Vec<Vec3>,
    ) -> bool {
        self.course.set(t, state.position(), to, to_height);
        self.course.aim_x = jump.aim.x;
        self.course.aim_z = jump.aim.z;
        self.motion.copy_from(state);
        self.run.path = Some(Vec::new());
        let hop_ticks = self.run.hop(
            t,
            &mut self.course,
            &mut self.motion,
            self.abilities.can_sprint,
            jump.hops,
        );
        let success = hop_ticks >= 0;
        if success {
            let ground_ticks = jump.takeoff - hop_ticks - if jump.jumps { 0 } else { 1 };
            for tick in 0..ground_ticks {
                self.run_tick(
                    t,
                    RunUp::y_rot_at(&self.course, &self.motion, tick, y_rot),
                    false,
                );
            }
            self.run_tick(
                t,
                RunUp::y_rot_at(&self.course, &self.motion, ground_ticks, y_rot),
                jump.jumps,
            );
            state.copy_from(&self.motion);
        }
        if let Some(mut added) = self.run.path.take() {
            path.append(&mut added);
        }
        success
    }
    fn around<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
    ) -> Option<Jump> {
        let mut best = None;
        for spot in around_spots() {
            let run_up = Vec3::new(
                f64::from(from.x) + spot.x,
                self.course.from_y,
                f64::from(from.z) + spot.z,
            );
            if !self.course.can_stand_at(t, run_up) {
                continue;
            }
            for offset in [0.0, 0.3, -0.3, 0.7, -0.7, 1.2, -1.2] {
                self.course.aim_beside(offset);
                if self.course.is_way_clear(t, run_up) {
                    best = better(best, self.from_rest(t, run_up, 0));
                    if has(&best, 0.45) {
                        return best;
                    }
                }
            }
        }
        best
    }
    fn hopped<L: TerrainLevel<B>>(&mut self, t: &mut Terrain<B, L>) -> Option<Jump> {
        let runway = self.course.run_up_length(t, 8.0);
        self.course.edge = self.course.edge_length(t);
        for hops in 1..=2 {
            let travel = self.run.hop_travel(
                t,
                &mut self.course,
                &mut self.motion,
                self.abilities.can_sprint,
                runway,
                hops,
            );
            let back = travel + 0.3 - self.course.edge;
            if travel.is_nan() || back > runway {
                return None;
            }
            let jump = self.from_rest(t, self.course.behind(back), hops);
            if has(&jump, self.min_margin) {
                return jump;
            }
        }
        None
    }
    #[allow(clippy::wrong_self_convention)] // Name retained from Agent fromRest.
    fn from_rest<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        run_up: Vec3,
        hops: i32,
    ) -> Option<Jump> {
        // The rest object is only reset, never ticked; its sneaking stays false.
        // The copy keeps the collider cache shared while self is borrowed.
        self.rest.reset(t, run_up, RESTING, true, false, 0);
        let state = self.rest.copy();
        self.attempt(t, &state, run_up, None, hops, 4)
    }
    fn attempt<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        state: &PlayerMotion,
        run_up: Vec3,
        first_y_rot: Option<f32>,
        hops: i32,
        validations: i32,
    ) -> Option<Jump> {
        self.pool.append(&mut self.ground);
        self.motion.copy_from(state);
        let hop_ticks = self.run.hop(
            t,
            &mut self.course,
            &mut self.motion,
            self.abilities.can_sprint,
            hops,
        );
        if hop_ticks < 0 {
            return None;
        }
        let mut best = None;
        if self.run_to_edge(t, first_y_rot)
            && (self.course.target_height < self.motion.y || self.motion.delta_y > 0.0)
        {
            best = self.flight.jump(
                t,
                &mut self.course,
                &mut self.motion,
                self.abilities.can_sprint,
                run_up,
                hop_ticks + self.ground.len() as i32,
                false,
                hops,
            );
        }
        self.estimate_takeoffs(t, first_y_rot);
        for i in 0..self.estimates.len().min(validations.max(0) as usize) {
            self.motion.copy_from(&self.estimates[i].state);
            let takeoff = self.estimates[i].takeoff;
            if let Some(jump) = self.flight.jump(
                t,
                &mut self.course,
                &mut self.motion,
                self.abilities.can_sprint,
                run_up,
                hop_ticks + takeoff,
                true,
                hops,
            ) {
                let good = jump.margin >= 0.45;
                best = better(best, Some(jump));
                if good {
                    break;
                }
            }
        }
        // Original keeps estimates referencing the returned pool objects until
        // the next estimateTakeoffs clears them. Rust moves them after use.
        self.pool.extend(self.estimates.drain(..).map(|e| e.state));
        best
    }
    fn run_to_edge<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        first_y_rot: Option<f32>,
    ) -> bool {
        for tick in 0..16 {
            let snapshot = self.snapshot();
            self.ground.push(snapshot);
            self.run_tick(
                t,
                RunUp::y_rot_at(&self.course, &self.motion, tick, first_y_rot),
                false,
            );
            if self.motion.horizontal_collision && self.motion.horizontal_speed() < 0.01 {
                return false;
            }
            if !self.motion.on_ground {
                return true;
            }
        }
        false
    }
    fn estimate_takeoffs<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        first_y_rot: Option<f32>,
    ) {
        self.estimates.clear();
        for takeoff in 0..self.ground.len() {
            self.motion.copy_from(&self.ground[takeoff]);
            self.run_tick(
                t,
                RunUp::y_rot_at(&self.course, &self.motion, takeoff as i32, first_y_rot),
                true,
            );
            let margin = if self.motion.on_ground {
                f64::NAN
            } else {
                self.course.estimate(CourseMotion {
                    position: self.motion.position(),
                    delta: self.motion.delta_movement(),
                    is_sprinting: self.motion.is_sprinting,
                })
            };
            if !margin.is_nan() {
                let state = self.snapshot();
                self.estimates.push(Estimate {
                    takeoff: takeoff as i32,
                    margin,
                    state,
                });
            }
        }
        self.estimates.sort_by(|a, b| b.margin.total_cmp(&a.margin));
    }
    fn snapshot(&mut self) -> PlayerMotion {
        if let Some(mut state) = self.pool.pop() {
            state.copy_from(&self.motion);
            state
        } else {
            self.motion.copy()
        }
    }
    fn run_tick<L: TerrainLevel<B>>(&mut self, t: &mut Terrain<B, L>, y_rot: f32, jump: bool) {
        self.run.tick(
            t,
            &mut self.course,
            &mut self.motion,
            self.abilities.can_sprint,
            y_rot,
            jump,
        );
    }
}
fn better(a: Option<Jump>, b: Option<Jump>) -> Option<Jump> {
    if a.is_none()
        || b.as_ref()
            .is_some_and(|b| b.margin > a.as_ref().expect("a exists").margin)
    {
        b
    } else {
        a
    }
}
fn has(jump: &Option<Jump>, margin: f64) -> bool {
    jump.as_ref().is_some_and(|j| j.margin >= margin)
}
fn around_spots() -> Vec<Vec3> {
    let mut spots = Vec::new();
    for x in [-0.25, 0.1, 0.5, 0.9, 1.25] {
        for z in [-0.25, 0.1, 0.5, 0.9, 1.25] {
            if x != 0.5 || z != 0.5 {
                spots.push(Vec3::new(x, 0.0, z));
            }
        }
    }
    spots.sort_by(|a, b| {
        let ad = (a.x - 0.5) * (a.x - 0.5) + (a.z - 0.5) * (a.z - 0.5);
        let bd = (b.x - 0.5) * (b.x - 0.5) + (b.z - 0.5) * (b.z - 0.5);
        ad.total_cmp(&bd)
    });
    spots
}
