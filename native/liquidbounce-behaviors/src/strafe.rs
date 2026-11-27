// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleStrafe, getMovementDirectionOfInput and withStrafe, ae0eb396.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strafe {
    pub strength_in_air: f32,
    pub strength_on_ground: f32,
    pub strict_movement: bool,
}
impl Default for Strafe {
    fn default() -> Self {
        Self {
            strength_in_air: 1.0,
            strength_on_ground: 1.0,
            strict_movement: false,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct StrafeContext {
    pub self_movement: bool,
    pub on_ground: bool,
    /// ClientInput.moveVector != Vec2.ZERO, independently of untransformed flags.
    pub moving: bool,
    /// Untransformed input: forward=1, backward=2, left=4, right=8.
    pub input: u8,
    pub yaw: f32,
}
/// Float arithmetic and opposing-key cancellation match the pinned helper.
pub fn movement_direction(mut yaw: f32, input: u8) -> f32 {
    let (forward, back, left, right) = (
        input & 1 != 0,
        input & 2 != 0,
        input & 4 != 0,
        input & 8 != 0,
    );
    let multiplier = if back && !forward {
        yaw += 180.0;
        -0.5
    } else if forward && !back {
        0.5
    } else {
        1.0
    };
    if left && !right {
        yaw -= 90.0 * multiplier;
    }
    if right && !left {
        yaw += 90.0 * multiplier;
    }
    yaw
}
impl Strafe {
    pub fn move_event(
        self,
        running: bool,
        context: StrafeContext,
        movement: [f64; 3],
        java_sin_cos: impl FnOnce(f64) -> [f64; 2],
    ) -> [f64; 3] {
        if !running || !context.self_movement {
            return movement;
        }
        let strength = if context.on_ground {
            self.strength_on_ground
        } else {
            self.strength_in_air
        };
        if strength == 0.0 {
            return movement;
        }
        if !context.moving {
            return if self.strict_movement {
                [0.0, movement[1], 0.0]
            } else {
                movement
            };
        }
        let input = context.input;
        if (input & 1 != 0) == (input & 2 != 0) && (input & 4 != 0) == (input & 8 != 0) {
            return [0.0, movement[1], 0.0];
        }
        let strength = f64::from(strength);
        let one_minus = 1.0 - strength;
        let previous_x = movement[0] * one_minus;
        let previous_z = movement[2] * one_minus;
        let used_speed = (movement[0] * movement[0] + movement[2] * movement[2]).sqrt() * strength;
        let angle = f64::from(movement_direction(context.yaw, input)).to_radians();
        let [sin, cos] = java_sin_cos(angle);
        [
            previous_x - sin * used_speed,
            movement[1],
            previous_z + cos * used_speed,
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_self_events_nonzero_strength_and_real_move_vector_enter_strafe() {
        let mut context = StrafeContext {
            self_movement: true,
            on_ground: false,
            moving: true,
            input: 1,
            yaw: 0.0,
        };
        let movement = [-0.4, -0.0, 0.3];
        let never = |_| panic!("gated handler must not query trig");
        assert_eq!(
            Strafe::default().move_event(false, context, movement, never),
            movement
        );
        context.self_movement = false;
        assert_eq!(
            Strafe::default().move_event(true, context, movement, never),
            movement
        );
        context.self_movement = true;
        let strict = Strafe {
            strength_in_air: -0.0,
            strict_movement: true,
            ..Default::default()
        };
        context.moving = false;
        assert_eq!(strict.move_event(true, context, movement, never), movement);
        context.on_ground = true;
        let result = strict.move_event(true, context, movement, never);
        assert_eq!(result.map(f64::to_bits), [0, (-0.0f64).to_bits(), 0]);
        context.moving = true;
        context.input = 15;
        assert_eq!(
            Strafe::default()
                .move_event(true, context, movement, never)
                .map(f64::to_bits),
            [0, (-0.0f64).to_bits(), 0]
        );
    }
    #[test]
    fn float_direction_uses_all_four_untransformed_flags() {
        assert_eq!(movement_direction(10.0, 1 | 4), -35.0);
        assert_eq!(movement_direction(10.0, 2 | 4), 235.0);
        assert_eq!(movement_direction(10.0, 1 | 2 | 4), -80.0);
        assert_eq!(movement_direction(10.0, 4 | 8), 10.0);
    }
}
