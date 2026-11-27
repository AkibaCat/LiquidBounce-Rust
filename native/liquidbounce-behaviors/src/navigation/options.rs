// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent navigation Options and the original default placement policy.
use super::walk_control::{
 ability_capture::{BreakPolicy, CaptureOptions},
 block_targets::PlaceableBlocks,
 bridging::Bridging,
};
use std::{collections::HashSet, sync::Arc};

pub type StackPolicy<T> = Arc<dyn Fn(&T) -> bool>;
pub struct PlacementPolicy<T> {
 pub placeable: StackPolicy<T>,
 pub unfavourable: StackPolicy<T>,
}
impl<T> Clone for PlacementPolicy<T> {
 fn clone(&self) -> Self { Self { placeable: self.placeable.clone(), unfavourable: self.unfavourable.clone() } }
}
impl<T> PlaceableBlocks<T> for PlacementPolicy<T> {
 fn is_placeable(&self, stack: &T) -> bool { (self.placeable)(stack) }
 fn is_unfavourable(&self, stack: &T) -> bool { (self.unfavourable)(stack) }
}
/// Raw default-block facts read by PlaceableBlocks.DEFAULT, in source order.
pub trait DefaultPlaceableBlock {
 type State;
 fn default_state(&self) -> Self::State;
 fn is_entity_block(&self) -> bool;
 fn is_falling(&self) -> bool;
 fn is_interactive(&self) -> bool;
 fn collision_full_in_empty_world(&self, state: &Self::State) -> bool;
 fn friction(&self) -> f32;
 fn speed_factor(&self) -> f32;
 fn jump_factor(&self) -> f32;
}
pub trait DefaultPlaceableStack {
 type Block: DefaultPlaceableBlock;
 fn item_block(&self) -> Option<Self::Block>;
}
impl<T: DefaultPlaceableStack + 'static> PlacementPolicy<T> {
 pub fn default_blocks() -> Self {
  Self {
   placeable: Arc::new(|stack| {
    let Some(block) = stack.item_block() else { return false; };
    let state = block.default_state();
    !block.is_entity_block() && !block.is_falling() && !block.is_interactive()
      && block.collision_full_in_empty_world(&state)
   }),
   unfavourable: Arc::new(|stack| {
    let Some(block) = stack.item_block() else { return false; };
    block.friction() != 0.6_f32 || block.speed_factor() != 1.0 || block.jump_factor() != 1.0
   }),
  }
 }
}
#[derive(Clone)]
pub struct Options<B, T> {
 pub can_break: bool,
 pub can_place: bool,
 pub can_parkour: bool,
 pub risky_jumps: bool,
 pub max_fall: i32,
 pub path_range: i32,
 pub can_fly: bool,
 pub can_elytra: bool,
 pub elytra_rockets: bool,
 pub look_ahead: bool,
 pub look_ahead_turn_speed: f32,
 pub look_ahead_distance: f32,
 pub look_ahead_x_rot: f32,
 pub fast_bridges: bool,
 pub places_bridge_floors: bool,
 pub placeable_blocks: PlacementPolicy<T>,
 pub can_click_through_blocks: bool,
 pub glide_clearance: f64,
 pub min_press_ticks: i32,
 pub min_release_ticks: i32,
 pub bridging: Bridging,
 pub risky_bridges: bool,
 pub breakable_blocks: BreakPolicy<B>,
 pub can_boat: bool,
 pub can_eat: bool,
 pub reserved: Arc<HashSet<i64>>,
}
impl<B: 'static, T> Options<B, T> {
 pub fn new(placeable_blocks: PlacementPolicy<T>) -> Self {
  Self {
   can_break: false, can_place: false, can_parkour: true, risky_jumps: false,
   max_fall: 3, path_range: 64, can_fly: true, can_elytra: true, elytra_rockets: true,
   look_ahead: true, look_ahead_turn_speed: 10.0, look_ahead_distance: 4.0,
   look_ahead_x_rot: 8.0, fast_bridges: false, places_bridge_floors: false,
   placeable_blocks, can_click_through_blocks: false, glide_clearance: 0.0,
   min_press_ticks: 0, min_release_ticks: 0, bridging: Bridging::Auto,
   risky_bridges: false, breakable_blocks: Arc::new(|_| true), can_boat: false,
   can_eat: false, reserved: Arc::new(HashSet::new()),
  }
 }
 pub fn max_length(&self) -> f64 { f64::from(self.path_range) }
 pub fn capture(&self) -> CaptureOptions<B, T> {
  CaptureOptions {
   can_break: self.can_break, can_place: self.can_place, can_parkour: self.can_parkour,
   risky_jumps: self.risky_jumps, max_fall: self.max_fall, fast_bridges: self.fast_bridges,
   bridging: self.bridging, risky_bridges: self.risky_bridges,
   placeable: self.placeable_blocks.placeable.clone(), breakable: self.breakable_blocks.clone(),
   reserved: self.reserved.clone(),
  }
 }
}
impl<B: 'static, T: DefaultPlaceableStack + 'static> Default for Options<B, T> {
 fn default() -> Self { Self::new(PlacementPolicy::default_blocks()) }
}
