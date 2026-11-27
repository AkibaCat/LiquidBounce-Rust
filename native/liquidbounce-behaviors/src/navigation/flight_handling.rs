// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent flight Handling.kt, including the original live player getter order.
use super::{
    flight::{Acceleration, FlyAbilities},
    geometry::Vec3,
    input::Input,
};

pub trait FlightPlayer {
    fn position(&self) -> Vec3;
    fn delta_movement(&self) -> Vec3;
    fn tick_count(&self) -> i32;
    fn horizontal_collision(&self) -> bool;
    fn vertical_collision(&self) -> bool;
    fn y_rot(&self) -> f32;
    fn is_sprinting(&self) -> bool;
}

#[derive(Clone, Debug)]
pub struct Handling {
    control: Acceleration,
    base_speed: f64,
    base_vertical_speed: f64,
    measured_drag: f64,
    measured_vertical_drag: f64,
    measured_thrust: f64,
    measured_sprint_thrust: f64,
    measured_vertical_thrust: f64,
    last_position: Option<Vec3>,
    last_move: Option<Vec3>,
    last_tick: i32,
    last_pressed: Option<Input>,
}
impl Default for Handling {
    fn default() -> Self {
        Self {
            control: Acceleration::Direct,
            base_speed: f64::NAN,
            base_vertical_speed: f64::NAN,
            measured_drag: f64::NAN,
            measured_vertical_drag: f64::NAN,
            measured_thrust: f64::NAN,
            measured_sprint_thrust: f64::NAN,
            measured_vertical_thrust: f64::NAN,
            last_position: None,
            last_move: None,
            last_tick: 0,
            last_pressed: None,
        }
    }
}
fn if_nan(value: f64, fallback: impl FnOnce() -> f64) -> f64 {
    if value.is_nan() { fallback() } else { value }
}
fn at_least(value: f64, minimum: f64) -> f64 {
    if value < minimum { minimum } else { value }
}
fn at_most(value: f64, maximum: f64) -> f64 {
    if value > maximum { maximum } else { value }
}
fn blend(measured: f64, sample: f64) -> f64 {
    if measured.is_nan() {
        sample
    } else {
        measured + (sample - measured) * 0.5
    }
}
impl Handling {
    pub fn last_move(&self) -> Option<Vec3> {
        self.last_move
    }
    pub fn drag(&self) -> f64 {
        if_nan(self.measured_drag, || {
            if self.control == Acceleration::Momentum {
                f64::from(0.91_f32)
            } else {
                0.0
            }
        })
    }
    pub fn vertical_drag(&self) -> f64 {
        if_nan(self.measured_vertical_drag, || {
            if self.control == Acceleration::Momentum {
                FlyAbilities::VERTICAL_DRAG
            } else {
                0.0
            }
        })
    }
    fn sprint_thrust(&self) -> f64 {
        if_nan(
            if_nan(self.measured_sprint_thrust, || self.measured_thrust),
            || self.base_speed * (1.0 - self.drag()),
        )
    }
    fn thrust(&self) -> f64 {
        if_nan(
            if_nan(self.measured_thrust, || self.measured_sprint_thrust),
            || {
                self.base_speed
                    * (1.0 - self.drag())
                    * if self.control == Acceleration::Momentum {
                        0.5
                    } else {
                        1.0
                    }
            },
        )
    }
    fn vertical_thrust(&self) -> f64 {
        if_nan(self.measured_vertical_thrust, || {
            self.base_vertical_speed * (1.0 - self.vertical_drag())
        })
    }
    pub fn speed(&self) -> f64 {
        at_least(self.sprint_thrust() / (1.0 - self.drag()), 0.05)
    }
    pub fn vertical_speed(&self) -> f64 {
        at_least(self.vertical_thrust() / (1.0 - self.vertical_drag()), 0.05)
    }
    pub fn step(&self) -> f64 {
        self.thrust() / (1.0 - self.drag())
    }
    pub fn coasts(&self) -> bool {
        self.drag() > 0.5
    }
    pub fn coast_distance(&self) -> f64 {
        self.speed() * self.drag() / (1.0 - self.drag())
    }
    pub fn abilities(&mut self, base: FlyAbilities) -> FlyAbilities {
        if base.control() != self.control
            || base.horizontal_speed() != self.base_speed
            || base.vertical_speed() != self.base_vertical_speed
        {
            self.control = base.control();
            self.base_speed = base.horizontal_speed();
            self.base_vertical_speed = base.vertical_speed();
            self.measured_drag = f64::NAN;
            self.measured_vertical_drag = f64::NAN;
            self.measured_thrust = f64::NAN;
            self.measured_sprint_thrust = f64::NAN;
            self.measured_vertical_thrust = f64::NAN;
        }
        FlyAbilities::new(
            self.control,
            self.speed(),
            self.vertical_speed(),
            base.is_flying(),
        )
    }
    pub fn record(&mut self, player: &impl FlightPlayer, pressed: Option<Input>) {
        let position = player.position();
        let movement = self.last_position.and_then(|last| {
            (player.tick_count() == self.last_tick.wrapping_add(1)).then(|| {
                Vec3::new(
                    position.x - last.x,
                    position.y - last.y,
                    position.z - last.z,
                )
            })
        });
        if let (Some(pressed), Some(movement), Some(before)) = (pressed, movement, self.last_move)
            && movement.x * movement.x + movement.y * movement.y + movement.z * movement.z < 100.0
        {
            if !player.horizontal_collision() {
                self.measure_level(
                    movement,
                    before,
                    pressed,
                    player.y_rot(),
                    player.is_sprinting(),
                );
            }
            let held = self
                .last_pressed
                .is_some_and(|last| last.jump == pressed.jump && last.shift == pressed.shift);
            if !player.vertical_collision() && held {
                self.measure_vertical(movement.y, before.y, pressed);
            }
        }
        self.last_position = Some(position);
        self.last_move = movement;
        self.last_tick = player.tick_count();
        self.last_pressed = pressed;
    }
    pub fn rest(&self, player: &impl FlightPlayer) -> Vec3 {
        let movement = self.last_move.unwrap_or_else(|| player.delta_movement());
        let coast = self.drag() / (1.0 - self.drag());
        let position = player.position();
        Vec3::new(
            position.x + movement.x * coast,
            position.y + movement.y * self.vertical_drag() / (1.0 - self.vertical_drag()),
            position.z + movement.z * coast,
        )
    }
    fn measure_level(
        &mut self,
        movement: Vec3,
        before: Vec3,
        pressed: Input,
        y_rot: f32,
        sprint: bool,
    ) {
        let strafe = f64::from(u8::from(pressed.left)) - f64::from(u8::from(pressed.right));
        let forward = f64::from(u8::from(pressed.forward)) - f64::from(u8::from(pressed.backward));
        if strafe == 0.0 && forward == 0.0 {
            self.measure_drag(movement, before.x, before.z);
            return;
        }
        let yaw = f64::from(y_rot).to_radians();
        let length = (strafe * strafe + forward * forward).sqrt();
        let direction_x = (strafe * super::flight_math::cos(yaw)
            - forward * super::flight_math::sin(yaw))
            / length;
        let direction_z = (forward * super::flight_math::cos(yaw)
            + strafe * super::flight_math::sin(yaw))
            / length;
        let along = before.x * direction_x + before.z * direction_z;
        self.measure_drag(
            movement,
            before.x - along * direction_x,
            before.z - along * direction_z,
        );
        let drag = self.drag();
        let x = movement.x - before.x * drag;
        let z = movement.z - before.z * drag;
        let added = (x * x + z * z).sqrt();
        if sprint {
            self.measured_sprint_thrust = blend(self.measured_sprint_thrust, added);
        } else {
            self.measured_thrust = blend(self.measured_thrust, added);
        }
    }
    fn measure_drag(&mut self, movement: Vec3, x: f64, z: f64) {
        let length_sqr = x * x + z * z;
        if length_sqr <= 0.02 * 0.02 {
            return;
        }
        if self.measured_drag.is_nan() {
            self.measured_thrust = f64::NAN;
            self.measured_sprint_thrust = f64::NAN;
        }
        let kept = (movement.x * x + movement.z * z) / length_sqr;
        self.measured_drag = blend(self.measured_drag, at_most(at_least(kept, 0.0), 0.95));
    }
    fn measure_vertical(&mut self, movement: f64, before: f64, pressed: Input) {
        if pressed.jump != pressed.shift {
            self.measured_vertical_thrust = blend(
                self.measured_vertical_thrust,
                (movement - before * self.vertical_drag()).abs(),
            );
        } else if before.abs() > 0.02 {
            let kept = movement / before;
            if (0.0..=1.0).contains(&kept) {
                if self.measured_vertical_drag.is_nan() {
                    self.measured_vertical_thrust = f64::NAN;
                }
                self.measured_vertical_drag =
                    blend(self.measured_vertical_drag, at_most(kept, 0.95));
            }
        }
    }
}
