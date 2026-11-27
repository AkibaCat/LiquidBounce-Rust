// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::anti_bot::{ARMOR, CONDITIONS, MODES, Settings, State, VALIDATORS};
#[derive(Clone, Debug, PartialEq)]
pub struct AntiBotSetting(pub String, pub Value);
const HELMET: [&str; 10] = [
    "Nothing",
    "Leather",
    "Chain",
    "Iron",
    "Gold",
    "Diamond",
    "Netherite",
    "TurtleScute",
    "Pumpkin",
    "Skull",
];
impl AntiBotSetting {
    pub fn apply(self, state: &mut State) -> Result<(), ConfigError> {
        let p = self.0.as_str();
        let v = &self.1;
        if p == "Mode" {
            let tag = scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected mode"))?;
            let tag = match tag.as_str() {
                "Matrix" => MODES[1],
                "IntaveHeavy" => MODES[2],
                "Horizon" => MODES[3],
                _ => &tag,
            };
            let mode = MODES
                .iter()
                .position(|m| *m == tag)
                .ok_or_else(|| ConfigError::new(p, "unknown case-sensitive mode"))?;
            state.set_mode(mode);
            return Ok(());
        }
        let s = &mut state.settings;
        fn multi(dst: &mut u16, v: &Value, path: &str, tags: &[&str]) -> Result<(), ConfigError> {
            *dst = 0;
            let a = match v {
                Value::Array(a) => a.clone(),
                Value::String(_) | Value::Number(_) | Value::Bool(_) => vec![v.clone()],
                _ => return Ok(()),
            };
            for v in a {
                let tag = scalar_string(&v)
                    .ok_or_else(|| ConfigError::new(path, "invalid multi-choice"))?;
                if let Some(i) = tags.iter().position(|t| t.eq_ignore_ascii_case(&tag)) {
                    *dst |= 1 << i;
                }
            }
            Ok(())
        }
        match p {
            "LiteralNPC" => s.literal_npc = parse_bool(v, p)?,
            "NotInTabList.Enabled" => s.not_in_tab = parse_bool(v, p)?,
            "NotInTabList.Grace" => s.grace = parse_int(v, p)?,
            "Mode.Custom.Conditions" => multi(&mut s.conditions, v, p, &CONDITIONS)?,
            "Mode.Custom.InvalidGround.Enabled" => s.invalid_ground = parse_bool(v, p)?,
            "Mode.Custom.InvalidGround.VLToConsiderAsBot" => s.ground_vl = parse_int(v, p)?,
            "Mode.Custom.AlwaysInRadius.Enabled" => s.radius = parse_bool(v, p)?,
            "Mode.Custom.AlwaysInRadius.AlwaysInRadiusRange" => s.radius_range = parse_float(v, p)?,
            "Mode.Custom.Age.Enabled" => s.age = parse_bool(v, p)?,
            "Mode.Custom.Age.Minimum" => s.minimum_age = parse_int(v, p)?,
            "Mode.Custom.Moved.Enabled" => s.moved = parse_bool(v, p)?,
            "Mode.Custom.Moved.Minimum" => s.minimum_moved = parse_int(v, p)?,
            "Mode.Custom.Metadata.Enabled" => s.metadata = parse_bool(v, p)?,
            "Mode.Custom.Metadata.Maximum" => s.maximum_metadata = parse_int(v, p)?,
            "Mode.Custom.Armor.Enabled" => s.armor = parse_bool(v, p)?,
            "Mode.Custom.Armor.Helmet" => {
                let mut selected = 0;
                let result = multi(&mut selected, v, p, &HELMET);
                s.armor_choices[0] = (0..HELMET.len())
                    .filter(|i| selected & (1 << i) != 0)
                    .fold(0, |mask, i| {
                        mask | (1 << ARMOR.iter().position(|t| *t == HELMET[i]).unwrap())
                    });
                result?;
            }
            "Mode.Custom.Armor.Chestplate" => multi(&mut s.armor_choices[1], v, p, &ARMOR[..8])?,
            "Mode.Custom.Armor.Leggings" => multi(&mut s.armor_choices[2], v, p, &ARMOR[..7])?,
            "Mode.Custom.Armor.Boots" => multi(&mut s.armor_choices[3], v, p, &ARMOR[..7])?,
            "Mode.Custom.Name.Enabled" => s.name = parse_bool(v, p)?,
            "Mode.Custom.Name.Length" => s.length = parse_int_pair(v, p)?,
            "Mode.Custom.Name.ValidateChars" => {
                let mut value = u16::from(s.validators);
                let result = multi(&mut value, v, p, &VALIDATORS);
                s.validators = value as u8;
                result?;
            }
            _ => return Err(ConfigError::new(p, "unknown AntiBot setting")),
        }
        Ok(())
    }
}
pub const LEAVES: &[(&str, &str, f64, f64, &str)] = &[
    ("Conditions", "MULTI_CHOOSE", 0., 0., ""),
    ("InvalidGround.Enabled", "BOOLEAN", 0., 0., ""),
    ("InvalidGround.VLToConsiderAsBot", "INT", 1., 50., "flags"),
    ("AlwaysInRadius.Enabled", "BOOLEAN", 0., 0., ""),
    ("AlwaysInRadius.AlwaysInRadiusRange", "FLOAT", 5., 30., ""),
    ("Age.Enabled", "BOOLEAN", 0., 0., ""),
    ("Age.Minimum", "INT", 0., 120., "ticks"),
    ("Moved.Enabled", "BOOLEAN", 0., 0., ""),
    ("Moved.Minimum", "INT", 20., 400., "ticks"),
    ("Metadata.Enabled", "BOOLEAN", 0., 0., ""),
    ("Metadata.Maximum", "INT", 0., 10., "entries"),
    ("Armor.Enabled", "BOOLEAN", 0., 0., ""),
    ("Armor.Helmet", "MULTI_CHOOSE", 0., 0., ""),
    ("Armor.Chestplate", "MULTI_CHOOSE", 0., 0., ""),
    ("Armor.Leggings", "MULTI_CHOOSE", 0., 0., ""),
    ("Armor.Boots", "MULTI_CHOOSE", 0., 0., ""),
    ("Name.Enabled", "BOOLEAN", 0., 0., ""),
    ("Name.Length", "INT_RANGE", 1., 32., ""),
    ("Name.ValidateChars", "MULTI_CHOOSE", 0., 0., ""),
];
fn value(s: &Settings, p: &str) -> Value {
    match p {
        "Conditions" => json!(s.conditions),
        "InvalidGround.Enabled" => json!(s.invalid_ground),
        "InvalidGround.VLToConsiderAsBot" => json!(s.ground_vl),
        "AlwaysInRadius.Enabled" => json!(s.radius),
        "AlwaysInRadius.AlwaysInRadiusRange" => json!(s.radius_range),
        "Age.Enabled" => json!(s.age),
        "Age.Minimum" => json!(s.minimum_age),
        "Moved.Enabled" => json!(s.moved),
        "Moved.Minimum" => json!(s.minimum_moved),
        "Metadata.Enabled" => json!(s.metadata),
        "Metadata.Maximum" => json!(s.maximum_metadata),
        "Armor.Enabled" => json!(s.armor),
        "Armor.Helmet" => json!(s.armor_choices[0]),
        "Armor.Chestplate" => json!(s.armor_choices[1]),
        "Armor.Leggings" => json!(s.armor_choices[2]),
        "Armor.Boots" => json!(s.armor_choices[3]),
        "Name.Enabled" => json!(s.name),
        "Name.Length" => json!({"from":s.length[0],"to":s.length[1]}),
        "Name.ValidateChars" => json!(s.validators),
        _ => Value::Null,
    }
}
fn key(p: &str) -> String {
    format!(
        "liquidbounce.module.antiBot.{}",
        p.split('.')
            .map(|s| {
                let mut c = s.chars();
                format!("{}{}", c.next().unwrap().to_lowercase(), c.as_str())
            })
            .collect::<Vec<_>>()
            .join(".")
    )
}
pub fn serialize_anti_bot(
    s: &State,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::AntiBot, c);
    let mut custom = vec![];
    for &(p, kind, lo, hi, unit) in LEAVES {
        let name = p.rsplit('.').next().unwrap();
        let k = key(&format!("Mode.Custom.{p}"));
        let v = if p == "AlwaysInRadius.AlwaysInRadiusRange" {
            float_json(s.settings.radius_range, &k)?
        } else {
            value(&s.settings, p)
        };
        let node = if kind == "MULTI_CHOOSE" {
            let tags: &[&str] = match p {
                "Conditions" => &CONDITIONS,
                "Name.ValidateChars" => &VALIDATORS,
                "Armor.Chestplate" => &ARMOR[..8],
                "Armor.Helmet" => &HELMET,
                _ => &ARMOR[..7],
            };
            let mut mask = v.as_u64().unwrap() as u32;
            if p == "Armor.Helmet" {
                mask = tags.iter().enumerate().fold(0, |out, (i, tag)| {
                    if mask & (1 << ARMOR.iter().position(|v| v == tag).unwrap()) != 0 {
                        out | (1 << i)
                    } else {
                        out
                    }
                });
            }
            t.multi(name, mask, tags, &k)
        } else if kind == "BOOLEAN" {
            t.leaf(name, kind, v, &k, None)
        } else {
            let range = if matches!(kind, "INT" | "INT_RANGE") {
                json!({"from":lo as i32,"to":hi as i32})
            } else {
                json!({"from":lo,"to":hi})
            };
            t.ranged(name, kind, v, range, unit, &k)
        };
        if let Some((group, _)) = p.split_once('.') {
            if !custom.iter().any(|v: &Value| v["name"] == group) {
                custom.push(t.group(group, "TOGGLEABLE", vec![]));
            }
            custom.iter_mut().find(|v| v["name"] == group).unwrap()["value"]
                .as_array_mut()
                .unwrap()
                .push(node);
        } else {
            custom.push(node);
        }
    }
    let choices = MODES
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            (
                tag.to_string(),
                t.group(
                    tag,
                    "CONFIGURABLE",
                    if i == 0 { custom.clone() } else { vec![] },
                ),
            )
        })
        .collect();
    values.push(t.mode(MODES[s.settings.mode], choices));
    values.push(t.leaf(
        "LiteralNPC",
        "BOOLEAN",
        json!(s.settings.literal_npc),
        &key("LiteralNPC"),
        None,
    ));
    values.push(t.group(
        "NotInTabList",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.settings.not_in_tab),
                &key("NotInTabList.Enabled"),
                None,
            ),
            t.ranged(
                "Grace",
                "INT",
                json!(s.settings.grace),
                json!({"from":0,"to":200}),
                "ticks",
                &key("NotInTabList.Grace"),
            ),
        ],
    ));
    Ok(t.group("AntiBot", "TOGGLEABLE", values))
}
pub(super) fn parse_anti_bot(p: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(mode) = root.get("Mode") {
        if let Some(active) = mode.get("active") {
            p.changes.push(SettingChange::AntiBot(AntiBotSetting(
                "Mode".into(),
                active.clone(),
            )));
        }
        if let Some(custom) = mode.get("choices").and_then(|v| v.get("Custom")) {
            match nodes(custom, "Custom", "AntiBot.Mode.Custom") {
                Ok(custom) => {
                    leaf(
                        p,
                        &custom,
                        "Conditions",
                        "AntiBot.Mode.Custom.Conditions",
                        |v, _| {
                            Ok(SettingChange::AntiBot(AntiBotSetting(
                                "Mode.Custom.Conditions".into(),
                                v.clone(),
                            )))
                        },
                    );
                    for group in [
                        "InvalidGround",
                        "AlwaysInRadius",
                        "Age",
                        "Moved",
                        "Metadata",
                        "Armor",
                        "Name",
                    ] {
                        if let Some(value) = custom.get(group) {
                            match nodes(value, group, &format!("AntiBot.Mode.Custom.{group}")) {
                                Ok(children) => {
                                    for &(path, _, _, _, _) in LEAVES {
                                        if let Some(name) = path.strip_prefix(&format!("{group}."))
                                        {
                                            leaf(
                                                p,
                                                &children,
                                                name,
                                                &format!("AntiBot.Mode.Custom.{path}"),
                                                |v, _| {
                                                    Ok(SettingChange::AntiBot(AntiBotSetting(
                                                        format!("Mode.Custom.{path}"),
                                                        v.clone(),
                                                    )))
                                                },
                                            );
                                        }
                                    }
                                }
                                Err(e) => p.issues.push(e),
                            }
                        }
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
    leaf(p, root, "LiteralNPC", "AntiBot.LiteralNPC", |v, _| {
        Ok(SettingChange::AntiBot(AntiBotSetting(
            "LiteralNPC".into(),
            v.clone(),
        )))
    });
    if let Some(group) = root.get("NotInTabList") {
        match nodes(group, "NotInTabList", "AntiBot.NotInTabList") {
            Ok(n) => {
                for name in ["Enabled", "Grace"] {
                    leaf(
                        p,
                        &n,
                        name,
                        &format!("AntiBot.NotInTabList.{name}"),
                        |v, _| {
                            Ok(SettingChange::AntiBot(AntiBotSetting(
                                format!("NotInTabList.{name}"),
                                v.clone(),
                            )))
                        },
                    );
                }
            }
            Err(e) => p.issues.push(e),
        }
    }
}
