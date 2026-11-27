// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from CombatExtensions, GlobalSettingsTarget, EntityTaggingManager and
// FriendManager at LiquidBounce ae0eb396. Copyright (c) 2015-2026 CCBlueX.
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Target {
    SelfEntity,
    Players,
    Hostile,
    Angerable,
    WaterCreature,
    Passive,
    ArmorStand,
    Invisible,
    Dead,
    Sleeping,
    Friends,
}
impl Target {
    pub const ALL: [Self; 11] = [
        Self::SelfEntity,
        Self::Players,
        Self::Hostile,
        Self::Angerable,
        Self::WaterCreature,
        Self::Passive,
        Self::ArmorStand,
        Self::Invisible,
        Self::Dead,
        Self::Sleeping,
        Self::Friends,
    ];
    pub const fn tag(self) -> &'static str {
        match self {
            Self::SelfEntity => "Self",
            Self::Players => "Players",
            Self::Hostile => "Hostile",
            Self::Angerable => "Angerable",
            Self::WaterCreature => "WaterCreature",
            Self::Passive => "Passive",
            Self::ArmorStand => "ArmorStand",
            Self::Invisible => "Invisible",
            Self::Dead => "Dead",
            Self::Sleeping => "Sleeping",
            Self::Friends => "Friends",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|item| java_tag_eq(item.tag(), tag))
    }
}

