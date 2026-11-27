// SPDX-License-Identifier: GPL-3.0-or-later
//! Client-side RemotePlayer inventories and ItemStack object references.
//! A handle names one received stack object, never a value-based item key.
use super::{Game, ItemStack};
use horizon_client::mobs::Mob;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

pub(super) const SCREEN: &str = "liquidbounce:invsee";
pub(super) const SLOTS: [&str; 8] = [
    "mainhand", "offhand", "feet", "legs", "chest", "head", "body", "saddle",
];

#[derive(Clone)]
pub(super) struct Player {
    pub token: u64,
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub current: bool,
    pub saved: bool,
    pub tracked: bool,
    pub inventory: [u64; 36],
    pub equipment: [u64; 8],
    pub crafting: [u64; 5],
    pub carried: u64,
    pub model: Option<Mob>,
}
impl Player {
    fn new(token: u64, id: i32, uuid: String, name: String) -> Self {
        Self {
            token,
            id,
            uuid,
            name,
            current: true,
            saved: false,
            tracked: false,
            inventory: [0; 36],
            equipment: [0; 8],
            crafting: [0; 5],
            carried: 0,
            model: None,
        }
    }
}

#[derive(Default)]
pub(super) struct State {
    pub players: HashMap<u64, Player>,
    pub stacks: HashMap<u64, ItemStack>,
    pub viewed: Option<String>,
    pub pending_open: Option<String>,
    pub tooltip: Option<u64>,
    pub completion_signature: Option<String>,
    pub completion_owned: bool,
}
impl State {
    fn allocate(&mut self, stack: Option<&ItemStack>) -> u64 {
        let Some(stack) = stack.filter(|s| s.count > 0 && &*s.item != "minecraft:air") else {
            return 0;
        };
        let handle = stack.object_identity();
        self.stacks.insert(handle, stack.clone());
        handle
    }
    fn same_or_new(&mut self, old: u64, stack: Option<&ItemStack>) -> u64 {
        let _ = old;
        self.allocate(stack)
    }
    pub fn stack(&self, handle: u64) -> Option<&ItemStack> {
        self.stacks.get(&handle)
    }
    pub fn item(&self, handle: u64) -> Value {
        self.stack(handle).map_or(Value::Null, |s| {
            // ItemEnchantments is a map: its equality does not depend on
            // the order components happened to arrive in the packet.
            let enchantments: std::collections::BTreeMap<&str, u32> = s
                .enchantments()
                .iter()
                .map(|(id, level)| (&**id, *level))
                .collect();
            json!({"handle":handle,"item":s.item.as_ref(),"count":s.count,"enchantments":enchantments})
        })
    }
    pub fn resolve(&self, uuid: &str) -> Option<&Player> {
        self.players
            .values()
            .find(|p| p.uuid == uuid && p.current)
            .or_else(|| self.players.values().find(|p| p.uuid == uuid && p.saved))
    }
    fn reset(&mut self) {
        let tracked: HashSet<String> = self
            .players
            .values()
            .filter(|p| p.tracked)
            .map(|p| p.uuid.clone())
            .collect();
        for p in self.players.values_mut() {
            if p.current && tracked.contains(&p.uuid) {
                p.inventory[1..].fill(0);
            }
            p.saved = false;
            p.tracked = false;
        }
        self.players.retain(|_, p| p.current);
        let live: HashSet<u64> = self
            .players
            .values()
            .flat_map(|p| {
                p.inventory
                    .into_iter()
                    .chain(p.equipment)
                    .chain(p.crafting)
                    .chain([p.carried])
            })
            .collect();
        self.stacks.retain(|id, _| live.contains(id));
        // CommandInvsee.viewedPlayer is deliberately not reset.
    }
}

