// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientNotifierEvent, Host};
use liquidbounce_behaviors::{
    notifier::{Arg, DEFAULTS, Notice, Player, Settings, Stack, State},
    notifier_format::format_message,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
#[derive(Default)]
pub struct NotifierRuntime {
    pub model: State,
    pub bot_entities: BTreeSet<i32>,
}
fn stack(v: &Value) -> Option<Stack> {
    Some(Stack {
        id: v["id"].as_str()?.into(),
        components: v["components"].as_str()?.into(),
        name: v["name"].as_str()?.into(),
        count: v["count"].as_i64()? as i32,
        duration: v["duration"].as_i64()? as i32,
        consumable: v["consumable"] == true,
    })
}
fn player(v: &Value, bots: &BTreeSet<i32>) -> Option<Player> {
    let id = v["id"].as_i64()? as i32;
    Some(Player {
        id,
        uuid: v["uuid"].as_str()?.into(),
        name: v["name"].as_str()?.into(),
        bot: bots.contains(&id),
        using: v["using"] == true,
        active_hand: v["activeHand"].as_u64().unwrap_or(0) as usize,
        ticks_using: v["ticksUsing"].as_i64().unwrap_or(0) as i32,
        hands: [stack(&v["hands"][0]), stack(&v["hands"][1])],
    })
}
fn render(n: Notice, s: &Settings) -> Result<String, String> {
    let lang = liquidbounce_interop::localization::Locale::from_game_language(
        &horizon_client::lang::code(),
    );
    let mut args = n.args;
    let pattern = if s.formats[n.kind] == DEFAULTS[n.kind]
        && lang != liquidbounce_interop::localization::Locale::English
    {
        let key = format!("liquidbounce.module.notifier.messages.default{}", n.kind);
        if n.kind == 2 {
            if let Some(Arg::Text(mode)) = args.get_mut(1) {
                *mode = liquidbounce_interop::localization::describe(
                    lang.code(),
                    &format!("liquidbounce.module.notifier.messages.{mode}"),
                );
            }
        }
        if n.kind == 4 {
            if let Some(Arg::Text(hand)) = args.get_mut(3) {
                *hand = liquidbounce_interop::localization::describe(
                    lang.code(),
                    &format!("liquidbounce.module.notifier.messages.{hand}"),
                );
            }
        }
        liquidbounce_interop::localization::describe(lang.code(), &key)
    } else {
        s.formats[n.kind].clone()
    };
    format_message(&pattern, &args)
}
impl LiquidBounceHooks {
    fn notifier_initialize(&self, host: &dyn Host) -> bool {
        let (active, initialize) = {
            let state = self.state();
            let mut modules = state.behaviors.borrow_mut();
            let enabled = modules
                .query_ids
                .get("Notifier")
                .is_some_and(|id| state.registry.enabled_without_world(*id));
            let active = modules.query_running(&state.registry, "Notifier");
            modules.notifier.model.sync(enabled);
            (active, modules.notifier.model.needs_initialize())
        };
        if initialize {
            if let Some(v) = host
                .client_notifier_snapshot()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            {
                let online = v["online"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|p| {
                        Some((
                            p["uuid"].as_str()?.into(),
                            p["name"].as_str()?.into(),
                            p["mode"].as_i64()? as i32,
                        ))
                    })
                    .collect::<Vec<_>>();
                self.state()
                    .behaviors
                    .borrow_mut()
                    .notifier
                    .model
                    .initialize(online);
            }
        }
        active
    }
    fn notifier_messages(&self, host: &mut dyn Host, notices: Vec<Notice>) {
        let settings = self
            .state()
            .behaviors
            .borrow()
            .notifier
            .model
            .settings
            .clone();
        for n in notices {
            match render(n, &settings) {
                Ok(message) => {
                    if settings.notification {
                        #[cfg(feature="webview")]
    self.state().events.borrow_mut().push(("notification".into(),json!({"title":liquidbounce_interop::localization::display_module_name(&horizon_client::lang::code(),"Notifier"),"message":message,"severity":"INFO"})));
                    } else {
                        host.chat(&message);
                    }
                }
                Err(error) => eprintln!("[LiquidBounce] Notifier format: {error}"),
            }
        }
    }
    pub(crate) fn notifier_event(&self, host: &mut dyn Host, event: &mut ClientNotifierEvent<'_>) {
        let active = self.notifier_initialize(host);
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let notices = {
            let state = self.state();
            let friend = v["name"]
                .as_str()
                .is_some_and(|n| state.friends.contains(n));
            let mut b = state.behaviors.borrow_mut();
            let m = &mut b.notifier.model;
            let mut out = vec![];
            match v["kind"].as_str() {
                Some("info") if active => {
                    let a = v["actions"].as_u64().unwrap_or(0);
                    let rows = v["entries"].as_array().into_iter().flatten();
                    if a & 1 != 0 {
                        for p in rows.clone() {
                            if let (Some(id), Some(name)) = (p["uuid"].as_str(), p["name"].as_str())
                            {
                                out.extend(m.add(id.into(), name.into()));
                            }
                        }
                    }
                    if a & 4 != 0 {
                        for p in rows {
                            if let (Some(id), Some(mode)) = (p["uuid"].as_str(), p["mode"].as_i64())
                            {
                                out.extend(m.mode(id, mode as i32, a & 1 != 0));
                            }
                        }
                    }
                }
                Some("remove") if active => {
                    for id in v["ids"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                    {
                        out.extend(m.remove(id));
                    }
                }
                Some("removeEntities") => {
                    for id in v["ids"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                    {
                        m.remove_entity(id);
                    }
                }
                Some("entity") => {
                    if let (Some(id), Some(name), Some(code)) =
                        (v["uuid"].as_str(), v["name"].as_str(), v["event"].as_u64())
                    {
                        let notice =
                            m.entity_event(id, name, code as u8, v["local"] == true, friend);
                        if active {
                            out.extend(notice);
                        }
                    }
                }
                _ => {}
            }
            out
        };
        self.notifier_messages(host, notices);
    }
    pub(crate) fn notifier_tick(&self, host: &mut dyn Host) {
        if !self.notifier_initialize(host) {
            return;
        }
        let Some(v) = host
            .client_notifier_snapshot()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return;
        };
        let notices = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let r = &mut b.notifier;
            let players = v["players"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| player(v, &r.bot_entities))
                .collect::<Vec<_>>();
            r.model.tick(&players)
        };
        self.notifier_messages(host, notices);
    }
}
