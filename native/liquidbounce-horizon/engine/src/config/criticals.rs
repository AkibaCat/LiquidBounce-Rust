// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::criticals::{MODES, PACKET_TYPES, PACKETS, SPRINT_MODES, Settings};
#[derive(Clone, Debug, PartialEq)]
pub struct CriticalsSetting {
    pub path: String,
    pub value: Value,
}
fn choose(v: &Value, path: &str, tags: &[&str]) -> Result<usize, ConfigError> {
    let mut value = scalar_string(v).ok_or_else(|| ConfigError::new(path, "expected choice"))?;
    if path == "Packet.Mode" {
        if value.eq_ignore_ascii_case("NoCheatPlus") {
            value = PACKETS[1].into();
        }
        if value.eq_ignore_ascii_case("Grim") {
            value = PACKETS[5].into();
        }
    }
    tags.iter()
        .position(|s| s.eq_ignore_ascii_case(&value))
        .ok_or_else(|| ConfigError::new(path, "unknown Criticals choice"))
}
impl CriticalsSetting {
    pub fn apply(self, s: &mut Settings) -> Result<(), ConfigError> {
        let v = &self.value;
        let p = self.path.as_str();
        match p {
            "Mode" => s.mode = choose(v, p, &MODES)?,
            "Packet.Mode" => s.packet_mode = choose(v, p, &PACKETS)?,
            "Packet.PacketType" => s.packet_type = choose(v, p, &PACKET_TYPES)?,
            "Jump.Height" => s.jump_height = parse_float(v, p)?,
            "Jump.Range" => s.jump_range = parse_float(v, p)?,
            "Jump.OptimizeForCooldown" => s.optimize = parse_bool(v, p)?,
            "Jump.CheckKillaura" => s.check_killaura = parse_bool(v, p)?,
            "Jump.CheckAutoClicker" => s.check_auto_clicker = parse_bool(v, p)?,
            "Jump.CheckTriggerBot" => s.check_trigger_bot = parse_bool(v, p)?,
            "Jump.CanBeSeen" => s.can_be_seen = parse_bool(v, p)?,
            "Blink.Delay" => {
                let r = parse_int_pair(v, p)?;
                if r[0] > r[1] {
                    return Err(ConfigError::new(p, "empty delay range"));
                }
                s.blink_delay = r;
            }
            "Blink.Range" => s.blink_range = parse_float(v, p)?,
            "Timer.Speed" => s.timer_speed = parse_float(v, p)?,
            "Timer.Range" => s.timer_range = parse_float(v, p)?,
            "WhenSprinting.Enabled" => s.sprint_enabled = parse_bool(v, p)?,
            "WhenSprinting.StopSprinting" => s.stop_sprinting = choose(v, p, &SPRINT_MODES)?,
            "WhenSprinting.Range" => s.sprint_range = parse_float(v, p)?,
            "Visuals.Enabled" => s.visuals = parse_bool(v, p)?,
            "Visuals.Fake" => s.fake = parse_bool(v, p)?,
            "Visuals.Critical" => s.critical = parse_int(v, p)?,
            "Visuals.Magic" => s.magic = parse_int(v, p)?,
            _ => return Err(ConfigError::new(p, "unknown Criticals setting")),
        };
        Ok(())
    }
}
fn key(path: &str) -> String {
    format!("liquidbounce.module.criticals.{path}")
}
fn boolean(t: &Tree<'_>, name: &str, v: bool, path: &str) -> Value {
    t.leaf(name, "BOOLEAN", json!(v), &key(path), None)
}
fn choice(t: &Tree<'_>, name: &str, index: usize, tags: &[&str], path: &str) -> Value {
    let mut v = t.leaf(name, "CHOOSE", json!(tags[index]), &key(path), None);
    if t.format == JsonFormat::Interop {
        v["choices"] = json!(tags);
    }
    v
}
fn float(
    t: &Tree<'_>,
    name: &str,
    v: f32,
    range: [f32; 2],
    path: &str,
) -> Result<Value, ConfigError> {
    Ok(t.ranged(
        name,
        "FLOAT",
        float_json(v, path)?,
        json!({"from":range[0],"to":range[1]}),
        "",
        &key(path),
    ))
}
pub fn serialize_criticals(
    s: &Settings,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::Criticals, common);
    let mut choices = Map::new();
    for mode in MODES {
        let v = match mode {
            "Packet" => vec![
                choice(&t, "Mode", s.packet_mode, &PACKETS, "mode.packet.mode"),
                choice(
                    &t,
                    "PacketType",
                    s.packet_type,
                    &PACKET_TYPES,
                    "mode.packet.packetType",
                ),
            ],
            "Jump" => vec![
                float(&t, "Height", s.jump_height, [0.1, 0.42], "mode.jump.height")?,
                float(&t, "Range", s.jump_range, [1., 6.], "mode.jump.range")?,
                boolean(
                    &t,
                    "OptimizeForCooldown",
                    s.optimize,
                    "mode.jump.optimizeForCooldown",
                ),
                boolean(
                    &t,
                    "CheckKillaura",
                    s.check_killaura,
                    "mode.jump.checkKillaura",
                ),
                boolean(
                    &t,
                    "CheckAutoClicker",
                    s.check_auto_clicker,
                    "mode.jump.checkAutoClicker",
                ),
                boolean(
                    &t,
                    "CheckTriggerBot",
                    s.check_trigger_bot,
                    "mode.jump.checkTriggerBot",
                ),
                boolean(&t, "CanBeSeen", s.can_be_seen, "mode.jump.canBeSeen"),
            ],
            "Blink" => vec![
                t.ranged(
                    "Delay",
                    "INT_RANGE",
                    json!({"from":s.blink_delay[0],"to":s.blink_delay[1]}),
                    json!({"from":0,"to":1000}),
                    "ms",
                    &key("mode.blink.delay"),
                ),
                float(&t, "Range", s.blink_range, [0., 10.], "mode.blink.range")?,
            ],
            "Timer" => vec![
                float(&t, "Speed", s.timer_speed, [0.1, 1.], "mode.timer.speed")?,
                float(&t, "Range", s.timer_range, [0., 10.], "mode.timer.range")?,
            ],
            _ => vec![],
        };
        choices.insert(mode.into(), t.group(mode, "CONFIGURABLE", v));
    }
    values.push(t.mode(MODES[s.mode], choices));
    values.push(t.group(
        "WhenSprinting",
        "TOGGLEABLE",
        vec![
            boolean(&t, "Enabled", s.sprint_enabled, "whenSprinting.enabled"),
            choice(
                &t,
                "StopSprinting",
                s.stop_sprinting,
                &SPRINT_MODES,
                "whenSprinting.stopSprinting",
            ),
            float(
                &t,
                "Range",
                s.sprint_range,
                [0., 10.],
                "whenSprinting.range",
            )?,
        ],
    ));
    values.push(t.group(
        "Visuals",
        "TOGGLEABLE",
        vec![
            boolean(&t, "Enabled", s.visuals, "visuals.enabled"),
            boolean(&t, "Fake", s.fake, "visuals.fake"),
            t.ranged(
                "Critical",
                "INT",
                json!(s.critical),
                json!({"from":0,"to":20}),
                "",
                &key("visuals.critical"),
            ),
            t.ranged(
                "Magic",
                "INT",
                json!(s.magic),
                json!({"from":0,"to":20}),
                "",
                &key("visuals.magic"),
            ),
        ],
    ));
    Ok(t.group("Criticals", "TOGGLEABLE", values))
}
fn collect(p: &mut SettingsPatch, n: &Nodes<'_>, prefix: &str, names: &[&str]) {
    for name in names {
        if let Some(node) = n.get(*name) {
            let path = if prefix.is_empty() {
                (*name).into()
            } else {
                format!("{prefix}.{name}")
            };
            match value_of(node, &path) {
                Ok(value) => p.changes.push(SettingChange::Criticals(CriticalsSetting {
                    path,
                    value: value.clone(),
                })),
                Err(e) => p.issues.push(e),
            }
        }
    }
}
fn group<'a>(p: &mut SettingsPatch, n: &Nodes<'a>, name: &str) -> Option<Nodes<'a>> {
    let node = n.get(name)?;
    match nodes(node, name, name) {
        Ok(n) => Some(n),
        Err(e) => {
            p.issues.push(e);
            None
        }
    }
}
pub(super) fn parse_criticals(p: &mut SettingsPatch, n: &Nodes<'_>) {
    if let Some(mode) = n.get("Mode") {
        match mode["active"].as_str() {
            Some(v) => p.changes.push(SettingChange::Criticals(CriticalsSetting {
                path: "Mode".into(),
                value: json!(v),
            })),
            None => p.issues.push(ConfigError::new(
                "Criticals.Mode.active",
                "expected mode tag",
            )),
        }
        if let Some(choices) = mode["choices"].as_object() {
            for tag in MODES {
                if let Some(node) = choices.get(tag) {
                    let n = match nodes(node, tag, tag) {
                        Ok(n) => n,
                        Err(e) => {
                            p.issues.push(e);
                            continue;
                        }
                    };
                    let names: &[&str] = match tag {
                        "Packet" => &["Mode", "PacketType"],
                        "Jump" => &[
                            "Height",
                            "Range",
                            "OptimizeForCooldown",
                            "CheckKillaura",
                            "CheckAutoClicker",
                            "CheckTriggerBot",
                            "CanBeSeen",
                        ],
                        "Blink" => &["Delay", "Range"],
                        "Timer" => &["Speed", "Range"],
                        _ => &[],
                    };
                    collect(p, &n, tag, names);
                }
            }
        } else {
            p.issues.push(ConfigError::new(
                "Criticals.Mode.choices",
                "expected mode choices",
            ));
        }
    }
    if let Some(n) = group(p, n, "WhenSprinting") {
        collect(
            p,
            &n,
            "WhenSprinting",
            &["Enabled", "StopSprinting", "Range"],
        );
    }
    if let Some(n) = group(p, n, "Visuals") {
        collect(p, &n, "Visuals", &["Enabled", "Fake", "Critical", "Magic"]);
    }
}
