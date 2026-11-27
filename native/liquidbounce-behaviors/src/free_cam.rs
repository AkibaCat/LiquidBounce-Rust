// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleFreeCam and PositionState at upstream ae0eb396.
//! The native adapter supplies live options, world rays and lifecycle events.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub speed: f32,
    pub camera_interact: bool,
    pub look_at: bool,
    pub cancel_on: [bool; 4],
    pub mid_click_camera_teleport: bool,
    pub keep_sneaking: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            speed: 1.0,
            camera_interact: true,
            look_at: true,
            cancel_on: [false; 4],
            mid_click_camera_teleport: false,
            keep_sneaking: false,
        }
    }
}
pub const CANCEL_TAGS: [&str; 4] = ["Damage", "Teleport", "Move", "Liquid"];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionState {
    pub available: bool,
    pub pos: [f64; 3],
    pub last_pos: [f64; 3],
    pub rot: [f32; 2],
    pub last_rot: [f32; 2],
}
impl Default for PositionState {
    fn default() -> Self {
        Self {
            available: false,
            pos: [0.; 3],
            last_pos: [0.; 3],
            rot: [0.; 2],
            last_rot: [0.; 2],
        }
    }
}
impl PositionState {
    pub fn enable(&mut self, eye: [f64; 3], rotation: [f32; 2]) {
        *self = Self {
            available: true,
            pos: eye,
            last_pos: eye,
            rot: rotation,
            last_rot: rotation,
        };
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn set_position(&mut self, target: [f64; 3]) {
        self.last_pos = self.pos;
        self.pos = target;
    }
    pub fn set_rotation(&mut self, target: [f32; 2]) {
        self.last_rot = self.rot;
        self.rot = target;
    }
    pub fn mouse_rotation(&mut self, x: f64, y: f64) {
        // The event multiplies as Double, then PositionState narrows to Float.
        self.set_rotation([
            self.rot[0] + (x * f64::from(0.15f32)) as f32,
            (self.rot[1] + (y * f64::from(0.15f32)) as f32).clamp(-90., 90.),
        ]);
    }
    pub fn interpolate(&self, partial: f32) -> [f64; 3] {
        std::array::from_fn(|i| {
            self.last_pos[i] + f64::from(partial) * (self.pos[i] - self.last_pos[i])
        })
    }
    pub fn interpolate_rotation(&self, partial: f32) -> [f32; 2] {
        std::array::from_fn(|i| partial.mul_add(self.rot[i] - self.last_rot[i], self.last_rot[i]))
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FreeCam {
    pub settings: Settings,
    pub position: PositionState,
    pub initialize: bool,
    pub cancel_pending: bool,
}
impl FreeCam {
    pub fn enabled(&mut self) {
        self.initialize = true;
        self.cancel_pending = false;
    }
    pub fn disabled(&mut self) {
        self.position.clear();
        self.initialize = false;
        self.cancel_pending = false;
    }
    pub fn initialize(&mut self, eye: [f64; 3], rotation: [f32; 2]) {
        if self.initialize {
            self.position.enable(eye, rotation);
            self.initialize = false;
        }
    }
    pub fn active(&self, running: bool) -> bool {
        running && self.position.available && !self.cancel_pending
    }
    pub fn movement_yaw(yaw: f32, keys: [bool; 7]) -> f32 {
        let [forward, backward, left, right, ..] = keys;
        let mut yaw = yaw;
        let factor = if backward && !forward {
            yaw += 180.;
            -0.5
        } else if forward && !backward {
            0.5
        } else {
            1.
        };
        if left && !right {
            yaw -= 90. * factor;
        }
        if right && !left {
            yaw += 90. * factor;
        }
        yaw
    }
    pub fn tick(&mut self, keys: [bool; 7], grabbed: bool) {
        let speed = f64::from(self.settings.speed);
        let yaw = f64::from(Self::movement_yaw(self.position.rot[0], keys)).to_radians();
        let moving = keys[0] != keys[1] || keys[2] != keys[3];
        let dx = if moving { -yaw.sin() * speed } else { 0. };
        let dz = if moving { yaw.cos() * speed } else { 0. };
        let dy = (i32::from(keys[4]) - i32::from(keys[5])) as f64 * speed;
        let pos = self.position.pos;
        self.position
            .set_position([pos[0] + dx, pos[1] + dy, pos[2] + dz]);
        if !grabbed {
            self.position.mouse_rotation(0., 0.);
        }
    }
    pub fn mouse(&mut self, running: bool, x: f64, y: f64) -> bool {
        if !self.active(running) {
            return false;
        }
        self.position.mouse_rotation(x, y);
        true
    }
    pub fn cancel(&mut self, running: bool, trigger: usize, fired: bool) -> bool {
        if running && self.settings.cancel_on.get(trigger) == Some(&true) && fired {
            self.position.clear();
            self.initialize = false;
            self.cancel_pending = true;
            true
        } else {
            false
        }
    }
    pub fn middle_click(&mut self, running: bool, target: [f64; 3]) {
        if self.active(running) && self.settings.mid_click_camera_teleport {
            let pos = self.position.pos;
            self.position
                .set_position(std::array::from_fn(|i| pos[i] + 0.9 * (target[i] - pos[i])));
        }
    }
    pub fn first_input(&self, running: bool, keys: &mut [bool; 7]) {
        if self.active(running) {
            keys[..6].fill(false);
        }
    }
    pub fn final_sneak(&self, running: bool, keys: &mut [bool; 7]) {
        if self.active(running) && self.settings.keep_sneaking {
            keys[5] = true;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_defaults_and_lifecycle_reset_all_endpoints() {
        let mut f = FreeCam::default();
        assert_eq!(f.settings, Settings::default());
        assert!(!f.active(true));
        f.enabled();
        f.initialize([1., 64., 3.], [30., -15.]);
        assert!(f.active(true));
        assert_eq!(f.position.pos, f.position.last_pos);
        f.position.mouse_rotation(100., -100.);
        f.disabled();
        assert_eq!(f.position, PositionState::default());
    }
    #[test]
    fn all_key_chords_use_original_direction_and_constant_horizontal_speed() {
        for bits in 0..128 {
            let keys = std::array::from_fn(|i| bits & (1 << i) != 0);
            let mut f = FreeCam::default();
            f.enabled();
            f.initialize([0.; 3], [0.; 2]);
            f.tick(keys, true);
            let p = f.position.pos;
            let moving = keys[0] != keys[1] || keys[2] != keys[3];
            assert!(
                ((p[0] * p[0] + p[2] * p[2]).sqrt() - if moving { 1. } else { 0. }).abs() < 1e-12
            );
            assert_eq!(p[1], (i32::from(keys[4]) - i32::from(keys[5])) as f64);
        }
    }
    #[test]
    fn interpolation_has_no_yaw_wrap_and_pitch_is_limited() {
        let mut p = PositionState::default();
        p.enable([0.; 3], [179., 89.]);
        p.mouse_rotation(40., 40.);
        assert_eq!(p.rot[1], 90.);
        assert_eq!(p.interpolate_rotation(0.), [179., 89.]);
        assert!(p.interpolate_rotation(0.5)[0] > 179.);
        p.set_position([10., 20., 30.]);
        assert_eq!(p.interpolate(0.25), [2.5, 5., 7.5]);
    }
    #[test]
    fn selected_triggers_deactivate_camera_immediately_and_gravity_is_not_a_move_trigger() {
        for trigger in 0..4 {
            let mut f = FreeCam::default();
            f.settings.cancel_on[trigger] = true;
            f.enabled();
            f.initialize([1.; 3], [1.; 2]);
            assert!(!f.cancel(true, trigger, false));
            assert!(f.active(true));
            assert!(f.cancel(true, trigger, true));
            assert!(!f.active(true));
            assert!(f.cancel_pending);
        }
    }
    #[test]
    fn input_priority_keeps_other_modules_changes_and_camera_teleport_keeps_ten_percent_margin() {
        let mut f = FreeCam::default();
        f.enabled();
        f.initialize([0.; 3], [0.; 2]);
        let mut keys = [true; 7];
        f.first_input(true, &mut keys);
        assert_eq!(keys, [false, false, false, false, false, false, true]);
        keys[0] = true;
        f.settings.keep_sneaking = true;
        f.final_sneak(true, &mut keys);
        assert!(keys[0] && keys[5]);
        f.settings.mid_click_camera_teleport = true;
        f.middle_click(true, [10., 20., 30.]);
        assert_eq!(f.position.pos, [9., 18., 27.]);
    }
}
