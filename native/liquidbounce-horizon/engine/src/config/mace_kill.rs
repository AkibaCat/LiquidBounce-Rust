// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::mace_kill::MaceKill;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaceKillSetting {
    FallHeight(i32),
}
impl MaceKillSetting {
    pub fn apply(self, module: &mut MaceKill) {
        match self {
            Self::FallHeight(value) => module.fall_height = value,
        }
    }
}
pub fn serialize_mace_kill(
    module: &MaceKill,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::MaceKill, common);
    values.push(tree.ranged(
        "FallHeight",
        "INT",
        json!(module.fall_height),
        json!({"from":1,"to":170}),
        "",
        "liquidbounce.module.maceKill.fallHeight",
    ));
    Ok(tree.group("MaceKill", "TOGGLEABLE", values))
}
pub(super) fn parse_mace_kill(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "FallHeight",
        "MaceKill.FallHeight",
        |value, path| {
            parse_int(value, path)
                .map(|value| SettingChange::MaceKill(MaceKillSetting::FallHeight(value)))
        },
    );
}
