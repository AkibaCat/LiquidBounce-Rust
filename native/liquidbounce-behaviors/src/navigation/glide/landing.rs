// SPDX-License-Identifier: GPL-3.0-or-later
//! Original Landing pitch sweep and safe touchdown prediction.
use super::super::{geometry::Vec3, walk_jump::wrap_degrees};
use super::{Controls, Steering, elytra_motion, rockets::Rockets};
#[derive(Clone, Copy)]
struct Sweep {
    best_error: f64,
    best_x_rot: f32,
    best_along: f64,
    farthest_x_rot: f32,
    farthest_along: f64,
}
impl Default for Sweep {
    fn default() -> Self {
        Self {
            best_error: f64::INFINITY,
            best_x_rot: 0.0,
            best_along: f64::NEG_INFINITY,
            farthest_x_rot: 0.0,
            farthest_along: f64::NEG_INFINITY,
        }
    }
}
pub struct Landing {
    target: Vec3,
    pub rockets: Rockets,
    approach: f64,
    sweep: Sweep,
}
impl Landing {
    pub const SAFE_SINK: f64 = -0.45;
    pub fn new(target: Vec3, rockets: Rockets, approach: f64) -> Self {
        Self {
            target,
            rockets,
            approach,
            sweep: Sweep::default(),
        }
    }
    fn sweep_x_rots(
        &mut self,
        position: Vec3,
        motion: Vec3,
        bearing: f32,
        distance: f64,
        dx: f64,
        dz: f64,
    ) {
        let unit_x = if distance > 1.0e-6 {
            dx / distance
        } else {
            0.0
        };
        let unit_z = if distance > 1.0e-6 {
            dz / distance
        } else {
            1.0
        };
        self.sweep = Sweep::default();
        for x_rot in [
            -45.0, -35.0, -25.0, -15.0, -10.0, -5.0, 0.0, 5.0, 10.0, 20.0,
        ] {
            let Some(touchdown) = self.predict(position, motion, bearing, x_rot) else {
                continue;
            };
            let along =
                (touchdown.x - self.target.x) * unit_x + (touchdown.z - self.target.z) * unit_z;
            let mut error = (along + 1.0).abs();
            if touchdown.y < Self::SAFE_SINK {
                error += 100.0;
            }
            if error < self.sweep.best_error {
                self.sweep.best_error = error;
                self.sweep.best_x_rot = x_rot;
                self.sweep.best_along = along;
            }
            if along > self.sweep.farthest_along {
                self.sweep.farthest_along = along;
                self.sweep.farthest_x_rot = x_rot;
            }
        }
    }
    fn predict(&self, position: Vec3, mut motion: Vec3, y_rot: f32, x_rot: f32) -> Option<Vec3> {
        let mut x = position.x;
        let mut y = position.y;
        let mut z = position.z;
        for tick in 0..400 {
            motion = elytra_motion::update(motion, y_rot, x_rot, tick < self.rockets.boost_ticks());
            x += motion.x;
            y += motion.y;
            z += motion.z;
            if y <= self.target.y && motion.y < 0.0 {
                return Some(Vec3::new(x, motion.y, z));
            }
        }
        None
    }
}
impl Steering for Landing {
    fn target(&self) -> Vec3 {
        self.target
    }
    fn steer(&mut self, position: Vec3, motion: Vec3) -> Controls {
        let dx = self.target.x - position.x;
        let dz = self.target.z - position.z;
        let distance = (dx * dx + dz * dz).sqrt();
        let bearing = elytra_motion::y_rot_towards(dx, dz);
        self.sweep_x_rots(position, motion, bearing, distance, dx, dz);
        if self.approach > 0.0
            && distance > self.approach
            && self.rockets.boost_ticks() == 0
            && self.sweep.farthest_along >= 0.0
        {
            return Controls {
                y_rot: bearing,
                x_rot: self.sweep.farthest_x_rot,
                should_fire_rocket: false,
                is_boosted: false,
            };
        }
        let overhead = self.sweep.best_error == f64::INFINITY && position.y > self.target.y;
        let missed = self.sweep.best_error > 3.0;
        let fire = missed && !overhead && self.sweep.best_along < 0.0 && self.rockets.can_fire();
        if fire {
            self.rockets.fire();
        }
        let boosted = self.rockets.tick();
        let orbit = wrap_degrees(bearing + 90.0);
        let (y_rot, x_rot) = if boosted {
            let climb = self.target.y + 4.0 + distance * 0.15 - position.y;
            (
                bearing,
                elytra_motion::x_rot_towards(climb, distance).clamp(-40.0, 10.0),
            )
        } else if overhead {
            (orbit, 30.0)
        } else if missed && self.sweep.best_along > 0.0 {
            (orbit, -45.0)
        } else {
            (bearing, self.sweep.best_x_rot)
        };
        Controls {
            y_rot,
            x_rot,
            should_fire_rocket: fire,
            is_boosted: boosted,
        }
    }
}
