// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoF5 and InventoryValueGroups at LiquidBounce ae0eb396.
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    Whitelist,
    Blacklist,
}
impl Filter {
    fn accepts(self, matches: bool) -> bool {
        match self {
            Self::Whitelist => matches,
            Self::Blacklist => !matches,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerTitle {
    Barrel,
    Beacon,
    BlastFurnace,
    BrewingStand,
    Chest,
    LargeChest,
    Dispenser,
    Dropper,
    EnderChest,
    Furnace,
    Hopper,
    ShulkerBox,
    Smoker,
    ChestMinecart,
    ChestBoat,
    HopperMinecart,
}
impl ContainerTitle {
    pub const ALL: [Self; 16] = [
        Self::Barrel,
        Self::Beacon,
        Self::BlastFurnace,
        Self::BrewingStand,
        Self::Chest,
        Self::LargeChest,
        Self::Dispenser,
        Self::Dropper,
        Self::EnderChest,
        Self::Furnace,
        Self::Hopper,
        Self::ShulkerBox,
        Self::Smoker,
        Self::ChestMinecart,
        Self::ChestBoat,
        Self::HopperMinecart,
    ];
    pub const DEFAULT: [Self; 6] = [
        Self::Barrel,
        Self::Chest,
        Self::LargeChest,
        Self::ShulkerBox,
        Self::ChestMinecart,
        Self::ChestBoat,
    ];
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Barrel => "Barrel",
            Self::Beacon => "Beacon",
            Self::BlastFurnace => "BlastFurnace",
            Self::BrewingStand => "BrewingStand",
            Self::Chest => "Chest",
            Self::LargeChest => "LargeChest",
            Self::Dispenser => "Dispenser",
            Self::Dropper => "Dropper",
            Self::EnderChest => "EnderChest",
            Self::Furnace => "Furnace",
            Self::Hopper => "Hopper",
            Self::ShulkerBox => "ShulkerBox",
            Self::Smoker => "Smoker",
            Self::ChestMinecart => "ChestMinecart",
            Self::ChestBoat => "ChestBoat",
            Self::HopperMinecart => "HopperMinecart",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|entry| entry.tag().eq_ignore_ascii_case(tag))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutoF5 {
    pub check_screen_handler_type: bool,
    /// Unknown imported registry identifiers resolve to null in the original
    /// RegistryListValue and can therefore match a null container MenuType.
    pub types: BTreeSet<Option<String>>,
    pub type_filter: Filter,
    pub check_screen_title: bool,
    pub titles: Vec<ContainerTitle>,
    pub custom_titles: BTreeSet<String>,
    pub title_filter: Filter,
}
impl Default for AutoF5 {
    fn default() -> Self {
        Self {
            check_screen_handler_type: true,
            types: [
                "minecraft:generic_9x3",
                "minecraft:generic_9x6",
                "minecraft:shulker_box",
            ]
            .map(|name| Some(String::from(name)))
            .into(),
            type_filter: Filter::Whitelist,
            check_screen_title: true,
            titles: ContainerTitle::DEFAULT.into(),
            custom_titles: BTreeSet::new(),
            title_filter: Filter::Whitelist,
        }
    }
}
/// `None` means the actual screen is not an AbstractContainerScreen; a
/// container's absent MenuType (inventory, creative, mounts) stays `None`
/// in the tuple. A configured null registry entry matches it, as upstream does.
pub type ScreenContext<'a> = (Option<&'a str>, &'a str, &'a [String]);
impl AutoF5 {
    pub fn perspective(
        &self,
        running: bool,
        original: u8,
        screen: Option<ScreenContext<'_>>,
    ) -> u8 {
        let Some((menu, title, matching_title_tags)) = screen else {
            return original;
        };
        if !running {
            return original;
        }
        let matching_type = self.types.iter().any(|kind| kind.as_deref() == menu);
        if self.check_screen_handler_type && !self.type_filter.accepts(matching_type) {
            return original;
        }
        let matching_title = self.custom_titles.contains(title)
            || self
                .titles
                .iter()
                .any(|choice| matching_title_tags.iter().any(|tag| tag == choice.tag()));
        if self.check_screen_title && !self.title_filter.accepts(matching_title) {
            return original;
        }
        1 // CameraType.THIRD_PERSON_BACK, a temporary PerspectiveEvent override.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_defaults_require_both_type_and_current_language_title() {
        let module = AutoF5::default();
        let tags = vec!["Chest".into()];
        assert_eq!(
            module.perspective(
                true,
                0,
                Some((Some("minecraft:generic_9x3"), "箱子", &tags))
            ),
            1
        );
        assert_eq!(
            module.perspective(
                false,
                2,
                Some((Some("minecraft:generic_9x3"), "箱子", &tags))
            ),
            2
        );
        assert_eq!(
            module.perspective(true, 0, Some((Some("minecraft:hopper"), "箱子", &tags))),
            0
        );
        assert_eq!(
            module.perspective(true, 0, Some((Some("minecraft:generic_9x3"), "Shop", &[]))),
            0
        );
        assert_eq!(module.perspective(true, 2, None), 2);
    }
    #[test]
    fn absent_menu_type_blacklists_empty_choices_and_disabled_groups_follow_source() {
        let mut module = AutoF5 {
            check_screen_title: false,
            ..AutoF5::default()
        };
        assert_eq!(module.perspective(true, 0, Some((None, "", &[]))), 0);
        module.type_filter = Filter::Blacklist;
        assert_eq!(module.perspective(true, 0, Some((None, "", &[]))), 1);
        module.types.clear();
        module.type_filter = Filter::Whitelist;
        assert_eq!(
            module.perspective(true, 0, Some((Some("minecraft:generic_9x3"), "", &[]))),
            0
        );
        module.check_screen_handler_type = false;
        assert_eq!(module.perspective(true, 0, Some((None, "", &[]))), 1);
    }
    #[test]
    fn custom_titles_match_exactly_and_title_blacklist_inverts_the_combined_match() {
        let mut module = AutoF5 {
            check_screen_handler_type: false,
            ..AutoF5::default()
        };
        module.custom_titles.insert("Shop".into());
        assert_eq!(module.perspective(true, 0, Some((None, "Shop", &[]))), 1);
        assert_eq!(module.perspective(true, 0, Some((None, "shop", &[]))), 0);
        module.title_filter = Filter::Blacklist;
        assert_eq!(module.perspective(true, 0, Some((None, "Shop", &[]))), 0);
        assert_eq!(module.perspective(true, 0, Some((None, "shop", &[]))), 1);
        assert_eq!(ContainerTitle::ALL.len(), 16);
        assert_eq!(
            ContainerTitle::from_tag("largeCHEST"),
            Some(ContainerTitle::LargeChest)
        );
    }
    #[test]
    fn unknown_registry_entry_matches_null_menu_type_in_both_filters() {
        let mut module = AutoF5 {
            check_screen_title: false,
            types: [None].into(),
            ..AutoF5::default()
        };
        assert_eq!(module.perspective(true, 2, Some((None, "", &[]))), 1);
        assert_eq!(
            module.perspective(true, 2, Some((Some("minecraft:generic_9x3"), "", &[]))),
            2
        );
        module.type_filter = Filter::Blacklist;
        assert_eq!(module.perspective(true, 2, Some((None, "", &[]))), 2);
        assert_eq!(
            module.perspective(true, 2, Some((Some("minecraft:generic_9x3"), "", &[]))),
            1
        );
    }
}
