// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAntiBot and its four source modes, LiquidBounce ae0eb396.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const MODES: [&str; 4] = [
    "Custom",
    "Matrix-2021-10-18",
    "Intave-2022-07-28",
    "Horizon-2021-10-18",
];
pub const CONDITIONS: [&str; 11] = [
    "Duplicate",
    "NoGameMode",
    "Ping",
    "IllegalPitch",
    "FakeEntityID",
    "NeedHit",
    "IllegalHealth",
    "Swung",
    "Critted",
    "Attributes",
    "IllegalScale",
];
pub const ARMOR: [&str; 11] = [
    "Nothing",
    "Leather",
    "Chain",
    "Iron",
    "Gold",
    "Diamond",
    "Netherite",
    "Elytra",
    "TurtleScute",
    "Pumpkin",
    "Skull",
];
pub const VALIDATORS: [&str; 3] = ["Vanilla", "Cyrillic", "CJKUnifiedIdeographs"];

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub mode: usize,
    pub literal_npc: bool,
    pub not_in_tab: bool,
    pub grace: i32,
    pub conditions: u16,
    pub invalid_ground: bool,
    pub ground_vl: i32,
    pub radius: bool,
    pub radius_range: f32,
    pub age: bool,
    pub minimum_age: i32,
    pub moved: bool,
    pub minimum_moved: i32,
    pub metadata: bool,
    pub maximum_metadata: i32,
    pub armor: bool,
    pub armor_choices: [u16; 4],
    pub name: bool,
    pub length: [i32; 2],
    pub validators: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: 0,
            literal_npc: false,
            not_in_tab: false,
            grace: 0,
            conditions: (1 << 1) | (1 << 3) | (1 << 4),
            invalid_ground: true,
            ground_vl: 10,
            radius: false,
            radius_range: 20.,
            age: false,
            minimum_age: 20,
            moved: false,
            minimum_moved: 160,
            metadata: false,
            maximum_metadata: 3,
            armor: false,
            armor_choices: [1; 4],
            name: true,
            length: [3, 16],
            validators: 1,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Armor {
    pub identity: u64,
    pub predicates: u16,
    pub player_armor: bool,
    pub enchanted: bool,
}
impl Default for Armor {
    fn default() -> Self {
        Self {
            identity: 0,
            predicates: 1,
            player_armor: false,
            enchanted: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub id: i32,
    pub token: u64,
    pub uuid: String,
    pub name: String,
    pub local: bool,
    pub ticks: i32,
    pub position: [f64; 3],
    pub previous: [f64; 3],
    pub on_ground: bool,
    pub pitch: f32,
    pub health: f32,
    pub scale: f64,
    pub armor: [Armor; 4],
    pub properties_empty: bool,
    pub online: bool,
    pub listed: bool,
    pub game_mode: Option<i32>,
    pub ping: i32,
}
impl Player {
    pub fn fully_armored(&self) -> bool {
        self.armor.iter().all(|a| a.player_armor && a.enchanted)
    }
    fn armor_identity(&self) -> [u64; 4] {
        self.armor.map(|a| a.identity)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub uuid: String,
    pub name: String,
}
pub fn duplicate(profile: &Profile, online: &[Profile]) -> bool {
    online
        .iter()
        .filter(|p| p.name == profile.name && p.uuid != profile.uuid)
        .count()
        == 1
}
pub fn unique(profile: &Profile, online: &[Profile]) -> bool {
    online
        .iter()
        .filter(|p| p.name == profile.name && p.uuid == profile.uuid)
        .count()
        == 1
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub uuid: String,
    pub name: Option<String>,
    pub properties_empty: bool,
    pub mode: Option<i32>,
    pub latency: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Custom {
    pub flying: BTreeMap<i32, i32>,
    pub hit: BTreeSet<i32>,
    pub outside: BTreeSet<i32>,
    pub moved: BTreeSet<i32>,
    pub metadata_seen: BTreeSet<i32>,
    pub minimal_metadata: BTreeSet<i32>,
    pub swung: BTreeSet<i32>,
    pub critted: BTreeSet<i32>,
    pub attributes: BTreeSet<i32>,
    pub armor: BTreeSet<i32>,
}
#[derive(Clone, Debug)]
pub struct MatrixWait {
    pub members: VecDeque<(i32, u64)>,
    pub waiting: Option<((i32, u64), [u64; 4])>,
    pub callback: u64,
}
#[derive(Clone, Debug, Default)]
pub struct Matrix {
    pub suspects: BTreeSet<String>,
    pub bots: BTreeSet<String>,
    pub continuation: Option<MatrixWait>,
    next_callback: u64,
}
#[derive(Clone, Debug, Default)]
pub struct Intave {
    pub suspects: BTreeMap<String, (i32, i64)>,
    pub bots: BTreeSet<String>,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub settings: Settings,
    pub custom: Custom,
    pub matrix: Matrix,
    pub intave: Intave,
    pub horizon: BTreeSet<String>,
    // Suspended Matrix iterators retain the original entity objects across removal.
    pub entities: BTreeMap<(i32, u64), Player>,
}
impl State {
    pub fn set_mode(&mut self, mode: usize) {
        if self.settings.mode != mode {
            self.matrix.continuation = None;
            self.settings.mode = mode;
        }
    }
    pub fn reset(&mut self) {
        self.custom = Default::default();
        self.matrix.suspects.clear();
        self.matrix.bots.clear();
        self.matrix.continuation = None;
        self.intave = Default::default();
        self.horizon.clear();
        self.entities.clear();
    }
    pub fn observe(&mut self, players: &[Player]) {
        let mut keep = players
            .iter()
            .map(|p| (p.id, p.token))
            .collect::<BTreeSet<_>>();
        if let Some(task) = &self.matrix.continuation {
            keep.extend(task.members.iter().copied());
            if let Some((key, _)) = task.waiting {
                keep.insert(key);
            }
        }
        self.entities.retain(|key, _| keep.contains(key));
        for p in players {
            self.entities.insert((p.id, p.token), p.clone());
        }
    }
    pub fn info(&mut self, actions: u8, entries: &[Entry], online: &[Profile], now: i64) {
        match self.settings.mode {
            1 if actions & 1 != 0 => {
                for e in entries {
                    let Some(name) = &e.name else {
                        continue;
                    };
                    let profile = Profile {
                        uuid: e.uuid.clone(),
                        name: name.clone(),
                    };
                    if e.latency < 2 || !e.properties_empty || unique(&profile, online) {
                        continue;
                    }
                    if duplicate(&profile, online) {
                        self.matrix.bots.insert(e.uuid.clone());
                    } else {
                        self.matrix.suspects.insert(e.uuid.clone());
                    }
                }
            }
            // EnumSet.first() tests the lowest ordinal, not membership.
            2 if actions != 0 && actions.trailing_zeros() == 0 => {
                for e in entries {
                    let Some(name) = &e.name else {
                        continue;
                    };
                    let profile = Profile {
                        uuid: e.uuid.clone(),
                        name: name.clone(),
                    };
                    if e.latency >= 2 && !unique(&profile, online) {
                        self.intave
                            .suspects
                            .insert(e.uuid.clone(), (e.latency, now));
                    }
                }
            }
            2 if actions != 0 && actions.trailing_zeros() == 4 => {
                for e in entries {
                    if let Some((ping, time)) = self.intave.suspects.remove(&e.uuid) {
                        if ping.wrapping_sub(e.latency) == ping && now.wrapping_sub(time) <= 15 {
                            self.intave.bots.insert(e.uuid.clone());
                        }
                    }
                }
            }
            3 if actions & 1 != 0 => {
                for e in entries {
                    if e.mode.is_none() {
                        self.horizon.insert(e.uuid.clone());
                    }
                }
            }
            _ => {}
        }
    }
    pub fn remove_profiles(&mut self, ids: &[String]) {
        match self.settings.mode {
            1 => {
                for id in ids {
                    self.matrix.suspects.remove(id);
                    self.matrix.bots.remove(id);
                }
            }
            2 => {
                for id in ids {
                    self.intave.suspects.remove(id);
                    self.intave.bots.remove(id);
                }
            }
            3 => {
                for id in ids {
                    self.horizon.remove(id);
                }
            }
            _ => {}
        }
    }
    pub fn ground(&mut self, id: i32, on_ground: bool, previous_y: f64, y: f64) {
        if self.settings.mode != 0 || !self.settings.invalid_ground {
            return;
        }
        let c = &mut self.custom;
        let value = c.flying.get(&id).copied().unwrap_or(0);
        if on_ground && previous_y != y {
            c.flying.insert(id, value.wrapping_add(1));
        } else if !on_ground && value > 0 {
            let next = value / 2;
            if next <= 0 {
                c.flying.remove(&id);
            } else {
                c.flying.insert(id, next);
            }
        }
    }
    pub fn attack(&mut self, id: i32) {
        if self.settings.mode == 0 {
            self.custom.hit.insert(id);
        }
    }
    pub fn attributes(&mut self, id: i32) {
        if self.settings.mode == 0 {
            self.custom.attributes.insert(id);
        }
    }
    pub fn metadata(&mut self, id: i32, size: usize) {
        if self.settings.mode == 0
            && self.custom.metadata_seen.insert(id)
            && size as i64 <= i64::from(self.settings.maximum_metadata)
        {
            self.custom.minimal_metadata.insert(id);
        }
    }
    pub fn animate(&mut self, id: i32, action: u8) {
        if self.settings.mode == 0 && matches!(action, 4 | 5) {
            self.custom.critted.insert(id);
        }
    }
    pub fn swing(&mut self, id: i32) {
        if self.settings.mode == 0 {
            self.custom.swung.insert(id);
        }
    }
    pub fn remove_entities(&mut self, ids: &[i32]) {
        if self.settings.mode != 0 {
            return;
        }
        for id in ids {
            let c = &mut self.custom;
            c.flying.remove(id);
            c.hit.remove(id);
            c.outside.remove(id);
            c.moved.remove(id);
            c.metadata_seen.remove(id);
            c.minimal_metadata.remove(id);
            c.swung.remove(id);
            c.critted.remove(id);
            c.attributes.remove(id);
            c.armor.remove(id);
        }
    }
    pub fn armor_valid(&self, p: &Player) -> bool {
        self.settings
            .armor_choices
            .iter()
            .zip(p.armor)
            .all(|(selected, armor)| *selected == 0 || selected & armor.predicates != 0)
    }
    pub fn custom_tick(&mut self, own: [f64; 3], players: &[Player]) {
        if self.settings.mode != 0 {
            return;
        }
        let range_squared = f64::from(self.settings.radius_range * self.settings.radius_range);
        for p in players.iter().filter(|p| !p.local) {
            if (0..3)
                .map(|i| (own[i] - p.position[i]).powi(2))
                .sum::<f64>()
                > range_squared
            {
                self.custom.outside.insert(p.id);
            }
            if p.previous != p.position {
                self.custom.moved.insert(p.id);
            }
            if self.settings.armor && !self.armor_valid(p) {
                self.custom.armor.insert(p.id);
            }
        }
        let remove: Vec<_> = self
            .custom
            .armor
            .iter()
            .copied()
            .filter(|id| {
                players
                    .iter()
                    .find(|p| p.id == *id)
                    .is_none_or(|p| self.armor_valid(p))
            })
            .collect();
        for id in remove {
            self.custom.armor.remove(&id);
        }
    }
    pub fn matrix_start(&mut self, players: &[Player]) -> Option<u64> {
        if self.settings.mode != 1
            || self.matrix.continuation.is_some()
            || self.matrix.suspects.is_empty()
        {
            return None;
        }
        self.observe(players);
        self.matrix.next_callback = self
            .matrix
            .next_callback
            .checked_add(1)
            .expect("Matrix callback exhausted");
        self.matrix.continuation = Some(MatrixWait {
            members: players.iter().map(|p| (p.id, p.token)).collect(),
            waiting: None,
            callback: self.matrix.next_callback,
        });
        self.matrix_continue()
    }
    fn matrix_continue(&mut self) -> Option<u64> {
        let mut task = self.matrix.continuation.take()?;
        if let Some((key, previous)) = task.waiting.take() {
            if let Some(p) = self.entities.get(&key) {
                if (p.fully_armored() || p.armor_identity() != previous) && p.properties_empty {
                    self.matrix.bots.insert(p.uuid.clone());
                }
                self.matrix.suspects.remove(&p.uuid);
            }
        }
        while let Some(key) = task.members.pop_front() {
            let Some(p) = self.entities.get(&key) else {
                continue;
            };
            if !self.matrix.suspects.contains(&p.uuid) {
                continue;
            }
            if !p.fully_armored() {
                task.waiting = Some((key, p.armor_identity()));
                let id = task.callback;
                self.matrix.continuation = Some(task);
                return Some(id);
            }
            if p.properties_empty {
                self.matrix.bots.insert(p.uuid.clone());
            }
            self.matrix.suspects.remove(&p.uuid);
        }
        None
    }
    pub fn matrix_resume(&mut self, callback: u64, players: &[Player]) -> bool {
        if self.settings.mode != 1
            || !self
                .matrix
                .continuation
                .as_ref()
                .is_some_and(|t| t.callback == callback)
        {
            return false;
        }
        self.observe(players);
        self.matrix_continue().is_some()
    }
    pub fn is_bot(
        &self,
        running: bool,
        p: &Player,
        online: &[Profile],
        local_max_health: f32,
    ) -> bool {
        if !running || p.local {
            return false;
        }
        let s = &self.settings;
        if s.literal_npc && !p.online || s.not_in_tab && p.ticks >= s.grace && !p.listed {
            return true;
        }
        match s.mode {
            1 => self.matrix.bots.contains(&p.uuid),
            2 => self.intave.bots.contains(&p.uuid),
            3 => self.horizon.contains(&p.uuid),
            _ => {
                let c = &self.custom;
                let name_invalid = || {
                    let length = p.name.encode_utf16().count() as i32;
                    length < s.length[0]
                        || length > s.length[1]
                        || (0..3).any(|kind| {
                            s.validators & (1 << kind) != 0
                                && !p.name.chars().all(|v| match kind {
                                    0 => v.is_ascii_alphanumeric() || v == '_',
                                    1 => (0x400..=0x52f).contains(&(v as u32)),
                                    _ => (0x4e00..=0x9fa5).contains(&(v as u32)),
                                })
                        })
                };
                s.invalid_ground && c.flying.get(&p.id).copied().unwrap_or(0) >= s.ground_vl
                    || s.radius && !c.outside.contains(&p.id)
                    || s.age && p.ticks < s.minimum_age
                    || s.moved && p.ticks >= s.minimum_moved && !c.moved.contains(&p.id)
                    || s.metadata && c.minimal_metadata.contains(&p.id)
                    || s.armor && c.armor.contains(&p.id)
                    || s.name && name_invalid()
                    || (0..11).any(|i| {
                        s.conditions & (1 << i) != 0
                            && match i {
                                0 => duplicate(
                                    &Profile {
                                        uuid: p.uuid.clone(),
                                        name: p.name.clone(),
                                    },
                                    online,
                                ),
                                1 => p.game_mode.is_none(),
                                2 => p.ticks >= 20 && p.ping <= 0,
                                3 => p.pitch.abs() > 90.,
                                4 => !(0..=1_000_000_000).contains(&p.id),
                                5 => !c.hit.contains(&p.id),
                                6 => p.health > local_max_health,
                                7 => !c.swung.contains(&p.id),
                                8 => !c.critted.contains(&p.id),
                                9 => !c.attributes.contains(&p.id),
                                _ => p.scale != 1.,
                            }
                    })
            }
        }
    }
}
