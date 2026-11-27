// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::sneak::{MODES, Sneak};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SneakSetting {
    Mode(u8),
    NotDuringMove(bool),
    MagmaOnly(bool),
}
impl SneakSetting {
    pub fn apply(self, s: &mut Sneak) {
        match self {
            Self::Mode(v) => s.mode = v,
            Self::NotDuringMove(v) => s.not_during_move = v,
            Self::MagmaOnly(v) => s.magma_only = v,
        }
    }
}
pub fn serialize_sneak(
    s: &Sneak,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::Sneak, c);
    let mut choices = Map::new();
    for tag in MODES {
        let v = if tag == "Legit" {
            vec![t.leaf(
                "OnMagmaBlocksOnly",
                "BOOLEAN",
                json!(s.magma_only),
                "liquidbounce.module.sneak.mode.legit.onMagmaBlocksOnly",
                None,
            )]
        } else {
            vec![]
        };
        choices.insert(tag.into(), t.group(tag, "CONFIGURABLE", v));
    }
    let mut mode = t.group("Mode", "CHOICE", vec![]);
    mode["active"] = json!(s.tag());
    mode["choices"] = json!(choices);
    values.push(mode);
    values.push(t.leaf(
        "NotDuringMove",
        "BOOLEAN",
        json!(s.not_during_move),
        "liquidbounce.module.sneak.notDuringMove",
        None,
    ));
    Ok(t.group("Sneak", "TOGGLEABLE", values))
}
pub(super) fn parse_sneak(p: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(mode) = root.get("Mode") {
        match mode
            .get("active")
            .and_then(scalar_string)
            .and_then(|s| MODES.iter().position(|tag| *tag == s))
        {
            Some(v) => p
                .changes
                .push(SettingChange::Sneak(SneakSetting::Mode(v as u8))),
            None => p
                .issues
                .push(ConfigError::new("Sneak.Mode.active", "unknown choice")),
        }
        if let Some(v) = mode.get("choices").and_then(|v| v.get("Legit")) {
            match nodes(v, "Legit", "Sneak.Mode.Legit") {
                Ok(v) => leaf(
                    p,
                    &v,
                    "OnMagmaBlocksOnly",
                    "Sneak.Mode.Legit.OnMagmaBlocksOnly",
                    |v, p| {
                        parse_bool(v, p).map(|v| SettingChange::Sneak(SneakSetting::MagmaOnly(v)))
                    },
                ),
                Err(e) => p.issues.push(e),
            }
        }
    }
    leaf(p, root, "NotDuringMove", "Sneak.NotDuringMove", |v, p| {
        parse_bool(v, p).map(|v| SettingChange::Sneak(SneakSetting::NotDuringMove(v)))
    });
}
