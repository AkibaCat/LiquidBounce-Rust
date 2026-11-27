// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::macros::{ACTIONS, COUNT, MATCHES, Pick, SWINGS, State};
#[derive(Clone, Debug, PartialEq)]
pub struct MacrosSetting {
    pub path: String,
    pub value: Value,
}
fn choice(v: &Value, p: &str, tags: &[&str]) -> Result<usize, ConfigError> {
    scalar_string(v)
        .and_then(|v| tags.iter().position(|t| t.eq_ignore_ascii_case(&v)))
        .ok_or_else(|| ConfigError::new(p, "unknown choice"))
}
fn texts(v: &Value, p: &str) -> Result<Vec<String>, ConfigError> {
    match v {
        Value::Array(a) => a
            .iter()
            .map(|v| scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected text")))
            .collect(),
        _ => scalar_string(v)
            .map(|v| vec![v])
            .ok_or_else(|| ConfigError::new(p, "expected text list")),
    }
}
fn group_index(p: &str) -> Option<(usize, &str)> {
    let (g, p) = p.split_once('.')?;
    let (kind, num) = g.split_once('-')?;
    let i = num.parse::<usize>().ok()?.checked_sub(1)?;
    if i >= COUNT {
        return None;
    }
    Some((
        match kind {
            "Chat" => i,
            "Item" => COUNT + i,
            _ => return None,
        },
        p,
    ))
}
fn apply_pick(pick: &mut Pick, p: &str, v: &Value, items: &[String]) -> Result<(), ConfigError> {
    match p {
        "active" => pick.mode = choice(v, p, &["Name", "Item"])?,
        "Name.Names" => {
            let mut a = texts(v, p)?;
            a.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            a.dedup();
            pick.names = a;
        }
        "Name.Mode" => pick.match_mode = choice(v, p, &MATCHES)?,
        "Item.Items" => {
            let mut a = texts(v, p)?
                .into_iter()
                .map(|s| {
                    let s = if s.contains(':') {
                        s
                    } else {
                        format!("minecraft:{s}")
                    };
                    if items.contains(&s) {
                        s
                    } else {
                        "minecraft:air".into()
                    }
                })
                .collect::<Vec<_>>();
            a.sort_by(|a, b| {
                let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
                let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
                ap.cmp(bp).then(an.cmp(bn))
            });
            a.dedup();
            pick.items = a;
        }
        _ => return Err(ConfigError::new(p, "unknown item pick setting")),
    }
    Ok(())
}
impl MacrosSetting {
    pub fn apply(self, s: &mut State, items: &[String]) -> Result<(), ConfigError> {
        let (i, p) = group_index(&self.path)
            .ok_or_else(|| ConfigError::new(&self.path, "unknown macro group"))?;
        let v = &self.value;
        match p {
            "Trigger" => {
                let value = scalar_string(v)
                    .ok_or_else(|| ConfigError::new(p, "expected input key name"))?;
                let valid = crate::input::configured_bind(&value).is_some()
                    || value == "key.keyboard.unknown"
                    || value
                        .strip_prefix("key.keyboard.")
                        .is_some_and(|v| v.parse::<i32>().is_ok())
                    || value
                        .strip_prefix("key.mouse.")
                        .is_some_and(|v| v.parse::<i32>().is_ok());
                if !valid {
                    return Err(ConfigError::new(p, "unknown InputConstants key"));
                }
                s.trigger_mut(i).key = value;
            }
            "TriggerDelay" => {
                let range = parse_int_pair(v, p)?;
                let trigger = s.trigger_mut(i);
                if trigger.delay != range {
                    trigger.delay = range;
                    if range[0] > range[1] {
                        return Err(ConfigError::new(p, "empty trigger delay range"));
                    }
                    trigger.current =
                        liquidbounce_behaviors::random::random_i32_inclusive(range[0], range[1]);
                }
            }
            "Delay" if i < COUNT => s.chats[i].delay = parse_int_pair(v, p)?,
            "Messages" if i < COUNT => s.import_messages(i, texts(v, p)?),
            "UseClientCommand" if i < COUNT => s.chats[i].client_command = parse_bool(v, p)?,
            "Action" if i >= COUNT => s.items[i - COUNT].action = choice(v, p, &ACTIONS)?,
            "SwingMode" if i >= COUNT => s.items[i - COUNT].swing = choice(v, p, &SWINGS)?,
            "HoldTime" if i >= COUNT => s.items[i - COUNT].hold = parse_int_pair(v, p)?,
            _ if i >= COUNT && p.starts_with("PickMode.") => {
                apply_pick(&mut s.items[i - COUNT].pick, &p[9..], v, items)?
            }
            _ => return Err(ConfigError::new(p, "unknown macro setting")),
        }
        Ok(())
    }
}
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.macros.{}",
        path.split('.')
            .map(|p| {
                let mut c = p.chars();
                c.next()
                    .map_or_else(String::new, |f| f.to_lowercase().chain(c).collect())
            })
            .collect::<Vec<String>>()
            .join(".")
    )
}
fn leaf_m(t: &Tree, name: &str, kind: &str, v: Value, path: &str) -> Value {
    t.leaf(name, kind, v, &key(path), None)
}
fn range(
    t: &Tree,
    name: &str,
    pair: [i32; 2],
    bounds: [i32; 2],
    suffix: &str,
    path: &str,
) -> Value {
    t.ranged(
        name,
        "INT_RANGE",
        json!({"from":pair[0],"to":pair[1]}),
        json!({"from":bounds[0],"to":bounds[1]}),
        suffix,
        &key(path),
    )
}
fn select(t: &Tree, name: &str, i: usize, tags: &[&str], path: &str) -> Value {
    let mut v = leaf_m(t, name, "CHOOSE", json!(tags[i]), path);
    if t.format == JsonFormat::Interop {
        v["choices"] = json!(tags);
    }
    v
}
fn list(t: &Tree, name: &str, a: &[String], path: &str) -> Value {
    let mut v = leaf_m(t, name, "MUTABLE_LIST", json!(a), path);
    if t.format == JsonFormat::Interop {
        v["innerValueType"] = json!("TEXT");
    }
    v
}
fn pick(t: &Tree, s: &Pick, path: &str) -> Value {
    let name = t.group(
        "Name",
        "CONFIGURABLE",
        vec![
            list(t, "Names", &s.names, &format!("{path}.Name.Names")),
            select(
                t,
                "Mode",
                s.match_mode,
                &MATCHES,
                &format!("{path}.Name.Mode"),
            ),
        ],
    );
    let mut items = leaf_m(
        t,
        "Items",
        "REGISTRY_LIST",
        json!(s.items),
        &format!("{path}.Item.Items"),
    );
    if t.format == JsonFormat::Interop {
        items["registry"] = json!("item");
    }
    let mut choices = Map::new();
    choices.insert("Name".into(), name);
    choices.insert("Item".into(), t.group("Item", "CONFIGURABLE", vec![items]));
    let mut v = leaf_m(t, "PickMode", "CHOICE", json!([]), path);
    v["active"] = json!(["Name", "Item"][s.mode]);
    v["choices"] = Value::Object(choices);
    v
}
pub fn serialize_macros(
    s: &State,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::Macros, c);
    for i in 0..COUNT * 2 {
        let name = format!(
            "{}-{}",
            if i < COUNT { "Chat" } else { "Item" },
            i % COUNT + 1
        );
        let tr = s.trigger(i);
        let mut v = vec![
            range(
                &t,
                "TriggerDelay",
                tr.delay,
                [0, 15000],
                "ms",
                &format!("{name}.TriggerDelay"),
            ),
            leaf_m(
                &t,
                "Trigger",
                "KEY",
                json!(tr.key),
                &format!("{name}.Trigger"),
            ),
        ];
        if i < COUNT {
            let s = &s.chats[i];
            v.extend([
                range(
                    &t,
                    "Delay",
                    s.delay,
                    [0, 5000],
                    "ms",
                    &format!("{name}.Delay"),
                ),
                list(&t, "Messages", &s.messages, &format!("{name}.Messages")),
                leaf_m(
                    &t,
                    "UseClientCommand",
                    "BOOLEAN",
                    json!(s.client_command),
                    &format!("{name}.UseClientCommand"),
                ),
            ]);
        } else {
            let s = &s.items[i - COUNT];
            v.extend([
                select(&t, "Action", s.action, &ACTIONS, &format!("{name}.Action")),
                pick(&t, &s.pick, &format!("{name}.PickMode")),
                select(
                    &t,
                    "SwingMode",
                    s.swing,
                    &SWINGS,
                    &format!("{name}.SwingMode"),
                ),
                range(
                    &t,
                    "HoldTime",
                    s.hold,
                    [1, 20],
                    "ticks",
                    &format!("{name}.HoldTime"),
                ),
            ]);
        }
        values.push(t.group(&name, "CONFIGURABLE", v));
    }
    Ok(t.group("Macros", "TOGGLEABLE", values))
}
fn collect(patch: &mut SettingsPatch, n: &Value, path: &str) {
    if n.get("active").is_some() {
        patch.changes.push(SettingChange::Macros(MacrosSetting {
            path: format!("{path}.active"),
            value: n["active"].clone(),
        }));
        if let Some(c) = n["choices"].as_object() {
            for (name, v) in c {
                collect(patch, v, &format!("{path}.{name}"));
            }
        }
        return;
    }
    if let Some(a) = n["value"].as_array().filter(|a| {
        n["valueType"] == "CONFIGURABLE" || a.first().is_some_and(|v| v["name"].is_string())
    }) {
        for v in a {
            if let Some(name) = v["name"].as_str() {
                collect(patch, v, &format!("{path}.{name}"));
            }
        }
    } else if let Some(v) = n.get("value") {
        patch.changes.push(SettingChange::Macros(MacrosSetting {
            path: path.into(),
            value: v.clone(),
        }));
    } else {
        patch.issues.push(ConfigError::new(path, "missing value"));
    }
}
pub(super) fn parse_macros(patch: &mut SettingsPatch, root: &Nodes) {
    for kind in ["Chat", "Item"] {
        for i in 1..=COUNT {
            let name = format!("{kind}-{i}");
            if let Some(v) = root.get(&name) {
                collect(patch, v, &name);
            }
        }
    }
}
