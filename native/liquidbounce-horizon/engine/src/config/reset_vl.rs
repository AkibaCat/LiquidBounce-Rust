// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::reset_vl::ResetVl;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResetVlSetting {
    Times(i32),
    Timer(f32),
}
impl ResetVlSetting {
    pub fn apply(self, module: &mut ResetVl) {
        match self {
            Self::Times(value) => module.settings.times = value,
            Self::Timer(value) => module.settings.timer = value,
        }
    }
}
pub fn serialize_reset_vl(
    module: &ResetVl,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::ResetVl, common);
    values.push(tree.ranged(
        "Times",
        "INT",
        json!(module.settings.times),
        json!({"from":1,"to":50}),
        "",
        "liquidbounce.module.resetVL.times",
    ));
    values.push(tree.ranged(
        "Timer",
        "FLOAT",
        float_json(module.settings.timer, "ResetVL.Timer")?,
        json!({"from":0.1,"to":10.0}),
        "",
        "liquidbounce.module.resetVL.timer",
    ));
    Ok(tree.group("ResetVL", "TOGGLEABLE", values))
}
pub(super) fn parse_reset_vl(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Times", "ResetVL.Times", |value, path| {
        parse_int(value, path).map(|value| SettingChange::ResetVl(ResetVlSetting::Times(value)))
    });
    leaf(patch, root, "Timer", "ResetVL.Timer", |value, path| {
        parse_float(value, path).map(|value| SettingChange::ResetVl(ResetVlSetting::Timer(value)))
    });
}
