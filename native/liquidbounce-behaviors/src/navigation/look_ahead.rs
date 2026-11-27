// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent LookAhead.kt: view easing is separate from silent movement rotation.
use super::{geometry::Vec3, walk_control::{turning::LookPurpose, view_rotation::ViewRotation}, walk_jump::{vector_yaw, wrap_degrees}};

#[derive(Clone, Copy, Debug)]
pub struct LookSettings {
    pub enabled: bool,
    pub turn_speed: f32,
    pub distance: f32,
    pub x_rot: f32,
}
impl Default for LookSettings {
    fn default() -> Self { Self { enabled: true, turn_speed: 10.0, distance: 4.0, x_rot: 8.0 } }
}
/// All getters are live. The host keeps movement and visible camera rotations
/// separate, exactly as Body.look(MOVEMENT/VIEW) does.
pub trait ViewHost {
    fn look_settings(&self) -> LookSettings;
    fn view_taken_over(&self) -> bool;
    fn view_position(&self) -> Vec3;
    fn view_y_rot(&self) -> f32;
    fn view_x_rot(&self) -> f32;
    fn root_vehicle_motion(&self) -> Vec3;
    fn mouse_view_step(&self) -> f32;
    fn turn_look(&mut self, rotation: ViewRotation, purpose: LookPurpose);
}
#[derive(Default)]
pub struct LookAhead { point: Option<Vec3>, aimed: bool, y_speed: f32, x_speed: f32 }
impl LookAhead {
    pub fn aim(&mut self, point: Option<Vec3>) { self.point = point; self.aimed = true; }
    pub fn reset(&mut self) { self.point = None; self.aimed = false; self.stop_turning(); }
    pub fn tick(&mut self, host: &mut impl ViewHost) {
        let aimed = self.aimed;
        let point = self.point.take();
        self.aimed = false;
        if !host.look_settings().enabled || host.view_taken_over() { self.stop_turning(); return; }
        if aimed {
            let yaw = self.target_y_rot(host, point);
            let y_delta = if yaw.is_nan() { 0.0 } else { wrap_degrees(yaw - host.view_y_rot()) };
            let x_delta = self.x_rot(host, point) - host.view_x_rot();
            self.turn(host, y_delta, x_delta);
        } else if self.y_speed != 0.0 || self.x_speed != 0.0 { self.turn(host, 0.0, 0.0); }
    }
    fn target_y_rot(&self, host: &impl ViewHost, point: Option<Vec3>) -> f32 {
        let position = host.view_position();
        if let Some(point) = point {
            let dx = point.x - position.x; let dz = point.z - position.z;
            return if dx * dx + dz * dz < 1.0 { f32::NAN } else { vector_yaw(dx, dz) };
        }
        let motion = host.root_vehicle_motion();
        if motion.x * motion.x + motion.z * motion.z >= 0.05 * 0.05 { vector_yaw(motion.x, motion.z) } else { f32::NAN }
    }
    pub fn x_rot(&self, host: &impl ViewHost, point: Option<Vec3>) -> f32 {
        let x_rot = host.look_settings().x_rot;
        let Some(point) = point else { return x_rot; };
        let position = host.view_position();
        let rise = point.y - position.y;
        let dx = point.x - position.x; let dz = point.z - position.z;
        let horizontal = (dx * dx + dz * dz).sqrt();
        if horizontal < 1.0 && rise.abs() < 1.0 { return x_rot; }
        let slope = (rise.atan2(horizontal.max(1.0)) * f64::from(180.0_f32 / std::f32::consts::PI)) as f32;
        (x_rot - slope * 0.5).clamp(-30.0, 60.0)
    }
    fn turn(&mut self, host: &mut impl ViewHost, y_delta: f32, x_delta: f32) {
        let speed = host.look_settings().turn_speed;
        let change = speed * 0.3;
        self.y_speed = approach(self.y_speed, (y_delta * 0.2).clamp(-speed, speed), change);
        let x_max = speed * 0.5;
        self.x_speed = approach(self.x_speed, (x_delta * 0.2).clamp(-x_max, x_max), change);
        if self.y_speed.abs() < 0.05 && y_delta.abs() < 0.05 { self.y_speed = 0.0; }
        if self.x_speed.abs() < 0.05 && x_delta.abs() < 0.05 { self.x_speed = 0.0; }
        let yaw = host.view_y_rot() + steps(host, self.y_speed);
        let pitch = (host.view_x_rot() + steps(host, self.x_speed)).clamp(-90.0, 90.0);
        host.turn_look(ViewRotation::new(yaw, pitch), LookPurpose::View);
    }
    fn stop_turning(&mut self) { self.y_speed = 0.0; self.x_speed = 0.0; }
}
fn approach(current: f32, wanted: f32, change: f32) -> f32 { current + (wanted-current).clamp(-change, change) }
fn steps(host: &impl ViewHost, speed: f32) -> f32 {
    let step = host.mouse_view_step();
    if step > 0.0 { ((f64::from(speed / step) + 0.5).floor() as i32) as f32 * step } else { speed }
}
