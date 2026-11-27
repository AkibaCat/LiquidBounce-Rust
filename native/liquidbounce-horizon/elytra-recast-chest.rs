// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleElytraRecast reads CHEST's components, without equippable matching.
use crate::elytra_recast::Chest;
use horizon_assets::item_model::ComponentsView;

pub fn chest(components: ComponentsView<'_>) -> Chest {
    let integer = |name: &str| {
        components
            .get(name)
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0) as i32
    };
    let max = integer("minecraft:max_damage");
    let damageable = components.contains_key("minecraft:max_damage")
        && !components.contains_key("minecraft:unbreakable")
        && components.contains_key("minecraft:damage");
    // ItemStack.getDamageValue clamps its Int component before comparison.
    let damage = integer("minecraft:damage").max(0).min(max);
    Chest {
        glider: components.contains_key("minecraft:glider"),
        next_damage_will_break: damageable && damage >= max.wrapping_sub(1),
    }
}
