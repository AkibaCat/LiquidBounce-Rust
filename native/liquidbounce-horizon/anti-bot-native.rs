// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::items::ItemStack;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Default)]
pub(super) struct Native {
    pub(super) profiles: HashMap<u128, (bool, Option<i32>)>,
    pub(super) entities: HashMap<i32, bool>,
    pub(super) scales: HashMap<i32, f64>,
}
impl Game<'_> {
    pub(super) fn client_anti_bot_armor(&self, stack: Option<&ItemStack>) -> Value {
        let Some(stack) = stack.filter(|s| s.count > 0 && s.item.as_ref() != "minecraft:air")
        else {
            return json!({"identity":0,"predicates":1,"playerArmor":false,"enchanted":false});
        };
        let id = stack.item.as_ref();
        let items = self.items();
        let tag = |name: &str| items.item_in_tag(name, id);
        let mut mask = 0u16;
        for (i, prefix) in [
            "leather",
            "chainmail",
            "iron",
            "golden",
            "diamond",
            "netherite",
        ]
        .into_iter()
        .enumerate()
        {
            if ["helmet", "chestplate", "leggings", "boots"]
                .into_iter()
                .any(|part| id == format!("minecraft:{prefix}_{part}"))
            {
                mask |= 1 << (i + 1);
            }
        }
        let view = horizon_client::item_render::stack_view(stack);
        if self
            .pack
            .item_renderer
            .models
            .components_view(&view)
            .get("minecraft:glider")
            .is_some()
        {
            mask |= 1 << 7;
        }
        if id == "minecraft:turtle_helmet" {
            mask |= 1 << 8;
        }
        if id == "minecraft:carved_pumpkin" {
            mask |= 1 << 9;
        }
        if tag("minecraft:skulls") {
            mask |= 1 << 10;
        }
        let armor = [
            "minecraft:head_armor",
            "minecraft:chest_armor",
            "minecraft:leg_armor",
            "minecraft:foot_armor",
        ]
        .into_iter()
        .any(tag);
        json!({"identity":stack.object_identity(),"predicates":mask,"playerArmor":armor,"enchanted":!stack.enchantments().is_empty()})
    }
}
