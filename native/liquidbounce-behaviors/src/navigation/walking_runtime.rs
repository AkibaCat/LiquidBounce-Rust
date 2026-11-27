// SPDX-License-Identifier: GPL-3.0-or-later
//! Owning walking state shared by Course and Navigator.
use super::{
 background::Executor, geometry::{BlockPos, Vec3}, owned_search::OwnedTerrainSearch,
 walk::route::Route,
 walk_control::{
  block_actions::{ActionHost, BlockActions, PlacementOrder}, body::ControlState,
  break_times::BreakState, follower::Follower, hops::Hops,
  planning::{Planning, WalkSearchFactory}, setbacks::Setbacks,
 },
};
use std::sync::Arc;

pub struct WalkingRuntime<B: ControlState, T, S, O: PlacementOrder = StablePlacementOrder> {
 pub actions: BlockActions<O>,
 pub follower: Follower<B>,
 pub planning: Planning<B, T, S>,
 pub hops: Hops,
 setbacks: Setbacks,
}
impl<B: ControlState + BreakState + 'static, T: Send + 'static,
 S: OwnedTerrainSearch<T, Route = Route<B>> + Send + 'static, O: PlacementOrder>
 WalkingRuntime<B, T, S, O> {
 pub fn new(executor: Arc<dyn Executor>, search: WalkSearchFactory<B, T, S>, order: O) -> Self {
  Self { actions: BlockActions::new(order), follower: Follower::default(),
   planning: Planning::new(executor, search), hops: Hops::default(), setbacks: Setbacks::default() }
 }
 /// Called after the visible turn and before Heading looks along the route.
 pub fn start_tick(&mut self, host: &mut impl ActionHost<State = B>) {
  self.actions.tick(host);
  self.actions.aim(host);
 }
 pub fn reset(&mut self, host: &mut impl ActionHost<State = B>) {
  self.planning.reset();
  self.follower.clear();
  self.actions.idle(host);
 }
 pub fn block_changed(&mut self, pos: BlockPos) { self.planning.block_changed(pos); }
 pub fn block_confirmed(&mut self, host: &mut impl ActionHost<State = B>, pos: BlockPos, state: &B) -> bool {
  let put_back = self.actions.answer(host, pos, state);
  if put_back { self.planning.stuck_at(pos); }
  put_back
 }
 pub fn corrected(&mut self, refused: Vec3, ticks: i64) -> Vec<BlockPos> {
  self.follower.position_corrected();
  let spots = self.setbacks.refused(refused, ticks);
  for &pos in &spots { self.planning.stuck_at(pos); }
  spots
 }
 /// Does not read the old player, which may already be gone.
 pub fn level_changed(&mut self) {
  self.planning.reset();
  self.follower.clear();
  self.planning.forget_stuck_spots();
  self.actions.level_changed();
  self.planning.level_changed();
 }
}

#[derive(Clone, Copy)]
struct OrderedKey { pos: BlockPos, hash: u32, identity: u64 }
/// Deterministic native pending-map order. Ordinary JDK HashMap list buckets,
/// hash spreading, insertion, retained capacity, removal and resize splitting
/// are reproduced. A heavily colliding tree bin keeps stable linked order and
/// sets `has_tree_bin`; JVM identity-hash tree ordering remains an explicit
/// parity boundary. This usable native fallback does not claim that boundary.
#[derive(Default)]
pub struct StablePlacementOrder {
 buckets: Vec<Vec<OrderedKey>>,
 size: usize,
 serial: u64,
 has_tree_bin: bool,
}
impl StablePlacementOrder {
 fn hash(pos: BlockPos) -> u32 {
  let hash = pos.y.wrapping_add(pos.z.wrapping_mul(31)).wrapping_mul(31).wrapping_add(pos.x) as u32;
  hash ^ (hash >> 16)
 }
 fn resize(&mut self) {
  let capacity = if self.buckets.is_empty() { 16 } else { self.buckets.len() * 2 };
  let mut buckets = vec![Vec::new(); capacity];
  for bucket in self.buckets.drain(..) {
   for key in bucket { buckets[key.hash as usize & (capacity - 1)].push(key); }
  }
  self.buckets = buckets;
 }
 pub fn capacity(&self) -> usize { self.buckets.len() }
 pub fn has_tree_bin(&self) -> bool { self.has_tree_bin }
 pub fn identity(&self, pos: BlockPos) -> Option<u64> {
  if self.buckets.is_empty() { return None; }
  self.buckets[Self::hash(pos) as usize & (self.buckets.len()-1)].iter()
   .find(|key| key.pos == pos).map(|key| key.identity)
 }
}
impl PlacementOrder for StablePlacementOrder {
 fn inserted(&mut self, pos: BlockPos) {
  if self.buckets.is_empty() { self.resize(); }
  let hash = Self::hash(pos);
  let bucket = hash as usize & (self.buckets.len()-1);
  if self.buckets[bucket].iter().any(|key| key.pos == pos) { return; }
  self.serial = self.serial.wrapping_add(1);
  self.buckets[bucket].push(OrderedKey { pos, hash, identity: self.serial });
  self.size += 1;
  if self.buckets[bucket].len() >= 9 {
   if self.buckets.len() < 64 { self.resize(); } else { self.has_tree_bin = true; }
  }
  if self.size > self.buckets.len() * 3 / 4 { self.resize(); }
 }
 fn removed(&mut self, pos: BlockPos) {
  if self.buckets.is_empty() { return; }
  let bucket = Self::hash(pos) as usize & (self.buckets.len()-1);
  if let Some(index) = self.buckets[bucket].iter().position(|key| key.pos == pos) {
   self.buckets[bucket].remove(index); self.size -= 1;
  }
 }
 fn clear(&mut self) {
  for bucket in &mut self.buckets { bucket.clear(); }
  self.size = 0;
  self.has_tree_bin = false;
 }
 fn keys(&self) -> Vec<BlockPos> {
  self.buckets.iter().flat_map(|bucket| bucket.iter().map(|key| key.pos)).collect()
 }
}
