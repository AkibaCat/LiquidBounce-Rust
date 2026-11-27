// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::clip::Clip;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClipSetting {
    Mode(u8),
    Horizontal(i32),
    Vertical(i32),
    Liquids(u32),
    Stand(bool),
    OldHorizontal(f32),
    OldVertical(f32),
    Reset(bool),
}
impl ClipSetting {
    pub fn apply(self, s: &mut Clip) {
        match self {
            Self::Mode(v) => {
                if s.mode != v {
                    s.mode = v;
                    s.cancel();
                }
            }
            Self::Horizontal(v) => s.horizontal = v,
            Self::Vertical(v) => s.vertical = v,
            Self::Liquids(v) => s.liquids = v,
            Self::Stand(v) => s.requires_stand_on = v,
            Self::OldHorizontal(v) => s.old_horizontal = v,
            Self::OldVertical(v) => s.old_vertical = v,
            Self::Reset(v) => s.reset_velocity = v,
        }
    }
}
pub fn serialize_clip(
    s: &Clip,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::Clip, c);
    let mut choices = Map::new();
    choices.insert(
        "Fancy".into(),
        t.group(
            "Fancy",
            "CONFIGURABLE",
            vec![
                t.ranged(
                    "Horizontal",
                    "INT",
                    json!(s.horizontal),
                    json!({"from":0,"to":6}),
                    "blocks",
                    "liquidbounce.module.clip.choice.fancy.horizontal",
                ),
                t.ranged(
                    "Vertical",
                    "INT",
                    json!(s.vertical),
                    json!({"from":0,"to":10}),
                    "blocks",
                    "liquidbounce.module.clip.choice.fancy.vertical",
                ),
                t.multi(
                    "AllowedLiquids",
                    s.liquids,
                    &["Water", "Lava"],
                    "liquidbounce.module.clip.choice.fancy.allowedLiquids",
                ),
                t.leaf(
                    "RequiresStandOn",
                    "BOOLEAN",
                    json!(s.requires_stand_on),
                    "liquidbounce.module.clip.choice.fancy.requiresStandOn",
                    None,
                ),
            ],
        ),
    );
    choices.insert(
        "Old".into(),
        t.group(
            "Old",
            "CONFIGURABLE",
            vec![
                t.ranged(
                    "Horizontal",
                    "FLOAT",
                    float_json(s.old_horizontal, "Clip.Choice.Old.Horizontal")?,
                    json!({"from":-10.0,"to":10.0}),
                    "blocks",
                    "liquidbounce.module.clip.choice.old.horizontal",
                ),
                t.ranged(
                    "Vertical",
                    "FLOAT",
                    float_json(s.old_vertical, "Clip.Choice.Old.Vertical")?,
                    json!({"from":-10.0,"to":10.0}),
                    "blocks",
                    "liquidbounce.module.clip.choice.old.vertical",
                ),
                t.leaf(
                    "ResetVelocity",
                    "BOOLEAN",
                    json!(s.reset_velocity),
                    "liquidbounce.module.clip.choice.old.resetVelocity",
                    None,
                ),
            ],
        ),
    );
    let mut choice = t.group("Choice", "CHOICE", vec![]);
    choice["active"] = json!(if s.mode == 0 { "Fancy" } else { "Old" });
    choice["choices"] = json!(choices);
    values.push(choice);
    Ok(t.group("Clip", "TOGGLEABLE", values))
}
pub(super) fn parse_clip(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Choice") else {
        return;
    };
    match mode.get("active").and_then(scalar_string).as_deref() {
        Some("Fancy") => p.changes.push(SettingChange::Clip(ClipSetting::Mode(0))),
        Some("Old") => p.changes.push(SettingChange::Clip(ClipSetting::Mode(1))),
        _ => p
            .issues
            .push(ConfigError::new("Clip.Choice.active", "unknown choice")),
    }
    if let Err(e) = nodes(mode, "Choice", "Clip.Choice") {
        p.issues.push(e);
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        p.issues
            .push(ConfigError::new("Clip.Choice.choices", "expected choices"));
        return;
    };
    for tag in ["Fancy", "Old"] {
        let path = format!("Clip.Choice.{tag}");
        let Some(v) = choices.get(tag) else {
            p.issues.push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        let v = match nodes(v, tag, &path) {
            Ok(v) => v,
            Err(e) => {
                p.issues.push(e);
                continue;
            }
        };
        if tag == "Fancy" {
            leaf(
                p,
                &v,
                "Horizontal",
                &format!("{path}.Horizontal"),
                |v, p| parse_int(v, p).map(|v| SettingChange::Clip(ClipSetting::Horizontal(v))),
            );
            leaf(p, &v, "Vertical", &format!("{path}.Vertical"), |v, p| {
                parse_int(v, p).map(|v| SettingChange::Clip(ClipSetting::Vertical(v)))
            });
            leaf(
                p,
                &v,
                "RequiresStandOn",
                &format!("{path}.RequiresStandOn"),
                |v, p| parse_bool(v, p).map(|v| SettingChange::Clip(ClipSetting::Stand(v))),
            );
            leaf(
                p,
                &v,
                "AllowedLiquids",
                &format!("{path}.AllowedLiquids"),
                |v, p| {
                    parse_multi_mask(v, p, &["Water", "Lava"])
                        .map(|v| SettingChange::Clip(ClipSetting::Liquids(v)))
                },
            );
        } else {
            leaf(
                p,
                &v,
                "Horizontal",
                &format!("{path}.Horizontal"),
                |v, p| {
                    parse_float(v, p).map(|v| SettingChange::Clip(ClipSetting::OldHorizontal(v)))
                },
            );
            leaf(p, &v, "Vertical", &format!("{path}.Vertical"), |v, p| {
                parse_float(v, p).map(|v| SettingChange::Clip(ClipSetting::OldVertical(v)))
            });
            leaf(
                p,
                &v,
                "ResetVelocity",
                &format!("{path}.ResetVelocity"),
                |v, p| parse_bool(v, p).map(|v| SettingChange::Clip(ClipSetting::Reset(v))),
            );
        }
    }
}
