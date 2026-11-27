// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::mob_owners::MobOwners;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobOwnersSetting {
    Projectiles(bool),
}
impl MobOwnersSetting {
    pub fn apply(self, s: &mut MobOwners) {
        match self {
            Self::Projectiles(v) => s.projectiles = v,
        }
    }
}
pub fn serialize_mob_owners(
    s: &MobOwners,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::MobOwners, common);
    values.push(tree.leaf(
        "Projectiles",
        "BOOLEAN",
        json!(s.projectiles),
        "liquidbounce.module.mobOwners.projectiles",
        None,
    ));
    Ok(tree.group("MobOwners", "TOGGLEABLE", values))
}
pub(super) fn parse_mob_owners(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "Projectiles",
        "MobOwners.Projectiles",
        |v, p| parse_bool(v, p).map(|v| SettingChange::MobOwners(MobOwnersSetting::Projectiles(v))),
    );
}
