#!/usr/bin/env python3
"""Expose the actual container menu/title to AutoF5's temporary perspective event.

Titles are resolved afresh in the game's language; no English substring,
inventory-slot-count guess or previous-frame narration snapshot is used.
"""

TITLE_KEYS = [
    ('Barrel',['container.barrel']),('Beacon',['container.beacon']),
    ('BlastFurnace',['container.blast_furnace']),('BrewingStand',['container.brewing']),
    ('Chest',['container.chest']),('LargeChest',['container.chestDouble']),
    ('Dispenser',['container.dispenser']),('Dropper',['container.dropper']),
    ('EnderChest',['container.enderchest']),('Furnace',['container.furnace']),
    ('Hopper',['container.hopper']),('ShulkerBox',['container.shulkerBox']),
    ('Smoker',['container.smoker']),('ChestMinecart',['entity.minecraft.chest_minecart']),
    ('ChestBoat',['entity.minecraft.chest_boat']+[
        'entity.minecraft.'+wood+'_chest_'+('raft' if wood=='bamboo' else 'boat')
        for wood in ('acacia','bamboo','birch','cherry','dark_oak','jungle','mangrove','oak','pale_oak','poplar','spruce')]),
    ('HopperMinecart',['entity.minecraft.hopper_minecart']),
]

NATIVE = '''    /// AutoF5 sees AbstractContainerScreen's actual menu type and exact title.
    /// Inventory, creative and mount menus have no registered MenuType.
    pub(super) fn native_auto_f5_screen(&self) -> Option<(Option<String>, String, Vec<String>)> {
        if !self.native_window() { return None; }
        let (menu, title) = self.auto_f5_menu_title()?;
        let tags = super::auto_f5::matching_titles(&title,
            |key| horizon_client::text::Text::translate(key, &[]).plain());
        Some((menu, title, tags))
    }
    fn auto_f5_menu_title(&self) -> Option<(Option<String>, String)> {
        if self.title.is_some() || !matches!(self.screen, Screen::Inventory | Screen::Creative) {
            return None;
        }
        if self.screen == Screen::Creative { return Some((None, String::new())); }
        // Horse/NautilusInventoryMenu stores a null MenuType in vanilla.
        if let Some(id) = self.mount_menu.opened_mob() {
            let title = self.mobs.iter().find(|m| m.id == id).map_or_else(String::new, |m| {
                m.custom_name.clone().unwrap_or_else(|| {
                    let key = format!("entity.minecraft.{}", m.kind.id.trim_start_matches("minecraft:"));
                    self.translate(&key, &key)
                })
            });
            return Some((None, title));
        }
        if let Some(menu) = self.remote_menu_type() {
            return Some((Some(menu.to_owned()), self.remote_menu_title().map_or_else(String::new, |t| t.plain())));
        }
        if let Some((menu, title)) = self.auto_f5_storage_menu() {
            return Some((Some(menu), title));
        }
        if let Some(station) = &self.station {
            use super::stations::Kind;
            let menu = match station.kind {
                Kind::Enchanting => "minecraft:enchantment", Kind::Anvil => "minecraft:anvil",
                Kind::Grindstone => "minecraft:grindstone", Kind::Smithing => "minecraft:smithing",
                Kind::Stonecutter => "minecraft:stonecutter", Kind::Merchant => "minecraft:merchant",
                Kind::Beacon => "minecraft:beacon", Kind::Cartography => "minecraft:cartography_table",
                Kind::Loom => "minecraft:loom",
            };
            let title = if let Some(merchant) = &station.merchant {
                merchant.name.clone().unwrap_or_else(|| {
                    let key = if merchant.progress() {
                        format!("entity.minecraft.villager.{}", merchant.session.profession.trim_start_matches("minecraft:"))
                    } else { "entity.minecraft.wandering_trader".to_owned() };
                    self.translate(&key, &key)
                })
            } else { self.translate(station.kind.title_key(), "") };
            return Some((Some(menu.to_owned()), title));
        }
        let menu = (self.craft_size == 3).then(|| "minecraft:crafting".to_owned());
        Some((menu, self.translate("container.crafting", "Crafting")))
    }

'''

STORAGE = '''    /// AutoF5 shares the same title builders as the actual storage screens.
    pub(super) fn auto_f5_storage_menu(&self) -> Option<(String, String)> {
        use horizon_client::containers::BlockEntity;
        let entity = self.container_kind()?;
        if self.container_block_is_shulker() {
            return Some(("minecraft:shulker_box".to_owned(), self.container_title(self.translate("container.shulkerBox", "Shulker Box"))));
        }
        if let Some(rows) = self.chest_rows() {
            return Some((format!("minecraft:generic_9x{rows}"), self.chest_title(rows).plain()));
        }
        let (menu, key) = match entity {
            BlockEntity::Chest(_) if self.container_size() == 5 => ("minecraft:hopper", if self.open_cart_kind().is_some() { "entity.minecraft.hopper_minecart" } else { "container.hopper" }),
            BlockEntity::Chest(_) if self.container_size() == 9 => ("minecraft:generic_3x3", if self.container_block().as_deref() == Some("minecraft:dropper") { "container.dropper" } else { "container.dispenser" }),
            BlockEntity::Brewing(_) => ("minecraft:brewing_stand", "container.brewing"),
            BlockEntity::Crafter(_) => ("minecraft:crafter_3x3", "container.crafter"),
            BlockEntity::Furnace(furnace) => match furnace.kind.recipe_type().trim_start_matches("minecraft:") {
                "blasting" => ("minecraft:blast_furnace", "container.blast_furnace"),
                "smoking" => ("minecraft:smoker", "container.smoker"),
                _ => ("minecraft:furnace", "container.furnace"),
            },
            BlockEntity::Mod(_) => {
                let menu = self.mod_menu_at(self.container?)?;
                return Some((menu.to_owned(), self.remote_menu_title().map_or_else(String::new, |t| t.plain())));
            }
            _ => return None,
        };
        Some((menu.to_owned(), horizon_client::text::Text::literal(self.container_title(self.translate(key, key))).resolve_legacy().plain()))
    }

'''


def title_source():
    import json
    rows='\n'.join('    ('+json.dumps(tag)+', &['+', '.join(map(json.dumps,keys))+']),' for tag,keys in TITLE_KEYS)
    return '''// SPDX-License-Identifier: GPL-3.0-or-later
//! CheckScreenTitle's complete original enum, resolved in the current language.
pub(super) const TITLE_KEYS: &[(&str, &[&str])] = &[
'''+rows+'''
];
pub(super) fn matching_titles(title: &str, tr: impl Fn(&str) -> String) -> Vec<String> {
    TITLE_KEYS.iter().filter(|(_, keys)| keys.iter().any(|key| tr(key) == title))
        .map(|(tag, _)| (*tag).to_owned()).collect()
}
'''


def apply(edit, replace_once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:replace_once(t,anchor,NATIVE+anchor))
    anchor='    /// The custom name of the open container, or its default title\n'
    edit('crates/horizon-client/src/game/storage.rs',lambda t:replace_once(t,anchor,STORAGE+anchor))
    edit('crates/horizon-client/src/game/auto_f5.rs',lambda _:title_source())
    edit('crates/horizon-client/src/game.rs',lambda t:replace_once(t,'mod screen_info;\n','mod screen_info;\nmod auto_f5;\n'))
    return {'hook_names':[],'helper_names':['auto_f5_screen']}
