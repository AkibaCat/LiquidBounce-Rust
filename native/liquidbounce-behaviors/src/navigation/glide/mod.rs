// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent glide prediction and controllers.
pub mod abilities;
pub mod airway_search;
pub mod airways;
pub mod body;
pub mod capture;
pub mod contacts;
pub mod cruise;
pub mod elytra_motion;
pub mod gliding;
pub mod landing;
pub mod passage;
pub mod rockets;
pub mod roof_bounce;
pub mod route_search;
pub mod search_factory;
use super::geometry::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Controls {
    pub y_rot: f32,
    pub x_rot: f32,
    pub should_fire_rocket: bool,
    pub is_boosted: bool,
}
pub trait Steering {
    fn target(&self) -> Vec3;
    fn steer(&mut self, position: Vec3, delta_movement: Vec3) -> Controls;
}