impl Game<'_> {
    pub(super) fn client_inventory_tracker_world_refresh(&mut self) {
        if self.client_inventory_tracker_world_pending.replace(false) {
            let viewed = self.client_inventory_tracker.viewed.take();
            self.client_inventory_tracker = State {
                viewed,
                ..Default::default()
            };
        }
    }
    /// Render the exact received stacks, including an off-hand stack whose
    /// components the base client's mob mirror does not otherwise retain.
    pub(super) fn client_inventory_tracker_held_stack(
        &self,
        mob: &Mob,
        off: bool,
    ) -> Option<ItemStack> {
        if self.native_mod_screen() != Some(SCREEN) {
            return None;
        }
        let p = self
            .client_inventory_tracker
            .viewed
            .as_deref()
            .and_then(|uuid| self.client_inventory_tracker.resolve(uuid))?;
        if !p.model.as_ref().is_some_and(|m| m.join == mob.join) {
            return None;
        }
        self.client_inventory_tracker
            .stack(p.equipment[usize::from(off)])
            .cloned()
    }
    pub(super) fn client_inventory_tracker_model(&self, p: &Player) -> Option<Mob> {
        let mut m = p.model.clone()?;
        let stack = |slot| {
            self.client_inventory_tracker
                .stack(p.equipment[slot])
                .cloned()
        };
        m.extra.set_main_hand_stack(stack(0));
        m.extra.off_hand = stack(1).map(|s| s.item);
        for (slot, index) in [(2, 3), (3, 2), (4, 1), (5, 0)] {
            m.extra.armor[index] = stack(slot);
        }
        m.extra.feet = stack(2).map(|s| s.item);
        m.extra.legs = stack(3).map(|s| s.item);
        m.extra.chest = stack(4).map(|s| s.item);
        m.extra.head = stack(5).map(|s| s.item);
        Some(m)
    }
    /// Refresh entity object membership without manufacturing equipment events.
    /// Mob.join is Horizon's actual object identity and changes when an id is reused.
    pub(super) fn client_inventory_tracker_refresh(&mut self) {
        let rows = self.client_anti_bot_world()["players"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for p in self.client_inventory_tracker.players.values_mut() {
            p.current = false;
        }
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
        for row in rows {
            let (Some(token), Some(id), Some(uuid), Some(name)) = (
                row["token"].as_u64(),
                row["id"].as_i64().and_then(|v| i32::try_from(v).ok()),
                row["uuid"].as_str(),
                row["name"].as_str(),
            ) else {
                continue;
            };
            let model = if id == local {
                self.player_model.clone()
            } else {
                self.mobs
                    .iter()
                    .find(|m| m.id as i32 == id && !m.removed)
                    .cloned()
            };
            let p = self
                .client_inventory_tracker
                .players
                .entry(token)
                .or_insert_with(|| Player::new(token, id, uuid.into(), name.into()));
            p.id = id;
            p.uuid = uuid.into();
            p.name = name.into();
            p.current = true;
            if model.is_some() {
                p.model = model;
            }
            if id == local {
                let inventory = self.inventory.clone();
                let armor = self.armor.clone();
                let off = self.offhand.clone();
                let selected = self.selected;
                let crafting = (self.craft_size == 2).then(|| self.craft_grid.clone());
                let result = (self.craft_size == 2)
                    .then(|| self.craft_result())
                    .flatten();
                let carried = self.carried.clone();
                let s = &mut self.client_inventory_tracker;
                let mut p = s.players.remove(&token).expect("current local player");
                for i in 0..36 {
                    p.inventory[i] =
                        s.same_or_new(p.inventory[i], inventory.get(i).and_then(Option::as_ref));
                }
                p.equipment[0] = p.inventory.get(selected).copied().unwrap_or(0);
                p.equipment[1] = s.same_or_new(p.equipment[1], off.as_ref());
                for (slot, index) in [(2, 3), (3, 2), (4, 1), (5, 0)] {
                    p.equipment[slot] = s.same_or_new(p.equipment[slot], armor[index].as_ref());
                }
                p.crafting[0] = s.same_or_new(p.crafting[0], result.as_ref());
                for i in 0..4 {
                    p.crafting[i + 1] = s.same_or_new(
                        p.crafting[i + 1],
                        crafting
                            .as_ref()
                            .and_then(|g| g.get(i))
                            .and_then(Option::as_ref),
                    );
                }
                p.carried = s.same_or_new(p.carried, carried.as_ref());
                s.players.insert(token, p);
            }
        }
    }

    /// Client RemotePlayer.setItemSlot HEAD: each accepted assignment is heard,
    /// including equal-valued replacement stacks and updates while disabled.
    pub(super) fn client_inventory_tracker_equipment(
        &mut self,
        id: i32,
        slot: u8,
        stack: Option<&ItemStack>,
    ) {
        if !self.client_auto_tool_player() || slot as usize >= SLOTS.len() {
            return;
        }
        self.client_inventory_tracker_world_refresh();
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
        if id == local {
            return;
        }
        let Some(player) = self.client_anti_bot_player(id) else {
            return;
        };
        let Some(token) = player["token"].as_u64() else {
            return;
        };
        let (Some(uuid), Some(name)) = (player["uuid"].as_str(), player["name"].as_str()) else {
            return;
        };
        let anti_bot = self.client_anti_bot_query(id);
        let time = Self::client_inventory_tracker_time();
        let model = self
            .mobs
            .iter()
            .find(|m| m.id as i32 == id && !m.removed)
            .cloned();
        let s = &mut self.client_inventory_tracker;
        s.players
            .entry(token)
            .or_insert_with(|| Player::new(token, id, uuid.into(), name.into()));
        let handle = s.allocate(stack);
        let mut hands = [
            s.players[&token].equipment[0],
            s.players[&token].equipment[1],
        ];
        if slot <= 1 {
            hands[slot as usize] = handle;
        }
        let input = json!({"kind":"equipment","uuid":uuid,"name":name,"playerToken":token,
            "remote":true,"slot":SLOTS[slot as usize],"new":s.item(handle),"main":s.item(hands[0]),
            "off":s.item(hands[1]),"time":time,"antiBot":anti_bot})
        .to_string();
        let mut output = String::new();
        self.native_client_inventory_tracker(&input, &mut output);
        let answer = serde_json::from_str::<Value>(&output).unwrap_or(Value::Null);
        let s = &mut self.client_inventory_tracker;
        {
            let p = s.players.get_mut(&token).expect("equipment player");
            p.current = true;
            p.equipment[slot as usize] = handle;
            if slot == 0 {
                p.inventory[0] = handle;
            }
        }
        if answer["reset"] == true {
            s.reset();
        }
        if answer["clearSaved"] == true {
            for p in s.players.values_mut() {
                p.saved = false;
            }
        }
        if answer["track"] == true && answer["save"] == true {
            for p in s.players.values_mut().filter(|p| p.uuid == uuid) {
                p.saved = false;
            }
        }
        let projection = answer["projection"]
            .as_array()
            .filter(|v| v.len() == 36)
            .and_then(|v| {
                v.iter()
                    .map(|v| {
                        if v.is_null() {
                            Some(0)
                        } else {
                            v.as_u64()
                                .filter(|id| *id == 0 || s.stacks.contains_key(id))
                        }
                    })
                    .collect::<Option<Vec<_>>>()
            });
        let p = s.players.get_mut(&token).expect("equipment player");
        p.current = true;
        p.name = name.into();
        if model.is_some() {
            p.model = model;
        }
        if answer["running"] == true && answer["track"] == true {
            if answer["save"] == true {
                p.saved = true;
            }
            p.tracked = true;
            if let Some(projection) = projection {
                p.inventory.copy_from_slice(&projection);
            }
        }
    }

    pub(super) fn client_inventory_tracker_tick(&mut self) {
        self.client_inventory_tracker_world_refresh();
        if !self.client_auto_tool_player() {
            return;
        }
        self.client_inventory_tracker_refresh();
        if let Some(uuid) = self.client_inventory_tracker.pending_open.take() {
            if self.client_inventory_tracker.resolve(&uuid).is_some() {
                // Minecraft.schedule { setScreen(...) } replaces the current
                // screen after the command returns, including in-bed chat.
                self.screen = super::Screen::Playing;
                self.native_mod_screen = None;
                if self.native_mod_screen_open(SCREEN) {
                    self.client_inventory_tracker.viewed = Some(uuid);
                }
            }
        }
        if self.native_mod_screen() == Some(SCREEN)
            && self
                .client_inventory_tracker
                .viewed
                .as_deref()
                .and_then(|uuid| self.client_inventory_tracker.resolve(uuid))
                .is_none()
        {
            self.native_mod_screen_close(SCREEN);
        }
    }

    pub(super) fn client_inventory_tracker_query(&mut self, request: &str) -> String {
        self.client_inventory_tracker_world_refresh();
        let Ok(q) = serde_json::from_str::<Value>(request) else {
            return "null".into();
        };
        if matches!(q["op"].as_str(), Some("reset" | "clearSaved")) {
            if q["op"] == "reset" {
                self.client_inventory_tracker.reset();
            } else {
                for p in self.client_inventory_tracker.players.values_mut() {
                    p.saved = false;
                }
            }
            return "true".into();
        }
        if !self.client_auto_tool_player() {
            return "null".into();
        }
        self.client_inventory_tracker_refresh();
        let out = match q["op"].as_str() {
            Some("world") => {
                let players=self.client_inventory_tracker.players.values().filter(|p|p.current||p.saved)
                    .map(|p|json!({"uuid":p.uuid,"name":p.name,"playerToken":p.token,"current":p.current,"saved":p.saved})).collect::<Vec<_>>();
                json!({"players":players,"online":self.client_anti_bot_online()})
            }
            Some("open") => {
                let uuid = q["uuid"]
                    .as_str()
                    .filter(|uuid| self.client_inventory_tracker.resolve(uuid).is_some());
                if let Some(uuid) = uuid {
                    self.client_inventory_tracker.pending_open = Some(uuid.into());
                    json!(true)
                } else {
                    json!(false)
                }
            }
            Some("clearSaved") => {
                for p in self.client_inventory_tracker.players.values_mut() {
                    p.saved = false;
                }
                json!(true)
            }
            Some("reset") => {
                self.client_inventory_tracker.reset();
                json!(true)
            }
            _ => Value::Null,
        };
        out.to_string()
    }

    pub(super) fn client_inventory_tracker_time() -> i64 {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_millis().min(i64::MAX as u128) as i64,
            Err(e) => -(e.duration().as_millis().min(i64::MAX as u128) as i64),
        }
    }
}
