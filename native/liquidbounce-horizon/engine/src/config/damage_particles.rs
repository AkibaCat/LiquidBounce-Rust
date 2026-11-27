// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::damage_particles::{DamageParticles, TrackMode};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DamageParticlesSetting {
    TimeToLive(f32),
    Scale(f32),
    ScaleTransition(Easing),
    Displacement([f64; 3]),
    DisplacementTransition(Easing),
    TrackMode(TrackMode),
    Damage(i32),
    Death(i32),
    Heal(i32),
    MaxHealth(i32),
}
impl DamageParticlesSetting {
    pub fn apply(self, s: &mut DamageParticles) {
        match self {
            Self::TimeToLive(v) => s.ttl = v,
            Self::Scale(v) => s.scale = v,
            Self::ScaleTransition(v) => s.scale_transition = v,
            Self::Displacement(v) => s.displacement = v,
            Self::DisplacementTransition(v) => s.displacement_transition = v,
            Self::TrackMode(v) => s.track_mode = v,
            Self::Damage(v) => s.damage = v,
            Self::Death(v) => s.death = v,
            Self::Heal(v) => s.heal = v,
            Self::MaxHealth(v) => s.max_health = v,
        }
    }
}
fn key(name: &str) -> String {
    let mut s = String::from("liquidbounce.module.damageParticles");
    for part in name.split('.') {
        s.push('.');
        let mut c = part.chars();
        if let Some(f) = c.next() {
            s.extend(f.to_lowercase());
            s.extend(c);
        }
    }
    s
}
fn choose(t: &Tree, name: &str, value: &str, choices: &[&str]) -> Value {
    let mut v = t.leaf(name, "CHOOSE", json!(value), &key(name), None);
    if t.format == JsonFormat::Interop {
        v["choices"] = json!(choices);
    }
    v
}
pub fn serialize_damage_particles(
    s: &DamageParticles,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut v = t.common(ModuleKind::DamageParticles, common);
    v.push(t.ranged(
        "TimeToLive",
        "FLOAT",
        float_json(s.ttl, "DamageParticles.TimeToLive")?,
        json!({"from":0.5,"to":5}),
        "s",
        &key("TimeToLive"),
    ));
    v.push(t.ranged(
        "Scale",
        "FLOAT",
        float_json(s.scale, "DamageParticles.Scale")?,
        json!({"from":0.25,"to":4}),
        "",
        &key("Scale"),
    ));
    let easings = Easing::ALL.map(Easing::tag);
    v.push(choose(
        &t,
        "ScaleTransition",
        s.scale_transition.tag(),
        &easings,
    ));
    if s.displacement.iter().any(|n| !n.is_finite()) {
        return Err(ConfigError::new(
            "DamageParticles.Displacement",
            "nonfinite vector",
        ));
    }
    let mut vector = t.leaf(
        "Displacement",
        "VECTOR3_D",
        json!({"x":s.displacement[0],"y":s.displacement[1],"z":s.displacement[2]}),
        &key("Displacement"),
        None,
    );
    if format == JsonFormat::Interop {
        vector["useLocateButton"] = json!(true);
    }
    v.push(vector);
    v.push(choose(
        &t,
        "DisplacementTransition",
        s.displacement_transition.tag(),
        &easings,
    ));
    v.push(choose(
        &t,
        "TrackMode",
        s.track_mode.tag(),
        &["OnTick", "OnUpdate"],
    ));
    v.push(
        t.group(
            "Colors",
            "CONFIGURABLE",
            [
                ("Damage", s.damage),
                ("Death", s.death),
                ("Heal", s.heal),
                ("MaxHealth", s.max_health),
            ]
            .into_iter()
            .map(|(n, c)| t.leaf(n, "COLOR", json!(c), &key(&format!("Colors.{n}")), None))
            .collect(),
        ),
    );
    Ok(t.group("DamageParticles", "TOGGLEABLE", v))
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
pub(super) fn parse_damage_particles(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for (n, setting) in [
        (
            "TimeToLive",
            DamageParticlesSetting::TimeToLive as fn(f32) -> DamageParticlesSetting,
        ),
        ("Scale", DamageParticlesSetting::Scale),
    ] {
        leaf(patch, root, n, &format!("DamageParticles.{n}"), |v, p| {
            parse_float(v, p).map(|v| SettingChange::DamageParticles(setting(v)))
        });
    }
    for (n, setting) in [
        (
            "ScaleTransition",
            DamageParticlesSetting::ScaleTransition as fn(Easing) -> DamageParticlesSetting,
        ),
        (
            "DisplacementTransition",
            DamageParticlesSetting::DisplacementTransition,
        ),
    ] {
        leaf(patch, root, n, &format!("DamageParticles.{n}"), |v, p| {
            scalar_string(v)
                .and_then(|tag| {
                    Easing::ALL
                        .into_iter()
                        .find(|e| e.tag().eq_ignore_ascii_case(&tag))
                })
                .map(|v| SettingChange::DamageParticles(setting(v)))
                .ok_or_else(|| ConfigError::new(p, "unknown Easing"))
        });
    }
    leaf(
        patch,
        root,
        "Displacement",
        "DamageParticles.Displacement",
        |v, p| {
            let mut a = [0.; 3];
            for (i, n) in ["x", "y", "z"].into_iter().enumerate() {
                a[i] = match &v[n] {
                    Value::Number(n) => n.to_string().parse::<f64>().ok(),
                    Value::String(s) => s.parse::<f64>().ok(),
                    _ => None,
                }
                .filter(|n| n.is_finite())
                .ok_or_else(|| ConfigError::new(p, "expected finite x/y/z vector"))?;
            }
            Ok(SettingChange::DamageParticles(
                DamageParticlesSetting::Displacement(a),
            ))
        },
    );
    leaf(
        patch,
        root,
        "TrackMode",
        "DamageParticles.TrackMode",
        |v, p| {
            scalar_string(v)
                .and_then(|t| {
                    [TrackMode::OnTick, TrackMode::OnUpdate]
                        .into_iter()
                        .find(|m| m.tag().eq_ignore_ascii_case(&t))
                })
                .map(|m| SettingChange::DamageParticles(DamageParticlesSetting::TrackMode(m)))
                .ok_or_else(|| ConfigError::new(p, "unknown tracking mode"))
        },
    );
    if let Some(node) = root.get("Colors") {
        match nodes(node, "Colors", "DamageParticles.Colors") {
            Ok(group) => {
                for (n, setting) in [
                    (
                        "Damage",
                        DamageParticlesSetting::Damage as fn(i32) -> DamageParticlesSetting,
                    ),
                    ("Death", DamageParticlesSetting::Death),
                    ("Heal", DamageParticlesSetting::Heal),
                    ("MaxHealth", DamageParticlesSetting::MaxHealth),
                ] {
                    leaf(
                        patch,
                        &group,
                        n,
                        &format!("DamageParticles.Colors.{n}"),
                        |v, p| color(v, p).map(|v| SettingChange::DamageParticles(setting(v))),
                    );
                }
            }
            Err(e) => patch.issues.push(e),
        }
    }
}
