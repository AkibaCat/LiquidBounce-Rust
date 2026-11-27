// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::target_strafe::{PRIORITIES, REQUIREMENTS, Settings};
#[derive(Clone, Debug, PartialEq)]
pub struct TargetStrafeSetting(pub String, pub Value);
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.targetStrafe.{}",
        path.split('.')
            .map(|p| if p == "FOV" {
                "fov".into()
            } else {
                p[..1].to_ascii_lowercase() + &p[1..]
            })
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn color(v: &Value, p: &str) -> Result<u32, ConfigError> {
    if let Some(s) = v.as_str() {
        let s = s.strip_prefix('#').unwrap_or(s);
        if !matches!(s.len(), 6 | 8) {
            return Err(ConfigError::new(p, "expected 6/8 hex digits"));
        }
        u32::from_str_radix(s, 16)
            .map(|n| n | if s.len() == 6 { 0xff000000 } else { 0 })
            .map_err(|_| ConfigError::new(p, "invalid ARGB color"))
    } else {
        v.as_i64()
            .map(|n| n as u32)
            .or_else(|| v.as_u64().map(|n| n as u32))
            .ok_or_else(|| ConfigError::new(p, "expected ARGB integer"))
    }
}
fn number(v: &Value, p: &str, lo: f32, hi: f32) -> Result<f32, ConfigError> {
    let n = parse_float(v, p)?;
    if n.is_finite() && (lo..=hi).contains(&n) {
        Ok(n)
    } else {
        Err(ConfigError::new(
            p,
            format!("expected number in {lo}..{hi}"),
        ))
    }
}
impl TargetStrafeSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = format!("TargetStrafe.{}", self.0);
        let v = &self.1;
        match self.0.as_str() {
            "Mode" => {
                s.input_mode = match scalar_string(v).as_deref() {
                    Some("Motion") => false,
                    Some("Input") => true,
                    _ => return Err(ConfigError::new(&p, "unknown case-sensitive mode")),
                }
            }
            "Requirements" => s.requirements = parse_multi_mask(v, &p, &REQUIREMENTS)?,
            "Target.Priority" => {
                let a = v
                    .as_array()
                    .map(Vec::as_slice)
                    .unwrap_or_else(|| std::slice::from_ref(v));
                let mut out = vec![];
                for n in a {
                    let tag = scalar_string(n)
                        .ok_or_else(|| ConfigError::new(&p, "expected target priority"))?;
                    let i = PRIORITIES
                        .iter()
                        .position(|n| n.eq_ignore_ascii_case(&tag))
                        .ok_or_else(|| ConfigError::new(&p, "unknown target priority"))?;
                    if !out.contains(&i) {
                        out.push(i);
                    }
                }
                if out.is_empty() {
                    return Err(ConfigError::new(&p, "priority cannot be empty"));
                }
                s.priority = out;
            }
            "Range" => {
                s.range = {
                    let n = number(v, &p, 0f32, 8f32)?;
                    s.follow_range = s.follow_range.max(n);
                    n
                }
            }
            "FollowRange" => s.follow_range = (number(v, &p, 0f32, 10f32)?).max(s.range),
            "VerticalRange" => s.vertical_range = number(v, &p, 0f32, 8f32)?,
            "Target.FOV" => s.fov = number(v, &p, 0f32, 180f32)?,
            "Target.HurtTime" => {
                let n = parse_int(v, &p)?;
                if !(0..=10).contains(&n) {
                    return Err(ConfigError::new(&p, "expected integer in 0..10"));
                }
                s.hurt_time = n;
            }
            "Mode.Motion.Hypixel" => s.hypixel = parse_bool(v, &p)?,
            "Planner.Enabled" => s.planner = parse_bool(v, &p)?,
            "Planner.ControlDirection" => s.control_direction = parse_bool(v, &p)?,
            "Planner.Validation.Enabled" => s.validation = parse_bool(v, &p)?,
            "Planner.Validation.EdgeCheck.Enabled" => s.edge = parse_bool(v, &p)?,
            "Planner.Validation.EdgeCheck.MaxFallHeight" => {
                s.max_fall = number(v, &p, 0.1f32, 4f32)?
            }
            "Planner.Validation.VoidCheck.Enabled" => s.void = parse_bool(v, &p)?,
            "Planner.Validation.VoidCheck.SafetyExpand" => {
                s.safety_expand = number(v, &p, 0f32, 5f32)?
            }
            "Planner.AdaptiveRange.Enabled" => s.adaptive = parse_bool(v, &p)?,
            "Planner.AdaptiveRange.MaxRange" => s.max_range = number(v, &p, 1f32, 5f32)?,
            "Planner.AdaptiveRange.RangeStep" => s.range_step = number(v, &p, 0.1f32, 1f32)?,
            "Planner.Visibility.Enabled" => s.visibility = parse_bool(v, &p)?,
            "Planner.Visibility.SearchDistance" => {
                s.search_distance = number(v, &p, 0.5f32, 10f32)?
            }
            "Visuals.Enabled" => s.visuals = parse_bool(v, &p)?,
            "Visuals.Width" => s.width = number(v, &p, 0.01f32, 1f32)?,
            "Visuals.HeightOffset" => s.height_offset = number(v, &p, -1f32, 1f32)?,
            "Visuals.OuterColor" => s.outer = color(v, &p)?,
            "Visuals.InnerColor" => s.inner = color(v, &p)?,
            "Visuals.OutlineColor" => s.outline = color(v, &p)?,
            "Visuals.ShowNextPoint" => s.show_point = parse_bool(v, &p)?,
            "Visuals.PointRadius" => s.point_radius = number(v, &p, 0.05f32, 1f32)?,
            "Visuals.PointColor" => s.point = color(v, &p)?,
            "Visuals.PointOutlineColor" => s.point_outline = color(v, &p)?,
            "Visuals.InvalidPointColor" => s.invalid = color(v, &p)?,
            "Visuals.InvalidPointOutlineColor" => s.invalid_outline = color(v, &p)?,
            _ => return Err(ConfigError::new(&p, "unknown setting")),
        }
        Ok(())
    }
}
fn value(s: &Settings, path: &str) -> Value {
    match path {
        "Requirements" => json!(
            REQUIREMENTS
                .iter()
                .enumerate()
                .filter_map(|(i, n)| (s.requirements & (1 << i) != 0).then_some(*n))
                .collect::<Vec<_>>()
        ),
        "Target.Priority" => json!(
            s.priority
                .iter()
                .map(|i| PRIORITIES[*i])
                .collect::<Vec<_>>()
        ),
        "Range" => json!(s.range),
        "FollowRange" => json!(s.follow_range),
        "VerticalRange" => json!(s.vertical_range),
        "Target.FOV" => json!(s.fov),
        "Target.HurtTime" => json!(s.hurt_time),
        "Mode.Motion.Hypixel" => json!(s.hypixel),
        "Planner.Enabled" => json!(s.planner),
        "Planner.ControlDirection" => json!(s.control_direction),
        "Planner.Validation.Enabled" => json!(s.validation),
        "Planner.Validation.EdgeCheck.Enabled" => json!(s.edge),
        "Planner.Validation.EdgeCheck.MaxFallHeight" => json!(s.max_fall),
        "Planner.Validation.VoidCheck.Enabled" => json!(s.void),
        "Planner.Validation.VoidCheck.SafetyExpand" => json!(s.safety_expand),
        "Planner.AdaptiveRange.Enabled" => json!(s.adaptive),
        "Planner.AdaptiveRange.MaxRange" => json!(s.max_range),
        "Planner.AdaptiveRange.RangeStep" => json!(s.range_step),
        "Planner.Visibility.Enabled" => json!(s.visibility),
        "Planner.Visibility.SearchDistance" => json!(s.search_distance),
        "Visuals.Enabled" => json!(s.visuals),
        "Visuals.Width" => json!(s.width),
        "Visuals.HeightOffset" => json!(s.height_offset),
        "Visuals.OuterColor" => json!(s.outer),
        "Visuals.InnerColor" => json!(s.inner),
        "Visuals.OutlineColor" => json!(s.outline),
        "Visuals.ShowNextPoint" => json!(s.show_point),
        "Visuals.PointRadius" => json!(s.point_radius),
        "Visuals.PointColor" => json!(s.point),
        "Visuals.PointOutlineColor" => json!(s.point_outline),
        "Visuals.InvalidPointColor" => json!(s.invalid),
        "Visuals.InvalidPointOutlineColor" => json!(s.invalid_outline),
        _ => Value::Null,
    }
}
fn leaf(t: &Tree, s: &Settings, path: &str) -> Value {
    let name = path.rsplit('.').next().unwrap();
    let (kind, range) = match path {
        "Requirements" => ("MULTI_CHOOSE", None),
        "Target.Priority" => ("MULTI_CHOOSE", None),
        "Range" => ("FLOAT", Some(json!({"from":0,"to":8}))),
        "FollowRange" => ("FLOAT", Some(json!({"from":0,"to":10}))),
        "VerticalRange" => ("FLOAT", Some(json!({"from":0,"to":8}))),
        "Target.FOV" => ("FLOAT", Some(json!({"from":0,"to":180}))),
        "Target.HurtTime" => ("INT", Some(json!({"from":0,"to":10}))),
        "Mode.Motion.Hypixel" => ("BOOLEAN", None),
        "Planner.Enabled" => ("BOOLEAN", None),
        "Planner.ControlDirection" => ("BOOLEAN", None),
        "Planner.Validation.Enabled" => ("BOOLEAN", None),
        "Planner.Validation.EdgeCheck.Enabled" => ("BOOLEAN", None),
        "Planner.Validation.EdgeCheck.MaxFallHeight" => ("FLOAT", Some(json!({"from":0.1,"to":4}))),
        "Planner.Validation.VoidCheck.Enabled" => ("BOOLEAN", None),
        "Planner.Validation.VoidCheck.SafetyExpand" => ("FLOAT", Some(json!({"from":0,"to":5}))),
        "Planner.AdaptiveRange.Enabled" => ("BOOLEAN", None),
        "Planner.AdaptiveRange.MaxRange" => ("FLOAT", Some(json!({"from":1,"to":5}))),
        "Planner.AdaptiveRange.RangeStep" => ("FLOAT", Some(json!({"from":0.1,"to":1}))),
        "Planner.Visibility.Enabled" => ("BOOLEAN", None),
        "Planner.Visibility.SearchDistance" => ("FLOAT", Some(json!({"from":0.5,"to":10}))),
        "Visuals.Enabled" => ("BOOLEAN", None),
        "Visuals.Width" => ("FLOAT", Some(json!({"from":0.01,"to":1}))),
        "Visuals.HeightOffset" => ("FLOAT", Some(json!({"from":-1,"to":1}))),
        "Visuals.OuterColor" => ("COLOR", None),
        "Visuals.InnerColor" => ("COLOR", None),
        "Visuals.OutlineColor" => ("COLOR", None),
        "Visuals.ShowNextPoint" => ("BOOLEAN", None),
        "Visuals.PointRadius" => ("FLOAT", Some(json!({"from":0.05,"to":1}))),
        "Visuals.PointColor" => ("COLOR", None),
        "Visuals.PointOutlineColor" => ("COLOR", None),
        "Visuals.InvalidPointColor" => ("COLOR", None),
        "Visuals.InvalidPointOutlineColor" => ("COLOR", None),
        _ => ("TEXT", None),
    };
    let mut v = t.leaf(name, kind, value(s, path), &key(path), None);
    if t.format == JsonFormat::Interop {
        if let Some(r) = range {
            v["range"] = r;
            v["suffix"] = json!(
                if ["Range", "FollowRange", "VerticalRange"].contains(&path) {
                    "m"
                } else {
                    ""
                }
            );
        }
        if path == "Requirements" || path == "Target.Priority" {
            v["choices"] = if path == "Requirements" {
                json!(REQUIREMENTS)
            } else {
                json!(PRIORITIES)
            };
            v["canBeNone"] = json!(path == "Requirements");
            v["isOrderSensitive"] = json!(path == "Target.Priority");
        }
    }
    v
}
fn group(t: &Tree, s: &Settings, path: &str, children: &[&str], toggle: bool) -> Value {
    let mut v = t.group(
        path.rsplit('.').next().unwrap(),
        if toggle { "TOGGLEABLE" } else { "CONFIGURABLE" },
        children
            .iter()
            .map(|name| leaf(t, s, &format!("{path}.{name}")))
            .collect(),
    );
    if t.format == JsonFormat::Interop {
        v["key"] = json!(key(path));
        v["description"] = json!((t.describe)(&format!("{}.description", key(path))));
    }
    v
}
fn append(group: &mut Value, child: Value) {
    group["value"].as_array_mut().unwrap().push(child);
}
pub fn serialize_target_strafe(
    s: &Settings,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut out = t.common(ModuleKind::TargetStrafe, c);
    let mut mode = t.mode(
        if s.input_mode { "Input" } else { "Motion" },
        Map::from_iter([
            (
                "Motion".into(),
                group(&t, s, "Mode.Motion", &["Hypixel"], false),
            ),
            ("Input".into(), group(&t, s, "Mode.Input", &[], false)),
        ]),
    );
    if format == JsonFormat::Interop {
        mode["key"] = json!(key("Mode"));
        mode["description"] = json!(describe(&format!("{}.description", key("Mode"))));
    }
    out.push(mode);
    out.push(leaf(&t, s, "Range"));
    out.push(group(
        &t,
        s,
        "Target",
        &["FOV", "HurtTime", "Priority"],
        false,
    ));
    out.extend([
        leaf(&t, s, "FollowRange"),
        leaf(&t, s, "VerticalRange"),
        leaf(&t, s, "Requirements"),
    ]);
    let mut planner = group(&t, s, "Planner", &["Enabled", "ControlDirection"], true);
    let mut validation = group(&t, s, "Planner.Validation", &["Enabled"], true);
    append(
        &mut validation,
        group(
            &t,
            s,
            "Planner.Validation.EdgeCheck",
            &["Enabled", "MaxFallHeight"],
            true,
        ),
    );
    append(
        &mut validation,
        group(
            &t,
            s,
            "Planner.Validation.VoidCheck",
            &["Enabled", "SafetyExpand"],
            true,
        ),
    );
    append(&mut planner, validation);
    append(
        &mut planner,
        group(
            &t,
            s,
            "Planner.AdaptiveRange",
            &["Enabled", "MaxRange", "RangeStep"],
            true,
        ),
    );
    append(
        &mut planner,
        group(
            &t,
            s,
            "Planner.Visibility",
            &["Enabled", "SearchDistance"],
            true,
        ),
    );
    out.push(planner);
    out.push(group(
        &t,
        s,
        "Visuals",
        &[
            "Enabled",
            "Width",
            "HeightOffset",
            "OuterColor",
            "InnerColor",
            "OutlineColor",
            "ShowNextPoint",
            "PointRadius",
            "PointColor",
            "PointOutlineColor",
            "InvalidPointColor",
            "InvalidPointOutlineColor",
        ],
        true,
    ));
    Ok(t.group("TargetStrafe", "TOGGLEABLE", out))
}
fn add(patch: &mut SettingsPatch, path: &str, v: &Value) {
    let change = TargetStrafeSetting(path.into(), v.clone());
    match change.apply(&mut Settings::default()) {
        Ok(()) => patch.changes.push(SettingChange::TargetStrafe(change)),
        Err(e) => patch.issues.push(e),
    }
}
pub(super) fn parse_target_strafe(patch: &mut SettingsPatch, root: &Nodes) {
    fn walk(patch: &mut SettingsPatch, n: &Nodes, prefix: &str) {
        for (name, v) in n {
            let path = if prefix.is_empty() {
                (*name).to_string()
            } else {
                format!("{prefix}.{name}")
            };
            if path == "Mode" {
                if let Some(active) = v.get("active") {
                    add(patch, "Mode", active);
                } else {
                    patch.issues.push(ConfigError::new(
                        "TargetStrafe.Mode.active",
                        "missing active mode",
                    ));
                }
                if let Some(choices) = v["choices"].as_object() {
                    for name in ["Motion", "Input"] {
                        if let Some(choice) = choices.get(name) {
                            match nodes(choice, name, &format!("TargetStrafe.Mode.{name}")) {
                                Ok(n) => walk(patch, &n, &format!("Mode.{name}")),
                                Err(e) => patch.issues.push(e),
                            }
                        }
                    }
                }
            } else if v["value"].is_array()
                && v["value"]
                    .as_array()
                    .is_some_and(|a| a.iter().all(|v| v.get("name").is_some()))
                && !matches!(path.as_str(), "Requirements" | "Target.Priority")
            {
                match nodes(v, name, &format!("TargetStrafe.{path}")) {
                    Ok(n) => walk(patch, &n, &path),
                    Err(e) => patch.issues.push(e),
                }
            } else if !["Enabled", "Bind", "Hidden"].contains(&path.as_str()) {
                match value_of(v, &format!("TargetStrafe.{path}")) {
                    Ok(v) => add(patch, &path, v),
                    Err(e) => patch.issues.push(e),
                }
            }
        }
    }
    walk(patch, root, "");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inactive_mode_all_nested_trees_colors_and_ordered_priorities_roundtrip() {
        let mut s = Settings::default();
        s.input_mode = true;
        s.hypixel = true;
        s.range = 5.;
        s.follow_range = 7.;
        s.priority = vec![5, 0, 3];
        s.requirements = 15;
        s.adaptive = true;
        s.visuals = false;
        s.invalid = 0x80abcdef;
        for f in [JsonFormat::File, JsonFormat::Interop] {
            let tree = serialize_target_strafe(
                &s,
                &CommonSettings {
                    enabled: false,
                    hidden: false,
                    key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Toggle"}),
                },
                f,
                &str::to_owned,
            )
            .unwrap();
            let patch = parse_settings(ModuleKind::TargetStrafe, &tree).unwrap();
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            let mut restored = Settings::default();
            for c in patch.changes {
                if let SettingChange::TargetStrafe(c) = c {
                    c.apply(&mut restored).unwrap();
                }
            }
            assert_eq!(s, restored);
        }
    }
    #[test]
    fn source_range_callbacks_and_invalid_values_are_atomic() {
        let mut s = Settings::default();
        TargetStrafeSetting("Range".into(), json!(6.))
            .apply(&mut s)
            .unwrap();
        assert_eq!(s.follow_range, 6.);
        TargetStrafeSetting("FollowRange".into(), json!(2.))
            .apply(&mut s)
            .unwrap();
        assert_eq!(s.follow_range, 6.);
        let before = s.clone();
        for value in [json!(11), json!(-1), json!("invalid")] {
            assert!(
                TargetStrafeSetting("Target.HurtTime".into(), value)
                    .apply(&mut s)
                    .is_err()
            );
            assert_eq!(s, before);
        }
        assert!(
            TargetStrafeSetting("Target.Priority".into(), json!([]))
                .apply(&mut s)
                .is_err()
        );
        assert!(
            TargetStrafeSetting("Planner.AdaptiveRange.RangeStep".into(), json!(0.))
                .apply(&mut s)
                .is_err()
        );
        assert!(
            TargetStrafeSetting("Mode".into(), json!("input"))
                .apply(&mut s)
                .is_err()
        );
    }
}
