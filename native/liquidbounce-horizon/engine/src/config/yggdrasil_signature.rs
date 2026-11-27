// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
pub fn serialize_yggdrasil_signature(
    force_slim: bool,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::YggdrasilSignatureFix, common);
    values.push(tree.leaf(
        "ForceSlimModel",
        "BOOLEAN",
        json!(force_slim),
        "liquidbounce.module.yggdrasilSignatureFix.forceSlimModel",
        None,
    ));
    Ok(tree.group("YggdrasilSignatureFix", "TOGGLEABLE", values))
}
pub(super) fn parse_yggdrasil_signature(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "ForceSlimModel",
        "YggdrasilSignatureFix.ForceSlimModel",
        |v, p| parse_bool(v, p).map(SettingChange::YggdrasilSignatureFix),
    );
}
