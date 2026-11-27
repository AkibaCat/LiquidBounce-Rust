// SPDX-License-Identifier: GPL-3.0-or-later
use super::super::{aabb::Aabb, geometry::Vec3, input::Input, terrain::TerrainLevel};
use super::{moves::RideState, vehicle::VehicleEntity};

pub trait RideEntity: VehicleEntity + Clone {
    fn position(&self) -> Vec3;
    fn interpolated_position(&self, partial_ticks: f32) -> Vec3;
    fn delta_movement(&self) -> Vec3;
    fn x(&self) -> f64;
    fn z(&self) -> f64;
    fn y_rot(&self) -> f32;
    fn x_rot(&self) -> f32;
    fn is_passenger(&self) -> bool;
    fn controlled_vehicle(&self) -> Option<Self>;
    fn bounding_box(&self) -> Aabb;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RideLook {
    View,
    Aim,
}
pub trait RideBody {
    type Entity: RideEntity;
    type State: RideState;
    type Level: TerrainLevel<Self::State>;
    fn player(&self) -> Self::Entity;
    fn level(&self) -> Self::Level;
    fn press(&mut self, input: Input, exact: bool);
    fn look(&mut self, y_rot: f32, x_rot: f32, purpose: RideLook);
}
