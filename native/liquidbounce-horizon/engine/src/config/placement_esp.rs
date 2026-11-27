// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    placement_renderer::PlacementRenderer, prophunt_esp::ProphuntEsp, void_esp::VoidEsp,
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlacementSetting {
    Enabled(bool),
    Clump(bool),
    StartSize(f32),
    StartCurve(Easing),
    EndSize(f32),
    EndCurve(Easing),
    FadeInCurve(Easing),
    FadeOutCurve(Easing),
    InTime(i32),
    OutTime(i32),
    Color(i32),
    OutlineColor(i32),
}
impl PlacementSetting {
    pub fn apply(self, s: &mut PlacementRenderer) {
        match self {
            Self::Enabled(v) => s.set_enabled(v),
            Self::Clump(v) => s.clump = v,
            Self::StartSize(v) => s.start_size = v,
            Self::StartCurve(v) => s.start_curve = v,
            Self::EndSize(v) => s.end_size = v,
            Self::EndCurve(v) => s.end_curve = v,
            Self::FadeInCurve(v) => s.fade_in_curve = v,
            Self::FadeOutCurve(v) => s.fade_out_curve = v,
            Self::InTime(v) => s.in_time = v,
            Self::OutTime(v) => s.out_time = v,
            Self::Color(v) => s.color = v,
            Self::OutlineColor(v) => s.outline = v,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProphuntEspSetting {
    Tracking(u32),
    Render(PlacementSetting),
}
impl ProphuntEspSetting {
    pub fn apply(self, s: &mut ProphuntEsp) {
        match self {
            Self::Tracking(v) => s.tracking = v,
            Self::Render(v) => v.apply(&mut s.render),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoidEspSetting {
    YThreshold(i32),
    RangeSide(i32),
    RangeFacing(i32),
    Render(PlacementSetting),
}
impl VoidEspSetting {
    pub fn apply(self, s: &mut VoidEsp) {
        match self {
            Self::YThreshold(v) => s.y_threshold = v,
            Self::RangeSide(v) => s.range_side = v,
            Self::RangeFacing(v) => s.range_facing = v,
            Self::Render(v) => v.apply(&mut s.render),
        }
    }
}
fn key(module: &str, path: &str) -> String {
    let mut s = format!("liquidbounce.module.{module}");
    for part in path.split('.') {
        let mut c = part.chars();
        s.push('.');
        if let Some(f) = c.next() {
            s.extend(f.to_lowercase());
            s.extend(c);
        }
    }
    s
}
fn choose(t: &Tree, name: &str, e: Easing, module: &str, group: &str) -> Value {
    let mut v = t.leaf(
        name,
        "CHOOSE",
        json!(e.tag()),
        &key(module, &format!("{group}.{name}")),
        None,
    );
    if t.format == JsonFormat::Interop {
        v["choices"] = json!(Easing::ALL.map(Easing::tag));
    }
    v
}
pub(super) fn placement_tree(
    t: &Tree,
    s: &PlacementRenderer,
    module: &str,
    group: &str,
) -> Result<Value, ConfigError> {
    let k = |n: &str| key(module, &format!("{group}.{n}"));
    let values = vec![
        t.leaf("Enabled", "BOOLEAN", json!(s.enabled), &k("Enabled"), None),
        t.leaf("Clump", "BOOLEAN", json!(s.clump), &k("Clump"), None),
        t.ranged(
            "StartSize",
            "FLOAT",
            float_json(s.start_size, "PlacementRenderer.StartSize")?,
            json!({"from":0,"to":2}),
            "",
            &k("StartSize"),
        ),
        choose(t, "StartCurve", s.start_curve, module, group),
        t.ranged(
            "EndSize",
            "FLOAT",
            float_json(s.end_size, "PlacementRenderer.EndSize")?,
            json!({"from":0,"to":2}),
            "",
            &k("EndSize"),
        ),
        choose(t, "EndCurve", s.end_curve, module, group),
        choose(t, "FadeInCurve", s.fade_in_curve, module, group),
        choose(t, "FadeOutCurve", s.fade_out_curve, module, group),
        t.ranged(
            "InTime",
            "INT",
            json!(s.in_time),
            json!({"from":0,"to":5000}),
            "ms",
            &k("InTime"),
        ),
        t.ranged(
            "OutTime",
            "INT",
            json!(s.out_time),
            json!({"from":0,"to":5000}),
            "ms",
            &k("OutTime"),
        ),
        t.leaf("Color", "COLOR", json!(s.color), &k("Color"), None),
        t.leaf(
            "OutlineColor",
            "COLOR",
            json!(s.outline),
            &k("OutlineColor"),
            None,
        ),
    ];
    Ok(t.group(group, "TOGGLEABLE", values))
}
pub fn serialize_prophunt_esp(
    s: &ProphuntEsp,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut v = t.common(ModuleKind::ProphuntEsp, common);
    let mut tracking = t.multi(
        "Tracking",
        s.tracking,
        &["FallingBlocks", "BlockUpdates", "ChunkDeltaUpdates"],
        &key("prophuntESP", "Tracking"),
    );
    if format == JsonFormat::Interop {
        tracking["canBeNone"] = json!(false);
    }
    v.push(tracking);
    v.push(placement_tree(
        &t,
        &s.render,
        "prophuntESP",
        "RenderBlockUpdates",
    )?);
    Ok(t.group("ProphuntESP", "TOGGLEABLE", v))
}
pub fn serialize_void_esp(
    s: &VoidEsp,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut v = t.common(ModuleKind::VoidEsp, common);
    for (name, value, lo, hi) in [
        ("YThreshold", s.y_threshold, 3, 32),
        ("RangeSide", s.range_side, 0, 32),
        ("RangeFacing", s.range_facing, 1, 32),
    ] {
        v.push(t.ranged(
            name,
            "INT",
            json!(value),
            json!({"from":lo,"to":hi}),
            "",
            &key("voidEsp", name),
        ));
    }
    v.push(placement_tree(&t, &s.render, "voidEsp", "Render")?);
    Ok(t.group("VoidESP", "TOGGLEABLE", v))
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
                .map_err(|_| ConfigError::new(p, "invalid color"))
        }
        _ => Err(ConfigError::new(p, "expected ARGB color")),
    }
}
pub(super) fn parse_placement(
    patch: &mut SettingsPatch,
    root: &Nodes<'_>,
    name: &str,
    module: &str,
    change: fn(PlacementSetting) -> SettingChange,
) {
    let Some(group) = root.get(name) else {
        return;
    };
    let group = match nodes(group, name, &format!("{module}.{name}")) {
        Ok(g) => g,
        Err(e) => {
            patch.issues.push(e);
            return;
        }
    };
    for (n, set) in [
        (
            "Enabled",
            PlacementSetting::Enabled as fn(bool) -> PlacementSetting,
        ),
        ("Clump", PlacementSetting::Clump),
    ] {
        leaf(patch, &group, n, &format!("{module}.{name}.{n}"), |v, p| {
            parse_bool(v, p).map(|v| change(set(v)))
        });
    }
    for (n, set) in [
        (
            "StartSize",
            PlacementSetting::StartSize as fn(f32) -> PlacementSetting,
        ),
        ("EndSize", PlacementSetting::EndSize),
    ] {
        leaf(patch, &group, n, &format!("{module}.{name}.{n}"), |v, p| {
            parse_float(v, p).map(|v| change(set(v)))
        });
    }
    for (n, set) in [
        (
            "StartCurve",
            PlacementSetting::StartCurve as fn(Easing) -> PlacementSetting,
        ),
        ("EndCurve", PlacementSetting::EndCurve),
        ("FadeInCurve", PlacementSetting::FadeInCurve),
        ("FadeOutCurve", PlacementSetting::FadeOutCurve),
    ] {
        leaf(patch, &group, n, &format!("{module}.{name}.{n}"), |v, p| {
            scalar_string(v)
                .and_then(|tag| {
                    Easing::ALL
                        .into_iter()
                        .find(|e| e.tag().eq_ignore_ascii_case(&tag))
                })
                .map(|v| change(set(v)))
                .ok_or_else(|| ConfigError::new(p, "unknown Easing"))
        });
    }
    for (n, set) in [
        (
            "InTime",
            PlacementSetting::InTime as fn(i32) -> PlacementSetting,
        ),
        ("OutTime", PlacementSetting::OutTime),
    ] {
        leaf(patch, &group, n, &format!("{module}.{name}.{n}"), |v, p| {
            parse_int(v, p).map(|v| change(set(v)))
        });
    }
    for (n, set) in [
        (
            "Color",
            PlacementSetting::Color as fn(i32) -> PlacementSetting,
        ),
        ("OutlineColor", PlacementSetting::OutlineColor),
    ] {
        leaf(patch, &group, n, &format!("{module}.{name}.{n}"), |v, p| {
            color(v, p).map(|v| change(set(v)))
        });
    }
}
pub(super) fn parse_prophunt_esp(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Tracking", "ProphuntESP.Tracking", |v, p| {
        let mask = parse_multi_mask(
            v,
            p,
            &["FallingBlocks", "BlockUpdates", "ChunkDeltaUpdates"],
        )?;
        Ok(SettingChange::ProphuntEsp(ProphuntEspSetting::Tracking(
            if mask == 0 { 7 } else { mask },
        )))
    });
    parse_placement(patch, root, "RenderBlockUpdates", "ProphuntESP", |v| {
        SettingChange::ProphuntEsp(ProphuntEspSetting::Render(v))
    });
}
pub(super) fn parse_void_esp(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for (n, set) in [
        (
            "YThreshold",
            VoidEspSetting::YThreshold as fn(i32) -> VoidEspSetting,
        ),
        ("RangeSide", VoidEspSetting::RangeSide),
        ("RangeFacing", VoidEspSetting::RangeFacing),
    ] {
        leaf(patch, root, n, &format!("VoidESP.{n}"), |v, p| {
            parse_int(v, p).map(|v| SettingChange::VoidEsp(set(v)))
        });
    }
    parse_placement(patch, root, "Render", "VoidESP", |v| {
        SettingChange::VoidEsp(VoidEspSetting::Render(v))
    });
}
