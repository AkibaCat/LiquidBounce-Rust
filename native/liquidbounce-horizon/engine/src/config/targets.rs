// SPDX-License-Identifier: GPL-3.0-or-later
//! GlobalManager Settings -> Targets, including the legacy Enemies group alias.
use super::*;
use liquidbounce_behaviors::targeting::{GlobalTargets, Target, TargetSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetSetting {
    Combat(Value),
    Visual(Value),
}
impl TargetSetting {
    pub fn apply(self, targets: &mut GlobalTargets) -> Result<(), ConfigError> {
        let (selected, value, allow_self, path) = match self {
            Self::Combat(value) => (&mut targets.combat, value, false, "Settings.Targets.Combat"),
            Self::Visual(value) => (&mut targets.visual, value, true, "Settings.Targets.Visual"),
        };
        // MultiChoiceListValue mutates its live EnumSet before set/callbacks.
        // A bad entry retains the successfully decoded prefix, even on error.
        *selected = TargetSet::EMPTY;
        let mut include = |value: &Value| -> Result<(), ConfigError> {
            let tag = gson_target_string(value)
                .ok_or_else(|| ConfigError::new(path, "target entry has no Gson string value"))?;
            if let Some(choice) =
                Target::from_tag(&tag).filter(|choice| allow_self || *choice != Target::SelfEntity)
            {
                selected.0 |= 1 << choice as u8;
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
        Ok(())
    }
}
fn gson_target_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(_) | Value::Bool(_) => Some(value.to_string()),
        Value::Array(values) if values.len() == 1 => gson_target_string(&values[0]),
        _ => None,
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlobalTargetsPatch {
    pub changes: Vec<TargetSetting>,
    pub issues: Vec<ConfigError>,
}

pub fn serialize_global_targets(
    targets: &GlobalTargets,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Value {
    let tree = Tree { format, describe };
    let mut leaves = Vec::new();
    for (name, selected, allow_self, key) in [
        (
            "Combat",
            targets.combat,
            false,
            "liquidbounce.settings.targets.combat",
        ),
        (
            "Visual",
            targets.visual,
            true,
            "liquidbounce.settings.targets.visual",
        ),
    ] {
        let choices = Target::ALL
            .into_iter()
            .filter(|choice| allow_self || *choice != Target::SelfEntity)
            .collect::<Vec<_>>();
        let values = choices
            .iter()
            .filter(|choice| selected.contains(**choice))
            .map(|choice| choice.tag())
            .collect::<Vec<_>>();
        let mut leaf = tree.leaf(name, "MULTI_CHOOSE", json!(values), key, None);
        if format == JsonFormat::Interop {
            leaf["choices"] = json!(
                choices
                    .iter()
                    .map(|choice| choice.tag())
                    .collect::<Vec<_>>()
            );
            leaf["canBeNone"] = json!(true);
            leaf["isOrderSensitive"] = json!(false);
        }
        leaves.push(leaf);
    }
    tree.group(
        "Settings",
        "CONFIGURABLE",
        vec![tree.group("Targets", "CONFIGURABLE", leaves)],
    )
}

pub fn parse_global_targets(json: &Value) -> Result<GlobalTargetsPatch, ConfigError> {
    let root = nodes(json, "Settings", "Settings")?;
    let mut patch = GlobalTargetsPatch::default();
    // ConfigSystem tries a current name before its aliases; JSON order is irrelevant.
    let Some((node, name)) = root
        .get("Targets")
        .map(|node| (*node, "Targets"))
        .or_else(|| root.get("Enemies").map(|node| (*node, "Enemies")))
    else {
        return Ok(patch);
    };
    let children = match nodes(node, name, "Settings.Targets") {
        Ok(children) => children,
        Err(error) => {
            patch.issues.push(error);
            return Ok(patch);
        }
    };
    for (name, allow_self) in [("Combat", false), ("Visual", true)] {
        let Some(node) = children.get(name) else {
            continue;
        };
        let path = format!("Settings.Targets.{name}");
        match node.get("value") {
            Some(value) => {
                patch.changes.push(if allow_self {
                    TargetSetting::Visual(value.clone())
                } else {
                    TargetSetting::Combat(value.clone())
                });
            }
            None => patch.issues.push(ConfigError::new(&path, "missing value")),
        }
    }
    Ok(patch)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_tree_keeps_original_order_default_and_independent_choices() {
        let state = GlobalTargets::default();
        let tree = serialize_global_targets(&state, JsonFormat::Interop, &|key| key.into());
        assert_eq!(tree["name"], "Settings");
        let targets = &tree["value"][0];
        assert_eq!(targets["name"], "Targets");
        assert_eq!(targets["value"][0]["name"], "Combat");
        assert_eq!(targets["value"][1]["name"], "Visual");
        assert_eq!(
            targets["value"][0]["value"],
            json!([
                "Players",
                "Hostile",
                "Angerable",
                "WaterCreature",
                "Invisible"
            ])
        );
        assert_eq!(targets["value"][0]["choices"].as_array().unwrap().len(), 10);
        assert_eq!(targets["value"][1]["choices"][0], "Self");
        assert_eq!(
            targets["value"][0]["key"],
            "liquidbounce.settings.targets.combat"
        );
        assert_eq!(targets["value"][0]["canBeNone"], true);
    }
    #[test]
    fn settings_file_round_trips_empty_and_all_sets_without_interop_metadata() {
        let state = GlobalTargets {
            combat: TargetSet::EMPTY,
            visual: TargetSet::ALL,
        };
        let tree = serialize_global_targets(&state, JsonFormat::File, &|_| {
            panic!("file has no description")
        });
        assert!(tree.get("valueType").is_none());
        assert!(tree["value"][0]["value"][0].get("choices").is_none());
        let patch = parse_global_targets(&tree).unwrap();
        assert!(patch.issues.is_empty());
        let mut restored = GlobalTargets::default();
        for change in patch.changes {
            change.apply(&mut restored).unwrap();
        }
        assert_eq!(restored, state);
    }
    #[test]
    fn alias_and_case_insensitive_tags_do_not_make_combat_self_selectable() {
        let patch = parse_global_targets(
            &json!({"name":"Settings","value":[{"name":"Enemies","value":[
            {"name":"Visual","value":["Self","pLaYeRs","Friends"]},
            {"name":"Combat","value":["Self","pLaYeRs","Unknown",true,1,"Players"]}]}]}),
        )
        .unwrap();
        assert!(patch.issues.is_empty());
        let mut state = GlobalTargets::default();
        for change in patch.changes {
            change.apply(&mut state).unwrap();
        }
        assert_eq!(state.combat, TargetSet::from_targets([Target::Players]));
        assert_eq!(
            state.visual,
            TargetSet::from_targets([Target::SelfEntity, Target::Players, Target::Friends])
        );
    }
    #[test]
    fn bad_combat_leaf_does_not_skip_visual_and_names_remain_case_sensitive() {
        let patch=parse_global_targets(&json!({"name":"Settings","value":[{"name":"Targets","value":[{"name":"Combat","value":[{}]},{"name":"Visual","value":[]}]}]})).unwrap();
        assert!(patch.issues.is_empty());
        let mut state = GlobalTargets::default();
        let mut errors = Vec::new();
        for change in patch.changes {
            if let Err(error) = change.apply(&mut state) {
                errors.push(error);
            }
        }
        assert_eq!(errors.len(), 1);
        assert_eq!(state.combat, TargetSet::EMPTY);
        assert_eq!(state.visual, TargetSet::EMPTY);
        assert!(parse_global_targets(&json!({"name":"settings","value":[]})).is_err());
    }
    #[test]
    fn malformed_array_keeps_prefix_and_json_null_clears_live_set() {
        let mut state = GlobalTargets::default();
        assert!(
            TargetSetting::Combat(json!(["Players", ["Hostile"], null, "Passive"]))
                .apply(&mut state)
                .is_err()
        );
        assert_eq!(
            state.combat,
            TargetSet::from_targets([Target::Players, Target::Hostile])
        );
        TargetSetting::Visual(json!(null))
            .apply(&mut state)
            .unwrap();
        assert_eq!(state.visual, TargetSet::EMPTY);
        TargetSetting::Combat(json!(["Invısıble", "ſleeping"]))
            .apply(&mut state)
            .unwrap();
        assert_eq!(
            state.combat,
            TargetSet::from_targets([Target::Invisible, Target::Sleeping])
        );
    }
}
