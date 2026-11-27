// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::murder_mystery::{MODES, State};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MurderMysterySetting {
    SetTeamPrefix(bool),
    Mode(usize),
}
impl MurderMysterySetting {
    pub fn apply(self, s: &mut State) {
        match self {
            Self::SetTeamPrefix(v) => s.set_team_prefix = v,
            Self::Mode(v) => s.set_mode(v),
        }
    }
}
pub fn serialize_murder_mystery(
    s: &State,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::MurderMystery, common);
    values.push(t.leaf(
        "SetTeamPrefix",
        "BOOLEAN",
        json!(s.set_team_prefix),
        "liquidbounce.module.murderMystery.setTeamPrefix",
        None,
    ));
    let choices = MODES
        .iter()
        .map(|tag| (tag.to_string(), t.group(tag, "CONFIGURABLE", vec![])))
        .collect();
    values.push(t.mode(MODES[s.mode], choices));
    Ok(t.group("MurderMystery", "TOGGLEABLE", values))
}
pub(super) fn parse_murder_mystery(p: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        p,
        root,
        "SetTeamPrefix",
        "MurderMystery.SetTeamPrefix",
        |v, path| {
            parse_bool(v, path)
                .map(|b| SettingChange::MurderMystery(MurderMysterySetting::SetTeamPrefix(b)))
        },
    );
    if let Some(mode) = root.get("Mode") {
        match mode
            .get("active")
            .and_then(scalar_string)
            .and_then(|s| MODES.iter().position(|m| *m == s))
        {
            Some(m) => p
                .changes
                .push(SettingChange::MurderMystery(MurderMysterySetting::Mode(m))),
            None => p.issues.push(ConfigError::new(
                "MurderMystery.Mode.active",
                "unknown mode",
            )),
        }
    }
}
