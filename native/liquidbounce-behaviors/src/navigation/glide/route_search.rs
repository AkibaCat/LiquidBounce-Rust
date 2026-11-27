// SPDX-License-Identifier: GPL-3.0-or-later
//! Original complete glide RouteSearch, including takeoff and two landing attempts.
use super::super::{
    geometry::{BlockPos, BlockRegion, Vec3, java_max, java_min},
    owned_search::OwnedTerrainSearch,
    search::{Failure, Outcome},
    terrain::{Terrain, TerrainLevel, TerrainState},
};
use super::{
    Steering, abilities::GlideAbilities, elytra_motion, landing::Landing, rockets::Rockets,
};
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    pub start: Vec3,
    pub points: Vec<Vec3>,
    pub landing: Vec3,
    pub rockets: i32,
    pub approach: f64,
}
pub struct RouteSearch<B: TerrainState, L: TerrainLevel<B>> {
    terrain: Terrain<B, L>,
    start: Vec3,
    goal: Vec3,
    radius: f64,
    abilities: GlideAbilities,
    attempt: Attempt,
    result: Outcome<Route>,
}
impl<B: TerrainState, L: TerrainLevel<B>> RouteSearch<B, L> {
    pub fn new(
        terrain: Terrain<B, L>,
        start: Vec3,
        goal: Vec3,
        radius: f64,
        abilities: GlideAbilities,
    ) -> Self {
        Self {
            terrain,
            start,
            goal,
            radius,
            abilities,
            attempt: Attempt::new(start, goal, abilities, 0.0),
            result: Outcome::Searching,
        }
    }
    pub fn advance(&mut self) -> Outcome<Route> {
        for _ in 0..200 {
            if self.result != Outcome::Searching {
                return self.result.clone();
            }
            self.result = self.attempt.tick(
                &mut self.terrain,
                self.start,
                self.goal,
                self.radius,
                self.abilities,
            );
            if matches!(self.result, Outcome::Failed(_)) && self.attempt.approach == 0.0 {
                self.attempt = Attempt::new(self.start, self.goal, self.abilities, 16.0);
                self.result = Outcome::Searching;
            }
        }
        self.result.clone()
    }
    pub fn preview(&self) -> Vec<Vec3> {
        self.attempt.points.clone()
    }
}
impl<B: TerrainState, L: TerrainLevel<B>> OwnedTerrainSearch<Terrain<B, L>> for RouteSearch<B, L> {
    type Route = Route;
    fn terrain_mut(&mut self) -> &mut Terrain<B, L> {
        &mut self.terrain
    }
    fn advance(&mut self) -> Outcome<Route> {
        RouteSearch::advance(self)
    }
    fn preview(&mut self) -> Option<Vec<Vec3>> {
        Some(RouteSearch::preview(self))
    }
}
struct Attempt {
    approach: f64,
    points: Vec<Vec3>,
    control: Landing,
    position: Vec3,
    motion: Vec3,
    tick: i32,
}
impl Attempt {
    fn new(start: Vec3, goal: Vec3, abilities: GlideAbilities, approach: f64) -> Self {
        Self {
            approach,
            points: vec![start],
            control: Landing::new(
                goal,
                Rockets::new(abilities.rockets, abilities.rocket_ticks),
                approach,
            ),
            position: start,
            motion: Vec3::ZERO,
            tick: 0,
        }
    }
    fn tick<B: TerrainState, L: TerrainLevel<B>>(
        &mut self,
        terrain: &mut Terrain<B, L>,
        start: Vec3,
        goal: Vec3,
        radius: f64,
        abilities: GlideAbilities,
    ) -> Outcome<Route> {
        if self.tick >= 2400 || self.tick >= abilities.glide_ticks {
            return Outcome::Failed(Failure::TooComplex);
        }
        let taking_off = self.tick < 2;
        let next = if taking_off {
            self.jump(start, goal)
        } else {
            let input = self.control.steer(self.position, self.motion);
            self.motion =
                elytra_motion::update(self.motion, input.y_rot, input.x_rot, input.is_boosted);
            Vec3::new(
                self.position.x + self.motion.x,
                self.position.y + self.motion.y,
                self.position.z + self.motion.z,
            )
        };
        let height = if taking_off {
            Terrain::<B, L>::PLAYER_HEIGHT
        } else {
            0.6
        };
        if !terrain.is_clear(terrain.body_box_height(next, height)) {
            return self.land(terrain, start, goal, radius, abilities, next);
        }
        if next.y < f64::from(terrain.min_y()) {
            return Outcome::Failed(Failure::NoPath);
        }
        self.position = next;
        self.tick = self.tick.wrapping_add(1);
        if self.tick % 4 == 0 {
            self.points.push(self.position);
        }
        Outcome::Searching
    }
    fn jump(&mut self, start: Vec3, goal: Vec3) -> Vec3 {
        if self.tick == 0 {
            let dx = goal.x - start.x;
            let dz = goal.z - start.z;
            let squared = dx * dx + dz * dz;
            let along = if squared.abs() < f64::from(1.0e-5_f32) {
                Vec3::ZERO
            } else {
                let scale = 0.3 / squared.sqrt();
                Vec3::new(dx * scale, 0.0, dz * scale)
            };
            self.motion = Vec3::new(along.x, f64::from(0.42_f32), along.z);
        }
        let next = Vec3::new(
            self.position.x + self.motion.x,
            self.position.y + self.motion.y,
            self.position.z + self.motion.z,
        );
        self.motion = Vec3::new(
            self.motion.x * f64::from(0.91_f32),
            (self.motion.y - 0.08) * f64::from(0.98_f32),
            self.motion.z * f64::from(0.91_f32),
        );
        next
    }
    fn land<B: TerrainState, L: TerrainLevel<B>>(
        &mut self,
        terrain: &mut Terrain<B, L>,
        start: Vec3,
        goal: Vec3,
        radius: f64,
        abilities: GlideAbilities,
        next: Vec3,
    ) -> Outcome<Route> {
        let bumped = !terrain
            .is_clear(terrain.body_box_height(Vec3::new(next.x, self.position.y, next.z), 0.6));
        if self.tick < 2
            || self.motion.y < Landing::SAFE_SINK
            || bumped
                && (self.motion.x * self.motion.x + self.motion.z * self.motion.z).sqrt() > 0.3
        {
            return Outcome::Failed(Failure::NoPath);
        }
        let spot = if bumped { self.position } else { next };
        let ground = self.ground_under(
            terrain,
            spot,
            if bumped {
                self.position.y - 2.0
            } else {
                next.y
            },
        );
        if ground.is_nan() {
            return Outcome::Failed(Failure::NoPath);
        }
        let landing = Vec3::new(spot.x, ground, spot.z);
        let dx = landing.x - goal.x;
        let dz = landing.z - goal.z;
        let near = (dx * dx + dz * dz).sqrt() <= java_max(radius, 3.0)
            && (landing.y - goal.y).abs() <= 1.0;
        if !near {
            return Outcome::Failed(Failure::NoPath);
        }
        let mut points = self.points.clone();
        points.push(landing);
        Outcome::Found(Route {
            start,
            points,
            landing,
            rockets: abilities.rockets.wrapping_sub(self.control.rockets.left()),
            approach: self.approach,
        })
    }
    fn ground_under<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        terrain: &mut Terrain<B, L>,
        spot: Vec3,
        bottom: f64,
    ) -> f64 {
        let mut ground = f64::NAN;
        let half = Terrain::<B, L>::PLAYER_WIDTH / 2.0;
        for x in (spot.x - half).floor() as i32..=(spot.x + half).floor() as i32 {
            for z in (spot.z - half).floor() as i32..=(spot.z + half).floor() as i32 {
                for y in (bottom.floor() as i32..=self.position.y.floor() as i32).rev() {
                    let height = terrain.stand_height(BlockPos::new(x, y, z));
                    #[allow(clippy::neg_cmp_op_on_partial_ord)]
                    // NaN ground deliberately accepts the first height.
                    if height <= self.position.y && height >= bottom - 0.01 && !(height <= ground) {
                        ground = height;
                    }
                }
            }
        }
        ground
    }
}
pub fn region(start: Vec3, goal: Vec3) -> BlockRegion {
    let dx = start.x - goal.x;
    let dz = start.z - goal.z;
    let margin = (java_min((dx * dx + dz * dz).sqrt(), 48.0) as i32).wrapping_add(2);
    BlockRegion {
        min: BlockPos::new(
            (java_min(start.x, goal.x).floor() as i32).wrapping_sub(margin),
            (java_min(start.y, goal.y).floor() as i32).wrapping_sub(2),
            (java_min(start.z, goal.z).floor() as i32).wrapping_sub(margin),
        ),
        max: BlockPos::new(
            (java_max(start.x, goal.x).floor() as i32).wrapping_add(margin),
            (java_max(start.y, goal.y).floor() as i32).wrapping_add(24),
            (java_max(start.z, goal.z).floor() as i32).wrapping_add(margin),
        ),
    }
}
