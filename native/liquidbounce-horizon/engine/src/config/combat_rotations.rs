// SPDX-License-Identifier: GPL-3.0-or-later
//! Reusable combat rotation tree, including the original unsupported-AI path.
use super::*;
use liquidbounce_behaviors::{
    combat_rotations::{Correction, Interpolation, Settings, SmoothMode},
    no_rotate_set::{FloatRange, NoRotateSet},
};
#[derive(Clone, Debug, PartialEq)]
pub enum CombatRotationSetting {
    Basic(NoRotateSetSetting),
    Value(String, Value),
}
impl CombatRotationSetting {
    pub fn apply(self, s: &mut Settings) -> Result<(), ConfigError> {
        let Self::Value(path, v) = self else {
            let Self::Basic(v) = self else { unreachable!() };
            let mut proxy = NoRotateSet {
                rotations: std::mem::take(&mut s.basic),
                ..Default::default()
            };
            v.apply(&mut proxy);
            s.basic = proxy.rotations;
            return Ok(());
        };
        let p = path.as_str();
        let range = |v: &Value| -> Result<FloatRange, ConfigError> {
            let r = parse_float_pair(v, p)?;
            Ok(FloatRange(r[0], r[1]))
        };
        if p == "AngleSmooth" {
            s.active = SmoothMode::ALL
                .into_iter()
                .find(|m| scalar_string(&v).as_deref() == Some(m.tag()))
                .ok_or_else(|| ConfigError::new(p, "unknown case-sensitive rotation mode"))?;
            return Ok(());
        }
        let i = if p.starts_with("AngleSmooth.Interpolation.") {
            Some((&mut s.interpolation, "AngleSmooth.Interpolation."))
        } else if p.starts_with("AngleSmooth.AI.Correction.Interpolation.") {
            Some((
                &mut s.ai.interpolation,
                "AngleSmooth.AI.Correction.Interpolation.",
            ))
        } else {
            None
        };
        if let Some((i, prefix)) = i {
            match &p[prefix.len()..] {
                "HorizontalSpeed" => i.horizontal = parse_int_pair(&v, p)?,
                "VerticalSpeed" => i.vertical = parse_int_pair(&v, p)?,
                "DirectionChangeFactor" => i.direction_change = parse_int_pair(&v, p)?,
                "Midpoint" => i.midpoint = parse_float(&v, p)?,
                _ => return Err(ConfigError::new(p, "unknown interpolation leaf")),
            };
            return Ok(());
        }
        match p {
            "AngleSmooth.AI.Model" => {
                s.ai.model = scalar_string(&v)
                    .filter(|v| ["21KC11KP", "19KC8KP"].contains(&v.as_str()))
                    .ok_or_else(|| ConfigError::new(p, "unknown case-sensitive model"))?
            }
            "AngleSmooth.AI.Correction" => {
                s.ai.correction = match scalar_string(&v).as_deref() {
                    Some("Interpolation") => Correction::Interpolation,
                    Some("Linear") => Correction::Linear,
                    Some("None") => Correction::None,
                    _ => {
                        return Err(ConfigError::new(
                            p,
                            "unknown case-sensitive correction mode",
                        ));
                    }
                }
            }
            "AngleSmooth.AI.Correction.Linear.HorizontalTurnSpeed" => s.ai.linear[0] = range(&v)?,
            "AngleSmooth.AI.Correction.Linear.VerticalTurnSpeed" => s.ai.linear[1] = range(&v)?,
            "AngleSmooth.AI.OutputMultiplier.Yaw" => s.ai.yaw_multiplier = parse_float(&v, p)?,
            "AngleSmooth.AI.OutputMultiplier.Pitch" => s.ai.pitch_multiplier = parse_float(&v, p)?,
            "ShortStop.Enabled" => s.short_stop.enabled = parse_bool(&v, p)?,
            "ShortStop.Rate" => s.short_stop.rate = parse_int(&v, p)?,
            "ShortStop.Duration" => s.short_stop.duration = parse_int_pair(&v, p)?,
            "Fail.Enabled" => s.fail.enabled = parse_bool(&v, p)?,
            "Fail.Rate" => s.fail.rate = parse_int(&v, p)?,
            "Fail.Factor" => s.fail.factor = parse_float(&v, p)?,
            "Fail.StrengthHorizontal" => s.fail.horizontal = range(&v)?,
            "Fail.StrengthVertical" => s.fail.vertical = range(&v)?,
            "Fail.TransitionInDuration" => s.fail.duration = parse_int_pair(&v, p)?,
            _ => return Err(ConfigError::new(p, "unknown combat rotation leaf")),
        }
        Ok(())
    }
}
const OLD: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
fn rename(v: &mut Value, base: &str) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(k)) = o.get_mut("key") {
                *k = k.replace(OLD, base);
            }
            for v in o.values_mut() {
                rename(v, base);
            }
        }
        Value::Array(a) => {
            for v in a {
                rename(v, base);
            }
        }
        _ => {}
    }
}
fn lower(s: &str) -> String {
    let mut c = s.chars();
    format!("{}{}", c.next().unwrap().to_lowercase(), c.as_str())
}
fn key(base: &str, path: &str) -> String {
    format!(
        "{base}.{}",
        path.split('.').map(lower).collect::<Vec<_>>().join(".")
    )
}
fn interpolation(t: &Tree<'_>, s: &Interpolation, path: &str, base: &str) -> Value {
    let mut values = vec![];
    for (name, v, from, to) in [
        ("HorizontalSpeed", s.horizontal, 1, 100),
        ("VerticalSpeed", s.vertical, 1, 100),
        ("DirectionChangeFactor", s.direction_change, 0, 100),
    ] {
        values.push(t.ranged(
            name,
            "INT_RANGE",
            json!({"from":v[0],"to":v[1]}),
            json!({"from":from,"to":to}),
            "%",
            &key(base, &format!("{path}.{name}")),
        ));
    }
    values.push(t.ranged(
        "Midpoint",
        "FLOAT",
        json!(s.midpoint),
        json!({"from":0.,"to":1.}),
        "",
        &key(base, &format!("{path}.Midpoint")),
    ));
    t.group("Interpolation", "CONFIGURABLE", values)
}
pub(super) fn serialize(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
    base: &str,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let proxy = NoRotateSet {
        rotations: s.basic.clone(),
        ..Default::default()
    };
    let mut value = serialize_no_rotate_set(&proxy, c, f, &|k| d(&k.replace(OLD, base)))?["value"]
        .as_array()
        .and_then(|a| a.iter().find(|v| v["name"] == "Mode"))
        .and_then(|v| v["choices"]["ResetRotation"]["value"].as_array())
        .and_then(|a| a.first())
        .cloned()
        .ok_or_else(|| ConfigError::new(base, "shared rotation tree changed"))?;
    rename(&mut value, base);
    let values = value["value"].as_array_mut().unwrap();
    let mut smoothing = values.remove(0);
    smoothing["active"] = json!(s.active.tag());
    let choices = smoothing["choices"].as_object_mut().unwrap();
    choices.insert(
        "Interpolation".into(),
        interpolation(&t, &s.interpolation, "AngleSmooth.Interpolation", base),
    );
    let mut models = Map::new();
    for name in ["21KC11KP", "19KC8KP"] {
        models.insert(name.into(), t.group(name, "CONFIGURABLE", vec![]));
    }
    let mut model = t.mode(&s.ai.model, models);
    model["name"] = json!("Model");
    let mut corrections = Map::new();
    corrections.insert(
        "Interpolation".into(),
        interpolation(
            &t,
            &s.ai.interpolation,
            "AngleSmooth.AI.Correction.Interpolation",
            base,
        ),
    );
    let mut linear = vec![];
    for (i, name) in ["HorizontalTurnSpeed", "VerticalTurnSpeed"]
        .into_iter()
        .enumerate()
    {
        linear.push(t.ranged(
            name,
            "FLOAT_RANGE",
            float_range(s.ai.linear[i].0, s.ai.linear[i].1, base)?,
            json!({"from":0.,"to":180.}),
            "°",
            &key(base, &format!("AngleSmooth.AI.Correction.Linear.{name}")),
        ));
    }
    corrections.insert("Linear".into(), t.group("Linear", "CONFIGURABLE", linear));
    corrections.insert("None".into(), t.group("None", "CONFIGURABLE", vec![]));
    let mut correction = t.mode(
        match s.ai.correction {
            Correction::Interpolation => "Interpolation",
            Correction::Linear => "Linear",
            Correction::None => "None",
        },
        corrections,
    );
    correction["name"] = json!("Correction");
    let multiplier = t.group(
        "OutputMultiplier",
        "CONFIGURABLE",
        [
            ("Yaw", s.ai.yaw_multiplier),
            ("Pitch", s.ai.pitch_multiplier),
        ]
        .into_iter()
        .map(|(name, v)| {
            t.ranged(
                name,
                "FLOAT",
                json!(v),
                json!({"from":0.5,"to":2.}),
                "",
                &key(base, &format!("AngleSmooth.AI.OutputMultiplier.{name}")),
            )
        })
        .collect(),
    );
    choices.insert(
        "AI".into(),
        t.group("AI", "CONFIGURABLE", vec![model, correction, multiplier]),
    );
    let bool_leaf = |p: &str, v: bool| {
        t.leaf(
            p.rsplit('.').next().unwrap(),
            "BOOLEAN",
            json!(v),
            &key(base, p),
            None,
        )
    };
    let int_leaf = |p: &str, v: i32, lo: i32, hi: i32| {
        t.ranged(
            p.rsplit('.').next().unwrap(),
            "INT",
            json!(v),
            json!({"from":lo,"to":hi}),
            "%",
            &key(base, p),
        )
    };
    let int_range = |p: &str, v: [i32; 2], lo: i32, hi: i32| {
        t.ranged(
            p.rsplit('.').next().unwrap(),
            "INT_RANGE",
            json!({"from":v[0],"to":v[1]}),
            json!({"from":lo,"to":hi}),
            "ticks",
            &key(base, p),
        )
    };
    let short = t.group(
        "ShortStop",
        "TOGGLEABLE",
        vec![
            bool_leaf("ShortStop.Enabled", s.short_stop.enabled),
            int_leaf("ShortStop.Rate", s.short_stop.rate, 1, 25),
            int_range("ShortStop.Duration", s.short_stop.duration, 1, 5),
        ],
    );
    let fail = t.group(
        "Fail",
        "TOGGLEABLE",
        vec![
            bool_leaf("Fail.Enabled", s.fail.enabled),
            int_leaf("Fail.Rate", s.fail.rate, 1, 100),
            t.ranged(
                "Factor",
                "FLOAT",
                json!(s.fail.factor),
                json!({"from":0.01,"to":0.99}),
                "",
                &key(base, "Fail.Factor"),
            ),
            t.ranged(
                "StrengthHorizontal",
                "FLOAT_RANGE",
                float_range(s.fail.horizontal.0, s.fail.horizontal.1, base)?,
                json!({"from":1.,"to":90.}),
                "°",
                &key(base, "Fail.StrengthHorizontal"),
            ),
            t.ranged(
                "StrengthVertical",
                "FLOAT_RANGE",
                float_range(s.fail.vertical.0, s.fail.vertical.1, base)?,
                json!({"from":0.,"to":90.}),
                "°",
                &key(base, "Fail.StrengthVertical"),
            ),
            int_range("Fail.TransitionInDuration", s.fail.duration, 0, 20),
        ],
    );
    values.splice(0..0, [smoothing, short, fail]);
    Ok(value)
}
pub(super) fn parse(root: &Value, prefix: &str) -> (Vec<CombatRotationSetting>, Vec<ConfigError>) {
    let mut proxy_root = root.clone();
    if let Some(v) = proxy_root["value"]
        .as_array_mut()
        .and_then(|a| a.iter_mut().find(|v| v["name"] == "AngleSmooth"))
    {
        if !["Linear", "Sigmoid", "Acceleration"]
            .iter()
            .any(|tag| v["active"] == *tag)
        {
            v["active"] = json!("Linear");
        }
    }
    let wrapped = json!({"name":"Mode","value":[],"active":"ResetRotation","choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[proxy_root]}}});
    let fake = json!({"name":"NoRotateSet","value":[wrapped]});
    let mut patch = SettingsPatch {
        module: ModuleKind::NoRotateSet,
        changes: vec![],
        issues: vec![],
    };
    if let Ok(root) = nodes(&fake, "NoRotateSet", "NoRotateSet") {
        no_rotate_set::parse_no_rotate_set(&mut patch, &root);
    }
    let mut changes = vec![];
    for change in patch.changes {
        if let SettingChange::NoRotateSet(v) = change {
            if !matches!(
                v,
                NoRotateSetSetting::Mode(_) | NoRotateSetSetting::AngleSmooth(_)
            ) {
                changes.push(CombatRotationSetting::Basic(v));
            }
        }
    }
    let mut issues: Vec<_> = patch
        .issues
        .into_iter()
        .map(|mut e| {
            e.path = e
                .path
                .replace("NoRotateSet.Mode.ResetRotation.Rotations", prefix);
            e
        })
        .collect();
    fn walk(
        n: &Value,
        p: &str,
        changes: &mut Vec<CombatRotationSetting>,
        issues: &mut Vec<ConfigError>,
        prefix: &str,
    ) {
        if n.get("active").is_some() {
            if p == "AngleSmooth" || p.starts_with("AngleSmooth.AI.") {
                changes.push(CombatRotationSetting::Value(p.into(), n["active"].clone()));
            }
            if let Some(choices) = n["choices"].as_object() {
                for (tag, v) in choices {
                    walk(v, &format!("{p}.{tag}"), changes, issues, prefix);
                }
            } else {
                issues.push(ConfigError::new(
                    &format!("{prefix}.{p}"),
                    "missing choices",
                ));
            }
            return;
        }
        if let Some(a) = n["value"].as_array() {
            for v in a {
                if let Some(name) = v["name"].as_str() {
                    walk(
                        v,
                        &if p.is_empty() {
                            name.into()
                        } else {
                            format!("{p}.{name}")
                        },
                        changes,
                        issues,
                        prefix,
                    );
                }
            }
            return;
        }
        if p.starts_with("AngleSmooth.Interpolation.")
            || p.starts_with("AngleSmooth.AI.")
            || p.starts_with("ShortStop.")
            || p.starts_with("Fail.")
        {
            changes.push(CombatRotationSetting::Value(p.into(), n["value"].clone()));
        }
    }
    walk(root, "", &mut changes, &mut issues, prefix);
    (changes, issues)
}
