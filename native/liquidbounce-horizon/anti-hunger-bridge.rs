// SPDX-License-Identifier: GPL-3.0-or-later
//! Borrowed access to the actual client player during ordinary packet dispatch.
//! This is installed only around a split Game send. It never owns a Game pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub swimming: bool,
    pub passenger: bool,
    pub destroying: bool,
    pub in_water: bool,
    pub under_water: bool,
    pub fall_distance: f64,
}
pub trait ClientPacketPlayer {
    fn snapshot(&self) -> Snapshot;
    /// Completes the actual MotionWorld jump before this callback returns.
    fn jump_from_ground(&self);
}
