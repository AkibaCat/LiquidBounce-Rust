// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::fast_exp_runtime::FastExpRuntime;
use liquidbounce_behaviors::no_rotate_set::NoRotateSet;
#[derive(Clone, Debug, PartialEq)]
pub enum FastExpSetting {
    Value(String, Value),
    Rotation(NoRotateSetSetting),
}
impl FastExpSetting {
    pub fn apply(self, s: &mut FastExpRuntime) -> Result<(), ConfigError> {
        if let Self::Rotation(v) = self {
            let mut proxy = NoRotateSet {
                rotations: std::mem::take(&mut s.rotations),
                ..Default::default()
            };
            v.apply(&mut proxy);
            s.rotations = proxy.rotations;
            return Ok(());
        }
        let Self::Value(path, value) = self else {
            unreachable!()
        };
        let p = path.as_str();
        let v = &value;
        let s = &mut s.behavior.settings;
        match p {
            "Rotate.Enabled" => s.rotate = parse_bool(v, p)?,
            "NoWaste.Enabled" => s.no_waste = parse_bool(v, p)?,
            "NoWaste.MinDurabilityToStartRepair" => s.min_durability = parse_int(v, p)?,
            "NoWaste.MaxDurabilityToContinueRepair" => s.max_percent = parse_int(v, p)?,
            "ThrowMode" => {
                s.fast = match scalar_string(v).as_deref() {
                    Some(s) if s.eq_ignore_ascii_case("Fast") => true,
                    Some(s) if s.eq_ignore_ascii_case("Normal") => false,
                    _ => return Err(ConfigError::new(p, "expected Normal or Fast")),
                }
            }
            "Normal.TicksPerItem" => s.ticks_per_item = parse_float_pair(v, p)?,
            "Fast.ItemsPerTick" => s.items_per_tick = parse_float_pair(v, p)?,
            "CombatPauseTime" => s.combat_pause = parse_int(v, p)?,
            "SlotResetDelay" => {
                let r = parse_int_pair(v, p)?;
                if r[0] > r[1] {
                    return Err(ConfigError::new(p, "empty slot reset range"));
                }
                s.slot_reset = r;
            }
            _ => return Err(ConfigError::new(p, "unknown FastExp setting")),
        };
        Ok(())
    }
}
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW: &str = "liquidbounce.module.fastExp.rotate.rotations";
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "FastExp.Rotate.Rotations",
        )
        .replace(OLD, NEW);
    e
}
fn keys(v: &mut Value) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(k)) = o.get_mut("key") {
                *k = k.replace(OLD, NEW);
            }
            for v in o.values_mut() {
                keys(v);
            }
        }
        Value::Array(a) => {
            for v in a {
                keys(v)
            }
        }
        _ => {}
    }
}
pub fn serialize_fast_exp(
    s: &FastExpRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::FastExp, c);
    let key = |p: &str| format!("liquidbounce.module.fastExp.{p}");
    let b = &s.behavior.settings;
    let describe = |k: &str| d(&k.replace(OLD, NEW));
    let proxy = NoRotateSet {
        rotations: s.rotations.clone(),
        ..Default::default()
    };
    let mut tree = serialize_no_rotate_set(&proxy, c, f, &describe).map_err(remap)?;
    let mut rotations = tree["value"]
        .as_array_mut()
        .and_then(|v| v.iter_mut().find(|v| v["name"] == "Mode"))
        .and_then(|v| v["choices"]["ResetRotation"]["value"].as_array_mut())
        .and_then(|v| v.first_mut())
        .map(Value::take)
        .ok_or_else(|| {
            ConfigError::new("FastExp.Rotate.Rotations", "shared rotation tree changed")
        })?;
    keys(&mut rotations);
    values.push(t.group(
        "Rotate",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(b.rotate),
                &key("rotate.enabled"),
                None,
            ),
            rotations,
        ],
    ));
    values.push(t.group(
        "NoWaste",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(b.no_waste),
                &key("noWaste.enabled"),
                None,
            ),
            t.ranged(
                "MinDurabilityToStartRepair",
                "INT",
                json!(b.min_durability),
                json!({"from":0,"to":2048}),
                "",
                &key("noWaste.minDurabilityToStartRepair"),
            ),
            t.ranged(
                "MaxDurabilityToContinueRepair",
                "INT",
                json!(b.max_percent),
                json!({"from":1,"to":100}),
                "%",
                &key("noWaste.maxDurabilityToContinueRepair"),
            ),
        ],
    ));
    let mut choices = Map::new();
    choices.insert(
        "Normal".into(),
        t.group(
            "Normal",
            "CONFIGURABLE",
            vec![t.ranged(
                "TicksPerItem",
                "FLOAT_RANGE",
                json!({"from":b.ticks_per_item[0],"to":b.ticks_per_item[1]}),
                json!({"from":0.5,"to":10.}),
                "ticks",
                &key("throwMode.normal.ticksPerItem"),
            )],
        ),
    );
    choices.insert(
        "Fast".into(),
        t.group(
            "Fast",
            "CONFIGURABLE",
            vec![t.ranged(
                "ItemsPerTick",
                "FLOAT_RANGE",
                json!({"from":b.items_per_tick[0],"to":b.items_per_tick[1]}),
                json!({"from":0.5,"to":16.}),
                "",
                &key("throwMode.fast.itemsPerTick"),
            )],
        ),
    );
    let mut mode = t.mode(if b.fast { "Fast" } else { "Normal" }, choices);
    mode["name"] = json!("ThrowMode");
    values.push(mode);
    values.push(t.ranged(
        "CombatPauseTime",
        "INT",
        json!(b.combat_pause),
        json!({"from":0,"to":40}),
        "ticks",
        &key("combatPauseTime"),
    ));
    values.push(t.ranged(
        "SlotResetDelay",
        "INT_RANGE",
        json!({"from":b.slot_reset[0],"to":b.slot_reset[1]}),
        json!({"from":0,"to":40}),
        "ticks",
        &key("slotResetDelay"),
    ));
    Ok(t.group("FastExp", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
    leaf(p, n, name, &format!("FastExp.{path}"), |v, _| {
        let s = FastExpSetting::Value(path.into(), v.clone());
        s.clone().apply(&mut FastExpRuntime::default())?;
        Ok(SettingChange::FastExp(s))
    });
}
pub(super) fn parse_fast_exp(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["CombatPauseTime", "SlotResetDelay"] {
        add(p, root, name, name);
    }
    for (group, names) in [
        ("Rotate", vec!["Enabled"]),
        (
            "NoWaste",
            vec![
                "Enabled",
                "MinDurabilityToStartRepair",
                "MaxDurabilityToContinueRepair",
            ],
        ),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("FastExp.{group}")) {
                Ok(n) => {
                    for name in names {
                        add(p, &n, name, &format!("{group}.{name}"));
                    }
                    if group == "Rotate" {
                        if let Some(v) = n.get("Rotations") {
                            let proxy = json!({"name":"NoRotateSet","value":[{"name":"Mode","active":"ResetRotation","value":[],"choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[v]}}}]});
                            let root = nodes(&proxy, "NoRotateSet", "NoRotateSet").unwrap();
                            let mut parsed = SettingsPatch {
                                module: ModuleKind::NoRotateSet,
                                changes: vec![],
                                issues: vec![],
                            };
                            no_rotate_set::parse_no_rotate_set(&mut parsed, &root);
                            for c in parsed.changes {
                                if let SettingChange::NoRotateSet(v) = c {
                                    if !matches!(v, NoRotateSetSetting::Mode(_)) {
                                        p.changes.push(SettingChange::FastExp(
                                            FastExpSetting::Rotation(v),
                                        ));
                                    }
                                }
                            }
                            p.issues.extend(parsed.issues.into_iter().map(remap));
                        }
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
    if let Some(v) = root.get("ThrowMode") {
        if let Some(tag) = v["active"].as_str() {
            let s = FastExpSetting::Value("ThrowMode".into(), json!(tag));
            match s.clone().apply(&mut FastExpRuntime::default()) {
                Ok(()) => p.changes.push(SettingChange::FastExp(s)),
                Err(e) => p.issues.push(e),
            }
        }
        for (mode, name) in [("Normal", "TicksPerItem"), ("Fast", "ItemsPerTick")] {
            if let Some(v) = v["choices"].get(mode) {
                match nodes(v, mode, &format!("FastExp.ThrowMode.{mode}")) {
                    Ok(n) => add(p, &n, name, &format!("{mode}.{name}")),
                    Err(e) => p.issues.push(e),
                }
            }
        }
    }
}
