// SPDX-License-Identifier: GPL-3.0-or-later
//! Generic ModuleSpeed config adapter.  The source schema is generated from
//! LiquidBounce's Kotlin values and retained as raw JSON so every mode tree
//! (including inactive choices) round-trips through the native config bridge.
use super::*;
use liquidbounce_behaviors::speed::{self, Settings};
use serde_json::{Map, Value, json};

#[derive(Clone, Debug, PartialEq)]
pub struct SpeedSetting {
    pub path: String,
    pub value: Value,
}
impl SpeedSetting {
    pub fn apply(self, settings: &mut Settings) -> Result<(), String> {
        if settings.values.contains_key(&self.path) {
            settings.values.insert(self.path, self.value);
            Ok(())
        } else {
            Err(format!("unknown Speed setting {}", self.path))
        }
    }
}

fn key(path: &str) -> String {
    let mut out = String::from("liquidbounce.module.speed");
    for part in path.split('.') {
        let mut chars = part.chars();
        let name = match chars.next() {
            Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        };
        out.push('.');
        out.push_str(&name);
    }
    out
}

fn value(settings: &Settings, path: &str, default: &Value) -> Value {
    settings.values.get(path).cloned().unwrap_or_else(|| default.clone())
}

fn render_node(
    node: &Value,
    prefix: &str,
    tree: &Tree<'_>,
    settings: &Settings,
) -> Value {
    let name = node["name"].as_str().unwrap_or("");
    let path = if prefix.is_empty() { name.to_owned() } else { format!("{prefix}.{name}") };
    let kind = node["kind"].as_str().unwrap_or("CONFIGURABLE");
    if kind == "CHOICE" {
        let active = settings.tag(&path);
        let mut choices = Map::new();
        for mode in node["modes"].as_array().into_iter().flatten() {
            let mode_name = mode["name"].as_str().unwrap_or("");
            let mode_path = format!("{path}.{mode_name}");
            let children = mode["children"].as_array().map(|children| {
                children.iter().map(|child| render_node(child, &mode_path, tree, settings)).collect()
            }).unwrap_or_default();
            choices.insert(mode_name.to_owned(), tree.group(mode_name, "CONFIGURABLE", children));
        }
        return tree.mode(active, choices);
    }
    if kind == "MULTI_CHOOSE" {
        let selected = value(settings, &path, node.get("default").unwrap_or(&Value::Null));
        let options = node["options"].as_array().map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>()).unwrap_or_default();
        let mut leaf = tree.leaf(name, kind, selected, &key(&path), None);
        if tree.format == JsonFormat::Interop {
            leaf["choices"] = json!(options);
            leaf["canBeNone"] = json!(node["canBeNone"].as_bool().unwrap_or(true));
            leaf["isOrderSensitive"] = json!(false);
        }
        return leaf;
    }
    if node.get("children").is_some() {
        let children = node["children"].as_array().into_iter().flatten()
            .map(|child| render_node(child, &path, tree, settings)).collect();
        return tree.group(name, kind, children);
    }
    let default = node.get("default").unwrap_or(&Value::Null);
    let current = value(settings, &path, default);
    let mut leaf = tree.leaf(name, kind, current, &key(&path), None);
    if tree.format == JsonFormat::Interop {
        if let Some(range) = node.get("range") { leaf["range"] = range.clone(); }
        if let Some(options) = node.get("options") { leaf["choices"] = options.clone(); }
    }
    leaf
}

pub fn serialize_speed(
    state: &Settings,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::Speed, common);
    for node in speed::SCHEMA.iter() {
        values.push(render_node(node, "", &tree, state));
    }
    Ok(tree.group("Speed", "TOGGLEABLE", values))
}

fn parse_node(patch: &mut SettingsPatch, node: &Value, prefix: &str, root: &Nodes<'_>) {
    let name = node["name"].as_str().unwrap_or("");
    let path = if prefix.is_empty() { name.to_owned() } else { format!("{prefix}.{name}") };
    let Some(raw) = root.get(name) else { return };
    let kind = node["kind"].as_str().unwrap_or("CONFIGURABLE");
    if kind == "CHOICE" {
        let Some(active) = raw.get("active").and_then(|v| v.as_str()) else {
            patch.issues.push(ConfigError::new(&format!("{path}.active"), "expected active choice"));
            return;
        };
        patch.changes.push(SettingChange::Speed(SpeedSetting { path: path.clone(), value: json!(active) }));
        let Some(choices) = raw.get("choices").and_then(Value::as_object) else { return };
        if let Some(chosen) = choices.get(active) {
            if let Ok(group) = nodes(chosen, active, &format!("{path}.{active}")) {
                if let Some(mode) = node["modes"].as_array().and_then(|m| m.iter().find(|m| m["name"] == active)) {
                    for child in mode["children"].as_array().into_iter().flatten() {
                        parse_leaf_or_group(patch, child, &format!("{path}.{active}"), &group);
                    }
                }
            }
        }
        return;
    }
    parse_leaf_or_group(patch, node, prefix, root);
}

fn parse_leaf_or_group(patch: &mut SettingsPatch, node: &Value, prefix: &str, root: &Nodes<'_>) {
    let name = node["name"].as_str().unwrap_or("");
    let path = if prefix.is_empty() { name.to_owned() } else { format!("{prefix}.{name}") };
    let kind = node["kind"].as_str().unwrap_or("CONFIGURABLE");
    let Some(raw) = root.get(name) else { return };
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        if let Ok(group) = nodes(raw, name, &path) {
            for child in children { parse_leaf_or_group(patch, child, &path, &group); }
        }
        return;
    }
    let Some(value) = raw.get("value") else {
        patch.issues.push(ConfigError::new(&path, "missing value"));
        return;
    };
    if kind == "CHOICE" || kind == "ROTATIONS" || kind == "MULTI_CHOOSE" || value.is_boolean() || value.is_number() || value.is_string() || value.is_array() {
        patch.changes.push(SettingChange::Speed(SpeedSetting { path, value: value.clone() }));
    }
}

pub(super) fn parse_speed(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for node in speed::SCHEMA.iter() {
        parse_node(patch, node, "", root);
    }
}
