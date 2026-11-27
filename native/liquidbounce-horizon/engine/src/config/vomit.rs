// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete PlayerInventoryConstraints tree, including old tick->ms imports.
use super::*;
use liquidbounce_behaviors::vomit::{REQUIREMENTS, Vomit};

#[derive(Clone, Debug, PartialEq)]
pub enum VomitSetting {
    StartDelay([i32; 2]),
    ClickDelay([i32; 2]),
    CloseDelay([i32; 2]),
    MissChance([i32; 2]),
    SlotMultiplier(f32),
    MillisecondDelays(bool),
    Requires(Value),
}
impl VomitSetting {
    pub fn apply(self, state: &mut Vomit) -> Result<(), ConfigError> {
        let c = &mut state.constraints;
        match self {
            Self::StartDelay(value) => c.start_delay = value,
            Self::ClickDelay(value) => c.click_delay = value,
            Self::CloseDelay(value) => c.close_delay = value,
            Self::MissChance(value) => c.miss_chance = value,
            Self::SlotMultiplier(value) => c.slot_multiplier = value,
            Self::MillisecondDelays(value) => c.millisecond_delays = value,
            Self::Requires(value) => {
                c.requirements = 0;
                fn string(value: &Value) -> Option<String> {
                    match value {
                        Value::String(s) => Some(s.clone()),
                        Value::Number(_) | Value::Bool(_) => Some(value.to_string()),
                        Value::Array(v) if v.len() == 1 => string(&v[0]),
                        _ => None,
                    }
                }
                fn fold(c: char) -> char {
                    let mut upper = c.to_uppercase();
                    let first = upper.next().unwrap_or(c);
                    let upper = if upper.next().is_none() { first } else { c };
                    upper.to_lowercase().next().unwrap_or(upper)
                }
                let mut include = |value: &Value| -> Result<(), ConfigError> {
                    let tag = string(value).ok_or_else(|| {
                        ConfigError::new(
                            "Vomit.Constraints.Requires",
                            "entry has no Gson string value",
                        )
                    })?;
                    if let Some(index) = REQUIREMENTS
                        .iter()
                        .position(|choice| choice.chars().map(fold).eq(tag.chars().map(fold)))
                    {
                        c.requirements |= 1 << index;
                    }
                    Ok(())
                };
                match &value {
                    Value::Array(values) => {
                        for value in values {
                            include(value)?;
                        }
                    }
                    Value::String(_) | Value::Number(_) | Value::Bool(_) => include(&value)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
pub fn serialize_vomit(
    state: &Vomit,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let c = &state.constraints;
    let mut constraints = Vec::new();
    for (name, suffix, range, max, unit) in [
        ("StartDelay", "startDelay", c.start_delay, 1000, "ms"),
        ("ClickDelay", "clickDelay", c.click_delay, 1000, "ms"),
        ("CloseDelay", "closeDelay", c.close_delay, 1000, "ms"),
        ("MissChance", "missChance", c.miss_chance, 100, "%"),
    ] {
        constraints.push(tree.ranged(
            name,
            "INT_RANGE",
            json!({"from":range[0],"to":range[1]}),
            json!({"from":0,"to":max}),
            unit,
            &format!("liquidbounce.module.vomit.constraints.{suffix}"),
        ));
    }
    constraints.push(tree.ranged(
        "SlotMultiplier",
        "FLOAT",
        float_json(c.slot_multiplier, "Vomit.Constraints.SlotMultiplier")?,
        json!({"from":0.0,"to":1.0}),
        "x",
        "liquidbounce.module.vomit.constraints.slotMultiplier",
    ));
    if format == JsonFormat::File {
        constraints.push(tree.leaf(
            "MillisecondDelays",
            "BOOLEAN",
            json!(c.millisecond_delays),
            "liquidbounce.module.vomit.constraints.millisecondDelays",
            None,
        ));
    }
    constraints.push(tree.multi(
        "Requires",
        u32::from(c.requirements),
        &REQUIREMENTS,
        "liquidbounce.module.vomit.constraints.requires",
    ));
    let mut values = tree.common(ModuleKind::Vomit, common);
    values.push(tree.group("Constraints", "CONFIGURABLE", constraints));
    Ok(tree.group("Vomit", "TOGGLEABLE", values))
}

/// The original group mutates old JSON before deserializing children. A broken
/// legacy group aborts before any child value is applied; 32-bit *50 wraps.
fn prepare_constraints(group: &Value) -> Result<Value, ConfigError> {
    let path = "Vomit.Constraints";
    let mut group = group.clone();
    let values = group
        .get("value")
        .and_then(Value::as_array)
        .ok_or_else(|| ConfigError::new(path, "expected constraint values"))?;
    for value in values {
        value
            .as_object()
            .ok_or_else(|| ConfigError::new(path, "constraint entry is not an object"))?;
    }
    let has_type = group
        .as_object()
        .is_some_and(|v| v.contains_key("valueType"));
    let mut marker = false;
    if !has_type {
        for value in values {
            let name = value
                .get("name")
                .and_then(scalar_string)
                .ok_or_else(|| ConfigError::new(path, "constraint entry has no string name"))?;
            if name == "MillisecondDelays" {
                marker = true;
                break;
            }
        }
    }
    if has_type || marker {
        return Ok(group);
    }
    for value in group["value"].as_array_mut().unwrap() {
        let name = value["name"].as_str().unwrap_or_default();
        if ["StartDelay", "ClickDelay", "CloseDelay"].contains(&name) {
            let range = value
                .get_mut("value")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| ConfigError::new(path, "legacy delay must be a range object"))?;
            let from = parse_int(
                range
                    .get("from")
                    .ok_or_else(|| ConfigError::new(path, "missing legacy from"))?,
                path,
            )?
            .wrapping_mul(50);
            let to = parse_int(
                range
                    .get("to")
                    .ok_or_else(|| ConfigError::new(path, "missing legacy to"))?,
                path,
            )?
            .wrapping_mul(50);
            range.insert("from".into(), json!(from));
            range.insert("to".into(), json!(to));
        }
    }
    Ok(group)
}
pub(super) fn parse_vomit(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(group) = root.get("Constraints") else {
        return;
    };
    let group = match prepare_constraints(group) {
        Ok(group) => group,
        Err(error) => {
            patch.issues.push(error);
            return;
        }
    };
    let c = match nodes(&group, "Constraints", "Vomit.Constraints") {
        Ok(c) => c,
        Err(error) => {
            patch.issues.push(error);
            return;
        }
    };
    for (name, make) in [
        (
            "StartDelay",
            VomitSetting::StartDelay as fn([i32; 2]) -> VomitSetting,
        ),
        ("ClickDelay", VomitSetting::ClickDelay),
        ("CloseDelay", VomitSetting::CloseDelay),
        ("MissChance", VomitSetting::MissChance),
    ] {
        leaf(
            patch,
            &c,
            name,
            &format!("Vomit.Constraints.{name}"),
            |v, p| parse_int_pair(v, p).map(|v| SettingChange::Vomit(make(v))),
        );
    }
    leaf(
        patch,
        &c,
        "SlotMultiplier",
        "Vomit.Constraints.SlotMultiplier",
        |v, p| parse_float(v, p).map(|v| SettingChange::Vomit(VomitSetting::SlotMultiplier(v))),
    );
    leaf(
        patch,
        &c,
        "MillisecondDelays",
        "Vomit.Constraints.MillisecondDelays",
        |v, p| parse_bool(v, p).map(|v| SettingChange::Vomit(VomitSetting::MillisecondDelays(v))),
    );
    leaf(
        patch,
        &c,
        "Requires",
        "Vomit.Constraints.Requires",
        |v, _| Ok(SettingChange::Vomit(VomitSetting::Requires(v.clone()))),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        }
    }
    fn apply(json: &Value) -> (Vomit, SettingsPatch) {
        let patch = parse_settings(ModuleKind::Vomit, json).unwrap();
        let mut state = Vomit::default();
        for change in patch.changes.clone() {
            if let SettingChange::Vomit(value) = change {
                value.apply(&mut state).unwrap();
            }
        }
        (state, patch)
    }
    #[test]
    fn all_constraints_metadata_and_complete_file_roundtrip() {
        let mut state = Vomit::default();
        state.constraints.requirements = 63;
        let interop =
            serialize_vomit(&state, &common(), JsonFormat::Interop, &str::to_owned).unwrap();
        let c = interop["value"][2]["value"].as_array().unwrap();
        assert_eq!(
            c.iter()
                .map(|v| v["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "StartDelay",
                "ClickDelay",
                "CloseDelay",
                "MissChance",
                "SlotMultiplier",
                "Requires"
            ]
        );
        assert_eq!(c[0]["value"], json!({"from":50,"to":100}));
        assert_eq!(c[1]["suffix"], "ms");
        assert_eq!(c[3]["range"], json!({"from":0,"to":100}));
        assert_eq!(c[4]["suffix"], "x");
        assert_eq!(c[5]["choices"], json!(REQUIREMENTS));
        for mask in 0..64 {
            state.constraints.requirements = mask;
            let file =
                serialize_vomit(&state, &common(), JsonFormat::File, &str::to_owned).unwrap();
            let (actual, patch) = apply(&file);
            assert!(patch.issues.is_empty());
            assert_eq!(actual.constraints, state.constraints);
        }
        assert_eq!(ModuleKind::Vomit.category(), "Fun");
    }
    #[test]
    fn legacy_tick_migration_and_both_explicit_markers() {
        let node = |meta: Option<bool>, millis: bool| {
            let mut c = json!({"name":"Constraints","value":[{"name":"StartDelay","value":{"from":2,"to":3}},{"name":"ClickDelay","value":{"from":i32::MAX,"to":-1}},{"name":"CloseDelay","value":{"from":0,"to":1}}]});
            if let Some(value) = meta {
                c["valueType"] = json!(value);
            }
            if millis {
                c["value"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"name":"MillisecondDelays","value":false}));
            }
            json!({"name":"Vomit","value":[c]})
        };
        let (legacy, patch) = apply(&node(None, false));
        assert!(patch.issues.is_empty());
        assert_eq!(legacy.constraints.start_delay, [100, 150]);
        assert_eq!(legacy.constraints.click_delay, [-50, -50]);
        assert_eq!(legacy.constraints.close_delay, [0, 50]);
        for value in [node(Some(false), false), node(None, true)] {
            let (state, patch) = apply(&value);
            assert!(patch.issues.is_empty());
            assert_eq!(state.constraints.start_delay, [2, 3]);
            assert_eq!(state.constraints.click_delay, [i32::MAX, -1]);
        }
    }
    #[test]
    fn raw_values_and_mutable_requirement_prefix_are_preserved() {
        let mut state = Vomit::default();
        VomitSetting::Requires(json!([
            "inventoryopen",
            "NOmovemeNT",
            "NotUsingItem",
            true,
            123,
            "unknown"
        ]))
        .apply(&mut state)
        .unwrap();
        assert_eq!(state.constraints.requirements, 1 | 4 | 32);
        assert!(
            VomitSetting::Requires(json!(["NoRotation", {}, "NoMovement"]))
                .apply(&mut state)
                .is_err()
        );
        assert_eq!(state.constraints.requirements, 2);
        VomitSetting::Requires(json!([["NotBreaking"]]))
            .apply(&mut state)
            .unwrap();
        assert_eq!(state.constraints.requirements, 8);
        VomitSetting::Requires(json!(null))
            .apply(&mut state)
            .unwrap();
        assert_eq!(state.constraints.requirements, 0);
        VomitSetting::SlotMultiplier(f32::NAN)
            .apply(&mut state)
            .unwrap();
        assert!(state.constraints.slot_multiplier.is_nan());
        assert!(serialize_vomit(&state, &common(), JsonFormat::Interop, &str::to_owned).is_err());
        assert!(matches!(
            apply_setting(
                &SettingChange::Vomit(VomitSetting::ClickDelay([i32::MAX, i32::MIN])),
                &mut NoFov::default(),
                &mut Zoom::default(),
                &mut FullBright::default()
            ),
            ApplyResult::Runtime(RuntimeRequest::Vomit(VomitSetting::ClickDelay([
                i32::MAX,
                i32::MIN
            ])))
        ));
    }
    #[test]
    fn legacy_prepare_failure_aborts_group_but_preserves_common_values() {
        let (state, patch) = apply(
            &json!({"name":"Vomit","value":[{"name":"Hidden","value":true},{"name":"Constraints","value":[{"name":"StartDelay","value":{"from":1,"to":2}},{"name":"ClickDelay","value":2},{"name":"Requires","value":["NoMovement"]}]}]}),
        );
        assert_eq!(patch.issues.len(), 1);
        assert_eq!(state.constraints, Vomit::default().constraints);
        assert_eq!(patch.changes, [SettingChange::Hidden(true)]);
    }
}
