// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleGhostHand TargetedBlocks registry-list tree.
use super::*;
use liquidbounce_behaviors::ghost_hand::GhostHand;

#[derive(Clone, Debug, PartialEq)]
pub enum GhostHandSetting {
    TargetedBlocks(Value),
}
impl GhostHandSetting {
    pub fn apply(self, module: &mut GhostHand, registry: &[String]) -> Result<(), ConfigError> {
        match self {
            Self::TargetedBlocks(value) => module
                .deserialize_targeted_blocks(&value, registry)
                .map_err(|error| ConfigError::new("GhostHand.TargetedBlocks", error)),
        }
    }
}
pub fn serialize_ghost_hand(
    state: &GhostHand,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::GhostHand, common);
    let mut blocks = tree.leaf(
        "TargetedBlocks",
        "REGISTRY_LIST",
        json!(state.sorted_targeted_blocks()),
        "liquidbounce.module.ghostHand.targetedBlocks",
        None,
    );
    if format == JsonFormat::Interop {
        blocks["registry"] = json!("block");
    }
    values.push(blocks);
    Ok(tree.group("GhostHand", "TOGGLEABLE", values))
}
pub(super) fn parse_ghost_hand(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "TargetedBlocks",
        "GhostHand.TargetedBlocks",
        |value, _| {
            Ok(SettingChange::GhostHand(GhostHandSetting::TargetedBlocks(
                value.clone(),
            )))
        },
    );
}
