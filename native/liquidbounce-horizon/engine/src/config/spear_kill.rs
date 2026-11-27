// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::spear_kill::SpearKill;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpearKillSetting {
    MaxTargetDistance(f32),
    MaxSpeed(f32),
    Preview(bool),
    FillColor(i32),
    OutlineColor(i32),
}
impl SpearKillSetting {
    pub fn apply(self, s: &mut SpearKill) {
        match self {
            Self::MaxTargetDistance(v) => s.settings.max_target_distance = v,
            Self::MaxSpeed(v) => s.settings.max_speed = v,
            Self::Preview(v) => s.settings.preview = v,
            Self::FillColor(v) => s.settings.fill_color = v,
            Self::OutlineColor(v) => s.settings.outline_color = v,
        }
    }
}
pub fn serialize_spear_kill(
    s: &SpearKill,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let s = s.settings;
    let mut values = t.common(ModuleKind::SpearKill, common);
    values.push(t.ranged(
        "MaxTargetDistance",
        "FLOAT",
        json!(s.max_target_distance),
        json!({"from":3.0,"to":200.0}),
        "",
        "liquidbounce.module.spearKill.maxTargetDistance",
    ));
    values.push(t.ranged(
        "MaxSpeed",
        "FLOAT",
        json!(s.max_speed),
        json!({"from":2.0,"to":10.0}),
        "",
        "liquidbounce.module.spearKill.maxSpeed",
    ));
    values.push(t.group(
        "Preview",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.preview),
                "liquidbounce.module.spearKill.preview.enabled",
                None,
            ),
            t.leaf(
                "FillColor",
                "COLOR",
                json!(s.fill_color),
                "liquidbounce.module.spearKill.preview.fillColor",
                None,
            ),
            t.leaf(
                "OutlineColor",
                "COLOR",
                json!(s.outline_color),
                "liquidbounce.module.spearKill.preview.outlineColor",
                None,
            ),
        ],
    ));
    Ok(t.group("SpearKill", "TOGGLEABLE", values))
}
fn color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    let fail = || ConfigError::new(p, "expected ARGB number or 6/8 digit hex");
    match v {
        Value::Number(n) => decimal_int(&n.to_string()).ok_or_else(fail),
        Value::String(s) => {
            let h = s.strip_prefix('#').unwrap_or(s);
            if !matches!(h.len(), 6 | 8) {
                return Err(fail());
            }
            let n = i64::from_str_radix(h, 16).map_err(|_| fail())?;
            Ok(if h.len() == 6 {
                n as i32 | 0xff000000u32 as i32
            } else {
                n as i32
            })
        }
        _ => Err(fail()),
    }
}
pub(super) fn parse_spear_kill(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for (name, setting) in [
        (
            "MaxTargetDistance",
            SpearKillSetting::MaxTargetDistance as fn(f32) -> SpearKillSetting,
        ),
        ("MaxSpeed", SpearKillSetting::MaxSpeed),
    ] {
        leaf(patch, root, name, &format!("SpearKill.{name}"), |v, p| {
            parse_float(v, p).map(|v| SettingChange::SpearKill(setting(v)))
        });
    }
    if let Some(node) = root.get("Preview") {
        match nodes(node, "Preview", "SpearKill.Preview") {
            Ok(group) => {
                leaf(
                    patch,
                    &group,
                    "Enabled",
                    "SpearKill.Preview.Enabled",
                    |v, p| {
                        parse_bool(v, p)
                            .map(|v| SettingChange::SpearKill(SpearKillSetting::Preview(v)))
                    },
                );
                for (name, setting) in [
                    (
                        "FillColor",
                        SpearKillSetting::FillColor as fn(i32) -> SpearKillSetting,
                    ),
                    ("OutlineColor", SpearKillSetting::OutlineColor),
                ] {
                    leaf(
                        patch,
                        &group,
                        name,
                        &format!("SpearKill.Preview.{name}"),
                        |v, p| color(v, p).map(|v| SettingChange::SpearKill(setting(v))),
                    );
                }
            }
            Err(e) => patch.issues.push(e),
        }
    }
}
