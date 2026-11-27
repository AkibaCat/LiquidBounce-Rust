// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::client_commands::Settings;
pub fn serialize_client_commands(
    s: &Settings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Value {
    let t = Tree { format, describe };
    t.group(
        "CommandManager",
        "CONFIGURABLE",
        vec![
            t.leaf(
                "Prefix",
                "TEXT",
                json!(s.prefix),
                "liquidbounce.commandManager.prefix",
                None,
            ),
            t.ranged(
                "HintCount",
                "INT",
                json!(s.hint_count),
                json!({"from":0,"to":10}),
                "",
                "liquidbounce.commandManager.hintCount",
            ),
        ],
    )
}
pub fn apply_client_commands(s: &mut Settings, root: &Value) -> Vec<ConfigError> {
    let mut issues = Vec::new();
    let Some(group) = root["value"].as_array().and_then(|a| {
        a.iter()
            .find(|n| n["name"] == "CommandManager" || n["name"] == "Commands")
    }) else {
        return issues;
    };
    let n = match nodes(group, group["name"].as_str().unwrap(), "CommandManager") {
        Ok(n) => n,
        Err(e) => {
            issues.push(e);
            return issues;
        }
    };
    if let Some(n) = n.get("Prefix") {
        match value_of(n, "CommandManager.Prefix").and_then(|v| {
            scalar_string(v)
                .ok_or_else(|| ConfigError::new("CommandManager.Prefix", "expected text"))
        }) {
            Ok(v) => s.prefix = v,
            Err(e) => issues.push(e),
        }
    }
    if let Some(n) = n.get("HintCount") {
        match value_of(n, "CommandManager.HintCount")
            .and_then(|v| parse_int(v, "CommandManager.HintCount"))
        {
            Ok(v) => s.hint_count = v,
            Err(e) => issues.push(e),
        }
    }
    issues
}
