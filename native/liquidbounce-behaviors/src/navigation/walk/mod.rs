// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac walk source units. The complete AutoWalk remains
//! unregistered until route generation, control and native binding are complete.
pub mod abilities;
pub mod changes;
pub mod costs;
pub mod course;
pub mod nodes;
pub mod moves;
pub mod parkour;
pub mod route;
pub mod route_search;
pub mod straight_walk;
pub mod water;

use super::terrain::TerrainState;
/// Live registry/state facts used by the original walk and simulation sources.
/// Implementations must expose the real block values rather than infer them
/// from a navigation collision shape (which may represent an opened door).
pub trait WalkState: TerrainState {
    fn is_air(&self) -> bool;
    fn can_be_replaced(&self) -> bool;
    fn is_falling(&self) -> bool;
    fn is_slime(&self) -> bool;
    fn friction(&self) -> f32;
    fn speed_factor(&self) -> f32;
    fn jump_factor(&self) -> f32;
}
