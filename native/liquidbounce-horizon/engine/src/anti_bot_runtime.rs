// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientAntiBotEvent, Host};
use liquidbounce_behaviors::anti_bot::{Armor, Entry, Player, Profile, State};
use serde_json::Value;

#[derive(Default)]
pub struct AntiBotRuntime {
    pub state: State,
}
pub(crate) struct Query {
    pub player: Player,
    pub online: Vec<Profile>,
    pub max_health: f32,
}
fn profiles(v: &Value) -> Vec<Profile> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| {
            Some(Profile {
                uuid: v["uuid"].as_str()?.into(),
                name: v["name"].as_str()?.into(),
            })
        })
        .collect()
}
fn player(v: &Value) -> Option<Player> {
    let armor = v["armor"].as_array().filter(|a| a.len() == 4)?;
    Some(Player {
        id: v["id"].as_i64()? as i32,
        token: v["token"].as_u64()?,
        uuid: v["uuid"].as_str()?.into(),
        name: v["name"].as_str()?.into(),
        local: v["local"] == true,
        ticks: v["ticks"].as_i64()? as i32,
        position: serde_json::from_value(v["position"].clone()).ok()?,
        previous: serde_json::from_value(v["previous"].clone()).ok()?,
        on_ground: v["onGround"] == true,
        pitch: f32::from_bits(v["pitchBits"].as_u64()? as u32),
        health: f32::from_bits(v["healthBits"].as_u64()? as u32),
        scale: f64::from_bits(v["scaleBits"].as_str()?.parse().ok()?),
        armor: std::array::from_fn(|i| Armor {
            identity: armor[i]["identity"].as_u64().unwrap_or(0),
            predicates: armor[i]["predicates"].as_u64().unwrap_or(1) as u16,
            player_armor: armor[i]["playerArmor"] == true,
            enchanted: armor[i]["enchanted"] == true,
        }),
        properties_empty: v["propertiesEmpty"] == true,
        online: v["online"] == true,
        listed: v["listed"] == true,
        game_mode: v["mode"].as_i64().map(|v| v as i32),
        ping: v["ping"].as_i64().unwrap_or(0) as i32,
    })
}
pub(crate) fn query(v: &Value) -> Option<Query> {
    Some(Query {
        player: player(&v["player"])?,
        online: profiles(&v["online"]),
        max_health: f32::from_bits(v["maxHealthBits"].as_u64()? as u32),
    })
}
fn players(v: &Value) -> Vec<Player> {
    v["players"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(player)
        .collect()
}
fn world(host: &dyn Host) -> Option<Value> {
    serde_json::from_str(&host.client_anti_bot_snapshot()?).ok()
}
impl LiquidBounceHooks {
    pub(crate) fn anti_bot_packet(&self, _host: &mut dyn Host, event: &mut ClientAntiBotEvent<'_>) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        if !b.query_running(&state.registry, "AntiBot") {
            return;
        }
        let s = &mut b.anti_bot.state;
        match v["kind"].as_str() {
            Some("info") => {
                let entries = v["entries"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|e| {
                        Some(Entry {
                            uuid: e["uuid"].as_str()?.into(),
                            name: e["name"].as_str().map(str::to_owned),
                            properties_empty: e["propertiesEmpty"] == true,
                            mode: e["mode"].as_i64().map(|v| v as i32),
                            latency: e["latency"].as_i64().unwrap_or(0) as i32,
                        })
                    })
                    .collect::<Vec<_>>();
                s.info(
                    v["actions"].as_u64().unwrap_or(0) as u8,
                    &entries,
                    &profiles(&v["online"]),
                    v["time"].as_i64().unwrap_or(0),
                );
            }
            Some("removeProfiles") => s.remove_profiles(
                &v["ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect::<Vec<_>>(),
            ),
            Some("ground") => s.ground(
                v["id"].as_i64().unwrap_or(0) as i32,
                v["ground"] == true,
                v["previous"].as_f64().unwrap_or(0.),
                v["y"].as_f64().unwrap_or(0.),
            ),
            Some("attributes") => s.attributes(v["id"].as_i64().unwrap_or(0) as i32),
            Some("metadata") => s.metadata(
                v["id"].as_i64().unwrap_or(0) as i32,
                v["size"].as_u64().unwrap_or(0) as usize,
            ),
            Some("animate") => s.animate(
                v["id"].as_i64().unwrap_or(0) as i32,
                v["action"].as_u64().unwrap_or(0) as u8,
            ),
            Some("swing") => s.swing(v["id"].as_i64().unwrap_or(0) as i32),
            Some("removeEntities") => {
                s.observe(&players(&v["world"]));
                s.remove_entities(
                    &v["ids"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|v| v.as_i64().map(|i| i as i32))
                        .collect::<Vec<_>>(),
                );
            }
            _ => {}
        }
    }
    pub(crate) fn anti_bot_attack(&self, id: i32) {
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        if b.query_running(&state.registry, "AntiBot") {
            b.anti_bot.state.attack(id);
        }
    }
    pub(crate) fn anti_bot_tick(&self, host: &dyn Host) {
        let Some(v) = world(host) else {
            return;
        };
        let players = players(&v);
        let online = profiles(&v["online"]);
        let maximum = f32::from_bits(
            v["maxHealthBits"]
                .as_u64()
                .unwrap_or(20f32.to_bits() as u64) as u32,
        );
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let running = b.query_running(&state.registry, "AntiBot");
        let (bots, callback) = {
            let s = &mut b.anti_bot.state;
            s.observe(&players);
            if running {
                if let Ok(own) = serde_json::from_value::<[f64; 3]>(v["own"].clone()) {
                    s.custom_tick(own, &players);
                }
            }
            let callback = running.then(|| s.matrix_start(&players)).flatten();
            let bots = players
                .iter()
                .filter(|p| s.is_bot(running, p, &online, maximum))
                .map(|p| p.id)
                .collect();
            (bots, callback)
        };
        b.notifier.bot_entities = bots;
        if let Some(callback) = callback {
            b.coroutine_ticker
                .register(crate::coroutine_ticker::WaitOwner::AntiBotMatrix, callback);
        }
    }
    pub(crate) fn anti_bot_callback(&self, host: &dyn Host, in_game: bool, callback: u64) -> bool {
        let Some(v) = world(host) else {
            return false;
        };
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        let running = in_game
            && b.query_running(&state.registry, "AntiBot")
            && b.anti_bot.state.settings.mode == 1;
        if !running {
            b.anti_bot.state.matrix.continuation = None;
            return false;
        }
        b.anti_bot.state.matrix_resume(callback, &players(&v))
    }
    pub(crate) fn anti_bot_reset(&self) {
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        b.anti_bot.state.reset();
        b.notifier.bot_entities.clear();
    }
}
