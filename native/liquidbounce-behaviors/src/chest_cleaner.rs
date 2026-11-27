// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned ModuleChestCleaner and its shared screen checks.
use crate::vomit::Constraints;
use std::collections::BTreeSet;

pub const MENUS: [&str; 25] = [
    "anvil",
    "beacon",
    "blast_furnace",
    "brewing_stand",
    "cartography_table",
    "crafter_3x3",
    "crafting",
    "enchantment",
    "furnace",
    "generic_3x3",
    "generic_9x1",
    "generic_9x2",
    "generic_9x3",
    "generic_9x4",
    "generic_9x5",
    "generic_9x6",
    "grindstone",
    "hopper",
    "lectern",
    "loom",
    "merchant",
    "shulker_box",
    "smithing",
    "smoker",
    "stonecutter",
];
pub const TITLES: [&str; 16] = [
    "Barrel",
    "Beacon",
    "BlastFurnace",
    "BrewingStand",
    "Chest",
    "LargeChest",
    "Dispenser",
    "Dropper",
    "EnderChest",
    "Furnace",
    "Hopper",
    "ShulkerBox",
    "Smoker",
    "ChestMinecart",
    "ChestBoat",
    "HopperMinecart",
];
pub const TITLE_KEYS: [&[&str]; 16] = [
    &["container.barrel"],
    &["container.beacon"],
    &["container.blast_furnace"],
    &["container.brewing"],
    &["container.chest"],
    &["container.chestDouble"],
    &["container.dispenser"],
    &["container.dropper"],
    &["container.enderchest"],
    &["container.furnace"],
    &["container.hopper"],
    &["container.shulkerBox"],
    &["container.smoker"],
    &["entity.minecraft.chest_minecart"],
    &[
        "entity.minecraft.chest_boat",
        "entity.minecraft.acacia_chest_boat",
        "entity.minecraft.bamboo_chest_raft",
        "entity.minecraft.birch_chest_boat",
        "entity.minecraft.cherry_chest_boat",
        "entity.minecraft.dark_oak_chest_boat",
        "entity.minecraft.jungle_chest_boat",
        "entity.minecraft.mangrove_chest_boat",
        "entity.minecraft.oak_chest_boat",
        "entity.minecraft.pale_oak_chest_boat",
        "entity.minecraft.poplar_chest_boat",
        "entity.minecraft.spruce_chest_boat",
    ],
    &["entity.minecraft.hopper_minecart"],
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    Whitelist,
    Blacklist,
}
impl Filter {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Whitelist => "Whitelist",
            Self::Blacklist => "Blacklist",
        }
    }
    pub fn accepts(self, present: bool) -> bool {
        present == (self == Self::Whitelist)
    }
}
#[derive(Clone, Debug)]
pub struct ChestCleaner {
    pub filter: Filter,
    pub items: BTreeSet<String>,
    pub auto_close: bool,
    pub constraints: Constraints,
    pub check_type: bool,
    pub types: BTreeSet<String>,
    pub type_filter: Filter,
    pub check_title: bool,
    pub titles: u32,
    pub custom: BTreeSet<String>,
    pub title_filter: Filter,
}
impl Default for ChestCleaner {
    fn default() -> Self {
        Self {
            filter: Default::default(),
            items: Default::default(),
            auto_close: true,
            constraints: Default::default(),
            check_type: true,
            types: [
                "minecraft:generic_9x3",
                "minecraft:generic_9x6",
                "minecraft:shulker_box",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            type_filter: Default::default(),
            check_title: true,
            titles: [0, 4, 5, 11, 13, 14]
                .into_iter()
                .fold(0, |mask, i| mask | (1 << i)),
            custom: Default::default(),
            title_filter: Default::default(),
        }
    }
}
impl ChestCleaner {
    pub fn accepts_screen(
        &self,
        inventory_screen: bool,
        menu: Option<&str>,
        title: &str,
        translated: &[Vec<String>],
    ) -> bool {
        if inventory_screen {
            return false;
        }
        if self.check_type
            && !self
                .type_filter
                .accepts(menu.is_some_and(|m| self.types.contains(m)))
        {
            return false;
        }
        let present = self.custom.contains(title)
            || translated.iter().enumerate().any(|(i, keys)| {
                self.titles & (1 << i) != 0 && keys.iter().any(|key| key == title)
            });
        !self.check_title || self.title_filter.accepts(present)
    }
    pub fn selects(&self, item: &str, empty: bool) -> bool {
        !empty && self.filter.accepts(self.items.contains(item))
    }
}
