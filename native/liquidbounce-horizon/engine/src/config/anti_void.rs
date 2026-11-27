// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    anti_void::{MODES, State},
    ghost_hand::GhostHand,
};
#[derive(Clone, Debug, PartialEq)]
pub struct AntiVoidSetting(pub String, pub Value);
impl AntiVoidSetting {
    pub fn apply(self, state: &mut State, registry: &[String]) -> Result<(), ConfigError> {
        let p = format!("AntiVoid.{}", self.0);
        let v = &self.1;
        let s = &mut state.settings;
        match self.0.as_str() {
            "Mode" => {
                let tag = scalar_string(v).ok_or_else(|| ConfigError::new(&p, "expected mode"))?;
                s.mode = MODES
                    .iter()
                    .position(|t| *t == tag)
                    .ok_or_else(|| ConfigError::new(&p, "unknown case-sensitive mode"))?;
            }
            "VoidLevel" => s.void_level = parse_int(v, &p)?,
            "Mode.Flag.FallDistance" => s.fall_distance = parse_float(v, &p)?,
            "Mode.Flag.Height" => s.height = parse_float(v, &p)?,
            "Mode.Flag.Silent" => s.silent = parse_bool(v, &p)?,
            "Mode.UseItem.YMotionThreshold" => s.y_motion_threshold = parse_float(v, &p)?,
            "Mode.UseItem.SlotResetDelay" => {
                let range = parse_int_pair(v, &p)?;
                if range[0] > range[1] {
                    return Err(ConfigError::new(&p, "empty integer range"));
                }
                s.slot_reset = range;
            }
            "Mode.UseItem.Items" => {
                let mut proxy = GhostHand {
                    targeted_blocks: s.items.clone(),
                };
                proxy
                    .deserialize_targeted_blocks(v, registry)
                    .map_err(|e| ConfigError::new(&p, e))?;
                s.items = proxy.targeted_blocks;
            }
            _ => return Err(ConfigError::new(&p, "unknown AntiVoid setting")),
        }
        Ok(())
    }
}
pub fn serialize_anti_void(
    s: &State,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = &s.settings;
    let mut values = t.common(ModuleKind::AntiVoid, c);
    let key = |p: &str| format!("liquidbounce.module.antiVoid.{p}");
    let flag = vec![
        t.ranged(
            "FallDistance",
            "FLOAT",
            float_json(s.fall_distance, &key("mode.flag.fallDistance"))?,
            json!({"from":0.,"to":6.}),
            "",
            &key("mode.flag.fallDistance"),
        ),
        t.ranged(
            "Height",
            "FLOAT",
            float_json(s.height, &key("mode.flag.height"))?,
            json!({"from":0.01,"to":10.}),
            "",
            &key("mode.flag.height"),
        ),
        t.leaf(
            "Silent",
            "BOOLEAN",
            json!(s.silent),
            &key("mode.flag.silent"),
            None,
        ),
    ];
    let proxy = GhostHand {
        targeted_blocks: s.items.clone(),
    };
    let mut items = t.leaf(
        "Items",
        "REGISTRY_LIST",
        json!(proxy.sorted_targeted_blocks()),
        &key("mode.useItem.items"),
        None,
    );
    if f == JsonFormat::Interop {
        items["registry"] = json!("item");
    }
    let use_item = vec![
        t.ranged(
            "YMotionThreshold",
            "FLOAT",
            float_json(s.y_motion_threshold, &key("mode.useItem.yMotionThreshold"))?,
            json!({"from":-1.,"to":-0.1}),
            "",
            &key("mode.useItem.yMotionThreshold"),
        ),
        t.ranged(
            "SlotResetDelay",
            "INT_RANGE",
            json!({"from":s.slot_reset[0],"to":s.slot_reset[1]}),
            json!({"from":0,"to":40}),
            "ticks",
            &key("mode.useItem.slotResetDelay"),
        ),
        items,
    ];
    let choices = MODES
        .iter()
        .enumerate()
        .map(|(i, name)| {
            (
                (*name).into(),
                t.group(
                    name,
                    "CONFIGURABLE",
                    match i {
                        1 => flag.clone(),
                        3 => use_item.clone(),
                        _ => vec![],
                    },
                ),
            )
        })
        .collect();
    values.push(t.mode(MODES[s.mode], choices));
    values.push(t.ranged(
        "VoidLevel",
        "INT",
        json!(s.void_level),
        json!({"from":-256,"to":0}),
        "",
        &key("voidLevel"),
    ));
    Ok(t.group("AntiVoid", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
    leaf(p, n, name, &format!("AntiVoid.{path}"), |v, _| {
        Ok(SettingChange::AntiVoid(AntiVoidSetting(
            path.into(),
            v.clone(),
        )))
    });
}
pub(super) fn parse_anti_void(p: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(mode) = root.get("Mode") {
        if let Some(active) = mode.get("active") {
            p.changes.push(SettingChange::AntiVoid(AntiVoidSetting(
                "Mode".into(),
                active.clone(),
            )));
        }
        for (tag, names) in [
            ("GhostBlock", &[][..]),
            ("Flag", &["FallDistance", "Height", "Silent"][..]),
            ("Blink", &[][..]),
            (
                "UseItem",
                &["YMotionThreshold", "SlotResetDelay", "Items"][..],
            ),
        ] {
            if let Some(choice) = mode.get("choices").and_then(|v| v.get(tag)) {
                match nodes(choice, tag, &format!("AntiVoid.Mode.{tag}")) {
                    Ok(n) => {
                        for name in names {
                            add(p, &n, name, &format!("Mode.{tag}.{name}"));
                        }
                    }
                    Err(e) => p.issues.push(e),
                }
            }
        }
    }
    add(p, root, "VoidLevel", "VoidLevel");
}
