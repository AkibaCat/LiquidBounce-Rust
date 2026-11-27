// SPDX-License-Identifier: GPL-3.0-or-later
//! navigation/glide/GlideCapture.kt, including offhand/hotbar/chest getter order.
use super::abilities::{GlideAbilities, GlideStack};
pub trait GlideInventory {
    type Stack: GlideStack;
    fn item(&self, slot: i32) -> Self::Stack;
}
pub trait EquippedGlidePlayer {
    type Stack: GlideStack;
    type Inventory: GlideInventory<Stack = Self::Stack>;
    fn inventory(&self) -> &Self::Inventory;
    fn offhand_item(&self) -> Self::Stack;
    fn chest_item(&self) -> Self::Stack;
}
pub fn capture_glide<P: EquippedGlidePlayer>(
    can_elytra: bool,
    use_rockets: bool,
    player: impl FnOnce() -> P,
) -> Option<GlideAbilities> {
    if !can_elytra {
        return None;
    }
    let player = player();
    let inventory = player.inventory();
    let mut stacks = Vec::with_capacity(10);
    stacks.push(player.offhand_item());
    for slot in 0..9 {
        stacks.push(inventory.item(slot));
    }
    GlideAbilities::of(&player.chest_item(), &stacks, use_rockets)
}
