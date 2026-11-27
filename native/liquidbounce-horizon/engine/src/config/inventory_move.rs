// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::inventory_move::{BEHAVIORS, SNEAK, SPRINT, Settings};
#[derive(Clone, Debug, PartialEq)]
pub struct InventoryMoveSetting(pub String, pub Value);
impl InventoryMoveSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let key = self.0.as_str();
        let v = &self.1;
        let choice = |tags: &[&str]| -> Result<u8, ConfigError> {
            scalar_string(v)
                .and_then(|v| tags.iter().position(|t| t.eq_ignore_ascii_case(&v)))
                .map(|i| i as u8)
                .ok_or_else(|| ConfigError::new(key, "unknown choice"))
        };
        match key {
            "Behavior" => s.behavior = choice(&BEHAVIORS)?,
            "PassthroughSneak" => s.passthrough_sneak = parse_bool(v, key)?,
            "SprintControl.Enabled" => s.sprint_control = parse_bool(v, key)?,
            "SprintControl.Client" => s.client_sprint = choice(&SPRINT)?,
            "SprintControl.Server" => s.server_sprint = choice(&SPRINT)?,
            "SneakControl.Enabled" => s.sneak_control = parse_bool(v, key)?,
            "SneakControl.Client" => s.client_sneak = choice(&SNEAK)?,
            "Timer.Enabled" => s.timer = parse_bool(v, key)?,
            "Timer.Speed" => s.speed = parse_float(v, key)?.clamp(0.1, 2.),
            "Blink.Enabled" => s.blink = parse_bool(v, key)?,
            "Blink.MaximumTime" => s.maximum_time = parse_int(v, key)?.clamp(0, 30000),
            _ => return Err(ConfigError::new(key, "unknown InventoryMove setting")),
        };
        Ok(())
    }
}
pub fn serialize_inventory_move(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::InventoryMove, c);
    let choice = |name: &str, key: &str, tags: &[&str], index: u8| {
        let mut v = t.leaf(name, "CHOOSE", json!(tags[index as usize]), key, None);
        if f == JsonFormat::Interop {
            v["choices"] = json!(tags);
        }
        v
    };
    let k = |p: &str| format!("liquidbounce.module.inventoryMove.{p}");
    values.push(choice("Behavior", &k("behavior"), &BEHAVIORS, s.behavior));
    values.push(t.leaf(
        "PassthroughSneak",
        "BOOLEAN",
        json!(s.passthrough_sneak),
        &k("passthroughSneak"),
        None,
    ));
    values.push(t.group(
        "SprintControl",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.sprint_control),
                &k("sprintControl.enabled"),
                None,
            ),
            choice(
                "Client",
                &k("sprintControl.client"),
                &SPRINT,
                s.client_sprint,
            ),
            choice(
                "Server",
                &k("sprintControl.server"),
                &SPRINT,
                s.server_sprint,
            ),
        ],
    ));
    values.push(t.group(
        "SneakControl",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.sneak_control),
                &k("sneakControl.enabled"),
                None,
            ),
            choice("Client", &k("sneakControl.client"), &SNEAK, s.client_sneak),
        ],
    ));
    values.push(t.group(
        "Timer",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.timer),
                &k("timer.enabled"),
                None,
            ),
            t.ranged(
                "Speed",
                "FLOAT",
                json!(s.speed),
                json!({"from":0.1,"to":2.}),
                "",
                &k("timer.speed"),
            ),
        ],
    ));
    values.push(t.group(
        "Blink",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.blink),
                &k("blink.enabled"),
                None,
            ),
            t.ranged(
                "MaximumTime",
                "INT",
                json!(s.maximum_time),
                json!({"from":0,"to":30000}),
                "ms",
                &k("blink.maximumTime"),
            ),
        ],
    ));
    Ok(t.group("InventoryMove", "TOGGLEABLE", values))
}
pub(super) fn parse_inventory_move(p: &mut SettingsPatch, root: &Nodes<'_>) {
    fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
        leaf(p, n, name, &format!("InventoryMove.{path}"), |v, _| {
            let s = InventoryMoveSetting(path.into(), v.clone());
            s.apply(&mut Settings::default())?;
            Ok(SettingChange::InventoryMove(s))
        });
    }
    for name in ["Behavior", "PassthroughSneak"] {
        add(p, root, name, name);
    }
    for (group, names) in [
        ("SprintControl", &["Enabled", "Client", "Server"][..]),
        ("SneakControl", &["Enabled", "Client"][..]),
        ("Timer", &["Enabled", "Speed"][..]),
        ("Blink", &["Enabled", "MaximumTime"][..]),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("InventoryMove.{group}")) {
                Ok(n) => {
                    for name in names {
                        add(p, &n, name, &format!("{group}.{name}"));
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
}
