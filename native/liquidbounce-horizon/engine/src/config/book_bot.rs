// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{book_bot::BookBot, elytra_swap::ElytraSwap};
fn parse_string(v: &Value, p: &str) -> Result<String, ConfigError> {
    scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected text"))
}
#[derive(Clone, Debug, PartialEq)]
pub enum BookBotSetting {
    Constraints(ElytraSwapSetting),
    Mode(u8),
    Pages(usize, i32),
    Ascii(bool),
    Space(bool),
    Cyclic(bool),
    Source(String),
    Sign(bool),
    Name(String),
    Delay(f32),
}
impl BookBotSetting {
    pub fn apply(self, s: &mut BookBot) -> Result<(), ConfigError> {
        match self {
            Self::Constraints(c) => {
                let mut swap = ElytraSwap {
                    constraints: s.constraints.clone(),
                    ..Default::default()
                };
                let result = c.apply(&mut swap);
                s.constraints = swap.constraints;
                result.map_err(|mut e| {
                    e.path = e.path.replace("ElytraSwap", "BookBot");
                    e
                })?;
            }
            Self::Mode(v) => s.mode = v,
            Self::Pages(i, v) => s.pages[i] = v,
            Self::Ascii(v) => s.ascii_only = v,
            Self::Space(v) => s.allow_space = v,
            Self::Cyclic(v) => s.cyclic = v,
            Self::Source(v) => s.source = v,
            Self::Sign(v) => s.sign = v,
            Self::Name(v) => s.name = v,
            Self::Delay(v) => s.delay = v,
        }
        Ok(())
    }
}
fn rename_keys(v: &mut Value) {
    match v {
        Value::String(s) => *s = s.replace("elytraSwap", "bookBot"),
        Value::Array(a) => a.iter_mut().for_each(rename_keys),
        Value::Object(o) => o.values_mut().for_each(rename_keys),
        _ => {}
    }
}
pub fn serialize_book_bot(
    s: &BookBot,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::BookBot, c);
    let proxy = ElytraSwap {
        constraints: s.constraints.clone(),
        ..Default::default()
    };
    let mut constraints = serialize_elytra_swap(&proxy, c, format, &|key| {
        describe(&key.replace("elytraSwap", "bookBot"))
    })?["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Constraints")
        .unwrap()
        .clone();
    rename_keys(&mut constraints);
    values.push(constraints);
    let mut choices = Map::new();
    for (i, tag) in ["Random", "File"].into_iter().enumerate() {
        let key = format!("liquidbounce.module.bookBot.mode.{}", tag.to_lowercase());
        let mut v = vec![t.ranged(
            "Pages",
            "INT",
            json!(s.pages[i]),
            json!({"from":0,"to":100}),
            "",
            &format!("{key}.pages"),
        )];
        if i == 0 {
            v.push(t.leaf(
                "AsciiOnly",
                "BOOLEAN",
                json!(s.ascii_only),
                &format!("{key}.asciiOnly"),
                None,
            ));
            v.push(t.leaf(
                "AllowSpace",
                "BOOLEAN",
                json!(s.allow_space),
                &format!("{key}.allowSpace"),
                None,
            ));
        } else {
            v.push(t.leaf(
                "Cyclic",
                "BOOLEAN",
                json!(s.cyclic),
                &format!("{key}.cyclic"),
                None,
            ));
            let mut file = t.leaf(
                "Source",
                "FILE",
                json!(s.source),
                &format!("{key}.source"),
                None,
            );
            if format == JsonFormat::Interop {
                file["dialogMode"] = json!("OPEN_FILE");
                file["supportedExtensions"] = json!(["txt"]);
            }
            v.push(file);
        }
        choices.insert(tag.into(), t.group(tag, "CONFIGURABLE", v));
    }
    let mut mode = t.group("Mode", "CHOICE", vec![]);
    mode["active"] = json!(if s.mode == 0 { "Random" } else { "File" });
    mode["choices"] = json!(choices);
    values.push(mode);
    values.push(t.group(
        "Sign",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.sign),
                "liquidbounce.module.bookBot.sign",
                None,
            ),
            t.leaf(
                "Name",
                "TEXT",
                json!(s.name),
                "liquidbounce.module.bookBot.sign.name",
                None,
            ),
        ],
    ));
    values.push(t.ranged(
        "Delay",
        "FLOAT",
        float_json(s.delay, "BookBot.Delay")?,
        json!({"from":0.,"to":20.}),
        "s",
        "liquidbounce.module.bookBot.delay",
    ));
    Ok(t.group("BookBot", "TOGGLEABLE", values))
}
pub(super) fn parse_book_bot(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let mut proxy = SettingsPatch {
        module: ModuleKind::ElytraSwap,
        changes: vec![],
        issues: vec![],
    };
    super::elytra_swap::parse_elytra_swap(&mut proxy, root);
    for c in proxy.changes {
        if let SettingChange::ElytraSwap(v) = c {
            p.changes
                .push(SettingChange::BookBot(BookBotSetting::Constraints(v)));
        }
    }
    for mut e in proxy.issues {
        e.path = e.path.replace("ElytraSwap", "BookBot");
        p.issues.push(e);
    }
    parse_mode(p, root);
    if let Some(sign) = root.get("Sign") {
        match nodes(sign, "Sign", "BookBot.Sign") {
            Ok(s) => {
                leaf(p, &s, "Enabled", "BookBot.Sign.Enabled", |v, p| {
                    parse_bool(v, p).map(|v| SettingChange::BookBot(BookBotSetting::Sign(v)))
                });
                leaf(p, &s, "Name", "BookBot.Sign.Name", |v, p| {
                    parse_string(v, p).map(|v| SettingChange::BookBot(BookBotSetting::Name(v)))
                });
            }
            Err(e) => p.issues.push(e),
        }
    }
    leaf(p, root, "Delay", "BookBot.Delay", |v, p| {
        parse_float(v, p).map(|v| SettingChange::BookBot(BookBotSetting::Delay(v)))
    });
}
fn parse_mode(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Mode") else {
        return;
    };
    match mode.get("active").and_then(scalar_string).as_deref() {
        Some("Random") => p
            .changes
            .push(SettingChange::BookBot(BookBotSetting::Mode(0))),
        Some("File") => p
            .changes
            .push(SettingChange::BookBot(BookBotSetting::Mode(1))),
        _ => p
            .issues
            .push(ConfigError::new("BookBot.Mode.active", "unknown choice")),
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        p.issues
            .push(ConfigError::new("BookBot.Mode.choices", "expected choices"));
        return;
    };
    for (i, tag) in ["Random", "File"].into_iter().enumerate() {
        let path = format!("BookBot.Mode.{tag}");
        let Some(v) = choices.get(tag) else {
            p.issues.push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        let v = match nodes(v, tag, &path) {
            Ok(v) => v,
            Err(e) => {
                p.issues.push(e);
                continue;
            }
        };
        leaf(p, &v, "Pages", &format!("{path}.Pages"), |v, p| {
            parse_int(v, p).map(|v| SettingChange::BookBot(BookBotSetting::Pages(i, v)))
        });
        if i == 0 {
            for (name, make) in [
                (
                    "AsciiOnly",
                    BookBotSetting::Ascii as fn(bool) -> BookBotSetting,
                ),
                ("AllowSpace", BookBotSetting::Space),
            ] {
                leaf(p, &v, name, &format!("{path}.{name}"), |v, p| {
                    parse_bool(v, p).map(|v| SettingChange::BookBot(make(v)))
                });
            }
        } else {
            leaf(p, &v, "Cyclic", &format!("{path}.Cyclic"), |v, p| {
                parse_bool(v, p).map(|v| SettingChange::BookBot(BookBotSetting::Cyclic(v)))
            });
            leaf(p, &v, "Source", &format!("{path}.Source"), |v, p| {
                parse_string(v, p).map(|v| SettingChange::BookBot(BookBotSetting::Source(v)))
            });
        }
    }
}
