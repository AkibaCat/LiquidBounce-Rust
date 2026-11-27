// SPDX-License-Identifier: GPL-3.0-or-later
//! Observed equipment history. Stack identity is supplied by the native owner.
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub handle: u64,
    pub item: String,
    pub enchantments: String,
    pub count: i32,
}
impl Item {
    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item == "minecraft:air"
    }
    fn same_kind_and_enchantments(&self, other: &Self) -> bool {
        self.item == other.item && self.enchantments == other.enchantments
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    MainHand,
    OffHand,
    Armor,
    AnimalArmor,
    Other,
}
#[derive(Default, Debug)]
pub struct TrackedInventory {
    pub items: VecDeque<Item>,
    // ItemStack inherits identity equality. Evicting a history row does not
    // erase its timestamp; the actual stack may still be in an open screen.
    pub times: HashMap<u64, i64>,
}
impl TrackedInventory {
    fn update(&mut self, item: Option<&Item>, hand: bool, now: i64) {
        self.items.retain(|item| item.count != 0);
        let Some(item) = item.filter(|item| !item.is_empty()) else {
            return;
        };
        self.items
            .retain(|old| !old.same_kind_and_enchantments(item));
        if hand {
            self.items.push_front(item.clone());
            self.times.insert(item.handle, now);
            if self.items.len() > 36 {
                self.items.pop_back();
            }
        }
    }
    fn project(&self, main: Option<&Item>) -> [Option<u64>; 36] {
        let mut slots = [None; 36];
        slots[0] = main.filter(|item| !item.is_empty()).map(|item| item.handle);
        let empty = slots[0].is_none();
        // Slot zero belongs to the actual main-hand stack. With an empty
        // main hand the newest observed offhand moves to inventory slot one.
        for (slot, value) in slots.iter_mut().enumerate().skip(1) {
            *value = self
                .items
                .get(if empty { slot - 1 } else { slot })
                .map(|item| item.handle);
        }
        slots
    }
}

#[derive(Default, Debug)]
pub struct InventoryTracker {
    pub save_players: bool,
    pub inventories: HashMap<String, TrackedInventory>,
    pub saved_players: HashMap<String, u64>,
    // CommandInvsee.viewedPlayer survives module/world resets.
    pub viewed_player: Option<String>,
}
pub struct Equipment<'a> {
    pub uuid: &'a str,
    pub player_token: u64,
    pub remote: bool,
    pub bot: bool,
    pub slot: Slot,
    pub new_item: Option<&'a Item>,
    pub main: Option<&'a Item>,
    pub off: Option<&'a Item>,
    pub now: i64,
}
impl InventoryTracker {
    pub fn set_save_players(&mut self, enabled: bool) -> bool {
        if self.save_players == enabled {
            return false;
        }
        self.save_players = enabled;
        self.saved_players.clear();
        true
    }
    pub fn equipment(&mut self, event: Equipment<'_>) -> Option<[Option<u64>; 36]> {
        if !event.remote || event.bot || event.slot == Slot::AnimalArmor {
            return None;
        }
        let history = self.inventories.entry(event.uuid.to_owned()).or_default();
        if self.save_players {
            self.saved_players
                .insert(event.uuid.to_owned(), event.player_token);
        }
        match event.slot {
            Slot::MainHand | Slot::OffHand => {
                history.update(event.off, true, event.now);
                history.update(event.main, true, event.now);
            }
            Slot::Armor => history.update(event.new_item, false, event.now),
            Slot::AnimalArmor | Slot::Other => {}
        }
        Some(history.project(event.main))
    }
    pub fn timestamp(&self, handle: u64) -> Option<i64> {
        self.inventories
            .get(self.viewed_player.as_ref()?)?
            .times
            .get(&handle)
            .copied()
            .filter(|time| *time != 0)
    }
    pub fn reset(&mut self) {
        self.inventories.clear();
        self.saved_players.clear();
    }
}
pub fn elapsed(now: i64, timestamp: i64) -> String {
    let seconds = now.saturating_sub(timestamp) / 1000;
    format!("{:02}:{:02}", (seconds % 3600) / 60, seconds % 60)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Completion {
    /// ChatScreen uses UTF-8 byte ranges, while its cursor counts characters.
    pub start: usize,
    pub len: usize,
    pub suggestions: Vec<String>,
}
pub fn complete<'a>(
    text: &str,
    cursor: usize,
    prefix: &str,
    online: impl IntoIterator<Item = &'a str>,
) -> Option<Completion> {
    let cursor = text
        .char_indices()
        .nth(cursor)
        .map_or(text.len(), |(i, _)| i);
    let input = &text[..cursor];
    let body = input.strip_prefix(prefix)?;
    let command_end = body.find(char::is_whitespace).unwrap_or(body.len());
    let command = &body[..command_end];
    if command_end == body.len() {
        return "invsee"
            .starts_with(&command.to_ascii_lowercase())
            .then(|| Completion {
                start: prefix.len(),
                len: command.len(),
                suggestions: if command == "invsee" {
                    vec![]
                } else {
                    vec!["invsee".into()]
                },
            });
    }
    if !command.eq_ignore_ascii_case("invsee") {
        return None;
    }
    let argument = body[command_end..].trim_start_matches(char::is_whitespace);
    let start = input.len() - argument.len();
    let mut suggestions = vec![];
    // PlayerInfoArgumentType delegates directly to SharedSuggestionProvider:
    // quoted partial input does not match an unquoted profile name. Execution
    // still accepts quoted names through ClientStringArgumentType.
    if !argument.contains(char::is_whitespace) && !argument.starts_with(['\'', '"']) {
        let lower = argument.to_lowercase();
        suggestions = online
            .into_iter()
            .filter(|name| {
                *name != argument
                    && name
                        .to_lowercase()
                        .split(['.', '_', '/'])
                        .any(|part| part.starts_with(&lower))
            })
            .map(str::to_owned)
            .collect();
        suggestions.sort_by_key(|name| name.to_lowercase());
        suggestions.dedup();
    }
    Some(Completion {
        start,
        len: argument.len(),
        suggestions,
    })
}

