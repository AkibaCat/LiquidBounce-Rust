// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::hitbox::Hitbox;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitboxSetting {
    Size(f32),
    ApplyToDebugHitbox(bool),
    ApplyToComponent(bool),
}
impl HitboxSetting {
    pub fn apply(self, module: &mut Hitbox) {
        match self {
            Self::Size(value) => module.size = value,
            Self::ApplyToDebugHitbox(value) => module.apply_to_debug_hitbox = value,
            Self::ApplyToComponent(value) => module.apply_to_component = value,
        }
    }
}
pub fn serialize_hitbox(
    state: &Hitbox,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::Hitbox, common);
    values.push(tree.ranged(
        "Size",
        "FLOAT",
        float_json(state.size, "Hitbox.Size")?,
        json!({"from":0.0,"to":1.0}),
        "",
        "liquidbounce.module.hitbox.size",
    ));
    for (name, value, key) in [
        (
            "ApplyToDebugHitbox",
            state.apply_to_debug_hitbox,
            "liquidbounce.module.hitbox.applyToDebugHitbox",
        ),
        (
            "ApplyToComponent",
            state.apply_to_component,
            "liquidbounce.module.hitbox.applyToComponent",
        ),
    ] {
        values.push(tree.leaf(name, "BOOLEAN", json!(value), key, None));
    }
    Ok(tree.group("Hitbox", "TOGGLEABLE", values))
}
pub(super) fn parse_hitbox(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Size", "Hitbox.Size", |value, path| {
        parse_float(value, path).map(|value| SettingChange::Hitbox(HitboxSetting::Size(value)))
    });
    leaf(
        patch,
        root,
        "ApplyToDebugHitbox",
        "Hitbox.ApplyToDebugHitbox",
        |value, path| {
            parse_bool(value, path)
                .map(|value| SettingChange::Hitbox(HitboxSetting::ApplyToDebugHitbox(value)))
        },
    );
    leaf(
        patch,
        root,
        "ApplyToComponent",
        "Hitbox.ApplyToComponent",
        |value, path| {
            parse_bool(value, path)
                .map(|value| SettingChange::Hitbox(HitboxSetting::ApplyToComponent(value)))
        },
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn declaration_order_and_range_are_ui_metadata() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({}),
        };
        let tree = serialize_hitbox(&Hitbox::default(), &common, JsonFormat::Interop, &|key| {
            key.into()
        })
        .unwrap();
        let values = tree["value"].as_array().unwrap();
        assert_eq!(
            values
                .iter()
                .map(|leaf| leaf["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec![
                "Bind",
                "Hidden",
                "Size",
                "ApplyToDebugHitbox",
                "ApplyToComponent"
            ]
        );
        assert_eq!(values[2]["range"], json!({"from":0.0,"to":1.0}));
        assert_eq!(values[2]["value"].to_string(), "0.1");
        let patch=parse_settings(ModuleKind::Hitbox,&json!({"name":"Hitbox","value":[{"name":"ApplyToComponent","value":false},{"name":"Size","value":12.75},{"name":"ApplyToDebugHitbox","value":false}]})).unwrap();
        assert_eq!(
            patch.changes,
            vec![
                SettingChange::Hitbox(HitboxSetting::Size(12.75)),
                SettingChange::Hitbox(HitboxSetting::ApplyToDebugHitbox(false)),
                SettingChange::Hitbox(HitboxSetting::ApplyToComponent(false))
            ]
        );
    }
    #[test]
    fn malformed_size_preserves_later_flags_and_file_values_round_trip() {
        let patch=parse_settings(ModuleKind::Hitbox,&json!({"name":"Hitbox","value":[{"name":"Size","value":{}},{"name":"ApplyToComponent","value":"false"}]})).unwrap();
        assert_eq!(patch.issues.len(), 1);
        assert_eq!(
            patch.changes,
            vec![SettingChange::Hitbox(HitboxSetting::ApplyToComponent(
                false
            ))]
        );
        let common = CommonSettings {
            enabled: false,
            hidden: true,
            key_bind: json!({}),
        };
        let state = Hitbox {
            size: -0.2,
            ..Hitbox::default()
        };
        let file = serialize_hitbox(&state, &common, JsonFormat::File, &|_| String::new()).unwrap();
        assert!(file["value"][3].get("range").is_none());
        let patch = parse_settings(ModuleKind::Hitbox, &file).unwrap();
        assert!(
            patch
                .changes
                .contains(&SettingChange::Hitbox(HitboxSetting::Size(-0.2)))
        );
    }
}