// Tagged.makeLookupTable uses Java String.CASE_INSENSITIVE_ORDER; the
// single-character fold includes dotless/dotted I and long s for ASCII tags.
fn java_tag_eq(choice: &str, value: &str) -> bool {
    let fold = |character: char| {
        let mut upper = character.to_uppercase();
        let first = upper.next().unwrap_or(character);
        let upper = if upper.next().is_none() {
            first
        } else {
            character
        };
        upper.to_lowercase().next().unwrap_or(upper)
    };
    choice.chars().map(fold).eq(value.chars().map(fold))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetSet(pub u16);
impl Default for TargetSet {
    fn default() -> Self {
        Self::from_targets([
            Target::Players,
            Target::Hostile,
            Target::Angerable,
            Target::WaterCreature,
            Target::Invisible,
        ])
    }
}
impl TargetSet {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self((1 << 11) - 1);
    pub fn from_targets(values: impl IntoIterator<Item = Target>) -> Self {
        Self(
            values
                .into_iter()
                .fold(0, |mask, value| mask | (1 << value as u8)),
        )
    }
    pub const fn contains(self, value: Target) -> bool {
        self.0 & (1 << value as u8) != 0
    }
    pub fn tags(self) -> Vec<&'static str> {
        Target::ALL
            .into_iter()
            .filter(|value| self.contains(*value))
            .map(Target::tag)
            .collect()
    }
    fn interesting(self, entity: &TargetEntity, info: TargetInfo) -> bool {
        if !entity.kind.living
            || (!self.contains(Target::Dead) && !entity.alive)
            || (!self.contains(Target::Invisible) && entity.invisible)
        {
            return false;
        }
        match entity.kind.category {
            Some(Target::Players) => {
                !entity.is_local
                    && (!entity.sleeping || self.contains(Target::Sleeping))
                    && (self.contains(Target::Players)
                        || (info.is_friend && self.contains(Target::Friends)))
            }
            Some(category) => self.contains(category),
            None => false,
        }
    }
    pub fn should_attack(self, entity: &TargetEntity, info: TargetInfo) -> bool {
        !entity.is_local
            && !entity.has_local_passenger
            && (!info.is_friend || self.contains(Target::Friends))
            && info.classification == Classification::Target
            && self.interesting(entity, info)
    }
    pub fn should_be_attacked(self, entity: &TargetEntity, info: TargetInfo) -> bool {
        entity.kind.attackable && entity.within_border && self.should_attack(entity, info)
    }
    pub fn should_show(
        self,
        entity: &TargetEntity,
        info: TargetInfo,
        camera: VisualCamera,
    ) -> bool {
        if entity.is_local || entity.has_local_passenger {
            return self.contains(Target::SelfEntity)
                && (!camera.raw_first_person || camera.free_cam || camera.free_look);
        }
        (!info.is_friend || self.contains(Target::Friends))
            && info.classification != Classification::Ignored
            && self.interesting(entity, info)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlobalTargets {
    pub combat: TargetSet,
    pub visual: TargetSet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityKind {
    pub living: bool,
    pub attackable: bool,
    pub category: Option<Target>,
}
#[path = "targeting_facts.rs"]
mod facts;
impl EntityKind {
    pub const UNKNOWN: Self = Self {
        living: false,
        attackable: false,
        category: None,
    };
    /// Actual Java inheritance from the pinned 26.3 JAR, not MobCategory.
    pub fn for_type(id: &str) -> Self {
        facts::ENTITIES
            .binary_search_by_key(&id, |row| row.0)
            .map_or(Self::UNKNOWN, |index| facts::ENTITIES[index].1)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetEntity {
    pub kind: EntityKind,
    pub alive: bool,
    pub invisible: bool,
    pub sleeping: bool,
    pub is_local: bool,
    pub has_local_passenger: bool,
    pub within_border: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisualCamera {
    pub raw_first_person: bool,
    pub free_cam: bool,
    pub free_look: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Classification {
    #[default]
    Target,
    Interesting,
    Ignored,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TargetInfo {
    pub classification: Classification,
    pub is_friend: bool,
    pub tag_color: Option<(i32, u32)>,
}
impl TargetInfo {
    /// PriorityField accepts only a strictly greater priority; equal priority keeps the first handler.
    pub fn color(&mut self, color: u32, priority: i32) {
        if priority > self.tag_color.map_or(-20, |(p, _)| p) {
            self.tag_color = Some((priority, color));
        }
    }
    pub fn resolved_color(self) -> Option<u32> {
        self.tag_color.map(|(_, c)| c)
    }
    pub fn dont_target(&mut self) {
        if self.classification == Classification::Target {
            self.classification = Classification::Interesting;
        }
    }
    pub fn ignore(&mut self) {
        self.classification = Classification::Ignored;
    }
    pub fn assume_friend(&mut self) {
        self.is_friend = true;
    }
}
/// Clear at GameTickEvent FIRST_PRIORITY; lazy values remain stable during a tick.
#[derive(Clone, Debug, Default)]
pub struct TagCache {
    entries: HashMap<i32, TargetInfo>,
}
impl TagCache {
    pub fn begin_tick(&mut self) {
        self.entries.clear();
    }
    pub fn get(&mut self, id: i32, tag: impl FnOnce() -> TargetInfo) -> TargetInfo {
        *self.entries.entry(id).or_insert_with(tag)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FriendBook {
    /// Names use exact, case-sensitive gameProfile.name order.
    pub names: BTreeMap<String, Option<String>>,
    pub cancel_attack: bool,
}
impl FriendBook {
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }
    pub fn tag(&self, kind: EntityKind, name: Option<&str>) -> TargetInfo {
        TargetInfo {
            is_friend: kind.category == Some(Target::Players)
                && name.is_some_and(|name| self.contains(name)),
            ..TargetInfo::default()
        }
    }
    pub fn add(&mut self, name: String, alias: Option<String>) -> bool {
        if self.names.contains_key(&name) {
            return false;
        }
        self.names.insert(name, alias);
        true
    }
    pub fn remove(&mut self, name: &str) -> bool {
        self.names.remove(name).is_some()
    }
    pub fn clear(&mut self) -> bool {
        let changed = !self.names.is_empty();
        self.names.clear();
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entity(id: &str) -> TargetEntity {
        TargetEntity {
            kind: EntityKind::for_type(id),
            alive: true,
            invisible: false,
            sleeping: false,
            is_local: false,
            has_local_passenger: false,
            within_border: true,
        }
    }
    #[test]
    fn actual_inheritance_branch_precedence_is_preserved() {
        assert_eq!(facts::ENTITIES.len(), 161);
        for (id, expected) in [
            ("dolphin", Target::Passive),
            ("wolf", Target::Passive),
            ("polar_bear", Target::Passive),
            ("bee", Target::Passive),
            ("iron_golem", Target::Angerable),
            ("piglin", Target::Hostile),
            ("zombified_piglin", Target::Hostile),
            ("ender_dragon", Target::Hostile),
            ("allay", Target::Passive),
            ("bat", Target::Passive),
        ] {
            assert_eq!(
                EntityKind::for_type(&format!("minecraft:{id}")).category,
                Some(expected),
                "{id}"
            );
        }
        assert!(!EntityKind::for_type("minecraft:boat").living);
        assert!(EntityKind::for_type("minecraft:armor_stand").attackable);
    }
    #[test]
    fn default_targets_exclude_passive_armor_dead_and_sleeping_but_include_invisible() {
        let set = TargetSet::default();
        let info = TargetInfo::default();
        assert!(set.should_be_attacked(&entity("minecraft:zombie"), info));
        assert!(!set.should_be_attacked(&entity("minecraft:cow"), info));
        assert!(!set.should_be_attacked(&entity("minecraft:armor_stand"), info));
        let mut player = entity("minecraft:player");
        player.invisible = true;
        assert!(set.should_be_attacked(&player, info));
        player.alive = false;
        assert!(!set.should_be_attacked(&player, info));
        player.alive = true;
        player.sleeping = true;
        assert!(!set.should_be_attacked(&player, info));
        assert!(TargetSet::ALL.should_be_attacked(&player, info));
    }
    #[test]
    fn friend_gate_and_friends_only_override_are_player_specific() {
        let friend = TargetInfo {
            is_friend: true,
            ..TargetInfo::default()
        };
        assert!(!TargetSet::default().should_attack(&entity("minecraft:player"), friend));
        let set = TargetSet::from_targets([Target::Friends]);
        assert!(set.should_attack(&entity("minecraft:player"), friend));
        assert!(!set.should_attack(&entity("minecraft:player"), TargetInfo::default()));
        assert!(!set.should_attack(&entity("minecraft:cow"), friend));
    }
    #[test]
    fn visual_self_branch_precedes_all_living_tag_and_state_checks() {
        let mut boat = entity("minecraft:oak_boat");
        boat.has_local_passenger = true;
        let set = TargetSet::from_targets([Target::SelfEntity]);
        let ignored = TargetInfo {
            classification: Classification::Ignored,
            is_friend: true,
            tag_color: None,
        };
        assert!(!set.should_show(
            &boat,
            ignored,
            VisualCamera {
                raw_first_person: true,
                ..VisualCamera::default()
            }
        ));
        assert!(set.should_show(
            &boat,
            ignored,
            VisualCamera {
                raw_first_person: true,
                free_look: true,
                ..VisualCamera::default()
            }
        ));
        assert!(set.should_show(&boat, ignored, VisualCamera::default()));
        assert!(!TargetSet::ALL.should_attack(&boat, TargetInfo::default()));
    }
    #[test]
    fn classification_and_world_border_have_different_combat_and_visual_roles() {
        let mut zombie = entity("minecraft:zombie");
        zombie.within_border = false;
        let mut info = TargetInfo::default();
        info.dont_target();
        assert!(!TargetSet::ALL.should_be_attacked(&zombie, TargetInfo::default()));
        assert!(TargetSet::ALL.should_attack(&zombie, TargetInfo::default()));
        assert!(!TargetSet::ALL.should_attack(&zombie, info));
        assert!(TargetSet::ALL.should_show(&zombie, info, VisualCamera::default()));
        info.ignore();
        info.dont_target();
        assert_eq!(info.classification, Classification::Ignored);
        assert!(!TargetSet::ALL.should_show(&zombie, info, VisualCamera::default()));
    }
    #[test]
    fn lazy_tags_are_stable_until_first_priority_tick_and_names_are_case_sensitive() {
        let mut friends = FriendBook::default();
        let player = EntityKind::for_type("minecraft:player");
        assert!(friends.add("Alice".into(), Some("Alias".into())));
        assert!(!friends.add("Alice".into(), None));
        assert!(!friends.contains("alice"));
        assert!(
            !friends
                .tag(EntityKind::for_type("minecraft:cow"), Some("Alice"))
                .is_friend
        );
        let mut cache = TagCache::default();
        assert!(
            cache
                .get(1, || friends.tag(player, Some("Alice")))
                .is_friend
        );
        friends.remove("Alice");
        assert!(
            cache
                .get(1, || friends.tag(player, Some("Alice")))
                .is_friend
        );
        cache.begin_tick();
        assert!(
            !cache
                .get(1, || friends.tag(player, Some("Alice")))
                .is_friend
        );
    }
}