#[cfg(test)]
mod inventory_tracker_tests {
    use super::*;
    fn item(handle: u64, name: &str, enchantments: &str) -> Item {
        Item {
            handle,
            item: name.into(),
            enchantments: enchantments.into(),
            count: 1,
        }
    }
    fn equipment<'a>(main: Option<&'a Item>, off: Option<&'a Item>) -> Equipment<'a> {
        Equipment {
            uuid: "player",
            player_token: 4,
            remote: true,
            bot: false,
            slot: Slot::MainHand,
            new_item: main,
            main,
            off,
            now: 1_000,
        }
    }
    #[test]
    fn hand_updates_share_real_stack_identity_and_keep_both_hand_order() {
        let mut tracker = InventoryTracker::default();
        let sword = item(1, "sword", "{}");
        let shield = item(2, "shield", "{}");
        let slots = tracker
            .equipment(equipment(Some(&sword), Some(&shield)))
            .unwrap();
        assert_eq!(&slots[..3], &[Some(1), Some(2), None]);
        let slots = tracker.equipment(equipment(None, Some(&shield))).unwrap();
        assert_eq!(&slots[..4], &[None, Some(2), Some(1), None]);
    }
    #[test]
    fn duplicate_kind_and_enchantments_replace_identity_but_preserve_other_components_semantics() {
        let mut tracker = InventoryTracker::default();
        let old = item(1, "sword", "{sharpness:1}");
        let new = item(2, "sword", "{sharpness:1}");
        let different = item(3, "sword", "{sharpness:2}");
        tracker.equipment(equipment(Some(&old), None));
        tracker.equipment(equipment(Some(&new), Some(&different)));
        let history = &tracker.inventories["player"];
        assert_eq!(
            history.items.iter().map(|i| i.handle).collect::<Vec<_>>(),
            vec![2, 3]
        );
        assert_eq!(history.times[&1], 1000);
    }
    #[test]
    fn armor_equipping_removes_history_without_recording_a_new_timestamp() {
        let mut tracker = InventoryTracker::default();
        let helmet = item(1, "helmet", "{}");
        tracker.equipment(equipment(Some(&helmet), None));
        let mut event = equipment(None, None);
        event.slot = Slot::Armor;
        event.new_item = Some(&helmet);
        tracker.equipment(event);
        assert!(tracker.inventories["player"].items.is_empty());
        assert_eq!(tracker.inventories["player"].times[&1], 1000);
    }
    #[test]
    fn empty_hand_does_not_clear_history_and_capacity_is_thirty_six() {
        let mut tracker = InventoryTracker::default();
        for i in 1..=40 {
            let next = item(i, &format!("item{i}"), "{}");
            tracker.equipment(equipment(Some(&next), None));
        }
        let slots = tracker.equipment(equipment(None, None)).unwrap();
        assert_eq!(tracker.inventories["player"].items.len(), 36);
        assert_eq!(slots[1], Some(40));
        assert_eq!(slots[35], Some(6));
        assert!(tracker.inventories["player"].times.contains_key(&1));
    }
    #[test]
    fn local_bots_and_animal_armor_are_ignored_before_creating_history() {
        let mut tracker = InventoryTracker::default();
        for mode in 0..3 {
            let mut e = equipment(None, None);
            if mode == 0 {
                e.remote = false;
            } else if mode == 1 {
                e.bot = true;
            } else {
                e.slot = Slot::AnimalArmor;
            }
            assert!(tracker.equipment(e).is_none());
        }
        assert!(tracker.inventories.is_empty());
    }
    #[test]
    fn save_players_change_only_clears_saved_objects_and_reset_preserves_viewed_uuid() {
        let mut tracker = InventoryTracker::default();
        tracker.set_save_players(true);
        tracker.equipment(equipment(None, None));
        tracker.viewed_player = Some("player".into());
        assert_eq!(tracker.saved_players["player"], 4);
        assert!(!tracker.set_save_players(true));
        assert!(tracker.set_save_players(false));
        assert!(tracker.saved_players.is_empty());
        assert!(tracker.inventories.contains_key("player"));
        tracker.reset();
        assert!(tracker.inventories.is_empty());
        assert_eq!(tracker.viewed_player.as_deref(), Some("player"));
    }
    #[test]
    fn lore_looks_up_object_identity_and_formats_minutes_modulo_one_hour() {
        let mut tracker = InventoryTracker::default();
        let stack = item(4, "sword", "{}");
        tracker.equipment(equipment(Some(&stack), None));
        tracker.viewed_player = Some("player".into());
        assert_eq!(tracker.timestamp(4), Some(1000));
        assert_eq!(tracker.timestamp(5), None);
        assert_eq!(elapsed(3_724_000, 1000), "02:03");
    }
    #[test]
    fn completion_uses_live_prefix_online_profiles_and_shared_substring_rules() {
        let names = ["Other", "Team_Alice", "alex", "Alice"];
        let result = complete("!invsee a", 9, "!", names).unwrap();
        assert_eq!(result.start, 8);
        assert_eq!(result.len, 1);
        assert_eq!(result.suggestions, vec!["alex", "Alice", "Team_Alice"]);
        assert!(complete(".invsee a", 9, "!", names).is_none());
        assert!(complete("!say a", 6, "!", names).is_none());
        assert_eq!(
            complete("!inv", 4, "!", names).unwrap().suggestions,
            vec!["invsee"]
        );
    }
    #[test]
    fn completion_converts_character_cursor_to_byte_range_and_preserves_the_tail() {
        let result = complete("指令invsee 玩家suffix", 11, "指令", ["玩家甲"]).unwrap();
        assert_eq!(result.start, "指令invsee ".len());
        assert_eq!(result.len, "玩家".len());
        assert_eq!(result.suggestions, vec!["玩家甲"]);
    }
    #[test]
    fn completion_does_not_offer_extra_arguments_or_quoted_names() {
        for input in [".invsee Alice ", ".invsee \"Al", ".invsee 'Al"] {
            assert!(
                complete(input, input.chars().count(), ".", ["Alice"])
                    .unwrap()
                    .suggestions
                    .is_empty()
            );
        }
        assert!(
            complete(".invsee Alice", 13, ".", ["Alice"])
                .unwrap()
                .suggestions
                .is_empty()
        );
    }
}
