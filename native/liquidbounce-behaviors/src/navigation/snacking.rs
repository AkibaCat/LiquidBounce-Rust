// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent Snacking.kt and Hunger.kt selection without changing live getter order.
use super::walk_control::block_targets::Hand;
pub const HUNGRY: i32 = 14;
pub const FED: i32 = 18;
pub const INVENTORY_SIZE: i32 = 36;
pub const OFFHAND_SLOT: i32 = 40;
pub fn is_hotbar_slot(slot: i32) -> bool { (0..9).contains(&slot) }
pub trait PlainFoodStack {
 fn food_nutrition(&self) -> Option<i32>;
 fn is_golden_apple(&self) -> bool;
 fn is_enchanted_golden_apple(&self) -> bool;
 /// Rotten flesh, spider eye, poisonous potato, pufferfish, chorus fruit,
 /// raw chicken or suspicious stew, compared by actual item identity.
 fn is_bad_food(&self) -> bool;
}
pub trait FoodInventory {
 type Stack: PlainFoodStack;
 fn item(&self, slot: i32) -> Self::Stack;
}
pub fn is_plain_food(stack: &impl PlainFoodStack) -> bool {
 stack.food_nutrition().is_some() && !stack.is_golden_apple()
  && !stack.is_enchanted_golden_apple() && !stack.is_bad_food()
}
pub fn best_food_slot(inventory: &impl FoodInventory) -> i32 {
 let mut best = -1;
 let mut nutrition = 0;
 for slot in (0..INVENTORY_SIZE).chain(std::iter::once(OFFHAND_SLOT)) {
  let stack = inventory.item(slot);
  if !is_plain_food(&stack) { continue; }
  let value = stack.food_nutrition().unwrap_or(0);
  if value > nutrition { best = slot; nutrition = value; }
 }
 best
}
pub trait SnackHost {
 fn is_using_item(&self) -> bool;
 fn food_level(&self) -> i32;
 fn can_eat(&self) -> bool;
 fn is_in_water(&self) -> bool;
 fn on_ground(&self) -> bool;
 fn is_passenger(&self) -> bool;
 fn best_food_slot(&self) -> i32;
 fn can_click_inventory(&self) -> bool;
 fn should_stand(&mut self);
 fn suitable_hotbar_slot(&self) -> i32;
 fn swap_with_hotbar(&mut self, inventory: i32, hotbar: i32);
 fn select_hotbar_slot(&mut self, slot: i32);
 fn release_hotbar_slot(&mut self);
 fn use_item(&mut self, hand: Hand) -> bool;
}
pub struct Snacking { hotbar: i32, feeding: bool }
impl Default for Snacking {
 fn default() -> Self { Self { hotbar: -1, feeding: false } }
}
impl Snacking {
 pub fn tick(&mut self, host: &mut impl SnackHost, is_building: bool) {
  if self.hotbar >= 0 {
   if host.is_using_item() && !is_building {
    host.select_hotbar_slot(self.hotbar);
    return;
   }
   self.stop(host);
  }
  let hunger = host.food_level();
  if hunger >= FED { self.feeding = false; }
  if !host.can_eat() || hunger > HUNGRY && !self.feeding { return; }
  if is_building || host.is_using_item() || host.is_in_water()
    || !host.on_ground() && !host.is_passenger() { return; }
  let slot = host.best_food_slot();
  if slot < 0 { return; }
  if slot == OFFHAND_SLOT {
   if host.use_item(Hand::Off) { self.feeding = true; }
   return;
  }
  if !is_hotbar_slot(slot) && !host.can_click_inventory() {
   host.should_stand();
   return;
  }
  let held = if is_hotbar_slot(slot) { slot } else {
   let held = host.suitable_hotbar_slot();
   host.swap_with_hotbar(slot, held);
   held
  };
  host.select_hotbar_slot(held);
  if host.use_item(Hand::Main) { self.hotbar = held; self.feeding = true; }
 }
 pub fn stop(&mut self, host: &mut impl SnackHost) {
  if self.hotbar < 0 { return; }
  self.hotbar = -1;
  host.release_hotbar_slot();
 }
 pub fn state(&self) -> (i32, bool) { (self.hotbar, self.feeding) }
}
