// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleNoBlockInteract plus the exact Block/ItemStack.isInteractable predicates.
use serde_json::Value;

fn component_bool(value: &Value) -> Option<bool> {
    value
        .as_bool()
        .or_else(|| value.as_f64().map(|value| value != 0.0))
}

pub use crate::no_entity_interact::item_facts;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NoBlockInteract {
    pub sneaking: bool,
    pub interacting: bool,
}
impl NoBlockInteract {
    pub fn start_sneaking(&mut self) {
        self.sneaking = true;
    }
    pub fn movement_input(&mut self, running: bool, sneak: &mut bool) {
        if running && self.sneaking {
            *sneak = true;
            self.interacting = true;
        }
    }
    pub fn should_retry(&self, running: bool) -> bool {
        running && self.interacting
    }
    /// Call only after startUseItem returned: its nested block hook still sees
    /// both flags true, and any startSneaking from that call is cleared too.
    pub fn finish_retry(&mut self) {
        self.interacting = false;
        self.sneaking = false;
    }
}

pub fn block_is_interactable<'a>(
    has_class: impl Fn(&str) -> bool,
    property: impl Fn(&str) -> Option<&'a str>,
    needs_food: bool,
    game_master: bool,
) -> bool {
    const ALWAYS: &[&str] = &[
        "AbstractBedBlock",
        "AbstractChestBlock",
        "AbstractFurnaceBlock",
        "AnvilBlock",
        "BarrelBlock",
        "BeaconBlock",
        "BellBlock",
        "BrewingStandBlock",
        "ButtonBlock",
        "CandleCakeBlock",
        "CartographyTableBlock",
        "ComparatorBlock",
        "CrafterBlock",
        "CraftingTableBlock",
        "DaylightDetectorBlock",
        "DecoratedPotBlock",
        "DispenserBlock",
        "DoorBlock",
        "DragonEggBlock",
        "EnchantingTableBlock",
        "FenceGateBlock",
        "FlowerPotBlock",
        "GrindstoneBlock",
        "HopperBlock",
        "LecternBlock",
        "LeverBlock",
        "NoteBlock",
        "RedstoneWireBlock",
        "RepeaterBlock",
        "RespawnAnchorBlock",
        "ShulkerBoxBlock",
        "StonecutterBlock",
        "TrapDoorBlock",
    ];
    ALWAYS.iter().any(|class| has_class(class))
        || has_class("CakeBlock") && needs_food
        || (has_class("CaveVinesPlantBlock") || has_class("CaveVinesBlock"))
            && property("berries").unwrap_or("true") == "true"
        || has_class("ComposterBlock") && property("level").unwrap_or("8") == "8"
        || (has_class("GameMasterBlock") || has_class("LightBlock")) && game_master
        || has_class("JukeboxBlock") && property("has_record") == Some("true")
        || has_class("SweetBerryBushBlock")
            && property("age")
                .unwrap_or("2")
                .parse::<i32>()
                .is_ok_and(|age| age > 1)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ItemContext {
    pub equippable_allowed: bool,
    pub equipped_same_components: bool,
    pub equipped_prevents_armor_change: bool,
    pub creative: bool,
    pub has_inventory_arrow: bool,
    pub hand_has_firework: bool,
}

pub fn item_is_interactable(
    item: &str,
    count: u32,
    component: impl Fn(&str) -> Option<Value>,
    context: ItemContext,
) -> bool {
    if count == 0 || item == "minecraft:air" {
        return false;
    }
    if let Some(equippable) = component("minecraft:equippable")
        && equippable
            .get("swappable")
            .and_then(component_bool)
            .unwrap_or(true)
        && context.equippable_allowed
        && !context.equipped_same_components
        && (!context.equipped_prevents_armor_change || context.creative)
    {
        return true;
    }
    if [
        "minecraft:consumable",
        "minecraft:blocks_attacks",
        "minecraft:kinetic_weapon",
    ]
    .iter()
    .any(|id| component(id).is_some())
    {
        return true;
    }
    if let Some(effects) = component("minecraft:use_effects") {
        let speed = effects
            .get("speed_multiplier")
            .and_then(Value::as_f64)
            .unwrap_or(0.2) as f32;
        if speed.to_bits() != 0.2_f32.to_bits()
            || effects
                .get("can_sprint")
                .and_then(component_bool)
                .unwrap_or(false)
            || !effects
                .get("interact_vibrations")
                .and_then(component_bool)
                .unwrap_or(true)
        {
            return true;
        }
    }
    const ALWAYS: &[&str] = &[
        "BoatItem",
        "BucketItem",
        "EggItem",
        "EmptyMapItem",
        "EnderEyeItem",
        "EnderpearlItem",
        "ExperienceBottleItem",
        "FireworkRocketItem",
        "FishingRodItem",
        "BottleItem",
        "InstrumentItem",
        "KnowledgeBookItem",
        "PlaceOnWaterBlockItem",
        "SnowballItem",
        "SpawnEggItem",
        "SpyglassItem",
        "TridentItem",
        "WindChargeItem",
        "WritableBookItem",
        "WrittenBookItem",
        "ArmorStandItem",
        "BlockItem",
        "BrushItem",
        "HangingEntityItem",
        "FireChargeItem",
        "FlintAndSteelItem",
        "PotionItem",
    ];
    ALWAYS.iter().any(|class| item_facts::item_is(item, class))
        || item_facts::item_is(item, "BowItem") && context.has_inventory_arrow
        || item_facts::item_is(item, "CrossbowItem")
            && (context.has_inventory_arrow || context.hand_has_firework)
}

pub fn should_sneak(
    shift_down: bool,
    block_interactable: bool,
    main_interactable: bool,
    off_interactable: bool,
) -> bool {
    !shift_down && block_interactable && (main_interactable || off_interactable)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retry_waits_for_input_and_retains_flags_while_disabled() {
        let mut state = NoBlockInteract::default();
        state.start_sneaking();
        assert!(!state.should_retry(true));
        let mut sneak = false;
        state.movement_input(false, &mut sneak);
        assert!(!sneak);
        state.movement_input(true, &mut sneak);
        assert!(sneak && state.should_retry(true));
        assert!(!state.should_retry(false));
        assert!(state.sneaking && state.interacting);
        state.finish_retry();
        assert_eq!(state, NoBlockInteract::default());
    }
    #[test]
    fn source_special_block_properties_and_permissions() {
        let check = |class: &str, prop: Option<(&str, &str)>, hungry, master| {
            block_is_interactable(
                |name| name == class,
                |name| prop.filter(|(key, _)| *key == name).map(|(_, value)| value),
                hungry,
                master,
            )
        };
        assert!(!check("CakeBlock", None, false, false));
        assert!(check("CakeBlock", None, true, false));
        for class in ["CaveVinesPlantBlock", "CaveVinesBlock"] {
            assert!(check(class, None, false, false));
            assert!(!check(class, Some(("berries", "false")), false, false));
        }
        assert!(check("ComposterBlock", None, false, false));
        assert!(!check("ComposterBlock", Some(("level", "7")), false, false));
        assert!(!check("JukeboxBlock", None, false, false));
        assert!(check(
            "JukeboxBlock",
            Some(("has_record", "true")),
            false,
            false
        ));
        assert!(check("SweetBerryBushBlock", None, false, false));
        assert!(!check(
            "SweetBerryBushBlock",
            Some(("age", "1")),
            false,
            false
        ));
        for class in ["GameMasterBlock", "LightBlock"] {
            assert!(!check(class, None, false, false));
            assert!(check(class, None, false, true));
        }
    }
    #[test]
    fn arrow_and_firework_conditions_and_component_presence() {
        let check = |id, ctx| item_is_interactable(id, 1, |_| None, ctx);
        let mut ctx = ItemContext::default();
        assert!(!check("minecraft:bow", ctx));
        assert!(!check("minecraft:crossbow", ctx));
        ctx.hand_has_firework = true;
        assert!(!check("minecraft:bow", ctx));
        assert!(check("minecraft:crossbow", ctx));
        ctx.has_inventory_arrow = true;
        assert!(check("minecraft:bow", ctx));
        assert!(!check("minecraft:arrow", ctx));
        assert!(check("minecraft:glass_bottle", ctx));
        assert!(item_is_interactable(
            "minecraft:stick",
            1,
            |key| (key == "minecraft:consumable").then(|| serde_json::json!({})),
            ctx
        ));
        assert!(!item_is_interactable(
            "minecraft:air",
            1,
            |_| Some(serde_json::json!({})),
            ctx
        ));
        assert!(!item_is_interactable(
            "minecraft:stick",
            1,
            |key| (key == "minecraft:use_effects").then(|| serde_json::json!({})),
            ctx
        ));
        assert!(item_is_interactable(
            "minecraft:stick",
            1,
            |key| (key == "minecraft:use_effects")
                .then(|| serde_json::json!({"interact_vibrations":false})),
            ctx
        ));
    }
}
