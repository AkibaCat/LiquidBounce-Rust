// SPDX-License-Identifier: GPL-3.0-or-later
//! Full Minecraft Agent Motion1_8.kt, preserving all float promotions and steps.
use super::super::{
    aabb::{Aabb, Axis},
    geometry::{BlockPos, Vec3, java_max},
    walk::WalkState,
};
use super::{
    block_grid::{BlockGrid, SimulationState},
    player_motion::{HALF_WIDTH, HEIGHT, INPUT_FRICTION, PlayerMotion, STEP_HEIGHT},
    sine_table,
};
impl PlayerMotion {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn tick_1_8(
        &mut self,
        grid: &mut impl BlockGrid,
        yaw: f32,
        forward: i32,
        strafe: i32,
        jump: bool,
        sprint: bool,
        sneak: bool,
    ) {
        if self.jump_delay > 0 {
            self.jump_delay -= 1;
        }
        if self.delta_x.abs() < 0.005 {
            self.delta_x = 0.0;
        }
        if self.delta_y.abs() < 0.005 {
            self.delta_y = 0.0;
        }
        if self.delta_z.abs() < 0.005 {
            self.delta_z = 0.0;
        }
        self.sneaking = sneak;
        self.crouching = sneak;
        self.is_sprinting = sprint && forward > 0 && !sneak && !self.horizontal_collision;
        if !jump {
            self.jump_delay = 0;
        } else if self.on_ground && self.jump_delay == 0 {
            self.jump_1_8(yaw);
            self.jump_delay = 10;
        }
        let mut move_strafe = strafe as f32;
        let mut move_forward = forward as f32;
        if sneak {
            move_strafe *= 0.3_f32;
            move_forward *= 0.3_f32;
        }
        self.travel_1_8(
            grid,
            yaw,
            move_strafe * INPUT_FRICTION,
            move_forward * INPUT_FRICTION,
        );
        self.air_sprint = self.is_sprinting;
    }
    fn jump_1_8(&mut self, yaw: f32) {
        self.delta_y =
            f64::from(self.traits.jump_strength) + f64::from(self.traits.jump_boost_power);
        if self.is_sprinting {
            let angle = yaw * 0.017453292_f32;
            self.delta_x -= f64::from(sine_table::sin_1_8(angle) * 0.2_f32);
            self.delta_z += f64::from(sine_table::cos_1_8(angle) * 0.2_f32);
        }
    }
    fn travel_1_8(&mut self, grid: &mut impl BlockGrid, yaw: f32, strafe: f32, forward: f32) {
        let friction = if self.on_ground {
            grid.state(BlockPos::new(
                self.x.floor() as i32,
                (self.y.floor() as i32).wrapping_sub(1),
                self.z.floor() as i32,
            ))
            .friction()
                * 0.91_f32
        } else {
            0.91_f32
        };
        let speed = if self.on_ground {
            let walking = if self.is_sprinting {
                (f64::from(self.traits.walking_speed) * (1.0 + 0.30000001192092896)) as f32
            } else {
                self.traits.walking_speed
            };
            walking * (0.16277136_f32 / (friction * friction * friction))
        } else if self.air_sprint {
            (f64::from(0.02_f32) + f64::from(0.02_f32) * 0.3) as f32
        } else {
            0.02_f32
        };
        self.accelerate_1_8(yaw, strafe, forward, speed);
        self.move_1_8(grid, self.delta_x, self.delta_y, self.delta_z);
        self.delta_y -= 0.08;
        self.delta_y *= f64::from(0.98_f32);
        self.delta_x *= f64::from(friction);
        self.delta_z *= f64::from(friction);
    }
    fn accelerate_1_8(&mut self, yaw: f32, strafe: f32, forward: f32, speed: f32) {
        let mut length = strafe * strafe + forward * forward;
        if length < 1.0e-4_f32 {
            return;
        }
        length = f64::from(length).sqrt() as f32;
        if length < 1.0 {
            length = 1.0;
        }
        length = speed / length;
        let scaled_strafe = strafe * length;
        let scaled_forward = forward * length;
        let angle = yaw * std::f32::consts::PI / 180.0_f32;
        let sin = sine_table::sin_1_8(angle);
        let cos = sine_table::cos_1_8(angle);
        self.delta_x += f64::from(scaled_strafe * cos - scaled_forward * sin);
        self.delta_z += f64::from(scaled_forward * cos + scaled_strafe * sin);
    }
    fn body_1_8(&self) -> Aabb {
        Aabb::new(
            Vec3::new(self.x - HALF_WIDTH, self.y, self.z - HALF_WIDTH),
            Vec3::new(self.x + HALF_WIDTH, self.y + HEIGHT, self.z + HALF_WIDTH),
        )
    }
    fn collect_1_8(&self, grid: &mut impl BlockGrid, body: Aabb) {
        self.colliders
            .lock()
            .expect("simulation collider cache")
            .collect(grid, body);
    }
    fn offset_1_8(&self, axis: Axis, body: Aabb, movement: f64) -> f64 {
        self.colliders
            .lock()
            .expect("simulation collider cache")
            .offset(axis, body, movement)
    }
    fn move_1_8(&mut self, grid: &mut impl BlockGrid, wanted_x: f64, wanted_y: f64, wanted_z: f64) {
        let mut dx = wanted_x;
        let mut dy = wanted_y;
        let mut dz = wanted_z;
        let mut body = self.body_1_8();
        if self.touch_blocks_1_8(grid, body) {
            dx *= 0.25;
            dy *= f64::from(0.05_f32);
            dz *= 0.25;
            self.delta_x = 0.0;
            self.delta_y = 0.0;
            self.delta_z = 0.0;
        }
        if dx == 0.0 && dy == 0.0 && dz == 0.0 {
            return;
        }
        let start_x = self.x;
        let start_z = self.z;
        let edge = self.on_ground && self.sneaking;
        if edge {
            while dx != 0.0 && self.is_free_1_8(grid, body.moved(Vec3::new(dx, -1.0, 0.0))) {
                dx = back_off(dx);
            }
            while dz != 0.0 && self.is_free_1_8(grid, body.moved(Vec3::new(0.0, -1.0, dz))) {
                dz = back_off(dz);
            }
            while dx != 0.0
                && dz != 0.0
                && self.is_free_1_8(grid, body.moved(Vec3::new(dx, -1.0, dz)))
            {
                dx = back_off(dx);
                dz = back_off(dz);
            }
        }
        let want_x = dx;
        let want_y = dy;
        let want_z = dz;
        let start = body;
        self.collect_1_8(grid, body.expand_towards(Vec3::new(dx, dy, dz)));
        dy = self.offset_1_8(Axis::Y, body, dy);
        body = body.moved(Vec3::new(0.0, dy, 0.0));
        let grounded = self.on_ground || want_y != dy && want_y < 0.0;
        dx = self.offset_1_8(Axis::X, body, dx);
        body = body.moved(Vec3::new(dx, 0.0, 0.0));
        dz = self.offset_1_8(Axis::Z, body, dz);
        body = body.moved(Vec3::new(0.0, 0.0, dz));
        if grounded && (want_x != dx || want_z != dz) {
            let step = f64::from(STEP_HEIGHT);
            self.collect_1_8(grid, start.expand_towards(Vec3::new(want_x, step, want_z)));
            let mut wide = start;
            let wide_up = self.offset_1_8(
                Axis::Y,
                start.expand_towards(Vec3::new(want_x, 0.0, want_z)),
                step,
            );
            wide = wide.moved(Vec3::new(0.0, wide_up, 0.0));
            let wide_x = self.offset_1_8(Axis::X, wide, want_x);
            wide = wide.moved(Vec3::new(wide_x, 0.0, 0.0));
            let wide_z = self.offset_1_8(Axis::Z, wide, want_z);
            wide = wide.moved(Vec3::new(0.0, 0.0, wide_z));
            let mut narrow = start;
            let narrow_up = self.offset_1_8(Axis::Y, narrow, step);
            narrow = narrow.moved(Vec3::new(0.0, narrow_up, 0.0));
            let narrow_x = self.offset_1_8(Axis::X, narrow, want_x);
            narrow = narrow.moved(Vec3::new(narrow_x, 0.0, 0.0));
            let narrow_z = self.offset_1_8(Axis::Z, narrow, want_z);
            narrow = narrow.moved(Vec3::new(0.0, 0.0, narrow_z));
            let (step_x, step_z, mut down, mut stepped) =
                if wide_x * wide_x + wide_z * wide_z > narrow_x * narrow_x + narrow_z * narrow_z {
                    (wide_x, wide_z, -wide_up, wide)
                } else {
                    (narrow_x, narrow_z, -narrow_up, narrow)
                };
            down = self.offset_1_8(Axis::Y, stepped, down);
            stepped = stepped.moved(Vec3::new(0.0, down, 0.0));
            if dx * dx + dz * dz < step_x * step_x + step_z * step_z {
                dx = step_x;
                dy = down;
                dz = step_z;
                body = stepped;
            }
        }
        self.x = (body.min.x + body.max.x) / 2.0;
        self.y = body.min.y;
        self.z = (body.min.z + body.max.z) / 2.0;
        self.horizontal_collision = want_x != dx || want_z != dz;
        let vertical = want_y != dy;
        let was_on_ground = self.on_ground;
        self.on_ground = vertical && want_y < 0.0;
        self.update_support(grid, Vec3::new(dx, dy, dz));
        if self.on_ground {
            if !was_on_ground {
                self.landing_fall = if self.landed_on_1_8(grid).is_slime() {
                    0.0
                } else {
                    self.peak_y - self.y
                };
            }
            self.peak_y = self.y;
        } else {
            self.peak_y = java_max(self.peak_y, self.y);
        }
        if want_x != dx {
            self.delta_x = 0.0;
        }
        if want_z != dz {
            self.delta_z = 0.0;
        }
        let landed = self.landed_on_1_8(grid);
        if vertical {
            self.delta_y = if landed.is_slime() && !self.sneaking && self.delta_y < 0.0 {
                -self.delta_y
            } else {
                0.0
            };
        }
        if !edge {
            self.step_effects_1_8(start_x, start_z, &landed);
        }
    }
    fn is_free_1_8(&self, grid: &mut impl BlockGrid, body: Aabb) -> bool {
        for bx in (body.min.x.floor() as i32)..=((body.max.x + 1.0).floor() as i32).wrapping_sub(1)
        {
            for bz in
                (body.min.z.floor() as i32)..=((body.max.z + 1.0).floor() as i32).wrapping_sub(1)
            {
                for by in (body.min.y.floor() as i32).wrapping_sub(1)
                    ..=((body.max.y + 1.0).floor() as i32).wrapping_sub(1)
                {
                    if self
                        .colliders
                        .lock()
                        .expect("simulation collider cache")
                        .intersects(grid, BlockPos::new(bx, by, bz), body)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn touch_blocks_1_8(&mut self, grid: &mut impl BlockGrid, body: Aabb) -> bool {
        let mut web = false;
        for bx in ((body.min.x + 0.001).floor() as i32)..=((body.max.x - 0.001).floor() as i32) {
            for by in ((body.min.y + 0.001).floor() as i32)..=((body.max.y - 0.001).floor() as i32)
            {
                for bz in
                    ((body.min.z + 0.001).floor() as i32)..=((body.max.z - 0.001).floor() as i32)
                {
                    let block = grid.state(BlockPos::new(bx, by, bz));
                    if block.is_cobweb() {
                        web = true;
                    }
                    if block.speed_factor() != 1.0 {
                        self.delta_x *= f64::from(block.speed_factor());
                        self.delta_z *= f64::from(block.speed_factor());
                    }
                }
            }
        }
        web
    }
    fn landed_on_1_8<G: BlockGrid>(&self, grid: &mut G) -> G::State {
        let bx = self.x.floor() as i32;
        let by = (self.y - f64::from(0.2_f32)).floor() as i32;
        let bz = self.z.floor() as i32;
        let state = grid.state(BlockPos::new(bx, by, bz));
        if !state.is_air() {
            return state;
        }
        let below = grid.state(BlockPos::new(bx, by.wrapping_sub(1), bz));
        if below.is_fence() || below.is_wall() || below.is_fence_gate() {
            below
        } else {
            state
        }
    }
    fn step_effects_1_8(&mut self, start_x: f64, start_z: f64, block: &impl SimulationState) {
        let moved_x = self.x - start_x;
        let moved_z = self.z - start_z;
        self.walked = (f64::from(self.walked)
            + f64::from((moved_x * moved_x + moved_z * moved_z).sqrt() as f32) * 0.6)
            as f32;
        if self.walked <= self.next_step as f32 || block.is_step_air() {
            return;
        }
        self.next_step = (self.walked as i32).wrapping_add(1);
        if block.is_slime() && self.delta_y.abs() < 0.1 && !self.sneaking {
            let scale = 0.4 + self.delta_y.abs() * 0.2;
            self.delta_x *= scale;
            self.delta_z *= scale;
        }
    }
}
fn back_off(movement: f64) -> f64 {
    if (-0.05..0.05).contains(&movement) {
        0.0
    } else if movement > 0.0 {
        movement - 0.05
    } else {
        movement + 0.05
    }
}
