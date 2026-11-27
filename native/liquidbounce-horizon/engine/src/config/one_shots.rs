// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::auto_leave::{KickMode, Settings};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OneShotSetting {
    Kick(KickMode),
    Health(f32),
    Delay([i32; 2]),
    LeaveMode(KickMode),
}
fn mode(tree: &Tree<'_>, value: KickMode, key: &str) -> Value {
    let mut value = tree.leaf("Mode", "CHOOSE", json!(value.tag()), key, None);
    if tree.format == JsonFormat::Interop {
        value["choices"] = json!(KickMode::ALL.map(KickMode::tag));
    }
    value
}
pub fn serialize_kick(
    value: KickMode,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::Kick, common);
    values.push(mode(&tree, value, "liquidbounce.module.kick.mode"));
    Ok(tree.group("Kick", "TOGGLEABLE", values))
}
pub fn serialize_auto_leave(
    settings: &Settings,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::AutoLeave, common);
    values.push(tree.ranged(
        "Health",
        "FLOAT",
        float_json(settings.health, "AutoLeave.Health")?,
        json!({"from":0.0,"to":20.0}),
        "HP",
        "liquidbounce.module.autoLeave.health",
    ));
    values.push(tree.ranged(
        "Delay",
        "INT_RANGE",
        json!({"from":settings.delay[0],"to":settings.delay[1]}),
        json!({"from":0,"to":60}),
        "ticks",
        "liquidbounce.module.autoLeave.delay",
    ));
    values.push(mode(
        &tree,
        settings.mode,
        "liquidbounce.module.autoLeave.mode",
    ));
    Ok(tree.group("AutoLeave", "TOGGLEABLE", values))
}
fn parse_mode(value: &Value, path: &str) -> Result<KickMode, ConfigError> {
    value
        .as_str()
        .and_then(KickMode::parse)
        .ok_or_else(|| ConfigError::new(path, "unknown Kick mode"))
}
pub(super) fn parse_one_shots(patch: &mut SettingsPatch, root: &Nodes<'_>, kind: ModuleKind) {
    if kind == ModuleKind::Kick {
        leaf(patch, root, "Mode", "Kick.Mode", |v, p| {
            parse_mode(v, p).map(|v| SettingChange::OneShot(OneShotSetting::Kick(v)))
        });
    } else {
        leaf(patch, root, "Health", "AutoLeave.Health", |v, p| {
            parse_float(v, p).map(|v| SettingChange::OneShot(OneShotSetting::Health(v)))
        });
        leaf(patch, root, "Delay", "AutoLeave.Delay", |v, p| {
            parse_int_pair(v, p).map(|v| SettingChange::OneShot(OneShotSetting::Delay(v)))
        });
        leaf(patch, root, "Mode", "AutoLeave.Mode", |v, p| {
            parse_mode(v, p).map(|v| SettingChange::OneShot(OneShotSetting::LeaveMode(v)))
        });
    }
}
