// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleNotifier caches and TotemPopTracker from LiquidBounce ae0eb396.
use std::collections::{BTreeMap, BTreeSet};
pub const SWITCHES: [&str; 6] = [
    "JoinMessages",
    "LeaveMessages",
    "GameModeMessages",
    "ItemConsumptionMessages",
    "HeldItemMessages",
    "TotemPopMessages",
];
pub const FORMATS: [&str; 6] = [
    "JoinMessageFormat",
    "LeaveMessageFormat",
    "GameModeMessageFormat",
    "ItemConsumptionMessageFormat",
    "HeldItemMessageFormat",
    "TotemPopMessageFormat",
];
pub const DEFAULTS: [&str; 6] = [
    "%s joined",
    "%s left",
    "%s changed their game mode to %s",
    "%1$s used %2$s",
    "%1$s holds %2$s x%3$s in %4$s",
    "%1$s popped a totem %2$s times",
];
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub switches: [bool; 6],
    pub formats: [String; 6],
    pub held_items: Vec<String>,
    pub notification: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            switches: [true, true, false, true, false, true],
            formats: DEFAULTS.map(str::to_owned),
            held_items: vec![
                "minecraft:enchanted_golden_apple".into(),
                "minecraft:end_crystal".into(),
            ],
            notification: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stack {
    pub id: String,
    pub components: String,
    pub name: String,
    pub count: i32,
    pub duration: i32,
    pub consumable: bool,
}
impl Stack {
    fn same(&self, other: &Self) -> bool {
        self.id == other.id && self.components == other.components
    }
    fn same_held(&self, other: &Self) -> bool {
        self.same(other) && self.count == other.count
    }
}
#[derive(Clone, Debug)]
pub struct Player {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub bot: bool,
    pub using: bool,
    pub active_hand: usize,
    pub ticks_using: i32,
    pub hands: [Option<Stack>; 2],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg {
    Text(String),
    Int(i32),
}
impl From<String> for Arg {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub kind: usize,
    pub args: Vec<Arg>,
}
struct Consumption {
    stack: Stack,
    last_ticks: i32,
}
#[derive(Default)]
pub struct State {
    pub settings: Settings,
    enabled: bool,
    initialize: bool,
    names: BTreeMap<String, String>,
    modes: BTreeMap<String, i32>,
    consumption: BTreeMap<String, Consumption>,
    held: BTreeMap<String, [Option<Stack>; 2]>,
    pops: BTreeMap<String, i32>,
}
impl State {
    /// Current totem-pop count for a player UUID, used by Nametags.
    pub fn pop_count(&self, uuid: &str) -> i32 {
        self.pops.get(uuid).copied().unwrap_or(0)
    }
    pub fn sync(&mut self, enabled: bool) {
        if enabled != self.enabled {
            self.enabled = enabled;
            self.initialize = enabled;
            if !enabled {
                self.names.clear();
                self.modes.clear();
                self.consumption.clear();
                self.held.clear();
            }
        }
    }
    pub fn needs_initialize(&self) -> bool {
        self.enabled && self.initialize
    }
    pub fn initialize(&mut self, online: impl IntoIterator<Item = (String, String, i32)>) {
        if !self.needs_initialize() {
            return;
        }
        for (id, name, mode) in online {
            self.names.insert(id.clone(), name);
            self.modes.insert(id, mode);
        }
        self.initialize = false;
    }
    pub fn world_change(&mut self) {
        self.consumption.clear();
        self.held.clear();
        self.pops.clear();
    }
    pub fn add(&mut self, uuid: String, name: String) -> Option<Notice> {
        if !self.enabled || name.encode_utf16().count() <= 2 {
            return None;
        }
        self.names.insert(uuid, name.clone());
        self.settings.switches[0].then(|| Notice {
            kind: 0,
            args: vec![name.into()],
        })
    }
    pub fn remove(&mut self, uuid: &str) -> Option<Notice> {
        self.modes.remove(uuid);
        let name = self.names.remove(uuid)?;
        (self.enabled && self.settings.switches[1] && name.encode_utf16().count() >= 2).then(|| {
            Notice {
                kind: 1,
                args: vec![name.into()],
            }
        })
    }
    pub fn mode(&mut self, uuid: &str, mode: i32, initial: bool) -> Option<Notice> {
        if !self.enabled {
            return None;
        }
        let previous = self.modes.insert(uuid.into(), mode)?;
        if initial || previous == mode || !self.settings.switches[2] {
            return None;
        }
        Some(Notice {
            kind: 2,
            args: vec![
                self.names.get(uuid)?.clone().into(),
                Arg::Text(
                    match mode {
                        1 => "CREATIVE",
                        2 => "ADVENTURE",
                        3 => "SPECTATOR",
                        _ => "SURVIVAL",
                    }
                    .into(),
                ),
            ],
        })
    }
    pub fn entity_event(
        &mut self,
        uuid: &str,
        name: &str,
        event: u8,
        is_local: bool,
        friend: bool,
    ) -> Option<Notice> {
        if event == 3 {
            self.pops.remove(uuid);
            return None;
        }
        if event != 35 {
            return None;
        }
        let n = self.pops.entry(uuid.into()).or_default();
        *n = n.wrapping_add(1);
        (self.enabled && self.settings.switches[5] && !is_local && !friend).then(|| Notice {
            kind: 5,
            args: vec![Arg::Text(name.into()), Arg::Int(*n)],
        })
    }
    pub fn remove_entity(&mut self, uuid: &str) {
        self.pops.remove(uuid);
    }
    pub fn tick(&mut self, players: &[Player]) -> Vec<Notice> {
        if !self.enabled {
            return vec![];
        }
        let mut notices = vec![];
        let mut observed = BTreeSet::new();
        for p in players.iter().filter(|p| !p.bot) {
            observed.insert(p.uuid.clone());
            if self.settings.switches[3] {
                if !p.using {
                    if let Some(old) = self.consumption.remove(&p.uuid) {
                        if old.stack.duration > 0 && old.last_ticks >= old.stack.duration - 1 {
                            notices.push(Notice {
                                kind: 3,
                                args: vec![p.name.clone().into(), old.stack.name.into()],
                            });
                        }
                    }
                } else {
                    self.using(
                        &p.uuid,
                        p.hands.get(p.active_hand).cloned().flatten(),
                        p.ticks_using,
                    );
                }
            }
            if self.settings.switches[4] {
                let current = p
                    .hands
                    .clone()
                    .map(|s| s.filter(|s| s.count > 0 && self.settings.held_items.contains(&s.id)));
                if current.iter().all(Option::is_none) {
                    self.held.remove(&p.uuid);
                } else {
                    let previous = self.held.get(&p.uuid);
                    for hand in 0..2 {
                        if let Some(s) = &current[hand] {
                            if !previous
                                .and_then(|p| p[hand].as_ref())
                                .is_some_and(|p| s.same_held(p))
                            {
                                notices.push(Notice {
                                    kind: 4,
                                    args: vec![
                                        p.name.clone().into(),
                                        s.name.clone().into(),
                                        Arg::Int(s.count),
                                        Arg::Text(
                                            if hand == 0 { "MAIN_HAND" } else { "OFF_HAND" }.into(),
                                        ),
                                    ],
                                });
                            }
                        }
                    }
                    self.held.insert(p.uuid.clone(), current);
                }
            }
        }
        if self.settings.switches[3] {
            self.consumption.retain(|id, _| observed.contains(id));
        } else {
            self.consumption.clear();
        }
        if self.settings.switches[4] {
            self.held.retain(|id, _| observed.contains(id));
        } else {
            self.held.clear();
        }
        notices
    }
    pub fn using(&mut self, uuid: &str, item: Option<Stack>, ticks: i32) {
        if !self.enabled || !self.settings.switches[3] {
            return;
        }
        let Some(item) = item.filter(|s| s.count > 0 && s.consumable) else {
            self.consumption.remove(uuid);
            return;
        };
        match self.consumption.get_mut(uuid) {
            Some(s) if s.stack.same(&item) => s.last_ticks = s.last_ticks.max(ticks),
            _ => {
                self.consumption.insert(
                    uuid.into(),
                    Consumption {
                        stack: item,
                        last_ticks: ticks,
                    },
                );
            }
        }
    }
}
