// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::no_web_runtime::NoWebRuntime;
use liquidbounce_behaviors::{
    no_rotate_set::NoRotateSet,
    no_web::{MODES, Mode},
};
#[derive(Clone, Debug, PartialEq)]
pub enum NoWebSetting {
    Mode(Mode),
    BreakOnWorld(bool),
    Strength(f32),
    OnlyOnGround(bool),
    MotionY(bool),
    MotionYStrength(f32),
    Pickup(bool),
    PickupSpan([f32; 2]),
    Rotation(NoRotateSetSetting),
}
impl NoWebSetting {
    pub fn apply(self, s: &mut NoWebRuntime) {
        match self {
            Self::Mode(v) => {
                if s.behavior.settings.mode != v {
                    s.reset();
                    s.behavior.settings.mode = v;
                }
            }
            Self::BreakOnWorld(v) => s.behavior.settings.break_on_world = v,
            Self::Strength(v) => s.behavior.settings.strength = v,
            Self::OnlyOnGround(v) => s.behavior.settings.only_on_ground = v,
            Self::MotionY(v) => s.behavior.settings.motion_y = v,
            Self::MotionYStrength(v) => s.behavior.settings.motion_y_strength = v,
            Self::Pickup(v) => s.behavior.settings.pickup = v,
            Self::PickupSpan(v) => s.behavior.settings.pickup_span = v,
            Self::Rotation(v) => {
                let mut proxy = NoRotateSet {
                    rotations: std::mem::take(&mut s.rotations),
                    ..Default::default()
                };
                v.apply(&mut proxy);
                s.rotations = proxy.rotations;
            }
        }
    }
}
const OLD_KEY: &str = "liquidbounce.module.noRotateSet.mode.resetRotation.rotations";
const ROT_KEY: &str = "liquidbounce.module.noWeb.mode.placeWater.rotations";
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e
        .path
        .replace(
            "NoRotateSet.Mode.ResetRotation.Rotations",
            "NoWeb.Rotations",
        )
        .replace(OLD_KEY, ROT_KEY);
    e
}
fn rotation_tree(
    s: &NoWebRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let describe = |key: &str| d(&key.replace(OLD_KEY, ROT_KEY));
    let proxy = NoRotateSet {
        rotations: s.rotations.clone(),
        ..Default::default()
    };
    let mut tree = serialize_no_rotate_set(&proxy, c, f, &describe).map_err(remap)?;
    let mut rotations = tree["value"]
        .as_array_mut()
        .and_then(|v| v.iter_mut().find(|v| v["name"] == "Mode"))
        .and_then(|v| v["choices"]["ResetRotation"]["value"].as_array_mut())
        .and_then(|v| v.first_mut())
        .map(Value::take)
        .ok_or_else(|| {
            ConfigError::new("NoWeb.Rotations", "shared Rotations codec shape changed")
        })?;
    fn keys(v: &mut Value) {
        match v {
            Value::Object(o) => {
                if let Some(Value::String(k)) = o.get_mut("key") {
                    *k = k.replace(OLD_KEY, ROT_KEY);
                }
                for v in o.values_mut() {
                    keys(v);
                }
            }
            Value::Array(a) => {
                for v in a {
                    keys(v);
                }
            }
            _ => {}
        }
    }
    keys(&mut rotations);
    Ok(rotations)
}

