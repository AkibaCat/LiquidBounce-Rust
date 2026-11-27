// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent PlayerMotion land simulation; it intentionally has no fluids,
//! climbing or slow falling. SafeWalk's vanilla SimulatedPlayer is separate.
use super::super::{
    geometry::{BlockPos, Vec3, java_max},
    terrain::TerrainState,
    walk::WalkState,
};
use super::{
    Mechanics,
    block_colliders::BlockColliders,
    block_grid::{BlockGrid, SimulationState},
    collisions,
    edge_backoff::EdgeBackOff,
    motion_traits::MotionTraits,
    sine_table,
    support::Support,
};
use std::sync::{Arc, Mutex};

pub const HALF_WIDTH: f64 = 0.6_f32 as f64 / 2.0;
pub const HEIGHT: f64 = 1.8_f32 as f64;
pub const STEP_HEIGHT: f32 = 0.6;
pub const AIR_SPEED: f32 = 0.02;
pub const SPRINT_AIR_SPEED: f32 = 0.025999999;
pub const PLAIN_FRICTION: f32 = 0.6;
pub const INPUT_FRICTION: f32 = 0.98;
pub const RESTING: Vec3 = Vec3::new(0.0, -0.08 * (0.98_f32 as f64), 0.0);

/// Read from the real player on its game thread, in the original getter order.
pub trait MotionPlayer {
    fn position(&self) -> Vec3;
    fn delta_movement(&self) -> Vec3;
    fn on_ground(&self) -> bool;
    fn is_sprinting(&self) -> bool;
    fn jump_delay(&self) -> i32;
    fn is_crouching(&self) -> bool;
    fn supporting_block(&self) -> Option<BlockPos>;
}

