// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleVehicleBoost at LiquidBounce ae0eb396. The remembered passenger
//! state deliberately survives disable and world changes, as in the source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleBoost {
    pub horizontal_speed: f32,
    pub vertical_speed: f32,
    was_in_vehicle: bool,
}
impl Default for VehicleBoost {
    fn default() -> Self {
        Self {
            horizontal_speed: 2.0,
            vertical_speed: 1.0,
            was_in_vehicle: false,
        }
    }
}
impl VehicleBoost {
    pub fn game_tick(
        &mut self,
        running: bool,
        passenger: bool,
        yaw: f32,
        mut java_sin_cos: impl FnMut(f64) -> [f64; 2],
    ) -> Option<[f64; 3]> {
        if !running {
            return None;
        }
        let boost = if self.was_in_vehicle && !passenger {
            // JDK 25 Math.toRadians multiplies once by this stored double.
            let angle = f64::from(yaw) * 0.017453292519943295_f64;
            let [sin, cos] = java_sin_cos(angle);
            Some([
                -sin * f64::from(self.horizontal_speed),
                f64::from(self.vertical_speed),
                cos * f64::from(self.horizontal_speed),
            ])
        } else {
            None
        };
        self.was_in_vehicle = passenger;
        boost
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dismount_transition_overwrites_velocity_and_keeps_disabled_memory() {
        let mut state = VehicleBoost::default();
        let trig = |angle: f64| [angle.sin(), angle.cos()];
        assert_eq!(state.game_tick(true, false, 0.0, trig), None);
        assert_eq!(state.game_tick(true, true, 45.0, trig), None);
        assert_eq!(state.game_tick(false, false, 90.0, trig), None);
        let velocity = state.game_tick(true, false, 90.0, trig).unwrap();
        assert_eq!(velocity[0], -2.0);
        assert_eq!(velocity[1], 1.0);
        assert!(velocity[2].abs() < 1e-14);
        assert_eq!(state.game_tick(true, false, 90.0, trig), None);
        state.horizontal_speed = -9.0;
        state.vertical_speed = -7.0;
        state.game_tick(true, true, -180.0, trig);
        assert_eq!(
            state.game_tick(true, false, 0.0, trig),
            Some([0.0, -7.0, -9.0])
        );
    }
}
