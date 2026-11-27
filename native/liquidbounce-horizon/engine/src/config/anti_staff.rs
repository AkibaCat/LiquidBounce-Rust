// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AntiStaffSetting(pub bool);
pub fn serialize_anti_staff(
    show: bool,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::AntiStaff, common);
    values.push(t.leaf(
        "ShowInTabList",
        "BOOLEAN",
        json!(show),
        "liquidbounce.module.antiStaff.showInTabList",
        None,
    ));
    Ok(t.group("AntiStaff", "TOGGLEABLE", values))
}
pub(super) fn parse_anti_staff(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "ShowInTabList",
        "AntiStaff.ShowInTabList",
        |v, p| parse_bool(v, p).map(|v| SettingChange::AntiStaff(AntiStaffSetting(v))),
    );
}
