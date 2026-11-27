// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::block_outline::{BlockOutline, easing_tag};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockOutlineSetting {
    SideOnly(bool),
    Color(i32),
    Outline(i32),
    Slide(bool),
    Time(i32),
    Easing(Easing),
}
impl BlockOutlineSetting {
    pub fn apply(self, s: &mut BlockOutline) {
        match self {
            Self::SideOnly(v) => s.side_only = v,
            Self::Color(v) => s.color = v,
            Self::Outline(v) => s.outline = v,
            Self::Slide(v) => s.slide = v,
            Self::Time(v) => s.time = v,
            Self::Easing(v) => s.easing = v,
        }
    }
}
pub fn serialize_block_outline(
    s: &BlockOutline,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::BlockOutline, common);
    for (name, key, value) in [
        ("SideOnly", "sideOnly", json!(s.side_only)),
        ("Color", "color", json!(s.color)),
        ("Outline", "outline", json!(s.outline)),
    ] {
        values.push(t.leaf(
            name,
            if name == "SideOnly" {
                "BOOLEAN"
            } else {
                "COLOR"
            },
            value,
            &format!("liquidbounce.module.blockOutline.{key}"),
            None,
        ));
    }
    let mut easing = t.leaf(
        "Easing",
        "CHOOSE",
        json!(s.easing.tag()),
        "liquidbounce.module.blockOutline.slide.easing",
        None,
    );
    if format == JsonFormat::Interop {
        easing["choices"] = json!(Easing::ALL.map(Easing::tag));
    }
    values.push(t.group(
        "Slide",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.slide),
                "liquidbounce.module.blockOutline.slide.enabled",
                None,
            ),
            t.ranged(
                "Time",
                "INT",
                json!(s.time),
                json!({"from":1,"to":1000}),
                "ms",
                "liquidbounce.module.blockOutline.slide.time",
            ),
            easing,
        ],
    ));
    Ok(t.group("BlockOutline", "TOGGLEABLE", values))
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
pub(super) fn parse_block_outline(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "SideOnly", "BlockOutline.SideOnly", |v, p| {
        parse_bool(v, p).map(|v| SettingChange::BlockOutline(BlockOutlineSetting::SideOnly(v)))
    });
    for (name, setting) in [
        (
            "Color",
            BlockOutlineSetting::Color as fn(i32) -> BlockOutlineSetting,
        ),
        ("Outline", BlockOutlineSetting::Outline),
    ] {
        leaf(
            patch,
            root,
            name,
            &format!("BlockOutline.{name}"),
            |v, p| color(v, p).map(|v| SettingChange::BlockOutline(setting(v))),
        );
    }
    if let Some(node) = root.get("Slide") {
        match nodes(node, "Slide", "BlockOutline.Slide") {
            Ok(group) => {
                leaf(
                    patch,
                    &group,
                    "Enabled",
                    "BlockOutline.Slide.Enabled",
                    |v, p| {
                        parse_bool(v, p)
                            .map(|v| SettingChange::BlockOutline(BlockOutlineSetting::Slide(v)))
                    },
                );
                leaf(patch, &group, "Time", "BlockOutline.Slide.Time", |v, p| {
                    parse_int(v, p)
                        .map(|v| SettingChange::BlockOutline(BlockOutlineSetting::Time(v)))
                });
                leaf(
                    patch,
                    &group,
                    "Easing",
                    "BlockOutline.Slide.Easing",
                    |v, p| {
                        scalar_string(v)
                            .and_then(|tag| easing_tag(&tag))
                            .map(|v| SettingChange::BlockOutline(BlockOutlineSetting::Easing(v)))
                            .ok_or_else(|| ConfigError::new(p, "unknown Easing"))
                    },
                );
            }
            Err(e) => patch.issues.push(e),
        }
    }
}
