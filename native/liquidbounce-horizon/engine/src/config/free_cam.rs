// SPDX-License-Identifier: GPL-3.0-or-later
use super::{combat_rotations::CombatRotationSetting, *};
use crate::free_cam_runtime::FreeCamRuntime;
use liquidbounce_behaviors::free_cam::CANCEL_TAGS;
#[derive(Clone, Debug, PartialEq)]
pub enum FreeCamSetting {
    Speed(f32),
    Boolean(&'static str, bool),
    CancelOn([bool; 4]),
    NavigationKey(i32),
    Rotation(CombatRotationSetting),
    Navigation(AutoWalkSetting),
}
impl FreeCamSetting {
    pub fn apply(self, s: &mut FreeCamRuntime) -> Result<(), ConfigError> {
        match self {
            Self::Speed(v) => s.behavior.settings.speed = v,
            Self::CancelOn(v) => s.behavior.settings.cancel_on = v,
            Self::NavigationKey(v) => s.navigation_key = v,
            Self::Boolean(k, v) => match k {
                "AllowCameraInteract.Enabled" => s.behavior.settings.camera_interact = v,
                "AllowCameraInteract.LookAt" => s.behavior.settings.look_at = v,
                "MidClickCameraTeleport" => s.behavior.settings.mid_click_camera_teleport = v,
                "KeepSneaking" => s.behavior.settings.keep_sneaking = v,
                "Navigation.Enabled" => s.navigation_enabled = v,
                _ => return Err(ConfigError::new(k, "unknown FreeCam boolean")),
            },
            Self::Rotation(v) => return v.apply(&mut s.rotations),
            Self::Navigation(v) => v.apply(&mut s.navigation),
        }
        Ok(())
    }
}
fn remap(v: &mut Value, old: &str, new: &str) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(k)) = o.get_mut("key") {
                *k = k.replace(old, new);
            }
            for v in o.values_mut() {
                remap(v, old, new);
            }
        }
        Value::Array(a) => {
            for v in a {
                remap(v, old, new);
            }
        }
        _ => {}
    }
}
pub fn serialize_free_cam(
    s: &FreeCamRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let key = |tail: &str| format!("liquidbounce.module.freeCam.{tail}");
    let b = |name, value, tail: &str| t.leaf(name, "BOOLEAN", json!(value), &key(tail), None);
    let mut values = t.common(ModuleKind::FreeCam, c);
    values.push(t.ranged(
        "Speed",
        "FLOAT",
        float_json(s.behavior.settings.speed, "FreeCam.Speed")?,
        json!({"from":0.1,"to":2.0}),
        "",
        &key("speed"),
    ));
    let mut cancel = t.leaf(
        "CancelOn",
        "MULTI_CHOOSE",
        json!(
            CANCEL_TAGS
                .iter()
                .enumerate()
                .filter(|(i, _)| s.behavior.settings.cancel_on[*i])
                .map(|(_, s)| s)
                .collect::<Vec<_>>()
        ),
        &key("cancelOn"),
        None,
    );
    if f == JsonFormat::Interop {
        cancel["choices"] = json!(CANCEL_TAGS);
        cancel["canBeNone"] = json!(true);
    }
    values.push(cancel);
    values.push(b(
        "MidClickCameraTeleport",
        s.behavior.settings.mid_click_camera_teleport,
        "midClickCameraTeleport",
    ));
    values.push(b(
        "KeepSneaking",
        s.behavior.settings.keep_sneaking,
        "keepSneaking",
    ));
    values.push(combat_rotations::serialize(
        &s.rotations,
        c,
        f,
        d,
        &key("rotations"),
    )?);
    values.push(t.group(
        "AllowCameraInteract",
        "TOGGLEABLE",
        vec![
            b(
                "Enabled",
                s.behavior.settings.camera_interact,
                "allowCameraInteract.enabled",
            ),
            b(
                "LookAt",
                s.behavior.settings.look_at,
                "allowCameraInteract.lookAt",
            ),
        ],
    ));
    let old = "liquidbounce.module.autoWalk";
    let new = key("navigation");
    let nav =
        super::auto_walk::serialize_auto_walk(&s.navigation, c, f, &|k| d(&k.replace(old, &new)))?;
    let mut navigation = vec![b("Enabled", s.navigation_enabled, "navigation.enabled")];
    for mut v in nav["value"].as_array().cloned().unwrap_or_default() {
        if [
            "Mode",
            "Rotations",
            "Enabled",
            "Bind",
            "Hidden",
            "OnDisable",
            "OnEnable",
        ]
        .contains(&v["name"].as_str().unwrap_or(""))
        {
            continue;
        }
        remap(&mut v, old, &new);
        navigation.push(v);
    }
    navigation.push(t.leaf(
        "Key",
        "KEY",
        json!(s.navigation_key),
        &key("navigation.key"),
        None,
    ));
    values.push(t.group("Navigation", "TOGGLEABLE", navigation));
    Ok(t.group("FreeCam", "TOGGLEABLE", values))
}
pub(super) fn parse_free_cam(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let ch = |v| SettingChange::FreeCam(v);
    leaf(p, root, "Speed", "FreeCam.Speed", |v, path| {
        parse_float(v, path).map(|v| ch(FreeCamSetting::Speed(v)))
    });
    leaf(p, root, "CancelOn", "FreeCam.CancelOn", |v, path| {
        let a = v
            .as_array()
            .ok_or_else(|| ConfigError::new(path, "expected multi-choice array"))?;
        let mut selected = [false; 4];
        for v in a {
            let tag = scalar_string(v)
                .ok_or_else(|| ConfigError::new(path, "expected cancel trigger tag"))?;
            let i = CANCEL_TAGS
                .iter()
                .position(|t| t.eq_ignore_ascii_case(&tag))
                .ok_or_else(|| ConfigError::new(path, "unknown cancel trigger"))?;
            selected[i] = true;
        }
        Ok(ch(FreeCamSetting::CancelOn(selected)))
    });
    for (name, path) in [
        ("MidClickCameraTeleport", "MidClickCameraTeleport"),
        ("KeepSneaking", "KeepSneaking"),
    ] {
        leaf(p, root, name, &format!("FreeCam.{path}"), |v, p| {
            parse_bool(v, p).map(|v| ch(FreeCamSetting::Boolean(path, v)))
        });
    }
    if let Some(v) = root.get("Rotations") {
        let (changes, issues) = combat_rotations::parse(v, "FreeCam.Rotations");
        p.changes
            .extend(changes.into_iter().map(|v| ch(FreeCamSetting::Rotation(v))));
        p.issues.extend(issues);
    }
    for (group, fields) in [
        (
            "AllowCameraInteract",
            vec![
                ("Enabled", "AllowCameraInteract.Enabled"),
                ("LookAt", "AllowCameraInteract.LookAt"),
            ],
        ),
        ("Navigation", vec![("Enabled", "Navigation.Enabled")]),
    ] {
        let Some(v) = root.get(group) else {
            continue;
        };
        match nodes(v, group, &format!("FreeCam.{group}")) {
            Ok(n) => {
                for (name, path) in fields {
                    leaf(p, &n, name, &format!("FreeCam.{path}"), |v, p| {
                        parse_bool(v, p).map(|v| ch(FreeCamSetting::Boolean(path, v)))
                    });
                }
                if group == "Navigation" {
                    leaf(p, &n, "Key", "FreeCam.Navigation.Key", |v, path| {
                        parse_int(v, path).map(|v| ch(FreeCamSetting::NavigationKey(v)))
                    });
                    let mut q = SettingsPatch {
                        module: ModuleKind::AutoWalk,
                        changes: vec![],
                        issues: vec![],
                    };
                    super::auto_walk::parse_auto_walk(&mut q, &n);
                    p.changes.extend(q.changes.into_iter().filter_map(|c| {
                        if let SettingChange::AutoWalk(v) = c {
                            Some(ch(FreeCamSetting::Navigation(v)))
                        } else {
                            None
                        }
                    }));
                    p.issues.extend(q.issues.into_iter().map(|mut e| {
                        e.path = e.path.replace("AutoWalk", "FreeCam.Navigation");
                        e
                    }));
                }
            }
            Err(e) => p.issues.push(e),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_formats_preserve_complete_navigation_rotations_and_hidden_values() {
        let c = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Hold"}),
        };
        for f in [JsonFormat::Interop, JsonFormat::File] {
            let mut s = FreeCamRuntime::default();
            s.behavior.settings.speed = 1.75;
            s.behavior.settings.cancel_on = [true, false, true, false];
            s.behavior.settings.camera_interact = false;
            s.behavior.settings.look_at = false;
            s.navigation_enabled = true;
            s.navigation_key = 67;
            s.navigation.allow_break = false;
            s.navigation.render.ghost = false;
            s.navigation.render.route = 0xaabbccdd;
            s.rotations.basic.ticks_until_reset = 7;
            let tree = serialize_free_cam(&s, &c, f, &str::to_owned).unwrap();
            let parsed = parse_settings(ModuleKind::FreeCam, &tree).unwrap();
            assert!(parsed.issues.is_empty(), "{:?}", parsed.issues);
            let mut restored = FreeCamRuntime::default();
            for v in parsed.changes {
                if let SettingChange::FreeCam(v) = v {
                    v.apply(&mut restored).unwrap();
                }
            }
            assert_eq!(s.behavior.settings, restored.behavior.settings);
            assert_eq!(s.navigation_enabled, restored.navigation_enabled);
            assert_eq!(s.navigation_key, restored.navigation_key);
            assert_eq!(s.navigation, restored.navigation);
            assert_eq!(s.rotations, restored.rotations);
        }
    }
    #[test]
    fn malformed_trigger_does_not_hide_later_valid_sibling() {
        let tree = json!({"name":"FreeCam","value":[{"name":"CancelOn","value":["oops"]},{"name":"Speed","value":1.5}]});
        let p = parse_settings(ModuleKind::FreeCam, &tree).unwrap();
        assert_eq!(p.issues.len(), 1);
        assert!(
            p.changes
                .contains(&SettingChange::FreeCam(FreeCamSetting::Speed(1.5)))
        );
    }
}
