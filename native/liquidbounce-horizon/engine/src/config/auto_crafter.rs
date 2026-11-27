// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::auto_crafter::{CONTAINERS, OnFull, Settings};
#[derive(Clone, Debug, PartialEq)]
pub struct AutoCrafterSetting(pub String, pub Value);
impl AutoCrafterSetting {
    pub fn apply(&self, s: &mut Settings, items: &[String]) -> Result<(), ConfigError> {
        let p = format!("AutoCrafter.{}", self.0);
        let v = &self.1;
        match self.0.as_str() {
            "ItemsToCraft" => {
                let parse = |v: &Value| -> Result<String, ConfigError> {
                    let raw = scalar_string(v)
                        .ok_or_else(|| ConfigError::new(&p, "expected item identifier"))?;
                    let id = if raw.starts_with(':') {
                        format!("minecraft{raw}")
                    } else if !raw.contains(':') {
                        format!("minecraft:{raw}")
                    } else {
                        raw
                    };
                    Ok(if items.contains(&id) {
                        id
                    } else {
                        "minecraft:air".into()
                    })
                };
                s.items = match v {
                    Value::Array(a) => a.iter().map(parse).collect::<Result<_, _>>()?,
                    _ => vec![parse(v)?],
                };
            }
            "Delay" => {
                let range = parse_int_pair(v, &p)?;
                if range[0] > range[1] {
                    return Err(ConfigError::new(&p, "empty integer range"));
                }
                s.delay = range;
            }
            "StackCrafting" => s.stack = parse_bool(v, &p)?,
            "SequentialCrafting" => s.sequential = parse_bool(v, &p)?,
            "OnFull" => {
                s.on_full = scalar_string(v)
                    .and_then(|s| {
                        OnFull::ALL
                            .into_iter()
                            .find(|v| v.tag().eq_ignore_ascii_case(&s))
                    })
                    .ok_or_else(|| ConfigError::new(&p, "unknown OnFull choice"))?
            }
            "AllowedContainers" => {
                s.allowed = parse_multi_mask(v, &p, &CONTAINERS)?;
            }
            _ => return Err(ConfigError::new(&p, "unknown AutoCrafter setting")),
        }
        Ok(())
    }
}
fn key(name: &str) -> String {
    format!(
        "liquidbounce.module.autoCrafter.{}{}",
        name[..1].to_ascii_lowercase(),
        &name[1..]
    )
}
pub fn serialize_auto_crafter(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::AutoCrafter, c);
    let mut list = t.leaf(
        "ItemsToCraft",
        "REGISTRY_MUTABLE_LIST",
        json!(s.items),
        &key("ItemsToCraft"),
        None,
    );
    if f == JsonFormat::Interop {
        list["registry"] = json!("item");
        list["innerValueType"] = json!("ITEM");
    }
    values.push(list);
    values.push(t.ranged(
        "Delay",
        "INT_RANGE",
        json!({"from":s.delay[0],"to":s.delay[1]}),
        json!({"from":1,"to":20}),
        "ticks",
        &key("Delay"),
    ));
    for (name, value) in [
        ("StackCrafting", s.stack),
        ("SequentialCrafting", s.sequential),
    ] {
        values.push(t.leaf(name, "BOOLEAN", json!(value), &key(name), None));
    }
    let mut on_full = t.leaf(
        "OnFull",
        "CHOOSE",
        json!(s.on_full.tag()),
        &key("OnFull"),
        None,
    );
    if f == JsonFormat::Interop {
        on_full["choices"] = json!(OnFull::ALL.map(OnFull::tag));
    }
    values.push(on_full);
    values.push(t.multi(
        "AllowedContainers",
        s.allowed,
        &CONTAINERS,
        &key("AllowedContainers"),
    ));
    Ok(t.group("AutoCrafter", "TOGGLEABLE", values))
}
pub(super) fn parse_auto_crafter(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in [
        "ItemsToCraft",
        "Delay",
        "StackCrafting",
        "SequentialCrafting",
        "OnFull",
        "AllowedContainers",
    ] {
        leaf(
            p,
            root,
            name,
            &format!("AutoCrafter.{name}"),
            |value, path| {
                let setting = AutoCrafterSetting(name.into(), value.clone());
                if name == "ItemsToCraft" {
                    let values = match value {
                        Value::Array(a) => a.clone(),
                        _ => vec![value.clone()],
                    };
                    for value in values {
                        scalar_string(&value)
                            .ok_or_else(|| ConfigError::new(path, "expected item identifier"))?;
                    }
                } else {
                    setting.apply(&mut Settings::default(), &[])?;
                }
                Ok(SettingChange::AutoCrafter(setting))
            },
        );
    }
}
