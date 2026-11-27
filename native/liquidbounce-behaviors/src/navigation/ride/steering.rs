// SPDX-License-Identifier: GPL-3.0-or-later
use super::super::{
    geometry::{BlockPos, Vec3},
    input::Input,
    terrain::TerrainLevel,
    walk_jump::{vector_yaw, wrap_degrees},
};
use super::{
    body::{RideBody, RideEntity, RideLook},
    moves::RideState,
    vehicle::{JUMP_CHARGE_TICKS,VehicleEntity},
};
pub struct Steering {
    last_y_rot: f32,
    charge_ticks: i32,
}
impl Default for Steering {
    fn default() -> Self {
        Self {
            last_y_rot: f32::NAN,
            charge_ticks: 0,
        }
    }
}
impl Steering {
    pub fn reset(&mut self) {
        self.last_y_rot = f32::NAN;
        self.charge_ticks = 0;
    }
    pub fn boat<B: RideBody>(&mut self, body: &mut B, boat: &B::Entity, target: Vec3, brake: bool) {
        let y_rot = boat.y_rot();
        let turn_rate = if self.last_y_rot.is_nan() {
            0.0
        } else {
            wrap_degrees(y_rot - self.last_y_rot)
        };
        self.last_y_rot = y_rot;
        let below = BlockPos::containing(Vec3::new(boat.x(), boat.y() - 0.1, boat.z()));
        let friction = if body.level().block_state(below).friction() >= 0.9_f32 {
            0.98_f32
        } else {
            0.9_f32
        };
        let coast = turn_rate * friction / (1.0 - friction);
        let offset=sub(target,boat.position());
        let off = wrap_degrees(vector_yaw(offset.x,offset.z) - y_rot);
        let error = off - coast;
        let slow_enough = !brake || horizontal_length(boat.delta_movement()) < 0.12;
        body.press(
            Input {
                forward: off.abs() < 35.0 && slow_enough,
                backward: !slow_enough,
                left: error < -4.0,
                right: error > 4.0,
                jump: false,
                shift: false,
                sprint: false,
            },
            true,
        );
    }
    pub fn mount<B: RideBody>(
        &mut self,
        body: &mut B,
        mount: &B::Entity,
        target: Vec3,
        jump_to: Option<Vec3>,
        reach: f64,
    ) {
        let offset=sub(target,mount.position());
        let desired = vector_yaw(offset.x,offset.z);
        let facing = self.face(body, desired);
        let aligned = wrap_degrees(desired - facing).abs() < 50.0;
        let at_jump =
            jump_to.is_some_and(|jump_to| horizontal_distance(mount.position(), jump_to) < reach);
        self.charge_ticks = if at_jump {
            self.charge_ticks.wrapping_add(1)
        } else {
            0
        };
        body.press(
            Input {
                forward: aligned,
                jump: (1..=JUMP_CHARGE_TICKS).contains(&self.charge_ticks),
                ..Input::EMPTY
            },
            true,
        );
        if self.charge_ticks > JUMP_CHARGE_TICKS + 20 {
            self.charge_ticks = 0;
        }
    }
    pub fn stop(&mut self, body: &mut impl RideBody, sneak: bool) {
        self.reset();
        body.press(
            Input {
                shift: sneak,
                ..Input::EMPTY
            },
            true,
        );
    }
    pub fn face(&mut self, body: &mut impl RideBody, y_rot: f32) -> f32 {
        let player = body.player();
        let facing = player.y_rot() + wrap_degrees(y_rot - player.y_rot()).clamp(-35.0, 35.0);
        body.look(facing, player.x_rot(), RideLook::View);
        facing
    }
}
pub(crate) fn sub(to: Vec3, from: Vec3) -> Vec3 {
    Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z)
}
pub(crate) fn horizontal_distance(from: Vec3, to: Vec3) -> f64 {
    horizontal_length(sub(to, from))
}
pub(crate) fn horizontal_length(v: Vec3) -> f64 {
    (v.x * v.x + v.z * v.z).sqrt()
}