#[derive(Debug)]
pub struct PlayerMotion {
    pub traits: MotionTraits,
    pub(crate) colliders: Arc<Mutex<BlockColliders>>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub is_sprinting: bool,
    pub landing_fall: f64,
    pub peak_y: f64,
    pub jump_delay: i32,
    pub support: Support,
    pub ground_without_blocks: bool,
    pub crouching: bool,
    pub sneaking: bool,
    pub air_sprint: bool,
    pub walked: f32,
    pub next_step: i32,
}
impl Default for PlayerMotion {
    fn default() -> Self {
        Self::new(MotionTraits::default())
    }
}
impl Clone for PlayerMotion {
    fn clone(&self) -> Self {
        self.copy()
    }
}
impl PlayerMotion {
    pub fn new(traits: MotionTraits) -> Self {
        Self {
            traits,
            colliders: Arc::new(Mutex::new(BlockColliders::default())),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            delta_x: 0.0,
            delta_y: 0.0,
            delta_z: 0.0,
            on_ground: false,
            horizontal_collision: false,
            is_sprinting: false,
            landing_fall: 0.0,
            peak_y: 0.0,
            jump_delay: 0,
            support: Support::default(),
            ground_without_blocks: false,
            crouching: false,
            sneaking: false,
            air_sprint: false,
            walked: 0.0,
            next_step: 0,
        }
    }
    pub fn position(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
    pub fn delta_movement(&self) -> Vec3 {
        Vec3::new(self.delta_x, self.delta_y, self.delta_z)
    }
    pub fn horizontal_speed(&self) -> f64 {
        (self.delta_x * self.delta_x + self.delta_z * self.delta_z).sqrt()
    }
    pub fn supporting_block(&self) -> Option<BlockPos> {
        self.support
            .exists
            .then(|| BlockPos::new(self.support.x, self.support.y, self.support.z))
    }
    pub fn block_position(&self) -> BlockPos {
        let feet = BlockPos::containing(Vec3::new(self.x, self.y + 1.0e-3, self.z));
        if self.support.exists {
            BlockPos::new(self.support.x, feet.y, self.support.z)
        } else {
            feet
        }
    }
    pub fn reset(
        &mut self,
        grid: &mut impl BlockGrid,
        position: Vec3,
        motion: Vec3,
        on_ground: bool,
        is_sprinting: bool,
        jump_delay: i32,
    ) {
        self.x = position.x;
        self.y = position.y;
        self.z = position.z;
        self.delta_x = motion.x;
        self.delta_y = motion.y;
        self.delta_z = motion.z;
        self.on_ground = on_ground;
        self.is_sprinting = is_sprinting;
        self.jump_delay = jump_delay;
        self.horizontal_collision = false;
        self.crouching = false;
        self.peak_y = self.y;
        if on_ground {
            self.find_support(grid, 0.0, 0.0);
        } else {
            self.support.clear();
        }
        self.ground_without_blocks = on_ground && !self.support.exists;
        self.air_sprint = is_sprinting;
        self.walked = 0.0;
        self.next_step = 0;
    }
    pub fn reset_player(&mut self, grid: &mut impl BlockGrid, player: &impl MotionPlayer) {
        self.reset(
            grid,
            player.position(),
            player.delta_movement(),
            player.on_ground(),
            player.is_sprinting(),
            player.jump_delay(),
        );
        self.crouching = player.is_crouching();
        if let Some(support) = player.supporting_block() {
            self.support.set(support.x, support.y, support.z);
        }
    }
    /// The source copy shares both terrain and its collision cache. World input
    /// remains explicit; the shared cache is kept by all copies in this group.
    pub fn copy(&self) -> Self {
        let mut copy = Self::new(self.traits);
        copy.colliders = Arc::clone(&self.colliders);
        copy.copy_from(self);
        copy
    }
    /// The original copyFrom deliberately leaves sneaking and the target's
    /// traits/collision cache alone; copy() starts sneaking at false.
    pub fn copy_from(&mut self, other: &Self) {
        self.x = other.x;
        self.y = other.y;
        self.z = other.z;
        self.delta_x = other.delta_x;
        self.delta_y = other.delta_y;
        self.delta_z = other.delta_z;
        self.on_ground = other.on_ground;
        self.horizontal_collision = other.horizontal_collision;
        self.is_sprinting = other.is_sprinting;
        self.landing_fall = other.landing_fall;
        self.peak_y = other.peak_y;
        self.jump_delay = other.jump_delay;
        self.support = other.support;
        self.ground_without_blocks = other.ground_without_blocks;
        self.crouching = other.crouching;
        self.air_sprint = other.air_sprint;
        self.walked = other.walked;
        self.next_step = other.next_step;
    }
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        grid: &mut impl BlockGrid,
        yaw: f32,
        forward: i32,
        strafe: i32,
        jump: bool,
        sprint: bool,
        sneak: bool,
    ) {
        if self.traits.mechanics == Mechanics::V1_8 {
            self.tick_1_8(grid, yaw, forward, strafe, jump, sprint, sneak);
            return;
        }
        if self.jump_delay > 0 {
            self.jump_delay -= 1;
        }
        if self.delta_x * self.delta_x + self.delta_z * self.delta_z < 9.0e-6 {
            self.delta_x = 0.0;
            self.delta_z = 0.0;
        }
        if self.delta_y.abs() < 0.003 {
            self.delta_y = 0.0;
        }
        let slow = self.crouching;
        self.crouching = sneak;
        self.sneaking = sneak;
        self.is_sprinting =
            sprint && forward > 0 && !self.horizontal_collision && (self.is_sprinting || !slow);
        if !jump {
            self.jump_delay = 0;
        } else if self.on_ground && self.jump_delay == 0 {
            self.jump_from_ground(grid, yaw);
            self.jump_delay = 10;
        }
        self.travel(
            grid,
            yaw,
            forward,
            strafe,
            if slow {
                INPUT_FRICTION * 0.3_f32
            } else {
                INPUT_FRICTION
            },
        );
    }
    fn jump_from_ground(&mut self, grid: &mut impl BlockGrid, yaw: f32) {
        let jump_factor = self.jump_factor(grid);
        self.delta_y = java_max(
            f64::from(self.traits.jump_strength * jump_factor + self.traits.jump_boost_power),
            self.delta_y,
        );
        if self.is_sprinting {
            let angle = sine_table::radians(yaw);
            self.delta_x += -f64::from(sine_table::sin(angle)) * f64::from(0.2_f32);
            self.delta_z += f64::from(sine_table::cos(angle)) * f64::from(0.2_f32);
        }
    }
    fn travel(
        &mut self,
        grid: &mut impl BlockGrid,
        yaw: f32,
        forward: i32,
        strafe: i32,
        input_scale: f32,
    ) {
        let block_friction = if self.on_ground {
            self.movement_block(grid).friction()
        } else {
            1.0
        };
        let walking_speed = if self.is_sprinting {
            self.traits.walking_speed * 1.3_f32
        } else {
            self.traits.walking_speed
        };
        let speed = if !self.on_ground {
            if self.is_sprinting {
                SPRINT_AIR_SPEED
            } else {
                AIR_SPEED
            }
        } else if block_friction > PLAIN_FRICTION {
            walking_speed * (0.21600002_f32 / (block_friction * block_friction * block_friction))
        } else {
            walking_speed
        };
        self.accelerate(yaw, forward, strafe, speed, input_scale);
        self.move_modern(grid, self.delta_x, self.delta_y, self.delta_z);
        let friction = f64::from(block_friction * 0.91_f32);
        self.delta_x *= friction;
        self.delta_y = (self.delta_y - 0.08) * f64::from(0.98_f32);
        self.delta_z *= friction;
        if self.on_ground {
            self.step_on(grid);
        }
    }
    fn accelerate(&mut self, yaw: f32, forward: i32, strafe: i32, speed: f32, input_scale: f32) {
        if forward == 0 && strafe == 0 {
            return;
        }
        let length = f64::from(
            strafe
                .wrapping_mul(strafe)
                .wrapping_add(forward.wrapping_mul(forward)) as f32,
        )
        .sqrt() as f32;
        let mut input_x = strafe as f32 / length * input_scale;
        let mut input_z = forward as f32 / length * input_scale;
        let abs_x = input_x.abs();
        let abs_z = input_z.abs();
        let tan = if abs_z > abs_x {
            abs_x / abs_z
        } else {
            abs_z / abs_x
        };
        let root = f64::from(1.0_f32 + tan * tan).sqrt() as f32;
        let stretched = super::java_min_f32(input_scale * root, 1.0) / input_scale;
        input_x *= stretched;
        input_z *= stretched;
        let mut vector_x = f64::from(input_x);
        let mut vector_z = f64::from(input_z);
        let length_sqr = vector_x * vector_x + vector_z * vector_z;
        if length_sqr > 1.0 {
            let len = length_sqr.sqrt();
            vector_x /= len;
            vector_z /= len;
        }
        vector_x *= f64::from(speed);
        vector_z *= f64::from(speed);
        let angle = sine_table::radians(yaw);
        let sin = f64::from(sine_table::sin(angle));
        let cos = f64::from(sine_table::cos(angle));
        self.delta_x += vector_x * cos - vector_z * sin;
        self.delta_z += vector_z * cos + vector_x * sin;
    }
    fn move_modern(&mut self, grid: &mut impl BlockGrid, wanted_x: f64, dy: f64, wanted_z: f64) {
        let mut dx = wanted_x;
        let mut dz = wanted_z;
        if self.sneaking && dy <= 0.0 && self.is_above_ground(grid) {
            let mut edge = EdgeBackOff::default();
            edge.back_off(dx, dz, |x, z, depth| self.can_fall(grid, x, z, depth));
            dx = edge.x;
            dz = edge.z;
        }
        let movement = self.collide(grid, Vec3::new(dx, dy, dz));
        self.x += movement.x;
        self.y += movement.y;
        self.z += movement.z;
        let x_collision =
            (movement.x - dx).abs() >= f64::from(1.0e-5_f32) || (movement.x - dx).is_nan();
        let z_collision =
            (movement.z - dz).abs() >= f64::from(1.0e-5_f32) || (movement.z - dz).is_nan();
        let vertical_collision = dy != movement.y;
        let was_on_ground = self.on_ground;
        self.horizontal_collision = x_collision || z_collision;
        self.on_ground = vertical_collision && dy < 0.0;
        self.update_support(grid, movement);
        if x_collision {
            self.delta_x = 0.0;
        }
        if z_collision {
            self.delta_z = 0.0;
        }
        if vertical_collision {
            self.delta_y = self.bounce(grid, movement.y);
        }
        if self.on_ground {
            if !was_on_ground {
                self.landing_fall = (self.peak_y - self.y) * self.fall_scale(grid);
            }
            self.peak_y = self.y;
        } else {
            self.peak_y = java_max(self.peak_y, self.y);
        }
        let speed_factor = self.speed_factor(grid);
        self.delta_x *= speed_factor;
        self.delta_z *= speed_factor;
    }
    fn is_above_ground(&self, grid: &mut impl BlockGrid) -> bool {
        let fallen = self.peak_y - self.y;
        self.on_ground
            || (fallen < f64::from(STEP_HEIGHT)
                && !self.can_fall(grid, 0.0, 0.0, f64::from(STEP_HEIGHT) - fallen))
    }
    pub fn collide(&self, grid: &mut impl BlockGrid, delta: Vec3) -> Vec3 {
        collisions::collide(
            &mut self.colliders.lock().expect("simulation collider cache"),
            grid,
            self.position(),
            self.on_ground,
            delta,
        )
    }
    pub fn can_fall(&self, grid: &mut impl BlockGrid, dx: f64, dz: f64, depth: f64) -> bool {
        collisions::can_fall(
            &mut self.colliders.lock().expect("simulation collider cache"),
            grid,
            self.position(),
            dx,
            dz,
            depth,
        )
    }
    pub(crate) fn update_support(&mut self, grid: &mut impl BlockGrid, movement: Vec3) {
        if !self.on_ground {
            self.support.clear();
            self.ground_without_blocks = false;
            return;
        }
        self.find_support(grid, 0.0, 0.0);
        if !self.support.exists && !self.ground_without_blocks {
            self.find_support(grid, movement.x, movement.z);
        }
        self.ground_without_blocks = !self.support.exists;
    }
    fn bounce(&self, grid: &mut impl BlockGrid, moved: f64) -> f64 {
        if !self.on_ground || self.sneaking {
            return 0.0;
        }
        let restitution = f64::from(self.on_pos_legacy(grid).bounce_restitution());
        if -self.delta_y <= 0.08 || restitution <= 0.0 {
            return 0.0;
        }
        let portion = moved / self.delta_y;
        let compensation = portion * 0.08;
        let drag = 1.0 + portion * (f64::from(0.98_f32) - 1.0);
        (compensation - self.delta_y) * drag * restitution
    }
    fn step_on(&mut self, grid: &mut impl BlockGrid) {
        if !self.sneaking && self.on_pos_legacy(grid).is_slime() && self.delta_y.abs() < 0.1 {
            let scale = 0.4 + self.delta_y.abs() * 0.2;
            self.delta_x *= scale;
            self.delta_z *= scale;
        }
    }
    fn find_support(&mut self, grid: &mut impl BlockGrid, back_x: f64, back_z: f64) {
        self.support.find(
            &mut self.colliders.lock().expect("simulation collider cache"),
            grid,
            self.position(),
            back_x,
            back_z,
        );
    }
    pub fn stands_on(&self, x: i32, y: i32, z: i32) -> bool {
        self.on_ground
            && self.support.exists
            && self.support.x == x
            && self.support.y == y
            && self.support.z == z
    }
    pub fn footing(&self, grid: &mut impl BlockGrid, pos: BlockPos) -> f64 {
        self.colliders
            .lock()
            .expect("simulation collider cache")
            .footing(grid, pos, self.x, self.z, HALF_WIDTH)
    }
    pub(crate) fn fall_scale(&self, grid: &mut impl BlockGrid) -> f64 {
        let block = self.on_pos_legacy(grid);
        if block.is_slime() {
            0.0
        } else {
            1.0 - f64::from(block.fall_distance_reduction())
        }
    }
    pub(crate) fn on_pos_legacy<G: BlockGrid>(&self, grid: &mut G) -> G::State {
        let below = (self.y - f64::from(0.2_f32)).floor() as i32;
        if !self.support.exists {
            return grid.state(BlockPos::new(
                self.x.floor() as i32,
                below,
                self.z.floor() as i32,
            ));
        }
        let support = grid.state(BlockPos::new(
            self.support.x,
            self.support.y,
            self.support.z,
        ));
        if support.is_fence() || support.is_wall() || support.is_fence_gate() {
            support
        } else {
            grid.state(BlockPos::new(self.support.x, below, self.support.z))
        }
    }
    pub(crate) fn movement_block<G: BlockGrid>(&self, grid: &mut G) -> G::State {
        let below = (self.y - 0.500001).floor() as i32;
        if !self.support.exists {
            return grid.state(BlockPos::new(
                self.x.floor() as i32,
                below,
                self.z.floor() as i32,
            ));
        }
        let support = grid.state(BlockPos::new(
            self.support.x,
            self.support.y,
            self.support.z,
        ));
        if support.is_wall() || support.is_fence_gate() {
            support
        } else {
            grid.state(BlockPos::new(self.support.x, below, self.support.z))
        }
    }
    pub(crate) fn jump_factor(&self, grid: &mut impl BlockGrid) -> f32 {
        let here = grid
            .state(BlockPos::containing(self.position()))
            .jump_factor();
        if here == 1.0 {
            self.movement_block(grid).jump_factor()
        } else {
            here
        }
    }
    pub(crate) fn speed_factor(&self, grid: &mut impl BlockGrid) -> f64 {
        let here = grid.state(BlockPos::containing(self.position()));
        let factor = here.speed_factor();
        if factor != 1.0 || here.has_liquid() {
            f64::from(factor)
        } else {
            f64::from(self.movement_block(grid).speed_factor())
        }
    }
}
