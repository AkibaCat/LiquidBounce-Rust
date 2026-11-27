// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::auto_queue::{
    ACTIONS, CHAT_TYPES, GAMES, MATCHES, PAUSES, PRESETS, Pick, State, TRIGGERS,
};
#[derive(Clone, Debug, PartialEq)]
pub struct AutoQueueSetting {
    pub path: String,
    pub value: Value,
}
fn choice(v: &Value, p: &str, tags: &[&str]) -> Result<usize, ConfigError> {
    scalar_string(v)
        .and_then(|s| tags.iter().position(|t| *t == s))
        .ok_or_else(|| ConfigError::new(p, "unknown choice"))
}
fn texts(v: &Value, p: &str) -> Result<Vec<String>, ConfigError> {
    match v {
        Value::Array(a) => a
            .iter()
            .map(|v| scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected text entry")))
            .collect(),
        _ => scalar_string(v)
            .map(|s| vec![s])
            .ok_or_else(|| ConfigError::new(p, "expected text list")),
    }
}
fn apply_pick(pick: &mut Pick, path: &str, v: &Value, items: &[String]) -> Result<(), ConfigError> {
    match path {
        "Mode.active" => pick.mode = choice(v, path, &["Name", "Item"])?,
        "Mode.Name.Names" => {
            let mut names = texts(v, path)?;
            names.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            names.dedup();
            pick.names = names;
        }
        "Mode.Name.Mode" => pick.match_mode = choice(v, path, &MATCHES)?,
        "Mode.Item.Items" => {
            let mut entries = texts(v, path)?
                .into_iter()
                .map(|name| {
                    let name = if name.contains(':') {
                        name
                    } else {
                        format!("minecraft:{name}")
                    };
                    if items.contains(&name) {
                        name
                    } else {
                        "minecraft:air".into()
                    }
                })
                .collect::<Vec<_>>();
            entries.sort_by(|a, b| {
                let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
                let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
                ap.cmp(bp).then(an.cmp(bn))
            });
            entries.dedup();
            pick.items = entries;
        }
        _ => return Err(ConfigError::new(path, "unknown item selection setting")),
    }
    Ok(())
}
impl AutoQueueSetting {
    pub fn apply(self, s: &mut State, items: &[String]) -> Result<(), ConfigError> {
        let p = self.path.as_str();
        let v = &self.value;
        let text = || scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected text"));
        if let Some(p) = p.strip_prefix("Presets.Custom.Trigger.Item.") {
            return apply_pick(&mut s.settings.trigger_pick, p, v, items);
        }
        if let Some(p) = p.strip_prefix("Presets.Custom.Action.UseItem.") {
            return apply_pick(&mut s.settings.action_pick, p, v, items);
        }
        match p {
            "Presets.active" => s.set_preset(choice(v, p, &PRESETS)?),
            "Pause" => s.settings.pause = parse_multi_mask(v, p, &PAUSES)?,
            "Presets.HypixelSW.GameMode" => s.settings.game = choice(v, p, &GAMES)?,
            "Presets.GommeDuels.WinMessage" => s.settings.win = text()?,
            "Presets.GommeDuels.LoseMessage" => s.settings.lose = text()?,
            "Presets.GommeDuels.ControlKillAura" => s.settings.control_aura = parse_bool(v, p)?,
            "Presets.Custom.Trigger.active" => s.set_trigger(choice(v, p, &TRIGGERS)?),
            "Presets.Custom.Action.active" => s.settings.action = choice(v, p, &ACTIONS)?,
            "Presets.Custom.Trigger.Title.Keywords" => {
                let words = texts(v, p)?;
                s.settings.title = words.clone();
                s.settings.subtitle = words;
            }
            "Presets.Custom.Trigger.Message.Keywords" => s.settings.messages = texts(v, p)?,
            "Presets.Custom.Trigger.Message.ChatTypes" => {
                let mask = parse_multi_mask(v, p, &CHAT_TYPES)?;
                if mask == 0 {
                    return Err(ConfigError::new(p, "at least one chat type is required"));
                }
                s.settings.chat_types = mask;
            }
            "Presets.Custom.Trigger.TabHeader.Text" => s.settings.header = text()?,
            "Presets.Custom.Trigger.TabFooter.Text" => s.settings.footer = text()?,
            "Presets.Custom.Action.Chat.StartDelay" | "Presets.Custom.Action.Chat.MessageDelay" => {
                let range = parse_int_pair(v, p)?;
                if range[0] > range[1] {
                    return Err(ConfigError::new(p, "empty delay range"));
                }
                if p.ends_with(".StartDelay") {
                    s.settings.start_delay = range;
                } else {
                    s.settings.message_delay = range;
                }
            }
            "Presets.Custom.Action.Chat.Messages" => s.settings.sends = texts(v, p)?,
            "Presets.Custom.Control.Enabled" => s.settings.control = parse_bool(v, p)?,
            "Presets.Custom.Control.KillAura" => s.settings.kill_aura = parse_bool(v, p)?,
            "Presets.Custom.Control.Speed" => s.settings.speed = parse_bool(v, p)?,
            "Presets.Custom.WaitUntilWorldChange" => s.settings.wait_world = parse_bool(v, p)?,
            _ => return Err(ConfigError::new(p, "unknown AutoQueue setting")),
        }
        Ok(())
    }
}
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.autoQueue.{}",
        path.split('.')
            .map(|part| {
                let mut c = part.chars();
                c.next().map_or_else(String::new, |a| {
                    a.to_lowercase().chain(c).collect::<String>()
                })
            })
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn leaf_q(t: &Tree<'_>, name: &str, kind: &str, v: Value, path: &str) -> Value {
    t.leaf(name, kind, v, &key(path), None)
}
fn list(t: &Tree<'_>, name: &str, values: &[String], path: &str) -> Value {
    let mut v = leaf_q(t, name, "MUTABLE_LIST", json!(values), path);
    if t.format == JsonFormat::Interop {
        v["innerValueType"] = json!("TEXT");
    }
    v
}
fn choose(t: &Tree<'_>, name: &str, index: usize, tags: &[&str], path: &str) -> Value {
    let mut v = leaf_q(t, name, "CHOOSE", json!(tags[index]), path);
    if t.format == JsonFormat::Interop {
        v["choices"] = json!(tags);
    }
    v
}
fn modes(t: &Tree<'_>, name: &str, active: &str, groups: Vec<Value>, path: &str) -> Value {
    let mut v = leaf_q(t, name, "CHOICE", json!([]), path);
    v["active"] = json!(active);
    let choices = groups
        .into_iter()
        .map(|g| (g["name"].as_str().unwrap().to_owned(), g))
        .collect::<Map<_, _>>();
    v["choices"] = Value::Object(choices);
    v
}
fn pick_tree(t: &Tree<'_>, s: &Pick, prefix: &str) -> Value {
    let name = t.group(
        "Name",
        "CONFIGURABLE",
        vec![
            list(t, "Names", &s.names, &format!("{prefix}.Mode.Name.Names")),
            choose(
                t,
                "Mode",
                s.match_mode,
                &MATCHES,
                &format!("{prefix}.Mode.Name.Mode"),
            ),
        ],
    );
    let mut items = leaf_q(
        t,
        "Items",
        "REGISTRY_LIST",
        json!(s.items),
        &format!("{prefix}.Mode.Item.Items"),
    );
    if t.format == JsonFormat::Interop {
        items["registry"] = json!("item");
    }
    modes(
        t,
        "Mode",
        ["Name", "Item"][s.mode],
        vec![name, t.group("Item", "CONFIGURABLE", vec![items])],
        &format!("{prefix}.Mode"),
    )
}
pub fn serialize_auto_queue(
    state: &State,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let s = &state.settings;
    let mut values = t.common(ModuleKind::AutoQueue, common);
    let h = t.group(
        "HypixelSW",
        "CONFIGURABLE",
        vec![choose(
            &t,
            "GameMode",
            s.game,
            &GAMES,
            "Presets.HypixelSW.GameMode",
        )],
    );
    let g = t.group(
        "GommeDuels",
        "CONFIGURABLE",
        vec![
            leaf_q(
                &t,
                "WinMessage",
                "TEXT",
                json!(s.win),
                "Presets.GommeDuels.WinMessage",
            ),
            leaf_q(
                &t,
                "LoseMessage",
                "TEXT",
                json!(s.lose),
                "Presets.GommeDuels.LoseMessage",
            ),
            leaf_q(
                &t,
                "ControlKillAura",
                "BOOLEAN",
                json!(s.control_aura),
                "Presets.GommeDuels.ControlKillAura",
            ),
        ],
    );
    let mut types = t.multi(
        "ChatTypes",
        s.chat_types,
        &CHAT_TYPES,
        &key("Presets.Custom.Trigger.Message.ChatTypes"),
    );
    if format == JsonFormat::Interop {
        types["canBeNone"] = json!(false);
    }
    let triggers = modes(
        &t,
        "Trigger",
        TRIGGERS[s.trigger],
        vec![
            t.group(
                "Title",
                "CONFIGURABLE",
                vec![list(
                    &t,
                    "Keywords",
                    &s.title,
                    "Presets.Custom.Trigger.Title.Keywords",
                )],
            ),
            t.group(
                "Title",
                "CONFIGURABLE",
                vec![list(
                    &t,
                    "Keywords",
                    &s.subtitle,
                    "Presets.Custom.Trigger.Title.Keywords",
                )],
            ),
            t.group(
                "Message",
                "CONFIGURABLE",
                vec![
                    list(
                        &t,
                        "Keywords",
                        &s.messages,
                        "Presets.Custom.Trigger.Message.Keywords",
                    ),
                    types,
                ],
            ),
            t.group(
                "Item",
                "CONFIGURABLE",
                vec![pick_tree(
                    &t,
                    &s.trigger_pick,
                    "Presets.Custom.Trigger.Item",
                )],
            ),
            t.group(
                "TabHeader",
                "CONFIGURABLE",
                vec![leaf_q(
                    &t,
                    "Text",
                    "TEXT",
                    json!(s.header),
                    "Presets.Custom.Trigger.TabHeader.Text",
                )],
            ),
            t.group(
                "TabFooter",
                "CONFIGURABLE",
                vec![leaf_q(
                    &t,
                    "Text",
                    "TEXT",
                    json!(s.footer),
                    "Presets.Custom.Trigger.TabFooter.Text",
                )],
            ),
        ],
        "Presets.Custom.Trigger",
    );
    let range = |name: &str, pair: [i32; 2]| {
        t.ranged(
            name,
            "INT_RANGE",
            json!({"from":pair[0],"to":pair[1]}),
            json!({"from":0,"to":5000}),
            "ms",
            &key(&format!("Presets.Custom.Action.Chat.{name}")),
        )
    };
    let actions = modes(
        &t,
        "Action",
        ACTIONS[s.action],
        vec![
            t.group(
                "Chat",
                "CONFIGURABLE",
                vec![
                    range("StartDelay", s.start_delay),
                    range("MessageDelay", s.message_delay),
                    list(
                        &t,
                        "Messages",
                        &s.sends,
                        "Presets.Custom.Action.Chat.Messages",
                    ),
                ],
            ),
            t.group(
                "UseItem",
                "CONFIGURABLE",
                vec![pick_tree(
                    &t,
                    &s.action_pick,
                    "Presets.Custom.Action.UseItem",
                )],
            ),
        ],
        "Presets.Custom.Action",
    );
    let control = t.group(
        "Control",
        "TOGGLEABLE",
        vec![
            leaf_q(
                &t,
                "Enabled",
                "BOOLEAN",
                json!(s.control),
                "Presets.Custom.Control.Enabled",
            ),
            leaf_q(
                &t,
                "KillAura",
                "BOOLEAN",
                json!(s.kill_aura),
                "Presets.Custom.Control.KillAura",
            ),
            leaf_q(
                &t,
                "Speed",
                "BOOLEAN",
                json!(s.speed),
                "Presets.Custom.Control.Speed",
            ),
        ],
    );
    let custom = t.group(
        "Custom",
        "CONFIGURABLE",
        vec![
            triggers,
            actions,
            control,
            leaf_q(
                &t,
                "WaitUntilWorldChange",
                "BOOLEAN",
                json!(s.wait_world),
                "Presets.Custom.WaitUntilWorldChange",
            ),
        ],
    );
    values.push(modes(
        &t,
        "Presets",
        PRESETS[s.preset],
        vec![h, g, custom],
        "Presets",
    ));
    values.push(t.multi("Pause", s.pause, &PAUSES, &key("Pause")));
    Ok(t.group("AutoQueue", "TOGGLEABLE", values))
}
fn collect(patch: &mut SettingsPatch, node: &Value, path: &str) {
    if node.get("active").is_some() || node.get("choices").is_some() {
        if let Some(v) = node.get("active") {
            patch
                .changes
                .push(SettingChange::AutoQueue(AutoQueueSetting {
                    path: format!("{path}.active"),
                    value: v.clone(),
                }));
        } else {
            patch
                .issues
                .push(ConfigError::new(path, "missing active mode"));
        }
        if let Some(choices) = node.get("choices").and_then(Value::as_object) {
            for (name, node) in choices {
                collect(patch, node, &format!("{path}.{name}"));
            }
        } else {
            patch
                .issues
                .push(ConfigError::new(path, "expected choices object"));
        }
        return;
    }
    if let Some(value) = node
        .get("value")
        .and_then(Value::as_array)
        .filter(|a| a.first().is_some_and(|v| v.get("name").is_some()))
    {
        for child in value {
            if let Some(name) = child["name"].as_str() {
                collect(patch, child, &format!("{path}.{name}"));
            } else {
                patch
                    .issues
                    .push(ConfigError::new(path, "expected named config child"));
            }
        }
    } else if let Some(v) = node.get("value") {
        patch
            .changes
            .push(SettingChange::AutoQueue(AutoQueueSetting {
                path: path.into(),
                value: v.clone(),
            }));
    } else {
        patch.issues.push(ConfigError::new(path, "missing value"));
    }
}
pub(super) fn parse_auto_queue(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["Presets", "Pause"] {
        if let Some(v) = root.get(name) {
            collect(patch, v, name);
        }
    }
}
