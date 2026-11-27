// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    chest_cleaner::{ChestCleaner, Filter, MENUS, TITLES},
    elytra_swap::ElytraSwap,
};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq)]
pub enum ChestCleanerSetting {
    Constraints(ElytraSwapSetting),
    Value(&'static str, Value),
}
fn list(v: &Value, path: &str) -> Result<Vec<String>, ConfigError> {
    let parse = |v: &Value| {
        scalar_string(v).ok_or_else(|| ConfigError::new(path, "expected string list entry"))
    };
    match v {
        Value::Array(a) => a.iter().map(parse).collect(),
        _ => Ok(vec![parse(v)?]),
    }
}
fn filter(v: &Value, path: &str) -> Result<Filter, ConfigError> {
    match scalar_string(v).as_deref() {
        Some(s) if s.eq_ignore_ascii_case("Whitelist") => Ok(Filter::Whitelist),
        Some(s) if s.eq_ignore_ascii_case("Blacklist") => Ok(Filter::Blacklist),
        _ => Err(ConfigError::new(path, "unknown filter")),
    }
}
fn registry(
    v: &Value,
    available: &[String],
    default: Option<&str>,
    path: &str,
) -> Result<BTreeSet<String>, ConfigError> {
    let resolve = |v: &Value| -> Result<Option<String>, ConfigError> {
        let raw = match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => {
                return Err(ConfigError::new(
                    path,
                    "registry entry requires string or number",
                ));
            }
        };
        let id = match raw.find(':') {
            None => format!("minecraft:{raw}"),
            Some(0) => format!("minecraft{raw}"),
            _ => raw,
        };
        Ok(if available.contains(&id) {
            Some(id)
        } else {
            default.map(str::to_owned)
        })
    };
    let entries = match v {
        Value::Array(a) => a.iter().map(resolve).collect::<Result<Vec<_>, _>>()?,
        _ => vec![resolve(v)?],
    };
    Ok(entries.into_iter().flatten().collect())
}
impl ChestCleanerSetting {
    pub fn apply(self, s: &mut ChestCleaner, items: &[String]) -> Result<(), ConfigError> {
        let Self::Value(path, v) = self else {
            if let Self::Constraints(c) = self {
                let mut proxy = ElytraSwap {
                    constraints: s.constraints.clone(),
                    ..Default::default()
                };
                c.apply(&mut proxy).map_err(|mut e| {
                    e.path = e.path.replace("ElytraSwap", "ChestCleaner");
                    e
                })?;
                s.constraints = proxy.constraints;
            }
            return Ok(());
        };
        let full = format!("ChestCleaner.{path}");
        match path {
            "Filter" => s.filter = filter(&v, &full)?,
            "Items" => s.items = registry(&v, items, Some("minecraft:air"), &full)?,
            "AutoClose" => s.auto_close = parse_bool(&v, &full)?,
            "CheckScreenHandlerType.Enabled" => s.check_type = parse_bool(&v, &full)?,
            "CheckScreenHandlerType.Types" => {
                let menus = MENUS.map(|m| format!("minecraft:{m}"));
                s.types = registry(&v, &menus, None, &full)?;
            }
            "CheckScreenHandlerType.Filter" => s.type_filter = filter(&v, &full)?,
            "CheckScreenTitle.Enabled" => s.check_title = parse_bool(&v, &full)?,
            "CheckScreenTitle.Titles" => {
                s.titles = 0;
                for tag in list(&v, &full)? {
                    if let Some(i) = TITLES.iter().position(|t| t.eq_ignore_ascii_case(&tag)) {
                        s.titles |= 1 << i;
                    }
                }
            }
            "CheckScreenTitle.Custom" => s.custom = list(&v, &full)?.into_iter().collect(),
            "CheckScreenTitle.Filter" => s.title_filter = filter(&v, &full)?,
            _ => return Err(ConfigError::new(&full, "unknown setting")),
        }
        Ok(())
    }
}
fn ordered(values: &BTreeSet<String>) -> Value {
    let mut v: Vec<_> = values.iter().collect();
    v.sort_by(|a, b| {
        let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
        let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
        ap.cmp(bp).then_with(|| an.cmp(bn))
    });
    json!(v)
}
fn rename(v: &mut Value) {
    match v {
        Value::String(s) => *s = s.replace("elytraSwap", "chestCleaner"),
        Value::Array(a) => a.iter_mut().for_each(rename),
        Value::Object(o) => o.values_mut().for_each(rename),
        _ => {}
    }
}
pub fn serialize_chest_cleaner(
    s: &ChestCleaner,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let key = |p: &str| format!("liquidbounce.module.chestCleaner.{p}");
    let filter_leaf = |name: &str, path: &str, f: Filter| {
        let mut v = t.leaf(name, "CHOOSE", json!(f.tag()), &key(path), None);
        if format == JsonFormat::Interop {
            v["choices"] = json!(["Whitelist", "Blacklist"]);
        }
        v
    };
    let registry_leaf = |name: &str, path: &str, s: &BTreeSet<String>, r: &str| {
        let mut v = t.leaf(name, "REGISTRY_LIST", ordered(s), &key(path), None);
        if format == JsonFormat::Interop {
            v["registry"] = json!(r);
        }
        v
    };
    let mut values = t.common(ModuleKind::ChestCleaner, c);
    values.push(filter_leaf("Filter", "filter", s.filter));
    values.push(registry_leaf("Items", "items", &s.items, "item"));
    values.push(t.leaf(
        "AutoClose",
        "BOOLEAN",
        json!(s.auto_close),
        &key("autoClose"),
        None,
    ));
    let proxy = ElytraSwap {
        constraints: s.constraints.clone(),
        ..Default::default()
    };
    let mut constraints = serialize_elytra_swap(&proxy, c, format, &|k| {
        describe(&k.replace("elytraSwap", "chestCleaner"))
    })?["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Constraints")
        .unwrap()
        .clone();
    rename(&mut constraints);
    values.push(constraints);
    values.push(t.group(
        "CheckScreenHandlerType",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.check_type),
                &key("checkScreenHandlerType"),
                None,
            ),
            registry_leaf("Types", "checkScreenHandlerType.types", &s.types, "menu"),
            filter_leaf("Filter", "checkScreenHandlerType.filter", s.type_filter),
        ],
    ));
    let mut custom = t.leaf(
        "Custom",
        "MUTABLE_LIST",
        json!(s.custom),
        &key("checkScreenTitle.custom"),
        None,
    );
    if format == JsonFormat::Interop {
        custom["innerValueType"] = json!("TEXT");
    }
    values.push(t.group(
        "CheckScreenTitle",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.check_title),
                &key("checkScreenTitle"),
                None,
            ),
            t.multi("Titles", s.titles, &TITLES, &key("checkScreenTitle.titles")),
            custom,
            filter_leaf("Filter", "checkScreenTitle.filter", s.title_filter),
        ],
    ));
    Ok(t.group("ChestCleaner", "TOGGLEABLE", values))
}
pub(super) fn parse_chest_cleaner(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["Filter", "Items", "AutoClose"] {
        leaf(p, root, name, &format!("ChestCleaner.{name}"), |v, _| {
            Ok(SettingChange::ChestCleaner(ChestCleanerSetting::Value(
                name,
                v.clone(),
            )))
        });
    }
    let mut proxy = SettingsPatch {
        module: ModuleKind::ElytraSwap,
        changes: vec![],
        issues: vec![],
    };
    super::elytra_swap::parse_elytra_swap(&mut proxy, root);
    for c in proxy.changes {
        if let SettingChange::ElytraSwap(c) = c {
            p.changes.push(SettingChange::ChestCleaner(
                ChestCleanerSetting::Constraints(c),
            ));
        }
    }
    for mut e in proxy.issues {
        e.path = e.path.replace("ElytraSwap", "ChestCleaner");
        p.issues.push(e);
    }
    for (group, fields) in [
        (
            "CheckScreenHandlerType",
            &[
                ("Enabled", "CheckScreenHandlerType.Enabled"),
                ("Types", "CheckScreenHandlerType.Types"),
                ("Filter", "CheckScreenHandlerType.Filter"),
            ][..],
        ),
        (
            "CheckScreenTitle",
            &[
                ("Enabled", "CheckScreenTitle.Enabled"),
                ("Titles", "CheckScreenTitle.Titles"),
                ("Custom", "CheckScreenTitle.Custom"),
                ("Filter", "CheckScreenTitle.Filter"),
            ][..],
        ),
    ] {
        let Some(v) = root.get(group) else {
            continue;
        };
        match nodes(v, group, &format!("ChestCleaner.{group}")) {
            Ok(children) => {
                for &(name, path) in fields {
                    leaf(
                        p,
                        &children,
                        name,
                        &format!("ChestCleaner.{path}"),
                        |v, _| {
                            Ok(SettingChange::ChestCleaner(ChestCleanerSetting::Value(
                                path,
                                v.clone(),
                            )))
                        },
                    );
                }
            }
            Err(e) => p.issues.push(e),
        }
    }
}
