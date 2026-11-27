// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientMurderMysteryEvent, Host};
use liquidbounce_behaviors::{
    murder_mystery::{Item, Message, Player, State},
    targeting::TargetInfo,
};
use serde_json::{Value, json};
fn player(v: &Value) -> Option<Player> {
    Some(Player {
        uuid: v["uuid"].as_str()?.into(),
        name: v["name"].as_str()?.into(),
        position: serde_json::from_value(v["position"].clone()).ok()?,
        using_bow: v["usingBow"] == true,
        has_arrow: v["hasArrow"] == true,
    })
}
fn item(v: &Value) -> Item {
    Item {
        id: v["id"].as_str().unwrap_or("minecraft:air").into(),
        sword: v["sword"] == true,
        mining: v["mining"] == true,
        boat: v["boat"] == true,
    }
}
pub(crate) fn apply_tag(
    module: &mut State,
    running: bool,
    abstract_player: bool,
    name: Option<&str>,
    uuid: Option<&str>,
    info: &mut TargetInfo,
) {
    if !running || !abstract_player {
        return;
    }
    let Some((attack, role)) = name.and_then(|name| module.tag(name, uuid)) else {
        return;
    };
    if !attack {
        info.dont_target();
    }
    if let Some(color) = role.color() {
        info.color(color, 35);
    }
}
fn language() -> u8 {
    match horizon_client::lang::code()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "zh_cn" | "zh_sg" | "zh_hans" => 1,
        "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => 2,
        _ => 0,
    }
}
fn message(m: Message) -> String {
    match (language(), m) {
        (1, Message::Murderer(n)) => format!("凶手是 {n}"),
        (2, Message::Murderer(n)) => format!("兇手是 {n}"),
        (_, Message::Murderer(n)) => format!("It's {n}"),
        (1, Message::Bow(n)) => format!("{n} 持有弓。"),
        (2, Message::Bow(n)) => format!("{n} 持有弓。"),
        (_, Message::Bow(n)) => format!("{n} has a bow."),
        (1, Message::FirstInfected(n)) => format!("{n} 是首位感染者。"),
        (2, Message::FirstInfected(n)) => format!("{n} 是首位感染者。"),
        (_, Message::FirstInfected(n)) => format!("{n} is the first infected."),
        (1, Message::Assassin(n)) => format!("你的刺客：{n}"),
        (2, Message::Assassin(n)) => format!("你的刺客：{n}"),
        (_, Message::Assassin(n)) => format!("Your Assassin: {n}"),
        (1, Message::Target(n)) => format!("目标：{n}"),
        (2, Message::Target(n)) => format!("目標：{n}"),
        (_, Message::Target(n)) => format!("Target: {n}"),
        (1, Message::MissingTarget(n)) => format!("未找到目标，识别到的名称为：{n}"),
        (2, Message::MissingTarget(n)) => format!("未找到目標，辨識到的名稱為：{n}"),
        (_, Message::MissingTarget(n)) => format!("Failed to find target, but the name is: {n}"),
    }
}
impl LiquidBounceHooks {
    pub(crate) fn murder_mystery(
        &self,
        host: &mut dyn Host,
        event: &mut ClientMurderMysteryEvent<'_>,
    ) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let messages = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            let running = b.query_running(&state.registry, "MurderMystery");
            if !running {
                *event.output = "null".into();
                b.auto_dodge.murder_mystery_disallows = false;
                return;
            }
            let m = &mut b.murder_mystery;
            let mut messages = vec![];
            let players = v["players"]
                .as_array()
                .map(|a| a.iter().filter_map(player).collect::<Vec<_>>())
                .unwrap_or_default();
            for p in &players {
                m.profiles.insert(p.name.clone(), p.uuid.clone());
            }
            match v["kind"].as_str().unwrap_or("") {
                "reset" => m.reset(),
                "equipment" => {
                    if let Some(p) = player(&v["player"]) {
                        if let Some(items) = v["items"].as_array() {
                            for i in items {
                                if let Some(message) = m.item(&p, &item(i)) {
                                    messages.push(message);
                                }
                            }
                        }
                    }
                }
                "sound" => {
                    if let Ok(own) = serde_json::from_value::<[f64; 3]>(v["own"].clone()) {
                        if let Some(message) =
                            m.sound(v["volume"].as_f64().unwrap_or(0.) as f32, own, &players)
                        {
                            messages.push(message);
                        }
                    }
                }
                "tick" => {
                    let hands = std::array::from_fn(|i| item(&v["hands"][i]));
                    messages.extend(m.tick(&hands, &players));
                    let online = v["online"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| {
                                    Some((v["uuid"].as_str()?.into(), v["name"].as_str()?.into()))
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let colors = v["map"]["colors"].as_array().map(|a| {
                        a.iter()
                            .map(|v| v.as_u64().unwrap_or(0) as u8)
                            .collect::<Vec<_>>()
                    });
                    if let Some(message) = m.contract(
                        v["map"]["isMap"] == true,
                        v["map"]["id"].as_i64().map(|n| n as i32),
                        colors.as_deref(),
                        &online,
                    ) {
                        messages.push(message);
                    }
                }
                _ => {}
            }
            let (hurt, bow) = if v["kind"] == "frame" {
                m.take_sounds()
            } else {
                (false, false)
            };
            let prefixes = std::mem::take(&mut m.prefixes)
                .into_iter()
                .filter_map(|(name, role)| {
                    role.prefix()
                        .map(|(text, color)| json!({"name":name,"text":text,"color":color}))
                })
                .collect::<Vec<_>>();
            *event.output =
                json!({"active":true,"hurt":hurt,"bow":bow,"prefixes":prefixes}).to_string();
            b.auto_dodge.murder_mystery_disallows = m.disallows_arrow_dodge(true);
            b.timer.set_provider_running(
                crate::auto_dodge_runtime::TIMER_PROVIDER,
                b.query_running(&state.registry, "AutoDodge")
                    && !b.query_running(&state.registry, "Blink")
                    && !b.auto_dodge.murder_mystery_disallows,
            );
            messages
        };
        for m in messages {
            host.chat(&message(m));
        }
    }
    pub(crate) fn murder_mystery_reset(&self) {
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        b.murder_mystery.reset();
        b.auto_dodge.murder_mystery_disallows = b
            .murder_mystery
            .disallows_arrow_dodge(b.query_running(&state.registry, "MurderMystery"));
    }
}
