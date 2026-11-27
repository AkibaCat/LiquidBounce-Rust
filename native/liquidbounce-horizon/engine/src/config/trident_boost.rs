// SPDX-License-Identifier: GPL-3.0-or-later
//! Full original ModuleTridentBoost settings and raw ConfigSystem imports.
use super::*;
use liquidbounce_behaviors::trident_boost::TridentBoost;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TridentBoostSetting { Horizontal(f32), Vertical(f32), OnLand(bool) }
impl TridentBoostSetting {
    pub fn apply(self, state: &mut TridentBoost) {
        match self {
            Self::Horizontal(v) => state.horizontal_multiplier = v,
            Self::Vertical(v) => state.vertical_multiplier = v,
            Self::OnLand(v) => state.on_land = v,
        }
    }
}
pub fn serialize_trident_boost(state: &TridentBoost, common: &CommonSettings,
    format: JsonFormat, describe: &dyn Fn(&str) -> String) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::TridentBoost, common);
    for (name, key, value) in [
        ("HorizontalMultiplier", "horizontalMultiplier", state.horizontal_multiplier),
        ("VerticalMultiplier", "verticalMultiplier", state.vertical_multiplier),
    ] {
        values.push(tree.ranged(name, "FLOAT", float_json(value, &format!("TridentBoost.{name}"))?,
            json!({"from":0.1,"to":5.0}), "", &format!("liquidbounce.module.tridentBoost.{key}")));
    }
    values.push(tree.leaf("OnLand", "BOOLEAN", json!(state.on_land),
        "liquidbounce.module.tridentBoost.onLand", None));
    Ok(tree.group("TridentBoost", "TOGGLEABLE", values))
}
pub(super) fn parse_trident_boost(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "HorizontalMultiplier", "TridentBoost.HorizontalMultiplier", |value, path|
        parse_float(value, path).map(|value| SettingChange::TridentBoost(TridentBoostSetting::Horizontal(value))));
    leaf(patch, root, "VerticalMultiplier", "TridentBoost.VerticalMultiplier", |value, path|
        parse_float(value, path).map(|value| SettingChange::TridentBoost(TridentBoostSetting::Vertical(value))));
    leaf(patch, root, "OnLand", "TridentBoost.OnLand", |value, path|
        parse_bool(value, path).map(|value| SettingChange::TridentBoost(TridentBoostSetting::OnLand(value))));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_tree_imports_raw_values_and_continues_after_a_bad_leaf() {
        let common = CommonSettings { enabled: false, hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}) };
        let mut state = TridentBoost::default();
        let interop = serialize_trident_boost(&state, &common, JsonFormat::Interop, &str::to_owned).unwrap();
        let values = interop["value"].as_array().unwrap();
        assert_eq!(values.iter().map(|v| v["name"].as_str().unwrap()).collect::<Vec<_>>(),
            ["Bind", "Hidden", "HorizontalMultiplier", "VerticalMultiplier", "OnLand"]);
        assert_eq!(values[2]["range"], json!({"from":0.1,"to":5.0}));
        let patch = parse_settings(ModuleKind::TridentBoost, &json!({"name":"TridentBoost","value":[
            {"name":"HorizontalMultiplier","value":"bad"},
            {"name":"VerticalMultiplier","value":-17},
            {"name":"OnLand","value":false}
        ]})).unwrap();
        assert_eq!(patch.issues.len(), 1);
        for change in patch.changes {
            if let SettingChange::TridentBoost(setting) = change { setting.apply(&mut state); }
        }
        assert_eq!(state.horizontal_multiplier, 2.0);
        assert_eq!(state.vertical_multiplier, -17.0);
        assert!(!state.on_land);
        let file = serialize_trident_boost(&state, &common, JsonFormat::File, &str::to_owned).unwrap();
        assert!(parse_settings(ModuleKind::TridentBoost, &file).unwrap().issues.is_empty());
    }
}
