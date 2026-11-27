#!/usr/bin/env python3
"""Expose exact component and server tag facts to the shared inventory actor."""


def apply(edit, once):
    def native(text):
        return once(text, '''        value["wire"] = serde_json::json!(horizon_items::item_data::canon_stack(stack));
        value
''', '''        value["wire"] = serde_json::json!(horizon_items::item_data::canon_stack(stack));
        let stack = stack.filter(|stack| stack.count > 0 && &*stack.item != "minecraft:air");
        let chest = stack.map_or(Default::default(), |stack| {
            let view = horizon_client::item_render::stack_view(stack);
            horizon_client::elytra_recast_chest::chest(self.pack.item_renderer.models.components_view(&view))
        });
        let chest_armor = stack.is_some_and(|stack| self.items().item_tags
            .get("minecraft:chest_armor").is_some_and(|tag| tag.contains(&*stack.item)));
        value["elytraSwap"] = serde_json::json!({"empty":stack.is_none(),
            "glider":chest.glider,"nextDamageWillBreak":chest.next_damage_will_break,
            "chestArmor":chest_armor});
        value
''')
    edit('crates/horizon-client/src/game/native_hooks.rs', native)
    return {'hook_names': [], 'helper_names': []}
