// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::combine_mobs::CombineMobs;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombineMobsSetting {
    CombineArmorStands(bool),
    CombineMinecarts(bool),
}
impl CombineMobsSetting {
    pub fn apply(self, s: &mut CombineMobs) {
        match self {
            Self::CombineArmorStands(v) => s.combine_armor_stands = v,
            Self::CombineMinecarts(v) => s.combine_minecarts = v,
        }
    }
}
pub fn serialize_combine_mobs(
    s: &CombineMobs,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::CombineMobs, common);
    for (name, key, v) in [
        (
            "CombineArmorStands",
            "combineArmorStands",
            s.combine_armor_stands,
        ),
        ("CombineMinecarts", "combineMinecarts", s.combine_minecarts),
    ] {
        values.push(tree.leaf(
            name,
            "BOOLEAN",
            json!(v),
            &format!("liquidbounce.module.combineMobs.{key}"),
            None,
        ));
    }
    Ok(tree.group("CombineMobs", "TOGGLEABLE", values))
}
pub(super) fn parse_combine_mobs(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "CombineArmorStands",
        "CombineMobs.CombineArmorStands",
        |v, p| {
            parse_bool(v, p)
                .map(|v| SettingChange::CombineMobs(CombineMobsSetting::CombineArmorStands(v)))
        },
    );
    leaf(
        patch,
        root,
        "CombineMinecarts",
        "CombineMobs.CombineMinecarts",
        |v, p| {
            parse_bool(v, p)
                .map(|v| SettingChange::CombineMobs(CombineMobsSetting::CombineMinecarts(v)))
        },
    );
}
