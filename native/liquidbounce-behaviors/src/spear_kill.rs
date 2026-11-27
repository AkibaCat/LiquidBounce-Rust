// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSpearKill, LiquidBounce ae0eb396. State borrows never cross native calls.
use std::collections::VecDeque;
pub type Vector = [f64; 3];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub max_target_distance: f32,
    pub max_speed: f32,
    pub preview: bool,
    pub fill_color: i32,
    pub outline_color: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            max_target_distance: 50.0,
            max_speed: 7.0,
            preview: true,
            fill_color: 0x43ff0000,
            outline_color: 0xa7ffffffu32 as i32,
        }
    }
}
#[derive(Default, Debug)]
pub struct SpearKill {
    pub settings: Settings,
    pub attack_movements: VecDeque<Vector>,
    pub preview_target: Option<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub id: i32,
    pub distance: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kinetic {
    pub delay: i32,
    pub damage_duration: i32,
}
impl Kinetic {
    pub fn damage_use_duration(self) -> i32 {
        self.delay.wrapping_add(self.damage_duration)
    }
}
pub trait Host {
    fn with_spear_kill<R>(&mut self, operation: impl FnOnce(&mut SpearKill) -> R) -> R;
    fn using_spear(&mut self) -> bool;
    fn kinetic(&mut self) -> Option<Kinetic>;
    fn ticks_using(&mut self) -> i32;
    fn use_down(&mut self) -> bool;
    fn attack_down(&mut self) -> bool;
    fn restart_using(&mut self);
    fn target(&mut self, maximum: f32) -> Option<Target>;
    fn predict(&mut self, id: i32, ticks: i32) -> Option<Vector>;
    fn eye(&mut self) -> Vector;
    fn look(&mut self) -> Vector;
    fn velocity(&mut self, value: Vector);
}
fn normalize(value: Vector) -> Vector {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    if length < 1.0e-4 {
        [0.0; 3]
    } else {
        value.map(|v| v / length)
    }
}
fn length_squared(v: Vector) -> f64 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}
impl SpearKill {
    /// VehicleControl consults this only when SpearKill.enabled, including a
    /// suspended module. Disabling itself does not discard the source queue.
    pub fn current_charge_attack_movement(&self) -> Option<Vector> {
        self.attack_movements
            .front()
            .copied()
            .filter(|v| length_squared(*v) > 0.0)
    }
    fn reset(host: &mut impl Host) {
        host.with_spear_kill(|s| s.preview_target = None);
        if host.with_spear_kill(|s| !s.attack_movements.is_empty()) {
            host.velocity([0.0; 3]);
        }
        host.with_spear_kill(|s| s.attack_movements.clear());
    }
    pub fn tick(running: bool, host: &mut impl Host) {
        if !running {
            return;
        }
        if !host.using_spear() {
            Self::reset(host);
            return;
        }
        let settings = host.with_spear_kill(|s| s.settings);
        let target = if settings.preview {
            host.target(settings.max_target_distance)
        } else {
            None
        };
        host.with_spear_kill(|s| s.preview_target = target.map(|t| t.id));
        let Some(kinetic) = host.kinetic() else {
            Self::reset(host);
            return;
        };
        if host.use_down()
            && host.ticks_using() > kinetic.damage_use_duration()
            && host.with_spear_kill(|s| s.attack_movements.is_empty())
        {
            host.restart_using();
        }
        // Kinetic is the captured component; ticks are re-read after useItem.
        if host.ticks_using() < kinetic.delay {
            host.with_spear_kill(|s| s.attack_movements.clear());
            return;
        }
        if let Some(movement) = host.with_spear_kill(|s| s.attack_movements.pop_front()) {
            host.velocity(movement);
            return;
        }
        if host.ticks_using() >= kinetic.damage_use_duration() || !host.attack_down() {
            return;
        }
        let Some(target) = target.or_else(|| {
            let maximum = host.with_spear_kill(|s| s.settings.max_target_distance);
            host.target(maximum)
        }) else {
            return;
        };
        host.with_spear_kill(|s| s.preview_target = Some(target.id));
        let max_speed = host.with_spear_kill(|s| s.settings.max_speed);
        let ticks = (target.distance / f64::from(max_speed)).ceil() as i32;
        let ticks = ticks.max(1);
        let Some(predicted) = host.predict(target.id, ticks) else {
            return;
        };
        let eye = host.eye();
        let mut direction = normalize(std::array::from_fn(|a| predicted[a] - eye[a]));
        if length_squared(direction) <= 0.0 {
            direction = normalize(host.look());
        }
        let movement = direction.map(|v| v * (target.distance / f64::from(ticks)));
        host.with_spear_kill(|s| {
            s.attack_movements
                .extend(std::iter::repeat_n(movement, ticks as usize));
            s.attack_movements
                .extend(std::iter::repeat_n(movement.map(|v| -v), ticks as usize));
            s.attack_movements.push_back([0.0; 3]);
        });
        // Newly created movement is consumed on the following GameTickEvent.
    }
}
