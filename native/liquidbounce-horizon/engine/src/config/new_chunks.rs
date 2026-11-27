// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::new_chunks::NewChunks;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NewChunksSetting {
    RenderDistance(i32),
    RenderY(f32),
    AutoY(bool),
    Smooth(bool),
    Persist(bool),
    NewColor(i32),
    OldColor(i32),
}
impl NewChunksSetting {
    pub fn apply(self, s: &mut NewChunks) {
        match self {
            Self::RenderDistance(v) => s.render_distance = v,
            Self::RenderY(v) => s.render_y = v,
            Self::AutoY(v) => s.auto_y = v,
            Self::Smooth(v) => s.smooth = v,
            Self::Persist(v) => s.persist = v,
            Self::NewColor(v) => s.new_color = v,
            Self::OldColor(v) => s.old_color = v,
        }
    }
}
fn key(s: &str) -> String {
    let mut c = s.chars();
    format!(
        "liquidbounce.module.newChunks.{}{}",
        c.next().unwrap().to_lowercase(),
        c.as_str()
    )
}
pub fn serialize_new_chunks(
    s: &NewChunks,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::NewChunks, common);
    values.push(t.ranged(
        "RenderDistance",
        "INT",
        json!(s.render_distance),
        json!({"from":4,"to":128}),
        "chunks",
        &key("RenderDistance"),
    ));
    values.push(t.ranged(
        "RenderY",
        "FLOAT",
        float_json(s.render_y, "NewChunks.RenderY")?,
        json!({"from":-64,"to":320}),
        "",
        &key("RenderY"),
    ));
    for (name, value) in [
        ("AutoY", s.auto_y),
        ("Smooth", s.smooth),
        ("Persist", s.persist),
    ] {
        values.push(t.leaf(name, "BOOLEAN", json!(value), &key(name), None));
    }
    for (name, value) in [("NewColor", s.new_color), ("OldColor", s.old_color)] {
        values.push(t.leaf(name, "COLOR", json!(value), &key(name), None));
    }
    Ok(t.group("NewChunks", "TOGGLEABLE", values))
}
fn change(v: NewChunksSetting) -> SettingChange {
    SettingChange::NewChunks(v)
}
fn color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    match v {
        Value::Number(n) => {
            decimal_int(&n.to_string()).ok_or_else(|| ConfigError::new(p, "expected ARGB integer"))
        }
        Value::String(s) => {
            let h = s.strip_prefix('#').unwrap_or(s);
            if !matches!(h.len(), 6 | 8) {
                return Err(ConfigError::new(p, "expected 6/8 hex digits"));
            }
            i64::from_str_radix(h, 16)
                .map(|v| {
                    if h.len() == 6 {
                        v as i32 | 0xff000000u32 as i32
                    } else {
                        v as i32
                    }
                })
                .map_err(|_| ConfigError::new(p, "invalid hex color"))
        }
        _ => Err(ConfigError::new(p, "expected ARGB color")),
    }
}
pub(super) fn parse_new_chunks(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "RenderDistance",
        "NewChunks.RenderDistance",
        |v, p| parse_int(v, p).map(|v| change(NewChunksSetting::RenderDistance(v))),
    );
    leaf(patch, root, "RenderY", "NewChunks.RenderY", |v, p| {
        parse_float(v, p).map(|v| change(NewChunksSetting::RenderY(v)))
    });
    for (name, set) in [
        (
            "AutoY",
            NewChunksSetting::AutoY as fn(bool) -> NewChunksSetting,
        ),
        ("Smooth", NewChunksSetting::Smooth),
        ("Persist", NewChunksSetting::Persist),
    ] {
        leaf(patch, root, name, &format!("NewChunks.{name}"), |v, p| {
            parse_bool(v, p).map(|v| change(set(v)))
        });
    }
    for (name, set) in [
        (
            "NewColor",
            NewChunksSetting::NewColor as fn(i32) -> NewChunksSetting,
        ),
        ("OldColor", NewChunksSetting::OldColor),
    ] {
        leaf(patch, root, name, &format!("NewChunks.{name}"), |v, p| {
            color(v, p).map(|v| change(set(v)))
        });
    }
}
