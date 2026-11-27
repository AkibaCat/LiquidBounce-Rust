// SPDX-License-Identifier: GPL-3.0-or-later
//! Live facts/actions for complete Gliding. Host handles remain on the game thread.
use super::super::{
    aabb::Direction,
    geometry::{BlockPos, Vec3},
    input::Input,
    terrain::{TerrainLevel, TerrainState},
};
use super::{capture::EquippedGlidePlayer, contacts::ContactPlayer};
pub trait GlidePlayer: ContactPlayer + EquippedGlidePlayer {
    fn position(&self) -> Vec3;
    fn on_ground(&self) -> bool;
    fn delta_movement(&self) -> Vec3;
    fn block_position(&self) -> BlockPos;
}
pub trait GlideLevel<B: TerrainState>: TerrainLevel<B> + Clone {
    fn has_ceiling(&self) -> bool;
    fn has_chunk_at(&self, pos: BlockPos) -> bool;
    fn has_column(&self, x: i32, z: i32) -> bool;
    fn column_height(&self, x: i32, z: i32) -> i32;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlideLook {
    Movement,
    Steer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlideHand {
    Main,
    Off,
}
pub trait GlideBody {
    type Player: GlidePlayer;
    type State: TerrainState;
    type Level: GlideLevel<Self::State>;
    fn player(&self) -> &Self::Player;
    fn level(&self) -> Self::Level;
    fn glide_clearance(&self) -> f64;
    fn safe_fall(&self) -> i32;
    fn direction_hash(&self, direction: Direction) -> i32 {
        rust_direction_identity(direction)
    }
    fn press(&mut self, input: Input, exact: bool);
    fn look(&mut self, y_rot: f32, x_rot: f32, purpose: GlideLook);
    fn select_hotbar_slot(&mut self, slot: i32);
    fn release_hotbar_slot(&mut self);
    fn use_item(&mut self, hand: GlideHand);
}

/// Stable identities for Rust's six singleton directions. Java enum identity
/// hashes are unspecified process facts; only their stable assignment matters.
pub const fn rust_direction_identity(direction: Direction) -> i32 {
    match direction {
        Direction::Down => 1,
        Direction::Up => 2,
        Direction::North => 3,
        Direction::South => 4,
        Direction::West => 5,
        Direction::East => 6,
    }
}
