// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleNoPose's two original declarations, in their original order.
use super::*;
use liquidbounce_behaviors::no_pose::{NoPose, SneakHeight};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoPoseSetting {
    NoSwim(bool),
    SneakHeight(SneakHeight),
}
impl NoPoseSetting {
    pub fn apply(self, module: &mut NoPose) {
        match self {
            Self::NoSwim(value) => module.no_swim = value,
            Self::SneakHeight(value) => module.sneak_height = value,
        }
    }
}

pub fn serialize_no_pose(
    state: &NoPose,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::NoPose, common);
    values.push(tree.leaf(
        "NoSwim",
        "BOOLEAN",
        json!(state.no_swim),
        "liquidbounce.module.noPose.noSwim",
        None,
    ));
    let mut height = tree.leaf(
        "SneakHeight",
        "CHOOSE",
        json!(state.sneak_height.tag()),
        "liquidbounce.module.noPose.sneakHeight",
        None,
    );
    if format == JsonFormat::Interop {
        height["choices"] = json!(["1.8", "1.9", "1.15"]);
    }
    values.push(height);
    Ok(tree.group("NoPose", "TOGGLEABLE", values))
}

pub(super) fn parse_no_pose(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "NoSwim", "NoPose.NoSwim", |value, path| {
        parse_bool(value, path).map(|value| SettingChange::NoPose(NoPoseSetting::NoSwim(value)))
    });
    leaf(
        patch,
        root,
        "SneakHeight",
        "NoPose.SneakHeight",
        |value, path| {
            let tag =
                scalar_string(value).ok_or_else(|| ConfigError::new(path, "expected enum tag"))?;
            [SneakHeight::V1_8, SneakHeight::V1_9, SneakHeight::V1_15]
                .into_iter()
                .find(|choice| choice.tag().eq_ignore_ascii_case(&tag))
                .map(|choice| SettingChange::NoPose(NoPoseSetting::SneakHeight(choice)))
                .ok_or_else(|| ConfigError::new(path, format!("unknown SneakHeight {tag:?}")))
        },
    );
}
