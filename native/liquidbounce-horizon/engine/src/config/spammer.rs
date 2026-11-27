// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::spammer::{CONVERTERS, PATTERNS, SOURCES, Spammer};
#[derive(Clone, Debug, PartialEq)]
pub enum SpammerSetting {
    Delay([f32; 2]),
    Mps([i32; 2]),
    Source(u8),
    Messages(Vec<String>),
    File(String),
    Pattern(u8),
    Converter(u8),
    CustomFormatter(bool),
}
impl SpammerSetting {
    pub fn apply(self, s: &mut Spammer) {
        match self {
            Self::Delay(v) => s.delay = v,
            Self::Mps(v) => s.mps = v,
            Self::Source(v) => s.source = v,
            Self::Messages(v) => s.messages = v,
            Self::File(v) => s.file = v,
            Self::Pattern(v) => s.pattern = v,
            Self::Converter(v) => s.converter = v,
            Self::CustomFormatter(v) => s.custom_formatter = v,
        }
    }
}
pub fn serialize_spammer(
    s: &Spammer,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut v = t.common(ModuleKind::Spammer, common);
    v.push(t.ranged(
        "Delay",
        "FLOAT_RANGE",
        float_range(s.delay[0], s.delay[1], "Spammer.Delay")?,
        json!({"from":0.0,"to":300.0}),
        "secs",
        "liquidbounce.module.spammer.delay",
    ));
    v.push(t.ranged(
        "MPS",
        "INT_RANGE",
        json!({"from":s.mps[0],"to":s.mps[1]}),
        json!({"from":1,"to":500}),
        "messages",
        "liquidbounce.module.spammer.mps",
    ));
    let mut choices = Map::new();
    let mut messages = t.leaf(
        "Message",
        "MUTABLE_LIST",
        json!(s.messages),
        "liquidbounce.module.spammer.messageSource.setting.message",
        None,
    );
    if format == JsonFormat::Interop {
        messages["innerValueType"] = json!("TEXT");
    }
    choices.insert(
        "Setting".into(),
        t.group("Setting", "CONFIGURABLE", vec![messages]),
    );
    let mut source = t.leaf(
        "Source",
        "FILE",
        json!(s.file),
        "liquidbounce.module.spammer.messageSource.file.source",
        None,
    );
    if format == JsonFormat::Interop {
        source["dialogMode"] = json!("OPEN_FILE");
        source["supportedExtensions"] = json!(["txt"]);
    }
    choices.insert("File".into(), t.group("File", "CONFIGURABLE", vec![source]));
    let mut provider = t.group("MessageSource", "CHOICE", vec![]);
    provider["active"] = json!(SOURCES[usize::from(s.source).min(1)]);
    provider["choices"] = Value::Object(choices);
    v.push(provider);
    for (name, tag, choices, key) in [
        (
            "Pattern",
            PATTERNS[usize::from(s.pattern).min(1)],
            PATTERNS.as_slice(),
            "liquidbounce.module.spammer.pattern",
        ),
        (
            "MessageConverter",
            CONVERTERS[usize::from(s.converter).min(3)],
            CONVERTERS.as_slice(),
            "liquidbounce.module.spammer.messageConverter",
        ),
    ] {
        let mut value = t.leaf(name, "CHOOSE", json!(tag), key, None);
        if format == JsonFormat::Interop {
            value["choices"] = json!(choices);
        }
        v.push(value);
    }
    v.push(t.leaf(
        "CustomFormatter",
        "BOOLEAN",
        json!(s.custom_formatter),
        "liquidbounce.module.spammer.customFormatter",
        None,
    ));
    Ok(t.group("Spammer", "TOGGLEABLE", v))
}
fn choice(v: &Value, p: &str, tags: &[&str]) -> Result<u8, ConfigError> {
    scalar_string(v)
        .and_then(|s| tags.iter().position(|tag| tag.eq_ignore_ascii_case(&s)))
        .map(|i| i as u8)
        .ok_or_else(|| ConfigError::new(p, "unknown choice"))
}
pub(super) fn parse_spammer(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Delay", "Spammer.Delay", |v, p| {
        parse_float_pair(v, p).map(|v| SettingChange::Spammer(SpammerSetting::Delay(v)))
    });
    leaf(patch, root, "MPS", "Spammer.MPS", |v, p| {
        parse_int_pair(v, p).map(|v| SettingChange::Spammer(SpammerSetting::Mps(v)))
    });
    leaf(patch, root, "Pattern", "Spammer.Pattern", |v, p| {
        choice(v, p, &PATTERNS).map(|v| SettingChange::Spammer(SpammerSetting::Pattern(v)))
    });
    leaf(
        patch,
        root,
        "MessageConverter",
        "Spammer.MessageConverter",
        |v, p| {
            choice(v, p, &CONVERTERS).map(|v| SettingChange::Spammer(SpammerSetting::Converter(v)))
        },
    );
    leaf(
        patch,
        root,
        "CustomFormatter",
        "Spammer.CustomFormatter",
        |v, p| parse_bool(v, p).map(|v| SettingChange::Spammer(SpammerSetting::CustomFormatter(v))),
    );
    let Some(source) = root.get("MessageSource") else {
        return;
    };
    match source
        .get("active")
        .ok_or_else(|| ConfigError::new("Spammer.MessageSource.active", "missing choice"))
        .and_then(|v| choice(v, "Spammer.MessageSource.active", &SOURCES))
    {
        Ok(v) => patch
            .changes
            .push(SettingChange::Spammer(SpammerSetting::Source(v))),
        Err(e) => patch.issues.push(e),
    }
    if let Err(e) = nodes(source, "MessageSource", "Spammer.MessageSource") {
        patch.issues.push(e);
    }
    let Some(choices) = source.get("choices").and_then(Value::as_object) else {
        patch.issues.push(ConfigError::new(
            "Spammer.MessageSource.choices",
            "expected choices",
        ));
        return;
    };
    for tag in SOURCES {
        let path = format!("Spammer.MessageSource.{tag}");
        let Some(value) = choices.get(tag) else {
            patch
                .issues
                .push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        let children = match nodes(value, tag, &path) {
            Ok(v) => v,
            Err(e) => {
                patch.issues.push(e);
                continue;
            }
        };
        if tag == "Setting" {
            leaf(
                patch,
                &children,
                "Message",
                &format!("{path}.Message"),
                |v, p| {
                    let values = match v {
                        Value::Array(a) => a.iter().filter_map(scalar_string).collect(),
                        Value::String(s) => vec![s.clone()],
                        _ => return Err(ConfigError::new(p, "expected text list")),
                    };
                    Ok(SettingChange::Spammer(SpammerSetting::Messages(values)))
                },
            );
        } else {
            leaf(
                patch,
                &children,
                "Source",
                &format!("{path}.Source"),
                |v, p| {
                    scalar_string(v)
                        .map(|v| SettingChange::Spammer(SpammerSetting::File(v)))
                        .ok_or_else(|| ConfigError::new(p, "expected file path"))
                },
            );
        }
    }
}