pub fn serialize_no_web(
    s: &NoWebRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::NoWeb, c);
    let s0 = &s.behavior.settings;
    let mut choices = Map::new();
    let key = |v: &str| format!("liquidbounce.module.noWeb.{v}");
    for tag in MODES {
        let leaves = match tag {
            "Grim-2.3.65" => vec![t.leaf(
                "BreakOnWorld",
                "BOOLEAN",
                json!(s0.break_on_world),
                &key("mode.grim-2.3.65.breakOnWorld"),
                None,
            )],
            "Strafe" => vec![
                t.ranged(
                    "Strength",
                    "FLOAT",
                    float_json(s0.strength, "NoWeb.Strength")?,
                    json!({"from":0.01,"to":0.8}),
                    "",
                    &key("mode.strafe.strength"),
                ),
                t.leaf(
                    "OnlyOnGround",
                    "BOOLEAN",
                    json!(s0.only_on_ground),
                    &key("mode.strafe.onlyOnGround"),
                    None,
                ),
                t.group(
                    "MotionY",
                    "TOGGLEABLE",
                    vec![
                        t.leaf(
                            "Enabled",
                            "BOOLEAN",
                            json!(s0.motion_y),
                            &key("mode.strafe.motionY.enabled"),
                            None,
                        ),
                        t.ranged(
                            "MotionYStrength",
                            "FLOAT",
                            float_json(s0.motion_y_strength, "NoWeb.MotionYStrength")?,
                            json!({"from":-2.,"to":2.}),
                            "",
                            &key("mode.strafe.motionY.motionYStrength"),
                        ),
                    ],
                ),
            ],
            "PlaceWater" => vec![
                rotation_tree(s, c, f, d)?,
                t.group(
                    "Pickup",
                    "TOGGLEABLE",
                    vec![
                        t.leaf(
                            "Enabled",
                            "BOOLEAN",
                            json!(s0.pickup),
                            &key("mode.placeWater.pickup.enabled"),
                            None,
                        ),
                        t.ranged(
                            "PickupSpan",
                            "FLOAT_RANGE",
                            json!({"from":s0.pickup_span[0],"to":s0.pickup_span[1]}),
                            json!({"from":0.5,"to":20.}),
                            "s",
                            &key("mode.placeWater.pickup.pickupSpan"),
                        ),
                    ],
                ),
            ],
            _ => vec![],
        };
        choices.insert(tag.into(), t.group(tag, "CONFIGURABLE", leaves));
    }
    values.push(t.mode(s0.mode.tag(), choices));
    Ok(t.group("NoWeb", "TOGGLEABLE", values))
}
fn ch(v: NoWebSetting) -> SettingChange {
    SettingChange::NoWeb(v)
}
fn parse_rotation(p: &mut SettingsPatch, v: &Value) {
    let proxy = json!({"name":"NoRotateSet","value":[{"name":"Mode","active":"ResetRotation","value":[],"choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[v]}}}]});
    let root = nodes(&proxy, "NoRotateSet", "NoRotateSet").unwrap();
    let mut parsed = SettingsPatch {
        module: ModuleKind::NoRotateSet,
        changes: Vec::new(),
        issues: Vec::new(),
    };
    no_rotate_set::parse_no_rotate_set(&mut parsed, &root);
    for c in parsed.changes {
        if let SettingChange::NoRotateSet(v) = c
            && !matches!(v, NoRotateSetSetting::Mode(_))
        {
            p.changes.push(ch(NoWebSetting::Rotation(v)));
        }
    }
    p.issues.extend(parsed.issues.into_iter().map(remap));
}

pub(super) fn parse_no_web(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Mode") else {
        return;
    };
    match mode["active"].as_str().and_then(Mode::parse) {
        Some(v) => p.changes.push(ch(NoWebSetting::Mode(v))),
        None => p
            .issues
            .push(ConfigError::new("NoWeb.Mode.active", "unknown NoWeb mode")),
    };
    let Some(choices) = mode["choices"].as_object() else {
        p.issues.push(ConfigError::new(
            "NoWeb.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for tag in MODES {
        let path = format!("NoWeb.Mode.{tag}");
        let Some(v) = choices.get(tag) else {
            p.issues.push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        let n = match nodes(v, tag, &path) {
            Ok(n) => n,
            Err(e) => {
                p.issues.push(e);
                continue;
            }
        };
        match tag {
            "Grim-2.3.65" => leaf(
                p,
                &n,
                "BreakOnWorld",
                &format!("{path}.BreakOnWorld"),
                |v, path| parse_bool(v, path).map(|v| ch(NoWebSetting::BreakOnWorld(v))),
            ),
            "Strafe" => {
                leaf(p, &n, "Strength", &format!("{path}.Strength"), |v, path| {
                    parse_float(v, path).map(|v| ch(NoWebSetting::Strength(v)))
                });
                leaf(
                    p,
                    &n,
                    "OnlyOnGround",
                    &format!("{path}.OnlyOnGround"),
                    |v, path| parse_bool(v, path).map(|v| ch(NoWebSetting::OnlyOnGround(v))),
                );
                if let Some(v) = n.get("MotionY") {
                    match nodes(v, "MotionY", &format!("{path}.MotionY")) {
                        Ok(n) => {
                            leaf(
                                p,
                                &n,
                                "Enabled",
                                &format!("{path}.MotionY.Enabled"),
                                |v, path| parse_bool(v, path).map(|v| ch(NoWebSetting::MotionY(v))),
                            );
                            leaf(
                                p,
                                &n,
                                "MotionYStrength",
                                &format!("{path}.MotionY.MotionYStrength"),
                                |v, path| {
                                    parse_float(v, path)
                                        .map(|v| ch(NoWebSetting::MotionYStrength(v)))
                                },
                            );
                        }
                        Err(e) => p.issues.push(e),
                    }
                }
            }
            "PlaceWater" => {
                if let Some(v) = n.get("Rotations") {
                    parse_rotation(p, v);
                }
                if let Some(v) = n.get("Pickup") {
                    match nodes(v, "Pickup", &format!("{path}.Pickup")) {
                        Ok(n) => {
                            leaf(
                                p,
                                &n,
                                "Enabled",
                                &format!("{path}.Pickup.Enabled"),
                                |v, path| parse_bool(v, path).map(|v| ch(NoWebSetting::Pickup(v))),
                            );
                            leaf(
                                p,
                                &n,
                                "PickupSpan",
                                &format!("{path}.Pickup.PickupSpan"),
                                |v, path| {
                                    parse_float_pair(v, path)
                                        .map(|v| ch(NoWebSetting::PickupSpan(v)))
                                },
                            );
                        }
                        Err(e) => p.issues.push(e),
                    }
                }
            }
            _ => {}
        }
    }
}
