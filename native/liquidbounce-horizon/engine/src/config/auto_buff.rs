// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::auto_buff_runtime::AutoBuffRuntime;
use liquidbounce_behaviors::{
    auto_buff::{BowlMode, EFFECT_NAMES, FEATURES, RotationTiming},
    elytra_swap::ElytraSwap,
    no_rotate_set::NoRotateSet,
    vomit::Constraints,
};
#[derive(Clone, Debug, PartialEq)]
pub enum AutoBuffSetting {
    Value(String, Value),
    Rotation(NoRotateSetSetting),
    Constraints {
        stack: bool,
        value: ElytraSwapSetting,
    },
}
impl AutoBuffSetting {
    pub fn apply(self, runtime: &mut AutoBuffRuntime) -> Result<(), ConfigError> {
        match self {
            Self::Rotation(v) => {
                let mut p = NoRotateSet {
                    rotations: runtime.rotations.clone(),
                    ..Default::default()
                };
                v.apply(&mut p);
                runtime.rotations = p.rotations;
            }
            Self::Constraints { stack, value } => {
                let target = if stack {
                    &mut runtime.behavior.settings.stack_constraints
                } else {
                    &mut runtime.behavior.settings.refill_constraints
                };
                let mut p = ElytraSwap {
                    constraints: target.clone(),
                    ..Default::default()
                };
                value
                    .apply(&mut p)
                    .map_err(|e| constraints_error(e, stack))?;
                *target = p.constraints;
            }
            Self::Value(path, v) => {
                let p = format!("AutoBuff.{path}");
                let s = &mut runtime.behavior.settings;
                let parts = path.split('.').collect::<Vec<_>>();
                let boolean = || parse_bool(&v, &p);
                let int = || parse_int(&v, &p);
                let float = || parse_float(&v, &p);
                let range = || {
                    let r = parse_int_pair(&v, &p)?;
                    if r[0] > r[1] {
                        return Err(ConfigError::new(&p, "empty integer range"));
                    }
                    Ok(r)
                };
                match parts.as_slice() {
                    [feature, "Enabled"] if FEATURES.iter().any(|f| f.name() == *feature) => {
                        let i = FEATURES.iter().position(|f| f.name() == *feature).unwrap();
                        s.enabled[i] = boolean()?;
                        if !s.enabled[i]
                            && runtime.behavior.force_use.is_some_and(|f| f.index() == i)
                        {
                            runtime.behavior.force_use = None;
                        }
                    }
                    [feature, "Health"] if matches!(*feature, "Soup" | "Head" | "Gapple") => {
                        s.health[health_index(feature)].health = int()?
                    }
                    [feature, "ConsiderAbsorption"]
                        if matches!(*feature, "Soup" | "Head" | "Gapple") =>
                    {
                        s.health[health_index(feature)].absorption = boolean()?
                    }
                    ["Head", "MaxAbsorption"] => s.max_absorption = float()?,
                    ["Head", "Cooldown"] => s.head_cooldown = float()?,
                    ["Gapple", "Enchanted"] => s.enchanted = boolean()?,
                    [feature, "Potions", effect, leaf] if matches!(*feature, "Pot" | "Drink") => {
                        let Some(i) = EFFECT_NAMES.iter().position(|s| s == effect) else {
                            return Err(ConfigError::new(&p, "unknown effect"));
                        };
                        let config = &mut s.potion[usize::from(*feature == "Drink")];
                        match *leaf {
                            "Enabled" => config.enabled[i] = boolean()?,
                            "Health" if i < 2 => config.health[i] = int()?,
                            _ => return Err(ConfigError::new(&p, "unknown potion option")),
                        }
                    }
                    ["Pot", "TillGroundDistance"] => s.ground_distance = float()?,
                    ["Pot", "DoNotBenefitOthers"] => s.no_benefit_others = boolean()?,
                    ["Pot", "AllowLingering"] => s.lingering = boolean()?,
                    ["Soup", "AfterUse"] => {
                        let mode = match scalar_string(&v).as_deref() {
                            Some("None") => BowlMode::None,
                            Some("Drop") => BowlMode::Drop,
                            Some("Stack") => BowlMode::Stack,
                            _ => {
                                return Err(ConfigError::new(
                                    &p,
                                    "unknown case-sensitive bowl mode",
                                ));
                            }
                        };
                        if s.bowl_mode != mode {
                            runtime.behavior.bowl_slot = None;
                        }
                        s.bowl_mode = mode;
                    }
                    ["Soup", "AfterUse", mode, leaf] if matches!(*mode, "Drop" | "Stack") => {
                        let b = &mut s.bowl[usize::from(*mode == "Stack")];
                        match *leaf {
                            "AssumeEmptyBowl" => b.assume_empty = boolean()?,
                            "Wait" => b.wait = range()?,
                            "IgnoreInventoryFull" if *mode == "Stack" => {
                                s.ignore_inventory_full = boolean()?
                            }
                            _ => return Err(ConfigError::new(&p, "unknown bowl option")),
                        }
                    }
                    ["AutoSwap", "Enabled"] => s.auto_swap = boolean()?,
                    ["AutoSwap", "DelayIn"] => s.delay_in = range()?,
                    ["AutoSwap", "DelayOut"] => s.delay_out = range()?,
                    ["Refill", "Enabled"] => s.refill = boolean()?,
                    ["Rotations", "RotationTiming"] => {
                        s.timing = match scalar_string(&v).as_deref() {
                            Some("Normal") => RotationTiming::Normal,
                            Some("OnTick") => RotationTiming::OnTick,
                            Some("OnUse") => RotationTiming::OnUse,
                            _ => return Err(ConfigError::new(&p, "unknown case-sensitive timing")),
                        }
                    }
                    ["CombatPauseTime"] => s.combat_pause = int()?,
                    ["NotDuringCombat"] => s.not_in_combat = boolean()?,
                    ["IgnoreOpenInventory"] => s.ignore_inventory = boolean()?,
                    _ => return Err(ConfigError::new(&p, "unknown setting")),
                }
            }
        }
        Ok(())
    }
}
fn health_index(name: &str) -> usize {
    match name {
        "Soup" => 0,
        "Head" => 1,
        _ => 2,
    }
}
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const NEW: &str = "liquidbounce.module.autoBuff.rotations";
fn rotation_error(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "AutoBuff.Rotations",
        )
        .replace(OLD, NEW);
    e
}
fn constraints_error(mut e: ConfigError, stack: bool) -> ConfigError {
    e.path = e.path.replace(
        "ElytraSwap.Constraints",
        if stack {
            "AutoBuff.Soup.AfterUse.Stack.Constraints"
        } else {
            "AutoBuff.Refill.Constraints"
        },
    );
    e
}
fn replace_keys(v: &mut Value, old: &str, new: &str) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(k)) = o.get_mut("key") {
                *k = k.replace(old, new);
            }
            o.values_mut().for_each(|v| replace_keys(v, old, new));
        }
        Value::Array(a) => a.iter_mut().for_each(|v| replace_keys(v, old, new)),
        _ => {}
    }
}
fn constraints(
    s: &Constraints,
    stack: bool,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let new = if stack {
        "liquidbounce.module.autoBuff.soup.afterUse.stack.constraints"
    } else {
        "liquidbounce.module.autoBuff.refill.constraints"
    };
    let p = ElytraSwap {
        constraints: s.clone(),
        ..Default::default()
    };
    let tree = serialize_elytra_swap(&p, c, f, &|k| {
        d(&k.replace("liquidbounce.module.elytraSwap.constraints", new))
    })
    .map_err(|e| constraints_error(e, stack))?;
    let mut value = tree["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Constraints")
        .unwrap()
        .clone();
    replace_keys(
        &mut value,
        "liquidbounce.module.elytraSwap.constraints",
        new,
    );
    Ok(value)
}
fn low(s: &str) -> String {
    let mut chars = s.chars();
    format!("{}{}", chars.next().unwrap().to_lowercase(), chars.as_str())
}
pub fn serialize_auto_buff(
    r: &AutoBuffRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = &r.behavior.settings;
    let key = |p: &str| format!("liquidbounce.module.autoBuff.{p}");
    let b = |name: &str, value: bool, path: &str| {
        t.leaf(name, "BOOLEAN", json!(value), &key(path), None)
    };
    let int = |name: &str, value: i32, range: Value, suffix: &str, path: &str| {
        t.ranged(name, "INT", json!(value), range, suffix, &key(path))
    };
    let range = |name: &str, value: [i32; 2], path: &str| {
        t.ranged(
            name,
            "INT_RANGE",
            json!({"from":value[0],"to":value[1]}),
            json!({"from":0,"to":20}),
            "ticks",
            &key(path),
        )
    };
    let mut values = t.common(ModuleKind::AutoBuff, c);
    for feature in FEATURES {
        let name = feature.name();
        let prefix = low(name);
        let mut a = vec![b(
            "Enabled",
            s.enabled[feature.index()],
            &format!("{prefix}.enabled"),
        )];
        if matches!(name, "Soup" | "Head" | "Gapple") {
            let h = &s.health[health_index(name)];
            a.push(int(
                "Health",
                h.health,
                json!({"from":1,"to":100}),
                "%HP",
                &format!("{prefix}.health"),
            ));
            a.push(b(
                "ConsiderAbsorption",
                h.absorption,
                &format!("{prefix}.considerAbsorption"),
            ));
        }
        match name {
            "Soup" => {
                let mut choices = Map::new();
                choices.insert("None".into(), t.group("None", "CONFIGURABLE", vec![]));
                for (i, mode) in ["Drop", "Stack"].into_iter().enumerate() {
                    let p = format!("soup.afterUse.{}", low(mode));
                    let mut v = vec![
                        b(
                            "AssumeEmptyBowl",
                            s.bowl[i].assume_empty,
                            &format!("{p}.assumeEmptyBowl"),
                        ),
                        t.ranged(
                            "Wait",
                            "INT_RANGE",
                            json!({"from":s.bowl[i].wait[0],"to":s.bowl[i].wait[1]}),
                            json!({"from":1,"to":20}),
                            "ticks",
                            &key(&format!("{p}.wait")),
                        ),
                    ];
                    if i == 1 {
                        v.push(b(
                            "IgnoreInventoryFull",
                            s.ignore_inventory_full,
                            &format!("{p}.ignoreInventoryFull"),
                        ));
                        v.push(constraints(&s.stack_constraints, true, c, f, d)?);
                    }
                    choices.insert(mode.into(), t.group(mode, "CONFIGURABLE", v));
                }
                let mut mode = t.mode(
                    match s.bowl_mode {
                        BowlMode::None => "None",
                        BowlMode::Drop => "Drop",
                        BowlMode::Stack => "Stack",
                    },
                    choices,
                );
                mode["name"] = json!("AfterUse");
                a.push(mode);
            }
            "Head" => {
                a.push(t.ranged(
                    "MaxAbsorption",
                    "FLOAT",
                    float_json(s.max_absorption, "AutoBuff.Head.MaxAbsorption")?,
                    json!({"from":0.,"to":8.}),
                    "",
                    &key("head.maxAbsorption"),
                ));
                a.push(t.ranged(
                    "Cooldown",
                    "FLOAT",
                    float_json(s.head_cooldown, "AutoBuff.Head.Cooldown")?,
                    json!({"from":0.,"to":120.}),
                    "s",
                    &key("head.cooldown"),
                ));
            }
            "Gapple" => a.push(b("Enchanted", s.enchanted, "gapple.enchanted")),
            "Pot" | "Drink" => {
                let p = &s.potion[usize::from(name == "Drink")];
                a.push(
                    t.group(
                        "Potions",
                        "CONFIGURABLE",
                        EFFECT_NAMES
                            .iter()
                            .enumerate()
                            .map(|(i, name)| {
                                let path = format!("{prefix}.potions.{}", low(name));
                                let mut v =
                                    vec![b("Enabled", p.enabled[i], &format!("{path}.enabled"))];
                                if i < 2 {
                                    v.push(int(
                                        "Health",
                                        p.health[i],
                                        json!({"from":1,"to":100}),
                                        "%HP",
                                        &format!("{path}.health"),
                                    ));
                                }
                                t.group(name, "TOGGLEABLE", v)
                            })
                            .collect(),
                    ),
                );
                if name == "Pot" {
                    a.push(t.ranged(
                        "TillGroundDistance",
                        "FLOAT",
                        float_json(s.ground_distance, "AutoBuff.Pot.TillGroundDistance")?,
                        json!({"from":1.,"to":5.}),
                        "",
                        &key("pot.tillGroundDistance"),
                    ));
                    a.push(b(
                        "DoNotBenefitOthers",
                        s.no_benefit_others,
                        "pot.doNotBenefitOthers",
                    ));
                    a.push(b("AllowLingering", s.lingering, "pot.allowLingering"));
                }
            }
            _ => {}
        }
        values.push(t.group(name, "TOGGLEABLE", a));
    }
    values.push(t.group(
        "AutoSwap",
        "TOGGLEABLE",
        vec![
            b("Enabled", s.auto_swap, "autoSwap.enabled"),
            range("DelayIn", s.delay_in, "autoSwap.delayIn"),
            range("DelayOut", s.delay_out, "autoSwap.delayOut"),
        ],
    ));
    values.push(t.group(
        "Refill",
        "TOGGLEABLE",
        vec![
            b("Enabled", s.refill, "refill.enabled"),
            constraints(&s.refill_constraints, false, c, f, d)?,
        ],
    ));
    let proxy = NoRotateSet {
        rotations: r.rotations.clone(),
        ..Default::default()
    };
    let tree = serialize_no_rotate_set(&proxy, c, f, &|k| d(&k.replace(OLD, NEW)))
        .map_err(rotation_error)?;
    let mut rotations = tree["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Mode")
        .unwrap()["choices"]["ResetRotation"]["value"][0]
        .clone();
    replace_keys(&mut rotations, OLD, NEW);
    rotations["value"].as_array_mut().unwrap().push(t.leaf(
        "RotationTiming",
        "CHOOSE",
        json!(match s.timing {
            RotationTiming::Normal => "Normal",
            RotationTiming::OnTick => "OnTick",
            RotationTiming::OnUse => "OnUse",
        }),
        &key("rotations.rotationTiming"),
        None,
    ));
    if f == JsonFormat::Interop {
        let v = rotations["value"]
            .as_array_mut()
            .unwrap()
            .last_mut()
            .unwrap();
        v["choices"] = json!(["Normal", "OnTick", "OnUse"]);
    }
    values.push(rotations);
    values.push(int(
        "CombatPauseTime",
        s.combat_pause,
        json!({"from":0,"to":40}),
        "ticks",
        "combatPauseTime",
    ));
    values.push(b("NotDuringCombat", s.not_in_combat, "notDuringCombat"));
    values.push(b(
        "IgnoreOpenInventory",
        s.ignore_inventory,
        "ignoreOpenInventory",
    ));
    Ok(t.group("AutoBuff", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, path: &str, v: &Value) {
    let setting = AutoBuffSetting::Value(path.into(), v.clone());
    match setting.clone().apply(&mut AutoBuffRuntime::default()) {
        Ok(()) => p.changes.push(SettingChange::AutoBuff(setting)),
        Err(e) => p.issues.push(e),
    }
}
fn parse_constraints(p: &mut SettingsPatch, v: &Value, stack: bool) {
    let tree = json!({"name":"ElytraSwap","value":[v]});
    let root = nodes(&tree, "ElytraSwap", "ElytraSwap").unwrap();
    let mut proxy = SettingsPatch {
        module: ModuleKind::ElytraSwap,
        changes: vec![],
        issues: vec![],
    };
    elytra_swap::parse_elytra_swap(&mut proxy, &root);
    for change in proxy.changes {
        if let SettingChange::ElytraSwap(value) = change {
            p.changes
                .push(SettingChange::AutoBuff(AutoBuffSetting::Constraints {
                    stack,
                    value,
                }));
        }
    }
    p.issues.extend(
        proxy
            .issues
            .into_iter()
            .map(|e| constraints_error(e, stack)),
    );
}
fn parse_rotations(p: &mut SettingsPatch, v: &Value) {
    let tree = json!({"name":"NoRotateSet","value":[{"name":"Mode","active":"ResetRotation","value":[],"choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[v]}}}]});
    let root = nodes(&tree, "NoRotateSet", "NoRotateSet").unwrap();
    let mut proxy = SettingsPatch {
        module: ModuleKind::NoRotateSet,
        changes: vec![],
        issues: vec![],
    };
    no_rotate_set::parse_no_rotate_set(&mut proxy, &root);
    for change in proxy.changes {
        if let SettingChange::NoRotateSet(value) = change {
            if !matches!(value, NoRotateSetSetting::Mode(_)) {
                p.changes
                    .push(SettingChange::AutoBuff(AutoBuffSetting::Rotation(value)));
            }
        }
    }
    p.issues
        .extend(proxy.issues.into_iter().map(rotation_error));
    if let Ok(n) = nodes(v, "Rotations", "AutoBuff.Rotations") {
        if let Some(v) = n.get("RotationTiming").and_then(|v| v.get("value")) {
            add(p, "Rotations.RotationTiming", v);
        }
    }
}
fn parse_group(p: &mut SettingsPatch, v: &Value, path: &str) {
    let name = path.rsplit('.').next().unwrap();
    match nodes(v, name, &format!("AutoBuff.{path}")) {
        Ok(n) => parse_values(p, &n, path),
        Err(e) => p.issues.push(e),
    }
}
fn parse_values(p: &mut SettingsPatch, n: &Nodes<'_>, prefix: &str) {
    for (name, v) in n {
        if prefix.is_empty() && matches!(name.as_str(), "Enabled" | "Bind" | "Hidden") {
            continue;
        }
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        if path == "Rotations" {
            parse_rotations(p, v);
            continue;
        }
        if matches!(
            path.as_str(),
            "Refill.Constraints" | "Soup.AfterUse.Stack.Constraints"
        ) {
            parse_constraints(p, v, path.starts_with("Soup"));
            continue;
        }
        if path == "Soup.AfterUse" {
            if let Some(active) = v.get("active") {
                add(p, &path, active);
            } else {
                p.issues.push(ConfigError::new(
                    "AutoBuff.Soup.AfterUse.active",
                    "missing mode",
                ));
            }
            if let Some(choices) = v["choices"].as_object() {
                for mode in ["None", "Drop", "Stack"] {
                    if let Some(group) = choices.get(mode) {
                        parse_group(p, group, &format!("{path}.{mode}"));
                    } else {
                        p.issues.push(ConfigError::new(
                            &format!("AutoBuff.{path}.{mode}"),
                            "missing mode choice",
                        ));
                    }
                }
            } else {
                p.issues.push(ConfigError::new(
                    "AutoBuff.Soup.AfterUse.choices",
                    "expected choices object",
                ));
            }
            continue;
        }
        if v["value"].is_array() {
            parse_group(p, v, &path);
        } else if let Some(value) = v.get("value") {
            add(p, &path, value);
        }
    }
}
pub(super) fn parse_auto_buff(p: &mut SettingsPatch, root: &Nodes<'_>) {
    parse_values(p, root, "");
}
