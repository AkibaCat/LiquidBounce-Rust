// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogoffSpotSetting(pub bool);
pub fn serialize_logoff_spot(
    send_in_chat: bool,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::LogoffSpot, common);
    values.push(tree.leaf(
        "SendInChat",
        "BOOLEAN",
        json!(send_in_chat),
        "liquidbounce.module.logoffSpot.sendInChat",
        None,
    ));
    Ok(tree.group("LogoffSpot", "TOGGLEABLE", values))
}
pub(super) fn parse_logoff_spot(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "SendInChat",
        "LogoffSpot.SendInChat",
        |v, p| parse_bool(v, p).map(|v| SettingChange::LogoffSpot(LogoffSpotSetting(v))),
    );
}
