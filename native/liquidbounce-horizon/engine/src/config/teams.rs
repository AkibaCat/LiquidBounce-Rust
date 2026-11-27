// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::teams::{ARMOR_SLOTS, COLOR_SOURCES, ColorSource, MATCHES, Teams};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TeamsSetting {
    Matches(u32),
    ArmorColor(u32),
    ColorSources(Vec<ColorSource>),
}
impl TeamsSetting {
    pub fn apply(self, s: &mut Teams) {
        match self {
            Self::Matches(v) => s.matches = v,
            Self::ArmorColor(v) => s.armor_slots = v,
            Self::ColorSources(v) => s.color_sources = v,
        }
    }
}
pub fn serialize_teams(
    s: &Teams,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut v = t.common(ModuleKind::Teams, common);
    let mut matches = t.multi(
        "Matches",
        s.matches,
        &MATCHES,
        "liquidbounce.module.teams.matches",
    );
    let mut armor = t.multi(
        "ArmorColor",
        s.armor_slots,
        &ARMOR_SLOTS,
        "liquidbounce.module.teams.armorColor",
    );
    if format == JsonFormat::Interop {
        matches["canBeNone"] = json!(false);
        armor["canBeNone"] = json!(false);
    }
    v.push(matches);
    v.push(armor);
    let mut sources = t.leaf(
        "ColorSources",
        "MULTI_CHOOSE",
        json!(s.color_sources.iter().map(|s| s.tag()).collect::<Vec<_>>()),
        "liquidbounce.module.teams.colorSources",
        None,
    );
    if format == JsonFormat::Interop {
        sources["choices"] = json!(COLOR_SOURCES);
        sources["canBeNone"] = json!(true);
        sources["isOrderSensitive"] = json!(false);
    }
    v.push(sources);
    Ok(t.group("Teams", "TOGGLEABLE", v))
}
pub(super) fn parse_teams(p: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(p, root, "Matches", "Teams.Matches", |v, path| {
        parse_multi_mask(v, path, &MATCHES).map(|v| SettingChange::Teams(TeamsSetting::Matches(v)))
    });
    leaf(p, root, "ArmorColor", "Teams.ArmorColor", |v, path| {
        let mut v = v.clone();
        let remap = |v: &mut Value| {
            if let Some(s) = v.as_str() {
                for (alias, tag) in [
                    ("Boots", "Feet"),
                    ("Pants", "Legs"),
                    ("Chestplate", "Chest"),
                    ("Helmet", "Head"),
                ] {
                    if s.eq_ignore_ascii_case(alias) {
                        *v = json!(tag);
                        break;
                    }
                }
            }
        };
        if let Value::Array(a) = &mut v {
            for v in a {
                remap(v);
            }
        } else {
            remap(&mut v);
        }
        parse_multi_mask(&v, path, &ARMOR_SLOTS)
            .map(|v| SettingChange::Teams(TeamsSetting::ArmorColor(v)))
    });
    leaf(p, root, "ColorSources", "Teams.ColorSources", |v, path| {
        let mut sources = Vec::new();
        let mut add = |v: &Value| -> Result<(), ConfigError> {
            if !v.is_string() && !v.is_number() && !v.is_boolean() {
                return Err(ConfigError::new(
                    path,
                    "multichoice entries must be JSON primitives",
                ));
            }
            if let Some(s) = v.as_str().and_then(ColorSource::parse)
                && !sources.contains(&s)
            {
                sources.push(s);
            }
            Ok(())
        };
        if let Value::Array(a) = v {
            for v in a {
                add(v)?;
            }
        } else if !v.is_null() {
            add(v)?;
        }
        Ok(SettingChange::Teams(TeamsSetting::ColorSources(sources)))
    });
}
