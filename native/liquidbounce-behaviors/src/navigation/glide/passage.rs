// SPDX-License-Identifier: GPL-3.0-or-later
//! Original Passage route projection and model-predictive glide steering.
use super::super::{
    aabb::Aabb,
    flight::FlyRoute,
    geometry::{Vec3, java_max, java_min},
    terrain::{Terrain, TerrainLevel, TerrainState},
    walk_jump::wrap_degrees,
};
use super::{Controls, airway_search::body_box, elytra_motion, landing::Landing, rockets::Rockets};
#[derive(Clone, Copy)]
struct Projection {
    segment: usize,
    arc: f64,
    point: Vec3,
    distance: f64,
}
pub struct Passage {
    target: Vec3,
    pub rockets: Rockets,
    clearance: f64,
    pub lands_at_target: bool,
    points: Vec<Vec3>,
    arcs: Vec<f64>,
    end_arc: f64,
    segment: usize,
    last_x_rot: f32,
    reaches_target: bool,
    passes_target: bool,
    drift: f64,
    remaining: f64,
}
impl Passage {
    pub fn new(target: Vec3, rockets: Rockets, clearance: f64, lands_at_target: bool) -> Self {
        Self {
            target,
            rockets,
            clearance,
            lands_at_target,
            points: Vec::new(),
            arcs: Vec::new(),
            end_arc: 0.0,
            segment: 0,
            last_x_rot: 0.0,
            reaches_target: true,
            passes_target: false,
            drift: 0.0,
            remaining: f64::INFINITY,
        }
    }
    pub fn target(&self) -> Vec3 {
        self.target
    }
    pub fn route(&self) -> &[Vec3] {
        &self.points
    }
    pub fn drift(&self) -> f64 {
        self.drift
    }
    pub fn remaining(&self) -> f64 {
        self.remaining
    }
    pub fn follow(&mut self, route: &FlyRoute, reaches_target: Option<bool>) {
        self.set_points(&route.points, reaches_target.unwrap_or(route.can_reach));
    }
    pub fn steer_with<B: TerrainState, L: TerrainLevel<B>>(
        &mut self,
        terrain: impl FnOnce() -> Terrain<B, L>,
        position: Vec3,
        motion: Vec3,
    ) -> Controls {
        if self.points.is_empty() {
            self.set_points(&[position], true);
        }
        let mut level = terrain();
        let here = self.project(position, self.segment);
        self.segment = here.segment;
        self.drift = here.distance;
        self.remaining = java_max(0.0, self.end_arc - here.arc);
        self.passes_target = self.reaches_target && !self.lands_at_target && self.remaining < 64.0;
        let speed = length(motion);
        let below = here.point.y - position.y;
        let fire = self.rockets.can_fire() && (speed < 0.6 || below > 4.0);
        if fire {
            self.rockets.fire();
        }
        let bearing = bearing(position, self.look_ahead(here, speed));
        let mut best_cost = f64::INFINITY;
        let mut best_y_rot = bearing;
        let mut best_x_rot = 0.0;
        for turn in [-75.0, -45.0, -25.0, -12.0, 0.0, 12.0, 25.0, 45.0, 75.0] {
            for x_rot in [-35.0, -20.0, -8.0, 0.0, 8.0, 18.0, 30.0, 45.0, 60.0] {
                let cost = self.rollout(&mut level, position, motion, here, bearing + turn, x_rot);
                if cost < best_cost {
                    best_cost = cost;
                    best_y_rot = bearing + turn;
                    best_x_rot = x_rot;
                }
            }
        }
        let coarse_y_rot = best_y_rot;
        let coarse_x_rot = best_x_rot;
        for turn in [-5.0, 0.0, 5.0] {
            for pitch in [-4.0, 0.0, 4.0] {
                let y_rot = coarse_y_rot + turn;
                let x_rot = (coarse_x_rot + pitch).clamp(-60.0, 80.0);
                let cost = self.rollout(&mut level, position, motion, here, y_rot, x_rot);
                if cost < best_cost {
                    best_cost = cost;
                    best_y_rot = y_rot;
                    best_x_rot = x_rot;
                }
            }
        }
        self.last_x_rot = best_x_rot;
        Controls {
            y_rot: wrap_degrees(best_y_rot),
            x_rot: best_x_rot,
            should_fire_rocket: fire,
            is_boosted: self.rockets.tick(),
        }
    }
    fn rollout<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        level: &mut Terrain<B, L>,
        start: Vec3,
        mut motion: Vec3,
        here: Projection,
        y_rot: f32,
        x_rot: f32,
    ) -> f64 {
        let mut position = start;
        let mut at = here;
        let turn =
            wrap_degrees(y_rot - bearing(start, self.look_ahead(here, length(motion)))).abs();
        let mut cost = f64::from((x_rot - self.last_x_rot).abs()) * 0.05 + f64::from(turn) * 0.02;
        let mut closest = f64::MAX;
        for tick in 1..=30 {
            let look = if tick <= 6 {
                y_rot
            } else {
                bearing(position, self.look_ahead(at, length(motion)))
            };
            motion = elytra_motion::update(motion, look, x_rot, tick <= self.rockets.boost_ticks());
            let next = add(position, motion);
            if self.passes_target {
                closest = java_min(closest, next.distance_sqr(self.target));
            }
            if !is_clear(
                level,
                body_box(position, self.clearance).expand_towards(motion),
            ) {
                if !is_clear(level, body_box(position, 0.0).expand_towards(motion)) {
                    if self.is_landing(level, position, motion, next) {
                        return cost - 5000.0
                            + horizontal_distance(next, self.target) * 10.0
                            + horizontal_length(motion) * 10.0;
                    }
                    return cost + 100.0 * f64::from(30 - tick + 1);
                }
                cost += 8.0;
            }
            position = next;
            if tick > 6 || tick % 5 == 0 {
                at = self.project(position, at.segment);
            }
            if tick % 5 == 0 {
                cost += self.deviation(position, at) * 1.0;
            }
        }
        cost -= (at.arc - here.arc) * 2.0;
        cost += java_max(0.0, 1.6 - length(motion)) * 10.0;
        if self.passes_target {
            cost += closest.sqrt() * 4.0;
        }
        cost
    }
    fn deviation(&self, position: Vec3, at: Projection) -> f64 {
        let aside = horizontal_distance(position, at.point);
        let below = at.point.y - position.y;
        java_max(0.0, aside - 1.5) + java_max(0.0, below - 3.0)
    }
    fn is_landing<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        level: &mut Terrain<B, L>,
        position: Vec3,
        motion: Vec3,
        next: Vec3,
    ) -> bool {
        if !self.lands_at_target
            || motion.y < Landing::SAFE_SINK
            || horizontal_distance(next, self.target) > 3.0
            || (next.y - self.target.y).abs() > 1.5
        {
            return false;
        }
        let bumped = !level.is_clear(body_box(Vec3::new(next.x, position.y, next.z), 0.0));
        !bumped || horizontal_length(motion) <= 0.3
    }
    fn set_points(&mut self, route: &[Vec3], reaches_target: bool) {
        let mut corners = route.to_vec();
        self.reaches_target = reaches_target;
        if reaches_target
            && corners
                .last()
                .expect("glide route is nonempty")
                .distance_sqr(self.target)
                > 0.5 * 0.5
        {
            corners.push(self.target);
        }
        if corners.len() == 1 {
            corners.push(corners[0]);
        }
        self.arcs = vec![0.0; corners.len() + 1];
        for index in 1..corners.len() {
            self.arcs[index] = self.arcs[index - 1] + corners[index].distance(corners[index - 1]);
        }
        self.end_arc = self.arcs[corners.len() - 1];
        let last = corners[corners.len() - 1];
        let direction = subtract(last, corners[corners.len() - 2]);
        let squared = length_sqr(direction);
        if (!self.lands_at_target || !reaches_target) && squared > 1.0e-6 {
            let length = squared.sqrt();
            corners.push(add(
                last,
                Vec3::new(
                    (direction.x / length) * 64.0,
                    (direction.y / length) * 64.0,
                    (direction.z / length) * 64.0,
                ),
            ));
            self.arcs[corners.len() - 1] = self.end_arc + 64.0;
        } else {
            self.arcs.truncate(corners.len());
        }
        self.points = corners;
        self.segment = 0;
    }
    fn project(&self, position: Vec3, from: usize) -> Projection {
        let mut best: Option<Projection> = None;
        let first = from.saturating_sub(1);
        let last = (self.points.len() - 2).min(from + 24);
        for index in first..=last {
            let a = self.points[index];
            let ab = subtract(self.points[index + 1], a);
            let squared = length_sqr(ab);
            let t = if squared < 1.0e-9 {
                0.0
            } else {
                let ap = subtract(position, a);
                ((ap.x * ab.x + ap.y * ab.y + ap.z * ab.z) / squared).clamp(0.0, 1.0)
            };
            let point = add(a, Vec3::new(ab.x * t, ab.y * t, ab.z * t));
            let distance = point.distance(position);
            if best.is_none_or(|best| distance < best.distance) {
                best = Some(Projection {
                    segment: index,
                    arc: self.arcs[index] + t * (self.arcs[index + 1] - self.arcs[index]),
                    point,
                    distance,
                });
            }
        }
        best.unwrap_or_else(|| Projection {
            segment: 0,
            arc: 0.0,
            point: self.points[0],
            distance: self.points[0].distance(position),
        })
    }
    fn look_ahead(&self, at: Projection, speed: f64) -> Vec3 {
        let arc = at.arc + (speed * 10.0).clamp(6.0, 32.0);
        let mut index = at.segment;
        while index < self.points.len() - 2 && self.arcs[index + 1] < arc {
            index += 1;
        }
        let length = self.arcs[index + 1] - self.arcs[index];
        let t = if length < 1.0e-9 {
            1.0
        } else {
            ((arc - self.arcs[index]) / length).clamp(0.0, 1.0)
        };
        let a = self.points[index];
        let delta = subtract(self.points[index + 1], a);
        add(a, Vec3::new(delta.x * t, delta.y * t, delta.z * t))
    }
}
fn is_clear<B: TerrainState, L: TerrainLevel<B>>(level: &mut Terrain<B, L>, body: Aabb) -> bool {
    level.is_clear(body) && !level.contains_any_liquid(body)
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}
fn subtract(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}
fn length_sqr(v: Vec3) -> f64 {
    v.x * v.x + v.y * v.y + v.z * v.z
}
fn length(v: Vec3) -> f64 {
    length_sqr(v).sqrt()
}
fn horizontal_length(v: Vec3) -> f64 {
    (v.x * v.x + v.z * v.z).sqrt()
}
fn horizontal_distance(a: Vec3, b: Vec3) -> f64 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    (dx * dx + dz * dz).sqrt()
}
fn bearing(from: Vec3, to: Vec3) -> f32 {
    elytra_motion::y_rot_towards(to.x - from.x, to.z - from.z)
}
