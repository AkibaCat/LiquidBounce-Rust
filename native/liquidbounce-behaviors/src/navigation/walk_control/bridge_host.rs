// SPDX-License-Identifier: GPL-3.0-or-later
//! Direct host contracts of Crossing/Telly. Each operation retains its original
//! controller call site; the host binds real BlockActions/BlockTargets/StepCheck.
use super::{
    body::WalkBody, bridging::Bridging, turning::LookPurpose, view_rotation::ViewRotation,
};
use crate::navigation::geometry::{BlockPos, Vec3};

/// Stable identity of a PlacedBlock object, not its value or position. A host
/// should allocate a fresh identity for every successfully clicked placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacementIdentity(pub u64);

pub trait BridgeHost: WalkBody {
    fn sent_rotation(&self) -> ViewRotation;
    fn look(&mut self, rotation: ViewRotation, purpose: LookPurpose);
    fn is_placed(&self, pos: BlockPos) -> bool;
    fn place(&mut self, pos: BlockPos, ready: bool, held: ViewRotation);
    fn placement(&self) -> Option<PlacementIdentity>;
    fn rejections(&self) -> i32;
    fn action_rotation(&self) -> Option<ViewRotation>;
    fn is_unconfirmed(&self, pos: BlockPos) -> bool;
    fn bridging(&self) -> Bridging;
    fn risky_bridges(&self) -> bool;
    fn sneaking_speed(&self) -> f64;
    fn stride_moved(&self) -> Vec3;
    fn clicks_from(&self, eye: Vec3, rotation: ViewRotation, floor: BlockPos) -> bool;
    fn placeable_slot(&self) -> Option<i32>;
    fn placement_rotation(&self, pos: BlockPos, slot: i32, nearest: bool) -> Option<ViewRotation>;
    fn crossing_started(&self, length: usize, method: Bridging);
    fn crossing_fallback(&self, method: Bridging, fallback: Option<Bridging>);
    fn crossing_hopped(&self, method: Option<Bridging>, placed: i32);
}
