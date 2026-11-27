// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    elytra_swap::ElytraSwap,
    offhand::{Settings, SwitchMode},
};
#[derive(Clone, Debug, PartialEq)]
pub enum OffhandSetting {
    Constraints(ElytraSwapSetting),
    Value(String, Value),
}
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e.path.replace("ElytraSwap", "Offhand");
    e
}
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.offhand.{}",
        path.split('.')
            .map(|part| {
                let mut c = part.chars();
                c.next()
                    .map_or_else(String::new, |p| p.to_lowercase().chain(c).collect())
            })
            .collect::<Vec<_>>()
            .join(".")
    )
}
impl OffhandSetting {
    pub fn apply(self, s: &mut Settings) -> Result<(), ConfigError> {
        match self {
            Self::Constraints(v) => {
                let mut p = ElytraSwap {
                    constraints: s.constraints.clone(),
                    ..Default::default()
                };
                v.apply(&mut p).map_err(remap)?;
                s.constraints = p.constraints;
            }
            Self::Value(path, v) => {
                let p = format!("Offhand.{path}");
                match path.as_str() {
                    "SwitchMode" => {
                        s.switch_mode = match scalar_string(&v).as_deref() {
                            Some(v) if v.eq_ignore_ascii_case("Smart") => SwitchMode::Smart,
                            Some(v) if v.eq_ignore_ascii_case("Switch") => SwitchMode::Switch,
                            Some(v) if v.eq_ignore_ascii_case("PickUp") => SwitchMode::PickUp,
                            Some(v) if v.eq_ignore_ascii_case("Automatic") => SwitchMode::Automatic,
                            _ => return Err(ConfigError::new(&p, "unknown switch mode")),
                        };
                    }
                    "SwitchDelay" => {
                        s.delay = parse_int_pair(&v, &p)?;
                    }
                    "Cycle" => {
                        let value = scalar_string(&v)
                            .ok_or_else(|| ConfigError::new(&p, "expected key name"))?;
                        if crate::input::configured_bind(&value).is_none()
                            && value != "key.keyboard.unknown"
                            && !value
                                .strip_prefix("key.keyboard.")
                                .is_some_and(|v| v.parse::<i32>().is_ok())
                            && !value
                                .strip_prefix("key.mouse.")
                                .is_some_and(|v| v.parse::<i32>().is_ok())
                        {
                            return Err(ConfigError::new(&p, "unknown InputConstants key"));
                        }
                        s.cycle = value;
                    }
                    "Totem.Enabled" => {
                        s.totem = parse_bool(&v, &p)?;
                    }
                    "Totem.SwitchDelay" => {
                        s.totem_delay = parse_int_pair(&v, &p)?;
                    }
                    "Totem.SwitchBackDelay" => {
                        s.back_delay = parse_int(&v, &p)?;
                    }
                    "Totem.Health.Enabled" => {
                        s.health = parse_bool(&v, &p)?;
                    }
                    "Totem.Health.HealthThreshold" => {
                        s.threshold = parse_int(&v, &p)?;
                    }
                    "Totem.Health.Safety.Enabled" => {
                        s.safety = parse_bool(&v, &p)?;
                    }
                    "Totem.Health.Safety.SafeHealthThreshold" => {
                        s.safe_threshold = parse_int(&v, &p)?;
                    }
                    "Totem.Health.SubtractCalculatedDamage" => {
                        s.subtract = parse_bool(&v, &p)?;
                    }
                    "Totem.Health.PredictExplosionDamageEntities" => {
                        s.entities = parse_bool(&v, &p)?;
                    }
                    "Totem.Health.PredictExplosionDamageBlocks" => {
                        let b = parse_bool(&v, &p)?;
                        if s.blocks != b {
                            s.sphere_initialized = true;
                        }
                        s.blocks = b;
                    }
                    "Totem.Health.MissingArmor" => {
                        s.missing_armor = parse_bool(&v, &p)?;
                    }
                    "Totem.Health.PredictFallDamage.Enabled" => {
                        s.fall = parse_bool(&v, &p)?;
                    }
                    "Totem.Health.PredictFallDamage.IgnoreElytra" => {
                        s.ignore_elytra = parse_bool(&v, &p)?;
                    }
                    "Totem.Health.SwitchBack" => {
                        s.switch_back = parse_bool(&v, &p)?;
                    }
                    "Totem.MaceDive.Enabled" => {
                        s.mace = parse_bool(&v, &p)?;
                    }
                    "Totem.MaceDive.HorizontalRange" => {
                        s.mace_range = parse_float(&v, &p)?;
                    }
                    "Totem.MaceDive.MinFallSpeed" => {
                        s.mace_speed = parse_float(&v, &p)?;
                    }
                    "Totem.Urgency.Enabled" => {
                        s.urgency = parse_bool(&v, &p)?;
                    }
                    "Totem.Urgency.MinFactor" => {
                        s.min_factor = parse_float(&v, &p)?;
                    }
                    "Totem.Urgency.Range" => {
                        s.urgency_range = parse_float(&v, &p)?;
                    }
                    "Totem.SendDirectly" => {
                        s.direct = parse_bool(&v, &p)?;
                    }
                    "Crystal.Enabled" => {
                        s.crystal = parse_bool(&v, &p)?;
                    }
                    "Crystal.OnlyWhileCrystalAura" => {
                        s.crystal_only_ca = parse_bool(&v, &p)?;
                    }
                    "Crystal.WhenNoTotems" => {
                        s.crystal_no_totems = parse_bool(&v, &p)?;
                    }
                    "Crystal.CrystalBind" => {
                        let value = scalar_string(&v)
                            .ok_or_else(|| ConfigError::new(&p, "expected key name"))?;
                        if crate::input::configured_bind(&value).is_none()
                            && value != "key.keyboard.unknown"
                            && !value
                                .strip_prefix("key.keyboard.")
                                .is_some_and(|v| v.parse::<i32>().is_ok())
                            && !value
                                .strip_prefix("key.mouse.")
                                .is_some_and(|v| v.parse::<i32>().is_ok())
                        {
                            return Err(ConfigError::new(&p, "unknown InputConstants key"));
                        }
                        s.crystal_bind = value;
                    }
                    "Gapple.Enabled" => {
                        s.gapple = parse_bool(&v, &p)?;
                    }
                    "Gapple.GappleBind" => {
                        let value = scalar_string(&v)
                            .ok_or_else(|| ConfigError::new(&p, "expected key name"))?;
                        if crate::input::configured_bind(&value).is_none()
                            && value != "key.keyboard.unknown"
                            && !value
                                .strip_prefix("key.keyboard.")
                                .is_some_and(|v| v.parse::<i32>().is_ok())
                            && !value
                                .strip_prefix("key.mouse.")
                                .is_some_and(|v| v.parse::<i32>().is_ok())
                        {
                            return Err(ConfigError::new(&p, "unknown InputConstants key"));
                        }
                        s.gapple_bind = value;
                    }
                    "Gapple.WhileHoldingSword.Enabled" => {
                        s.gapple_sword = parse_bool(&v, &p)?;
                    }
                    "Gapple.WhileHoldingSword.OnlyWhileKillAura" => {
                        s.gapple_only_ka = parse_bool(&v, &p)?;
                    }
                    "StrengthPotion.Enabled" => {
                        s.strength = parse_bool(&v, &p)?;
                    }
                    "StrengthPotion.OnlyWhileHoldingSword" => {
                        s.strength_sword = parse_bool(&v, &p)?;
                    }
                    "StrengthPotion.OnlyWhileKillAura" => {
                        s.strength_only_ka = parse_bool(&v, &p)?;
                    }
                    "StrengthPotion.StrengthBind" => {
                        let value = scalar_string(&v)
                            .ok_or_else(|| ConfigError::new(&p, "expected key name"))?;
                        if crate::input::configured_bind(&value).is_none()
                            && value != "key.keyboard.unknown"
                            && !value
                                .strip_prefix("key.keyboard.")
                                .is_some_and(|v| v.parse::<i32>().is_ok())
                            && !value
                                .strip_prefix("key.mouse.")
                                .is_some_and(|v| v.parse::<i32>().is_ok())
                        {
                            return Err(ConfigError::new(&p, "unknown InputConstants key"));
                        }
                        s.strength_bind = value;
                    }
                    "Block.Enabled" => {
                        s.block = parse_bool(&v, &p)?;
                    }
                    "Block.WhileScaffold" => {
                        s.block_scaffold = parse_bool(&v, &p)?;
                    }
                    "Block.WhileEagle" => {
                        s.block_eagle = parse_bool(&v, &p)?;
                    }
                    _ => return Err(ConfigError::new(&p, "unknown Offhand setting")),
                }
            }
        }
        Ok(())
    }
}
fn leaf_value(s: &Settings, path: &str) -> Value {
    match path {
        "SwitchMode" => json!(s.switch_mode.name()),
        "SwitchDelay" => json!({"from":s.delay[0],"to":s.delay[1]}),
        "Cycle" => json!(s.cycle),
        "Totem.Enabled" => json!(s.totem),
        "Totem.SwitchDelay" => json!({"from":s.totem_delay[0],"to":s.totem_delay[1]}),
        "Totem.SwitchBackDelay" => json!(s.back_delay),
        "Totem.Health.Enabled" => json!(s.health),
        "Totem.Health.HealthThreshold" => json!(s.threshold),
        "Totem.Health.Safety.Enabled" => json!(s.safety),
        "Totem.Health.Safety.SafeHealthThreshold" => json!(s.safe_threshold),
        "Totem.Health.SubtractCalculatedDamage" => json!(s.subtract),
        "Totem.Health.PredictExplosionDamageEntities" => json!(s.entities),
        "Totem.Health.PredictExplosionDamageBlocks" => json!(s.blocks),
        "Totem.Health.MissingArmor" => json!(s.missing_armor),
        "Totem.Health.PredictFallDamage.Enabled" => json!(s.fall),
        "Totem.Health.PredictFallDamage.IgnoreElytra" => json!(s.ignore_elytra),
        "Totem.Health.SwitchBack" => json!(s.switch_back),
        "Totem.MaceDive.Enabled" => json!(s.mace),
        "Totem.MaceDive.HorizontalRange" => json!(s.mace_range),
        "Totem.MaceDive.MinFallSpeed" => json!(s.mace_speed),
        "Totem.Urgency.Enabled" => json!(s.urgency),
        "Totem.Urgency.MinFactor" => json!(s.min_factor),
        "Totem.Urgency.Range" => json!(s.urgency_range),
        "Totem.SendDirectly" => json!(s.direct),
        "Crystal.Enabled" => json!(s.crystal),
        "Crystal.OnlyWhileCrystalAura" => json!(s.crystal_only_ca),
        "Crystal.WhenNoTotems" => json!(s.crystal_no_totems),
        "Crystal.CrystalBind" => json!(s.crystal_bind),
        "Gapple.Enabled" => json!(s.gapple),
        "Gapple.GappleBind" => json!(s.gapple_bind),
        "Gapple.WhileHoldingSword.Enabled" => json!(s.gapple_sword),
        "Gapple.WhileHoldingSword.OnlyWhileKillAura" => json!(s.gapple_only_ka),
        "StrengthPotion.Enabled" => json!(s.strength),
        "StrengthPotion.OnlyWhileHoldingSword" => json!(s.strength_sword),
        "StrengthPotion.OnlyWhileKillAura" => json!(s.strength_only_ka),
        "StrengthPotion.StrengthBind" => json!(s.strength_bind),
        "Block.Enabled" => json!(s.block),
        "Block.WhileScaffold" => json!(s.block_scaffold),
        "Block.WhileEagle" => json!(s.block_eagle),
        _ => Value::Null,
    }
}
const LEAVES: &[(&str, &str, f64, f64, &str)] = &[
    ("SwitchMode", "CHOOSE", 0.0, 0.0, ""),
    ("SwitchDelay", "INT_RANGE", 0.0, 500.0, "ms"),
    ("Cycle", "KEY", 0.0, 0.0, ""),
    ("Totem.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    ("Totem.SwitchDelay", "INT_RANGE", 0.0, 500.0, "ms"),
    ("Totem.SwitchBackDelay", "INT", 0.0, 500.0, "ms"),
    ("Totem.Health.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    ("Totem.Health.HealthThreshold", "INT", 0.0, 20.0, ""),
    ("Totem.Health.Safety.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    (
        "Totem.Health.Safety.SafeHealthThreshold",
        "INT",
        0.0,
        20.0,
        "",
    ),
    (
        "Totem.Health.SubtractCalculatedDamage",
        "BOOLEAN",
        0.0,
        0.0,
        "",
    ),
    (
        "Totem.Health.PredictExplosionDamageEntities",
        "BOOLEAN",
        0.0,
        0.0,
        "",
    ),
    (
        "Totem.Health.PredictExplosionDamageBlocks",
        "BOOLEAN",
        0.0,
        0.0,
        "",
    ),
    ("Totem.Health.MissingArmor", "BOOLEAN", 0.0, 0.0, ""),
    (
        "Totem.Health.PredictFallDamage.Enabled",
        "BOOLEAN",
        0.0,
        0.0,
        "",
    ),
    (
        "Totem.Health.PredictFallDamage.IgnoreElytra",
        "BOOLEAN",
        0.0,
        0.0,
        "",
    ),
    ("Totem.Health.SwitchBack", "BOOLEAN", 0.0, 0.0, ""),
    ("Totem.MaceDive.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    ("Totem.MaceDive.HorizontalRange", "FLOAT", 1.0, 10.0, ""),
    ("Totem.MaceDive.MinFallSpeed", "FLOAT", 0.1, 2.0, "b/t"),
    ("Totem.Urgency.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    ("Totem.Urgency.MinFactor", "FLOAT", 0.0, 1.0, ""),
    ("Totem.Urgency.Range", "FLOAT", 1.0, 16.0, ""),
    ("Totem.SendDirectly", "BOOLEAN", 0.0, 0.0, ""),
    ("Crystal.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    ("Crystal.OnlyWhileCrystalAura", "BOOLEAN", 0.0, 0.0, ""),
    ("Crystal.WhenNoTotems", "BOOLEAN", 0.0, 0.0, ""),
    ("Crystal.CrystalBind", "KEY", 0.0, 0.0, ""),
    ("Gapple.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    ("Gapple.GappleBind", "KEY", 0.0, 0.0, ""),
    ("Gapple.WhileHoldingSword.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    (
        "Gapple.WhileHoldingSword.OnlyWhileKillAura",
        "BOOLEAN",
        0.0,
        0.0,
        "",
    ),
    ("StrengthPotion.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    (
        "StrengthPotion.OnlyWhileHoldingSword",
        "BOOLEAN",
        0.0,
        0.0,
        "",
    ),
    ("StrengthPotion.OnlyWhileKillAura", "BOOLEAN", 0.0, 0.0, ""),
    ("StrengthPotion.StrengthBind", "KEY", 0.0, 0.0, ""),
    ("Block.Enabled", "BOOLEAN", 0.0, 0.0, ""),
    ("Block.WhileScaffold", "BOOLEAN", 0.0, 0.0, ""),
    ("Block.WhileEagle", "BOOLEAN", 0.0, 0.0, ""),
];
fn node(t: &Tree<'_>, s: &Settings, path: &str) -> Value {
    let name = path.rsplit('.').next().unwrap_or(path);
    if let Some((_, kind, min, max, unit)) = LEAVES.iter().find(|v| v.0 == path) {
        let value = leaf_value(s, path);
        let mut v = if matches!(*kind, "INT" | "FLOAT" | "INT_RANGE") {
            t.ranged(
                name,
                kind,
                value,
                json!({"from":min,"to":max}),
                unit,
                &key(path),
            )
        } else {
            t.leaf(name, kind, value, &key(path), None)
        };
        if *kind == "CHOOSE" {
            v["choices"] = json!(["Smart", "Switch", "PickUp", "Automatic"]);
        }
        return v;
    }
    let prefix = format!("{path}.");
    let mut names = Vec::new();
    for (p, ..) in LEAVES {
        if let Some(tail) = p.strip_prefix(&prefix) {
            let child = tail.split('.').next().unwrap();
            if !names.contains(&child) {
                names.push(child);
            }
        }
    }
    t.group(
        name,
        "TOGGLEABLE",
        names
            .into_iter()
            .map(|name| node(t, s, &format!("{path}.{name}")))
            .collect(),
    )
}
fn rename(v: &mut Value) {
    match v {
        Value::String(s) => *s = s.replace("elytraSwap", "offhand"),
        Value::Array(v) => v.iter_mut().for_each(rename),
        Value::Object(v) => v.values_mut().for_each(rename),
        _ => {}
    }
}
pub fn serialize_offhand(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let p = ElytraSwap {
        constraints: s.constraints.clone(),
        ..Default::default()
    };
    let mut constraints =
        serialize_elytra_swap(&p, c, f, &|k| d(&k.replace("elytraSwap", "offhand")))?["value"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == "Constraints")
            .unwrap()
            .clone();
    rename(&mut constraints);
    let mut values = t.common(ModuleKind::Offhand, c);
    values.push(constraints);
    for name in [
        "SwitchMode",
        "SwitchDelay",
        "Cycle",
        "Totem",
        "Crystal",
        "Gapple",
        "StrengthPotion",
        "Block",
    ] {
        values.push(node(&t, s, name));
    }
    Ok(t.group("Offhand", "TOGGLEABLE", values))
}
fn collect(p: &mut SettingsPatch, n: &Value, path: &str) {
    if LEAVES.iter().any(|v| v.0 == path) {
        if let Some(v) = n.get("value") {
            p.changes.push(SettingChange::Offhand(OffhandSetting::Value(
                path.into(),
                v.clone(),
            )));
        } else {
            p.issues
                .push(ConfigError::new(&format!("Offhand.{path}"), "missing value"));
        }
        return;
    }
    if let Some(a) = n["value"].as_array() {
        for v in a {
            if let Some(name) = v["name"].as_str() {
                collect(p, v, &format!("{path}.{name}"));
            }
        }
    }
}
pub(super) fn parse_offhand(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let mut proxy = SettingsPatch {
        module: ModuleKind::ElytraSwap,
        changes: vec![],
        issues: vec![],
    };
    elytra_swap::parse_elytra_swap(&mut proxy, root);
    for c in proxy.changes {
        if let SettingChange::ElytraSwap(c) = c {
            p.changes
                .push(SettingChange::Offhand(OffhandSetting::Constraints(c)));
        }
    }
    p.issues.extend(proxy.issues.into_iter().map(remap));
    for name in [
        "SwitchMode",
        "SwitchDelay",
        "Cycle",
        "Totem",
        "Crystal",
        "Gapple",
        "StrengthPotion",
        "Block",
    ] {
        if let Some(v) = root.get(name) {
            collect(p, v, name);
        }
    }
}
