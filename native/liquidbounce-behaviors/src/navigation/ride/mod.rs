// SPDX-License-Identifier: GPL-3.0-or-later
//! Ridden vehicle pathfinding and navigation.
pub mod boating;
pub mod body;
pub mod moves;
pub mod riding;
pub mod route_search;
pub mod steering;
pub mod vehicle;
#[path = "../walk_control/view_rotation.rs"]
mod view_rotation;
