// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::anti_hunger::AntiHunger;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AntiHungerSetting {
    NoSprint(bool),
    WhileSwimming(bool),
    KeepFloating(bool),
}
impl AntiHungerSetting {
    pub fn apply(self, state: &mut AntiHunger) {
        match self {
            Self::NoSprint(v) => state.settings.no_sprint = v,
            Self::WhileSwimming(v) => state.settings.while_swimming = v,
            Self::KeepFloating(v) => state.settings.keep_floating = v,
        }
    }
}
pub fn serialize_anti_hunger(
    state: &AntiHunger,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::AntiHunger, common);
    values.push(tree.group(
        "NoSprint",
        "TOGGLEABLE",
        vec![
            tree.leaf(
                "Enabled",
                "BOOLEAN",
                json!(state.settings.no_sprint),
                "liquidbounce.module.antiHunger.noSprint.enabled",
                None,
            ),
            tree.leaf(
                "WhileSwimming",
                "BOOLEAN",
                json!(state.settings.while_swimming),
                "liquidbounce.module.antiHunger.noSprint.whileSwimming",
                None,
            ),
        ],
    ));
    values.push(tree.leaf(
        "KeepFloating",
        "BOOLEAN",
        json!(state.settings.keep_floating),
        "liquidbounce.module.antiHunger.keepFloating",
        None,
    ));
    Ok(tree.group("AntiHunger", "TOGGLEABLE", values))
}
pub(super) fn parse_anti_hunger(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(group) = root.get("NoSprint") {
        match nodes(group, "NoSprint", "AntiHunger.NoSprint") {
            Ok(group) => {
                leaf(
                    patch,
                    &group,
                    "Enabled",
                    "AntiHunger.NoSprint.Enabled",
                    |v, p| {
                        parse_bool(v, p)
                            .map(|v| SettingChange::AntiHunger(AntiHungerSetting::NoSprint(v)))
                    },
                );
                leaf(
                    patch,
                    &group,
                    "WhileSwimming",
                    "AntiHunger.NoSprint.WhileSwimming",
                    |v, p| {
                        parse_bool(v, p)
                            .map(|v| SettingChange::AntiHunger(AntiHungerSetting::WhileSwimming(v)))
                    },
                );
            }
            Err(e) => patch.issues.push(e),
        }
    }
    leaf(
        patch,
        root,
        "KeepFloating",
        "AntiHunger.KeepFloating",
        |v, p| {
            parse_bool(v, p).map(|v| SettingChange::AntiHunger(AntiHungerSetting::KeepFloating(v)))
        },
    );
}
