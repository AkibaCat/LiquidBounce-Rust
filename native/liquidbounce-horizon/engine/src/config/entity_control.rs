// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete original EntityControl Enforce tree and sequential import.
use super::*;
use liquidbounce_behaviors::entity_control::{ENFORCE, EntityControl};

#[derive(Clone, Debug, PartialEq)]
pub enum EntityControlSetting {
    Enforce(Value),
}
impl EntityControlSetting {
    pub fn apply(self, module: &mut EntityControl) -> Result<(), ConfigError> {
        match self {
            Self::Enforce(value) => module
                .deserialize_enforce(&value)
                .map_err(|error| ConfigError::new("EntityControl.Enforce", error)),
        }
    }
}

pub fn serialize_entity_control(
    state: &EntityControl,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::EntityControl, common);
    values.push(tree.multi(
        "Enforce",
        u32::from(state.enforce),
        &ENFORCE,
        "liquidbounce.module.entityControl.enforce",
    ));
    Ok(tree.group("EntityControl", "TOGGLEABLE", values))
}

pub(super) fn parse_entity_control(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(node) = root.get("Enforce") {
        // JsonNull clears this MultiChoiceListValue; a missing property fails.
        match node.get("value") {
            Some(value) => {
                patch
                    .changes
                    .push(SettingChange::EntityControl(EntityControlSetting::Enforce(
                        value.clone(),
                    )))
            }
            None => patch
                .issues
                .push(ConfigError::new("EntityControl.Enforce", "missing value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_enforce_subsets_roundtrip_with_original_choice_order_and_import_errors() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        for enforce in 0..4 {
            let state = EntityControl { enforce };
            for format in [JsonFormat::File, JsonFormat::Interop] {
                let value =
                    serialize_entity_control(&state, &common, format, &str::to_owned).unwrap();
                let leaf = value["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["name"] == "Enforce")
                    .unwrap();
                if format == JsonFormat::Interop {
                    assert_eq!(leaf["choices"], json!(["Saddled", "JumpStrength"]));
                }
                let patch = parse_settings(ModuleKind::EntityControl, &value).unwrap();
                assert!(patch.issues.is_empty());
                let mut restored = EntityControl::default();
                for change in patch.changes {
                    if let SettingChange::EntityControl(value) = change {
                        value.apply(&mut restored).unwrap();
                    }
                }
                assert_eq!(restored, state);
            }
        }
        let mut state = EntityControl::default();
        assert!(
            EntityControlSetting::Enforce(json!(["JumpStrength", null]))
                .apply(&mut state)
                .is_err()
        );
        assert_eq!(state.enforce, 2);
        let root = json!({"name":"EntityControl","value":[{"name":"Enforce"}]});
        assert_eq!(
            parse_settings(ModuleKind::EntityControl, &root)
                .unwrap()
                .issues
                .len(),
            1
        );
    }
}
