// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleNoEntityInteract and Filter, including the pinned registry-comparator bug.
use std::collections::BTreeSet;

#[path = "no_interact_items.rs"]
pub mod item_facts;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    Whitelist,
    Blacklist,
}
impl Filter {
    pub fn includes(self, member: bool) -> bool {
        match self {
            Self::Whitelist => member,
            Self::Blacklist => !member,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoEntityInteract {
    pub entity_type_filter: Filter,
    pub entity_types: BTreeSet<String>,
    pub holding_item_filter: Filter,
    pub holding_items: BTreeSet<String>,
}
impl Default for NoEntityInteract {
    fn default() -> Self {
        Self {
            entity_type_filter: Filter::Blacklist,
            entity_types: ["minecraft:armor_stand", "minecraft:villager"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            holding_item_filter: Filter::Whitelist,
            holding_items: item_facts::DEFAULT_HOLDING_ITEMS
                .iter()
                .map(|id| (*id).to_owned())
                .collect(),
        }
    }
}
impl NoEntityInteract {
    pub fn test(&self, running: bool, main_hand_item: &str) -> bool {
        // Source passes Entity, not EntityType, into the registry-sorted set.
        // DefaultedMappedRegistry.getKey(Entity) falls back to minecraft:pig.
        !running
            || (self
                .entity_type_filter
                .includes(self.entity_types.contains("minecraft:pig"))
                && self
                    .holding_item_filter
                    .includes(self.holding_items.contains(main_hand_item)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn entity_instance_lookup_uses_registry_default_pig_even_for_armor_stands() {
        let mut module = NoEntityInteract::default();
        assert!(module.test(true, "minecraft:air"));
        module.entity_type_filter = Filter::Whitelist;
        assert!(!module.test(true, "minecraft:air"));
        module.entity_types.insert("minecraft:pig".into());
        assert!(module.test(true, "minecraft:air"));
        module.entity_type_filter = Filter::Blacklist;
        assert!(!module.test(true, "minecraft:air"));
    }
    #[test]
    fn default_holding_items_are_four_tool_tags_and_six_explicit_items() {
        let module = NoEntityInteract::default();
        assert_eq!(module.holding_items.len(), 34);
        assert!(module.test(true, "minecraft:copper_pickaxe"));
        assert!(module.test(true, "minecraft:shears"));
        assert!(!module.test(true, "minecraft:diamond_sword"));
        assert!(!module.test(true, "minecraft:diamond_spear"));
    }
    #[test]
    fn holding_filter_is_independent_and_disabled_module_always_passes() {
        let mut module = NoEntityInteract {
            holding_item_filter: Filter::Blacklist,
            ..Default::default()
        };
        assert!(!module.test(true, "minecraft:air"));
        assert!(module.test(true, "minecraft:apple"));
        module.entity_type_filter = Filter::Whitelist;
        assert!(module.test(false, "minecraft:air"));
        assert!(module.test(false, "minecraft:apple"));
    }
}
