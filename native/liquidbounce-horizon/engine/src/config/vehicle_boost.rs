// SPDX-License-Identifier: GPL-3.0-or-later
//! Both original settings. UI ranges never clamp ConfigSystem imports.
use super::*;
use liquidbounce_behaviors::vehicle_boost::VehicleBoost;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VehicleBoostSetting {
    Horizontal(f32),
    Vertical(f32),
}
impl VehicleBoostSetting {
    pub fn apply(self, state: &mut VehicleBoost) {
        match self {
            Self::Horizontal(v) => state.horizontal_speed = v,
            Self::Vertical(v) => state.vertical_speed = v,
        }
    }
}
pub fn serialize_vehicle_boost(
    state: &VehicleBoost,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::VehicleBoost, common);
    for (name, key, value, max) in [
        (
            "HorizontalSpeed",
            "horizontalSpeed",
            state.horizontal_speed,
            10.0,
        ),
        ("VerticalSpeed", "verticalSpeed", state.vertical_speed, 5.0),
    ] {
        values.push(tree.ranged(
            name,
            "FLOAT",
            float_json(value, &format!("VehicleBoost.{name}"))?,
            json!({"from":0.1,"to":max}),
            "",
            &format!("liquidbounce.module.vehicleBoost.{key}"),
        ));
    }
    Ok(tree.group("VehicleBoost", "TOGGLEABLE", values))
}
pub(super) fn parse_vehicle_boost(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "HorizontalSpeed",
        "VehicleBoost.HorizontalSpeed",
        |value, path| {
            parse_float(value, path)
                .map(|v| SettingChange::VehicleBoost(VehicleBoostSetting::Horizontal(v)))
        },
    );
    leaf(
        patch,
        root,
        "VerticalSpeed",
        "VehicleBoost.VerticalSpeed",
        |value, path| {
            parse_float(value, path)
                .map(|v| SettingChange::VehicleBoost(VehicleBoostSetting::Vertical(v)))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_imports_preserve_both_speeds_and_continue_after_a_bad_leaf() {
        let patch = parse_settings(
            ModuleKind::VehicleBoost,
            &json!({"name":"VehicleBoost","value":[
                {"name":"HorizontalSpeed","value":null}, {"name":"VerticalSpeed","value":-7.25}
            ]}),
        )
        .unwrap();
        assert_eq!(patch.issues.len(), 1);
        let mut state = VehicleBoost::default();
        for change in patch.changes {
            if let SettingChange::VehicleBoost(value) = change {
                value.apply(&mut state);
            }
        }
        assert_eq!(state.horizontal_speed, 2.0);
        assert_eq!(state.vertical_speed, -7.25);
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        let file =
            serialize_vehicle_boost(&state, &common, JsonFormat::File, &str::to_owned).unwrap();
        let restored = parse_settings(ModuleKind::VehicleBoost, &file).unwrap();
        assert!(restored.issues.is_empty());
        assert!(restored.changes.contains(&SettingChange::VehicleBoost(
            VehicleBoostSetting::Vertical(-7.25)
        )));
    }
}
