// SPDX-License-Identifier: GPL-3.0-or-later
use super::super::geometry::{Vec3, java_max};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VehicleKind {
    Boat,
    Mount,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    Boat,
    Horse,
    Camel,
    Other,
}
pub trait VehicleEntity {
    fn entity_kind(&self) -> EntityKind;
    fn passenger_riding_position(&self, rider: &Self) -> Vec3;
    fn y(&self) -> f64;
    fn bounding_height(&self) -> f32;
    fn bounding_width(&self) -> f32;
    fn eye_height(&self) -> f32;
    fn max_up_step(&self) -> f32;
    fn can_jump(&self) -> bool;
    fn jump_strength(&self) -> f64;
    fn jump_boost_power(&self) -> f32;
    fn safe_fall_distance(&self) -> f64;
    fn movement_speed(&self) -> f64;
}
pub const JUMP_CHARGE_TICKS: i32 = 11;
pub const BOAT_WATER_SPEED: f64 = 0.35;
pub const BOAT_ICE_SPEED: f64 = 1.0;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vehicle {
    pub kind: VehicleKind,
    pub width: f64,
    pub height: f64,
    pub step_height: f64,
    pub jump_height: f64,
    pub safe_fall: i32,
    pub speed: f64,
    pub ice_speed: f64,
}
impl Vehicle {
    pub fn of<E: VehicleEntity>(rider: &E, vehicle: &E) -> Option<Self> {
        let seat = vehicle.passenger_riding_position(rider).y - vehicle.y();
        let height = java_max(
            f64::from(vehicle.bounding_height()),
            seat + f64::from(rider.eye_height()) + 0.05,
        );
        match vehicle.entity_kind() {
            EntityKind::Boat => Some(Self {
                kind: VehicleKind::Boat,
                width: f64::from(vehicle.bounding_width()),
                height,
                step_height: 0.0,
                jump_height: 0.0,
                safe_fall: 1,
                speed: BOAT_WATER_SPEED,
                ice_speed: BOAT_ICE_SPEED,
            }),
            EntityKind::Horse | EntityKind::Camel => {
                let width = f64::from(vehicle.bounding_width());
                let step_height = f64::from(vehicle.max_up_step());
                let jump_height = jump_height_of(vehicle);
                let safe_fall = (vehicle.safe_fall_distance() as i32).wrapping_sub(1);
                let speed = vehicle.movement_speed() * 2.158;
                Some(Self {
                    kind: VehicleKind::Mount,
                    width,
                    height,
                    step_height,
                    jump_height,
                    safe_fall,
                    speed,
                    ice_speed: speed,
                })
            }
            EntityKind::Other => None,
        }
    }
}
pub fn jump_height_of<E: VehicleEntity>(mount: &E) -> f64 {
    if mount.entity_kind() == EntityKind::Camel || !mount.can_jump() {
        return 0.0;
    }
    let mut velocity = mount.jump_strength() + f64::from(mount.jump_boost_power());
    let mut height = 0.0;
    while velocity > 0.0 {
        height += velocity;
        velocity = (velocity - 0.08) * f64::from(0.98_f32);
    }
    java_max(height - 0.2, 0.0)
}
