// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    anchor::Anchor,
    avoid_hazards::{AvoidHazards, Mode as HazardMode, TAGS},
    freeze::{MODES, Mode as FreezeMode, REASONS, Settings as FreezeSettings},
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnchorSetting {
    Distance(f32),
    Horizontal(f32),
    Vertical(f32),
}
impl AnchorSetting {
    pub fn apply(self, s: &mut Anchor) {
        match self {
            Self::Distance(v) => s.max_distance = v,
            Self::Horizontal(v) => s.horizontal_speed = v,
            Self::Vertical(v) => s.vertical_speed = v,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvoidHazardsSetting {
    Mode(HazardMode),
    Avoid(u32),
}
impl AvoidHazardsSetting {
    pub fn apply(self, s: &mut AvoidHazards) {
        match self {
            Self::Mode(v) => s.mode = v,
            Self::Avoid(v) => s.avoid = v,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreezeSetting {
    Mode(FreezeMode),
    DisableOn(u32),
    Notification(bool),
    QueueOrigins(u32),
    CancelOrigins(u32),
    CancelC0b(bool),
    Interval([i32; 2]),
}
impl FreezeSetting {
    pub fn apply(self, s: &mut FreezeSettings) {
        match self {
            Self::Mode(v) => s.mode = v,
            Self::DisableOn(v) => s.disable_on = v,
            Self::Notification(v) => s.notification = v,
            Self::QueueOrigins(v) => s.queue_origins = v,
            Self::CancelOrigins(v) => s.cancel_origins = v,
            Self::CancelC0b(v) => s.cancel_c0b = v,
            Self::Interval(v) => s.interval = v,
        }
    }
}
pub fn serialize_anchor(
    s: &Anchor,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::Anchor, c);
    for (name, key, v, from, to) in [
        ("MaxDistance", "maxDistance", s.max_distance, 0.1, 6.),
        (
            "HorizontalSpeed",
            "horizontalSpeed",
            s.horizontal_speed,
            0.,
            10.,
        ),
        ("VerticalSpeed", "verticalSpeed", s.vertical_speed, 0., 10.),
    ] {
        values.push(t.ranged(
            name,
            "FLOAT",
            float_json(v, &format!("Anchor.{name}"))?,
            json!({"from":from,"to":to}),
            "",
            &format!("liquidbounce.module.anchor.{key}"),
        ));
    }
    Ok(t.group("Anchor", "TOGGLEABLE", values))
}
pub fn serialize_avoid_hazards(
    s: &AvoidHazards,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::AvoidHazards, c);
    let mut mode = t.leaf(
        "Mode",
        "CHOOSE",
        json!(s.mode.tag()),
        "liquidbounce.module.avoidHazards.mode",
        None,
    );
    if f == JsonFormat::Interop {
        mode["choices"] = json!(["Shape", "Input"]);
    }
    values.extend([
        mode,
        t.multi(
            "Avoid",
            s.avoid,
            &TAGS,
            "liquidbounce.module.avoidHazards.avoid",
        ),
    ]);
    Ok(t.group("AvoidHazards", "TOGGLEABLE", values))
}
pub fn serialize_freeze(
    s: &FreezeSettings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::Freeze, c);
    let mut choices = Map::new();
    for tag in MODES {
        let key = format!(
            "liquidbounce.module.freeze.mode.{}",
            match tag {
                "Queue" => "queue",
                "Cancel" => "cancel",
                "Stationary" => "stationary",
                _ => "tickMovement",
            }
        );
        let leaves = match tag {
            "Queue" => vec![t.multi(
                "Origin",
                s.queue_origins,
                &["Incoming", "Outgoing"],
                &format!("{key}.origin"),
            )],
            "Cancel" => vec![t.multi(
                "Origin",
                s.cancel_origins,
                &["Incoming", "Outgoing"],
                &format!("{key}.origin"),
            )],
            "Stationary" => vec![t.leaf(
                "CancelC0B",
                "BOOLEAN",
                json!(s.cancel_c0b),
                &format!("{key}.cancelC0B"),
                None,
            )],
            _ => vec![t.ranged(
                "Interval",
                "INT_RANGE",
                json!({"from":s.interval[0],"to":s.interval[1]}),
                json!({"from":1,"to":200}),
                "ticks",
                &format!("{key}.interval"),
            )],
        };
        choices.insert(tag.into(), t.group(tag, "CONFIGURABLE", leaves));
    }
    values.extend([
        t.mode(s.mode.tag(), choices),
        t.multi(
            "DisableOn",
            s.disable_on,
            &REASONS,
            "liquidbounce.module.freeze.disableOn",
        ),
        t.leaf(
            "Notification",
            "BOOLEAN",
            json!(s.notification),
            "liquidbounce.module.freeze.notification",
            None,
        ),
    ]);
    Ok(t.group("Freeze", "TOGGLEABLE", values))
}
pub(super) fn parse_anchor(p: &mut SettingsPatch, n: &Nodes<'_>) {
    for (name, create) in [
        (
            "MaxDistance",
            AnchorSetting::Distance as fn(f32) -> AnchorSetting,
        ),
        ("HorizontalSpeed", AnchorSetting::Horizontal),
        ("VerticalSpeed", AnchorSetting::Vertical),
    ] {
        leaf(p, n, name, &format!("Anchor.{name}"), |v, path| {
            parse_float(v, path).map(|v| SettingChange::Anchor(create(v)))
        });
    }
}
pub(super) fn parse_avoid_hazards(p: &mut SettingsPatch, n: &Nodes<'_>) {
    leaf(p, n, "Mode", "AvoidHazards.Mode", |v, path| {
        scalar_string(v)
            .and_then(|s| HazardMode::parse(&s))
            .map(|v| SettingChange::AvoidHazards(AvoidHazardsSetting::Mode(v)))
            .ok_or_else(|| ConfigError::new(path, "unknown AvoidHazards mode"))
    });
    leaf(p, n, "Avoid", "AvoidHazards.Avoid", |v, path| {
        parse_multi_mask(v, path, &TAGS)
            .map(|v| SettingChange::AvoidHazards(AvoidHazardsSetting::Avoid(v)))
    });
}
pub(super) fn parse_freeze(p: &mut SettingsPatch, n: &Nodes<'_>) {
    leaf(p, n, "DisableOn", "Freeze.DisableOn", |v, path| {
        parse_multi_mask(v, path, &REASONS)
            .map(|v| SettingChange::Freeze(FreezeSetting::DisableOn(v)))
    });
    leaf(p, n, "Notification", "Freeze.Notification", |v, path| {
        parse_bool(v, path).map(|v| SettingChange::Freeze(FreezeSetting::Notification(v)))
    });
    let Some(mode) = n.get("Mode") else {
        return;
    };
    match mode["active"].as_str().and_then(FreezeMode::parse) {
        Some(v) => p
            .changes
            .push(SettingChange::Freeze(FreezeSetting::Mode(v))),
        None => p.issues.push(ConfigError::new(
            "Freeze.Mode.active",
            "unknown Freeze mode",
        )),
    }
    let Some(choices) = mode["choices"].as_object() else {
        p.issues.push(ConfigError::new(
            "Freeze.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for tag in MODES {
        let path = format!("Freeze.Mode.{tag}");
        let Some(v) = choices.get(tag) else {
            p.issues.push(ConfigError::new(&path, "choice not found"));
            continue;
        };
        match nodes(v, tag, &path) {
            Ok(n) => match tag {
                "Queue" | "Cancel" => {
                    leaf(p, &n, "Origin", &format!("{path}.Origin"), |v, path| {
                        parse_multi_mask(v, path, &["Incoming", "Outgoing"]).map(|v| {
                            SettingChange::Freeze(if tag == "Queue" {
                                FreezeSetting::QueueOrigins(v)
                            } else {
                                FreezeSetting::CancelOrigins(v)
                            })
                        })
                    })
                }
                "Stationary" => leaf(
                    p,
                    &n,
                    "CancelC0B",
                    &format!("{path}.CancelC0B"),
                    |v, path| {
                        parse_bool(v, path)
                            .map(|v| SettingChange::Freeze(FreezeSetting::CancelC0b(v)))
                    },
                ),
                _ => leaf(p, &n, "Interval", &format!("{path}.Interval"), |v, path| {
                    let from = parse_int(&v["from"], path)?;
                    let to = parse_int(&v["to"], path)?;
                    if from < 1 || from > to {
                        return Err(ConfigError::new(
                            path,
                            "interval must be positive and ascending",
                        ));
                    }
                    Ok(SettingChange::Freeze(FreezeSetting::Interval([from, to])))
                }),
            },
            Err(e) => p.issues.push(e),
        }
    }
}
