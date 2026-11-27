// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoWeapon's complete ae0eb396 setting declaration.
use super::*;
use liquidbounce_behaviors::auto_weapon::{AutoWeapon, CHANGE_ON, PREFERRED, PRIORITIES, Priority};

#[derive(Clone, Debug, PartialEq)]
pub enum AutoWeaponSetting {
    Preferred(Value),
    Priority(Priority),
    AutoShieldBreak(bool),
    AutoMace(bool),
    SwitchBack(i32),
    ChangeOn(Value),
}
impl AutoWeaponSetting {
    pub fn apply(self, state: &mut AutoWeapon) -> Result<(), ConfigError> {
        match self {
            Self::Preferred(value) => state
                .deserialize_preferred(&value)
                .map_err(|error| ConfigError::new("AutoWeapon.Preferred", error))?,
            Self::Priority(value) => state.priority = value,
            Self::AutoShieldBreak(value) => state.auto_shield_break = value,
            Self::AutoMace(value) => state.auto_mace = value,
            Self::SwitchBack(value) => state.switch_back = value,
            Self::ChangeOn(value) => state
                .deserialize_change_on(&value)
                .map_err(|error| ConfigError::new("AutoWeapon.ChangeOn", error))?,
        }
        Ok(())
    }
}
pub fn serialize_auto_weapon(
    state: &AutoWeapon,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::AutoWeapon, common);
    values.push(tree.multi(
        "Preferred",
        u32::from(state.preferred),
        &PREFERRED,
        "liquidbounce.module.autoWeapon.preferred",
    ));
    let mut priority = tree.leaf(
        "Priority",
        "CHOOSE",
        json!(state.priority.tag()),
        "liquidbounce.module.autoWeapon.priority",
        None,
    );
    if format == JsonFormat::Interop {
        priority["choices"] = json!(PRIORITIES);
    }
    values.push(priority);
    values.push(tree.leaf(
        "AutoShieldBreak",
        "BOOLEAN",
        json!(state.auto_shield_break),
        "liquidbounce.module.autoWeapon.autoShieldBreak",
        None,
    ));
    values.push(tree.leaf(
        "AutoMace",
        "BOOLEAN",
        json!(state.auto_mace),
        "liquidbounce.module.autoWeapon.autoMace",
        None,
    ));
    values.push(tree.ranged(
        "SwitchBack",
        "INT",
        json!(state.switch_back),
        json!({"from":1,"to":300}),
        "ticks",
        "liquidbounce.module.autoWeapon.switchBack",
    ));
    values.push(tree.multi(
        "ChangeOn",
        u32::from(state.change_on),
        &CHANGE_ON,
        "liquidbounce.module.autoWeapon.changeOn",
    ));
    Ok(tree.group("AutoWeapon", "TOGGLEABLE", values))
}
pub(super) fn parse_auto_weapon(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(node) = root.get("Preferred") {
        match node.get("value") {
            Some(value) => {
                patch
                    .changes
                    .push(SettingChange::AutoWeapon(AutoWeaponSetting::Preferred(
                        value.clone(),
                    )))
            }
            None => patch
                .issues
                .push(ConfigError::new("AutoWeapon.Preferred", "missing value")),
        }
    }
    leaf(
        patch,
        root,
        "Priority",
        "AutoWeapon.Priority",
        |value, path| {
            Priority::parse(value)
                .map(|value| SettingChange::AutoWeapon(AutoWeaponSetting::Priority(value)))
                .map_err(|error| ConfigError::new(path, error))
        },
    );
    leaf(
        patch,
        root,
        "AutoShieldBreak",
        "AutoWeapon.AutoShieldBreak",
        |value, path| {
            parse_bool(value, path)
                .map(|value| SettingChange::AutoWeapon(AutoWeaponSetting::AutoShieldBreak(value)))
        },
    );
    leaf(
        patch,
        root,
        "AutoMace",
        "AutoWeapon.AutoMace",
        |value, path| {
            parse_bool(value, path)
                .map(|value| SettingChange::AutoWeapon(AutoWeaponSetting::AutoMace(value)))
        },
    );
    leaf(
        patch,
        root,
        "SwitchBack",
        "AutoWeapon.SwitchBack",
        |value, path| {
            parse_int(value, path)
                .map(|value| SettingChange::AutoWeapon(AutoWeaponSetting::SwitchBack(value)))
        },
    );
    if let Some(node) = root.get("ChangeOn") {
        match node.get("value") {
            Some(value) => {
                patch
                    .changes
                    .push(SettingChange::AutoWeapon(AutoWeaponSetting::ChangeOn(
                        value.clone(),
                    )))
            }
            None => patch
                .issues
                .push(ConfigError::new("AutoWeapon.ChangeOn", "missing value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn auto_weapon_complete_source_order_defaults_and_raw_values() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        let state = AutoWeapon::default();
        let tree =
            serialize_auto_weapon(&state, &common, JsonFormat::Interop, &str::to_owned).unwrap();
        let values = tree["value"].as_array().unwrap();
        assert_eq!(
            values
                .iter()
                .map(|v| v["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "Bind",
                "Hidden",
                "Preferred",
                "Priority",
                "AutoShieldBreak",
                "AutoMace",
                "SwitchBack",
                "ChangeOn"
            ]
        );
        assert_eq!(values[2]["value"], json!(["Sword"]));
        assert_eq!(values[2]["choices"], json!(PREFERRED));
        assert_eq!(values[3]["value"], "Default");
        assert_eq!(values[3]["choices"], json!(PRIORITIES));
        assert_eq!(values[4]["value"], true);
        assert_eq!(values[5]["value"], true);
        assert_eq!(values[6]["value"], 20);
        assert_eq!(values[6]["range"], json!({"from":1,"to":300}));
        assert_eq!(values[6]["suffix"], "ticks");
        assert_eq!(values[7]["value"], json!(["OnAttack"]));
        assert_eq!(values[7]["choices"], json!(CHANGE_ON));
        let state = AutoWeapon {
            switch_back: i32::MIN,
            ..state
        };
        let stored =
            serialize_auto_weapon(&state, &common, JsonFormat::File, &str::to_owned).unwrap();
        assert_eq!(stored["value"][7]["value"], i32::MIN);
    }
    #[test]
    fn auto_weapon_imports_continue_after_bad_leaf_and_preserve_multichoice_prefix() {
        let root = json!({"name":"AutoWeapon","value":[
            {"name":"Preferred","value":["Axe",null,"Mace"]},
            {"name":"Priority","value":"Knockback"},
            {"name":"Priority","value":"Damage"},
            {"name":"AutoShieldBreak","value":false},
            {"name":"AutoMace","value":false},
            {"name":"SwitchBack","value":4294967297_i64},
            {"name":"ChangeOn","value":["OnTarget",{}]}
        ]});
        let patch = parse_settings(ModuleKind::AutoWeapon, &root).unwrap();
        assert!(patch.issues.is_empty());
        let mut state = AutoWeapon::default();
        let mut errors = 0;
        for change in patch.changes {
            if let SettingChange::AutoWeapon(setting) = change {
                errors += usize::from(setting.apply(&mut state).is_err());
            }
        }
        assert_eq!(errors, 2);
        assert_eq!(state.preferred, liquidbounce_behaviors::auto_weapon::AXE);
        assert_eq!(state.priority, Priority::Knockback);
        assert!(!state.auto_shield_break);
        assert!(!state.auto_mace);
        assert_eq!(state.switch_back, 1);
        assert_eq!(
            state.change_on,
            liquidbounce_behaviors::auto_weapon::ON_TARGET
        );
        let patch=parse_settings(ModuleKind::AutoWeapon,&json!({"name":"AutoWeapon","value":[{"name":"Priority","value":"unknown"},{"name":"SwitchBack","value":-900}]})).unwrap();
        assert_eq!(patch.issues.len(), 1);
        assert_eq!(
            patch.changes,
            [SettingChange::AutoWeapon(AutoWeaponSetting::SwitchBack(
                -900
            ))]
        );
    }
}
