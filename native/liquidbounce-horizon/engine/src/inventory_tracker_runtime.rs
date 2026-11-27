// SPDX-License-Identifier: GPL-3.0-or-later
//! Live remote equipment history and the shared CommandManager's invsee route.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::inventory_tracker::{self as model, Equipment, Item, Slot};
use serde_json::{Value, json};

#[derive(Default)]
pub struct InventoryTrackerRuntime {
    pub model: model::InventoryTracker,
    pub pending_reset: bool,
    pub pending_saved_clear: bool,
}
impl InventoryTrackerRuntime {
    pub fn save_players(&mut self, value: bool) {
        if self.model.set_save_players(value) {
            self.pending_saved_clear = true;
        }
    }
    pub fn reset(&mut self) {
        self.model.reset();
        self.pending_reset = true;
    }
    fn take_pending(&mut self) -> (bool, bool) {
        let reset = std::mem::take(&mut self.pending_reset);
        let saved = std::mem::take(&mut self.pending_saved_clear);
        (reset, saved)
    }
}
fn tr<'a>(en: &'a str, cn: &'a str, tw: &'a str) -> &'a str {
    match horizon_client::lang::code()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "zh_cn" | "zh_sg" | "zh_hans" => cn,
        "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => tw,
        _ => en,
    }
}
fn query(host: &mut dyn Host, request: Value) -> Value {
    serde_json::from_str(&host.client_inventory_tracker_query(&request.to_string()))
        .unwrap_or(Value::Null)
}
fn item(value: &Value) -> Result<Option<Item>, ()> {
    if value.is_null() {
        return Ok(None);
    }
    let count = i32::try_from(value["count"].as_i64().ok_or(())?).map_err(|_| ())?;
    Ok(Some(Item {
        handle: value["handle"].as_u64().filter(|v| *v != 0).ok_or(())?,
        item: value["item"].as_str().ok_or(())?.into(),
        enchantments: value["enchantments"].to_string(),
        count,
    }))
}
fn slot(value: &Value) -> Option<Slot> {
    Some(match value.as_str()? {
        "mainhand" => Slot::MainHand,
        "offhand" => Slot::OffHand,
        "head" | "chest" | "legs" | "feet" => Slot::Armor,
        "body" => Slot::AnimalArmor,
        "saddle" => Slot::Other,
        _ => return None,
    })
}
impl LiquidBounceHooks {
    /// Called inside a real native event; the complete facts arrive in input,
    /// so no Game helper or actor re-entry occurs while State is borrowed.
    pub(crate) fn inventory_tracker_event(&self, input: &str, output: &mut String) {
        let Ok(value) = serde_json::from_str::<Value>(input) else {
            return;
        };
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if value["kind"] == "completion" {
            let completion = value["text"].as_str().and_then(|text| {
                model::complete(
                    text,
                    value["cursor"]
                        .as_u64()
                        .and_then(|cursor| usize::try_from(cursor).ok())
                        .unwrap_or(0),
                    &modules.client_commands.prefix,
                    value["online"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|profile| profile["name"].as_str()),
                )
            });
            *output = completion.map_or_else(
                || json!({"handled":false}),
                |completion| json!({"handled":true,"range":[completion.start,completion.len],"suggestions":completion.suggestions}),
            ).to_string();
            return;
        }
        let running = modules.query_running(&state.registry, "InventoryTracker");
        if value["kind"] == "world" {
            modules.inventory_tracker.model.reset();
            modules.inventory_tracker.pending_reset = false;
            modules.inventory_tracker.pending_saved_clear = false;
            *output = json!({"running":running,"track":false}).to_string();
            return;
        }
        let (reset, clear_saved) = if value["kind"] == "equipment" {
            modules.inventory_tracker.take_pending()
        } else {
            (false, false)
        };
        let mut result =
            json!({"running":running,"track":false,"reset":reset,"clearSaved":clear_saved});
        if running && value["kind"] == "equipment" {
            if let (Some(uuid), Some(player_token), Some(slot), Ok(new), Ok(main), Ok(off)) = (
                value["uuid"].as_str().filter(|uuid| !uuid.is_empty()),
                value["playerToken"].as_u64(),
                slot(&value["slot"]),
                item(&value["new"]),
                item(&value["main"]),
                item(&value["off"]),
            ) {
                let bot = crate::anti_bot_runtime::query(&value["antiBot"]).is_some_and(|query| {
                    modules.anti_bot.state.is_bot(
                        modules.query_running(&state.registry, "AntiBot"),
                        &query.player,
                        &query.online,
                        query.max_health,
                    )
                });
                let tracker = &mut modules.inventory_tracker.model;
                if let Some(projection) = tracker.equipment(Equipment {
                    uuid,
                    player_token,
                    remote: value["remote"] == true,
                    bot,
                    slot,
                    new_item: new.as_ref(),
                    main: main.as_ref(),
                    off: off.as_ref(),
                    now: value["time"].as_i64().unwrap_or(0),
                }) {
                    result["track"] = json!(true);
                    result["save"] = json!(tracker.save_players);
                    result["projection"] = json!(&projection[..]);
                }
            }
        } else if running && value["kind"] == "lore" {
            result["lore"] = value["handle"]
                .as_u64()
                .and_then(|handle| modules.inventory_tracker.model.timestamp(handle))
                .map(|timestamp| {
                    format!(
                        "{}{}",
                        tr("Last Seen: ", "上次见到：", "上次見到："),
                        model::elapsed(value["time"].as_i64().unwrap_or(timestamp), timestamp)
                    )
                })
                .map_or(Value::Null, Value::String);
        }
        *output = result.to_string();
    }
    pub(crate) fn inventory_tracker_flush(&self, host: &mut dyn Host) {
        let (reset, saved) = self
            .state()
            .behaviors
            .borrow_mut()
            .inventory_tracker
            .take_pending();
        if reset {
            query(host, json!({"op":"reset"}));
        } else if saved {
            query(host, json!({"op":"clearSaved"}));
        }
    }
    pub(crate) fn inventory_tracker_command(
        &self,
        host: &mut dyn Host,
        reader: &mut liquidbounce_behaviors::client_commands::Reader,
    ) -> Result<(), String> {
        let name = reader.string()?.ok_or_else(|| "invsee <name>".to_owned())?;
        reader.finish()?;
        self.inventory_tracker_flush(host);
        let world = query(host, json!({"op":"world"}));
        if !host.client_state().is_some_and(|state| {
            state.world.is_some() && state.logged_in && state.eye_position.is_some()
        }) {
            return Err(tr(
                "A player in a world is required",
                "需要玩家进入世界",
                "需要玩家進入世界",
            )
            .into());
        }
        // PlayerInfoArgumentType resolves only a current tab-list profile,
        // even when SavePlayers retains a player outside render distance.
        let uuid = world["online"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|profile| {
                profile["name"]
                    .as_str()
                    .is_some_and(|value| value.eq_ignore_ascii_case(&name))
            })
            .and_then(|profile| profile["uuid"].as_str())
            .ok_or_else(|| {
                format!(
                    "{}: {name}",
                    tr("No such player", "没有此玩家", "沒有此玩家")
                )
            })?;
        if !world["players"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|player| player["uuid"] == uuid)
        {
            return Err(format!(
                "{}: {name}",
                tr("Player not found", "未找到玩家", "找不到玩家")
            ));
        }
        let opened = query(host, json!({"op":"open","uuid":uuid}));
        if opened != true && opened["opened"] != true {
            return Err(format!(
                "{}: {name}",
                tr("Player not found", "未找到玩家", "找不到玩家")
            ));
        }
        self.state()
            .behaviors
            .borrow_mut()
            .inventory_tracker
            .model
            .viewed_player = Some(uuid.into());
        Ok(())
    }
}
