// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Course.kt. Physics input is a value capture, and world
//! operations receive Terrain explicitly so Jumps can own all its work state.
use crate::navigation::{
    aabb::Aabb,
    geometry::{BlockPos, Vec3, java_min},
    terrain::{Terrain, TerrainLevel, TerrainState},
};

#[derive(Clone, Copy, Debug)]
pub struct CourseMotion {
    pub position: Vec3,
    pub delta: Vec3,
    pub is_sprinting: bool,
}
#[derive(Clone, Debug)]
pub struct Course {
    pub from_x: f64,
    pub from_y: f64,
    pub from_z: f64,
    pub target: BlockPos,
    pub target_x: f64,
    pub target_z: f64,
    pub target_height: f64,
    pub catches: bool,
    pub dir_x: f64,
    pub dir_z: f64,
    pub distance: f64,
    pub aim_x: f64,
    pub aim_z: f64,
    pub edge: f64,
    pub obstructed: bool,
    safe_fall: i32,
}
impl Course {
    pub const RUN_UP_STEP: f64 = 0.1;
    pub const MAX_AIR_TICKS: i32 = 60;
    pub const MAX_BELOW: f64 = 4.0;
    pub const LANDING_PROBE: f64 = 1.0e-4;
    pub const HALF_BLOCK: f64 = 0.5;
    pub const AIR_ACCELERATION: f64 = (0.02_f32 * 0.98_f32) as f64;
    pub const SPRINT_AIR_ACCELERATION: f64 = (0.025999999_f32 * 0.98_f32) as f64;
    pub fn new(safe_fall: i32) -> Self {
        Self {
            from_x: 0.0,
            from_y: 0.0,
            from_z: 0.0,
            target: BlockPos::default(),
            target_x: 0.0,
            target_z: 0.0,
            target_height: 0.0,
            catches: false,
            dir_x: 0.0,
            dir_z: 0.0,
            distance: 0.0,
            aim_x: 0.0,
            aim_z: 0.0,
            edge: f64::MAX,
            obstructed: false,
            safe_fall,
        }
    }
    pub fn set<B: TerrainState, L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        from: Vec3,
        to: BlockPos,
        to_height: f64,
    ) {
        self.from_x = from.x;
        self.from_y = from.y;
        self.from_z = from.z;
        self.target = to;
        self.target_x = f64::from(to.x) + 0.5;
        self.target_z = f64::from(to.z) + 0.5;
        self.target_height = to_height;
        self.catches = t.is_climbable(to);
        self.aim_x = self.target_x;
        self.aim_z = self.target_z;
        self.edge = f64::MAX;
        self.obstructed = false;
        self.dir_x = self.target_x - from.x;
        self.dir_z = self.target_z - from.z;
        self.distance = length(self.dir_x, self.dir_z);
        if self.distance > 1.0e-6 {
            self.dir_x /= self.distance;
            self.dir_z /= self.distance;
        }
    }
    pub fn aim_beside(&mut self, offset: f64) {
        self.aim_x = self.target_x - self.dir_z * offset;
        self.aim_z = self.target_z + self.dir_x * offset;
    }
    pub fn behind(&self, distance: f64) -> Vec3 {
        Vec3::new(
            self.from_x - self.dir_x * distance,
            self.from_y,
            self.from_z - self.dir_z * distance,
        )
    }
    pub fn is_way_clear<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        run_up: Vec3,
    ) -> bool {
        let samples = (length(self.aim_x - run_up.x, self.aim_z - run_up.z) / 0.25).ceil() as i32;
        for sample in 1..samples {
            let along = f64::from(sample) / f64::from(samples);
            let y = lerp(along, self.from_y, self.target_height) + 0.5;
            let b = t.body_box(Vec3::new(
                lerp(along, run_up.x, self.aim_x),
                y,
                lerp(along, run_up.z, self.aim_z),
            ));
            if !t.is_clear(Aabb::new(b.min, Vec3::new(b.max.x, b.max.y - 0.5, b.max.z))) {
                return false;
            }
        }
        true
    }
    pub fn run_up_length<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        max: f64,
    ) -> f64 {
        let mut distance = 0.0;
        while distance + Self::RUN_UP_STEP <= max
            && self.can_stand_at(t, self.behind(distance + Self::RUN_UP_STEP))
        {
            distance += Self::RUN_UP_STEP;
        }
        distance
    }
    pub fn can_stand_at<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        feet: Vec3,
    ) -> bool {
        t.is_supported(feet) && t.is_clear(t.body_box(feet))
    }
    pub fn edge_length<B: TerrainState, L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>) -> f64 {
        let mut distance = 0.0;
        while distance + Self::RUN_UP_STEP <= 1.5
            && t.is_supported(self.behind(-(distance + Self::RUN_UP_STEP)))
        {
            distance += Self::RUN_UP_STEP;
        }
        distance
    }
    pub fn estimate(&self, motion: CourseMotion) -> f64 {
        let Vec3 {
            mut x,
            mut y,
            mut z,
        } = motion.position;
        let Vec3 {
            x: mut dx,
            y: mut dy,
            z: mut dz,
        } = motion.delta;
        let acceleration = if motion.is_sprinting {
            Self::SPRINT_AIR_ACCELERATION
        } else {
            Self::AIR_ACCELERATION
        };
        for _ in 0..Self::MAX_AIR_TICKS {
            let tx = self.target_x - x;
            let tz = self.target_z - z;
            let len = length(tx, tz);
            if len > 1.0e-4 {
                let speed = (dx * tx + dz * tz) / len;
                let push = if self.catches || !self.overshoots(len, speed, y, dy) {
                    acceleration
                } else {
                    -Self::AIR_ACCELERATION
                };
                dx += tx / len * push;
                dz += tz / len * push;
            }
            let next_y = y + dy;
            if !self.catches
                && dy < 0.0
                && next_y <= self.target_height
                && y >= self.target_height - Self::LANDING_PROBE
            {
                return if self.is_over(x, z) {
                    self.margin(x + dx, z + dz)
                } else {
                    f64::NAN
                };
            }
            x += dx;
            y = next_y;
            z += dz;
            if self.catches && self.is_in_target(x, y, z) {
                return self.margin(x, z);
            }
            let hits_side = dy <= 0.0 && y < self.target_height && self.is_over(x, z);
            if hits_side || y < self.target_height - Self::MAX_BELOW {
                return f64::NAN;
            }
            dx *= f64::from(0.91_f32);
            dy = (dy - 0.08) * f64::from(0.98_f32);
            dz *= f64::from(0.91_f32);
        }
        f64::NAN
    }
    pub fn is_over(&self, x: f64, z: f64) -> bool {
        let half = f64::from(0.6_f32) / 2.0;
        x > f64::from(self.target.x) - half
            && x < f64::from(self.target.x) + 1.0 + half
            && z > f64::from(self.target.z) - half
            && z < f64::from(self.target.z) + 1.0 + half
    }
    pub fn is_in_target(&self, x: f64, y: f64, z: f64) -> bool {
        x.floor() as i32 == self.target.x
            && y.floor() as i32 == self.target.y
            && z.floor() as i32 == self.target.z
    }
    pub fn margin(&self, x: f64, z: f64) -> f64 {
        java_min(
            java_min(
                x - f64::from(self.target.x),
                f64::from(self.target.x) + 1.0 - x,
            ),
            java_min(
                z - f64::from(self.target.z),
                f64::from(self.target.z) + 1.0 - z,
            ),
        )
    }
    pub fn overshoots(&self, remaining: f64, speed: f64, height: f64, delta_y: f64) -> bool {
        let (mut forward, mut y, mut vertical, mut travel) = (speed, height, delta_y, 0.0);
        for _ in 0..Self::MAX_AIR_TICKS {
            forward += Self::SPRINT_AIR_ACCELERATION;
            travel += forward;
            y += vertical;
            if travel > remaining + 0.25 {
                return true;
            }
            if vertical < 0.0 && y <= self.target_height {
                return false;
            }
            forward *= f64::from(0.91_f32);
            vertical = (vertical - 0.08) * f64::from(0.98_f32);
        }
        false
    }
    pub fn is_safe<B: TerrainState, L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>) -> bool {
        let mut travelled = Self::HALF_BLOCK;
        while travelled < self.distance - Self::HALF_BLOCK {
            let x = (self.from_x + self.dir_x * travelled).floor() as i32;
            let z = (self.from_z + self.dir_z * travelled).floor() as i32;
            if !self.catches_fall(t, x, self.from_y.floor() as i32, z) {
                return false;
            }
            travelled += 0.5;
        }
        true
    }
    fn catches_fall<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        x: i32,
        top: i32,
        z: i32,
    ) -> bool {
        for y in (top
            .wrapping_sub(self.safe_fall)
            .wrapping_sub(1)
            .max(t.min_y())..=top)
            .rev()
        {
            let p = BlockPos::new(x, y, z);
            let stand = t.stand_height(p);
            if t.is_water(p) {
                return true;
            }
            if !stand.is_nan() {
                return self.from_y - stand <= f64::from(self.safe_fall);
            }
            if !t.is_passable(p) {
                return true;
            }
        }
        false
    }
}
fn length(x: f64, z: f64) -> f64 {
    (x * x + z * z).sqrt()
}
fn lerp(alpha: f64, from: f64, to: f64) -> f64 {
    from + alpha * (to - from)
}
