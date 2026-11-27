// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete original SkinDerp tree and mutable EnumSet import semantics.
use super::*;
use liquidbounce_behaviors::skin_derp::{PARTS, SkinDerp, part_index};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkinDerpSetting {
    Sync(bool),
    Delay(i32),
    Parts(Value),
}
impl SkinDerpSetting {
    pub fn apply(self, state: &mut SkinDerp) -> Result<(), ConfigError> {
        match self {
            Self::Sync(value) => state.settings.sync = value,
            Self::Delay(value) => state.settings.delay = value,
            Self::Parts(value) => {
                state.settings.parts = 0;
                fn string(value: &Value) -> Option<String> {
                    match value {
                        Value::String(value) => Some(value.clone()),
                        Value::Number(_) | Value::Bool(_) => Some(value.to_string()),
                        Value::Array(values) if values.len() == 1 => string(&values[0]),
                        _ => None,
                    }
                }
                let mut include = |value: &Value| -> Result<(), ConfigError> {
                    let tag = string(value).ok_or_else(|| {
                        ConfigError::new("SkinDerp.Parts", "entry has no Gson string value")
                    })?;
                    if let Some(index) = part_index(&tag) {
                        state.settings.parts |= 1 << index;
                    }
                    Ok(())
                };
                match value {
                    Value::Array(values) => {
                        for value in values {
                            include(&value)?;
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
pub fn serialize_skin_derp(
    state: &SkinDerp,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::SkinDerp, common);
    values.push(tree.leaf(
        "Sync",
        "BOOLEAN",
        json!(state.settings.sync),
        "liquidbounce.module.skinDerp.sync",
        None,
    ));
    values.push(tree.ranged(
        "Delay",
        "INT",
        json!(state.settings.delay),
        json!({"from":0,"to":20}),
        "ticks",
        "liquidbounce.module.skinDerp.delay",
    ));
    values.push(tree.multi(
        "Parts",
        u32::from(state.settings.parts),
        &PARTS,
        "liquidbounce.module.skinDerp.parts",
    ));
    Ok(tree.group("SkinDerp", "TOGGLEABLE", values))
}
pub(super) fn parse_skin_derp(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Sync", "SkinDerp.Sync", |v, p| {
        parse_bool(v, p).map(|v| SettingChange::SkinDerp(SkinDerpSetting::Sync(v)))
    });
    leaf(patch, root, "Delay", "SkinDerp.Delay", |v, p| {
        parse_int(v, p).map(|v| SettingChange::SkinDerp(SkinDerpSetting::Delay(v)))
    });
    leaf(patch, root, "Parts", "SkinDerp.Parts", |v, _| {
        Ok(SettingChange::SkinDerp(SkinDerpSetting::Parts(v.clone())))
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        }
    }
    #[test]
    fn complete_tree_and_raw_delays_roundtrip_all_part_subsets() {
        for parts in 0..128 {
            let mut state = SkinDerp::default();
            state.settings.parts = parts;
            state.settings.sync = true;
            state.settings.delay = i32::MIN;
            let tree = serialize_skin_derp(&state, &common(), JsonFormat::Interop, &str::to_owned)
                .unwrap();
            assert_eq!(
                tree["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v["name"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["Bind", "Hidden", "Sync", "Delay", "Parts"]
            );
            assert_eq!(tree["value"][3]["range"], json!({"from":0,"to":20}));
            assert_eq!(tree["value"][3]["suffix"], "ticks");
            let file =
                serialize_skin_derp(&state, &common(), JsonFormat::File, &str::to_owned).unwrap();
            let patch = parse_settings(ModuleKind::SkinDerp, &file).unwrap();
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            let mut restored = SkinDerp::default();
            for change in patch.changes {
                if let SettingChange::SkinDerp(value) = change {
                    value.apply(&mut restored).unwrap();
                }
            }
            assert_eq!(restored.settings, state.settings);
        }
        assert_eq!(ModuleKind::SkinDerp.category(), "Fun");
        assert_eq!(ModuleKind::SkinDerp.key(), "liquidbounce.module.skinDerp");
    }
    #[test]
    fn parts_import_mutates_live_enumset_before_error_and_preserves_wait() {
        let mut state = SkinDerp::default();
        state.settings.delay = 2;
        state.start_tick_handler(true, 127, &mut || unreachable!());
        SkinDerpSetting::Parts(json!(["cape", "HAT", "unknown", "leftpants", "hat"]))
            .apply(&mut state)
            .unwrap();
        assert_eq!(state.settings.parts, 1 | 4 | 64);
        assert!(state.busy());
        assert!(
            SkinDerpSetting::Parts(json!(["Jacket", {}, "Cape"]))
                .apply(&mut state)
                .is_err()
        );
        assert_eq!(state.settings.parts, 2);
        SkinDerpSetting::Parts(json!(null))
            .apply(&mut state)
            .unwrap();
        assert_eq!(state.settings.parts, 0);
        SkinDerpSetting::Parts(json!([["RightSleeve"]]))
            .apply(&mut state)
            .unwrap();
        assert_eq!(state.settings.parts, 1 << 5);
        assert!(matches!(
            apply_setting(
                &SettingChange::SkinDerp(SkinDerpSetting::Sync(true)),
                &mut NoFov::default(),
                &mut Zoom::default(),
                &mut FullBright::default()
            ),
            ApplyResult::Runtime(RuntimeRequest::SkinDerp(SkinDerpSetting::Sync(true)))
        ));
    }
}
