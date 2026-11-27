// SPDX-License-Identifier: GPL-3.0-or-later
//! MultiActions' one declared value; common values retain their source order.
use super::*;
use liquidbounce_behaviors::multi_actions::{ACTIONS, MultiActions};

#[derive(Clone, Debug, PartialEq)]
pub enum MultiActionsSetting {
    Actions(Value),
}
impl MultiActionsSetting {
    pub fn apply(self, module: &mut MultiActions) -> Result<(), ConfigError> {
        match self {
            Self::Actions(value) => module
                .deserialize_actions(&value)
                .map_err(|error| ConfigError::new("MultiActions.Actions", error)),
        }
    }
}

pub fn serialize_multi_actions(
    state: &MultiActions,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::MultiActions, common);
    values.push(tree.multi(
        "Actions",
        u32::from(state.actions),
        &ACTIONS,
        "liquidbounce.module.multiActions.actions",
    ));
    Ok(tree.group("MultiActions", "TOGGLEABLE", values))
}

pub(super) fn parse_multi_actions(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(node) = root.get("Actions") {
        // JsonNull is a real JsonElement and clears this MultiChoiceListValue.
        // A missing property is Java null and fails the Kotlin parameter guard.
        match node.get("value") {
            Some(value) => {
                patch
                    .changes
                    .push(SettingChange::MultiActions(MultiActionsSetting::Actions(
                        value.clone(),
                    )))
            }
            None => patch
                .issues
                .push(ConfigError::new("MultiActions.Actions", "missing value")),
        }
    }
}
