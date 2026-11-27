// SPDX-License-Identifier: GPL-3.0-or-later
use super::combat_rotations::CombatRotationSetting;
use super::*;
use crate::projectile_puncher_runtime::ProjectilePuncherRuntime;
use liquidbounce_behaviors::clicking::Technique;
pub const PROJECTILES: [&str; 3] = ["Fireball", "ShulkerBullet", "WindCharge"];
pub const SWINGS: [&str; 4] = ["DoNotHide", "HideForBoth", "HideForClient", "HideForServer"];
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectilePuncherSetting {
    Value(String, Value),
    Rotation(CombatRotationSetting),
}
impl ProjectilePuncherSetting {
    pub fn apply(self, s: &mut ProjectilePuncherRuntime) -> Result<(), ConfigError> {
        let Self::Value(path, v) = self else {
            let Self::Rotation(v) = self else {
                unreachable!()
            };
            return v.apply(&mut s.rotations);
        };
        let p = path.as_str();
        let choose = |tags: &[&str]| {
            scalar_string(&v)
                .and_then(|v| tags.iter().position(|t| t.eq_ignore_ascii_case(&v)))
                .ok_or_else(|| ConfigError::new(p, "unknown choice"))
        };
        match p {
            "Clicker.Technique" => {
                s.clicker.settings.technique = if choose(&["Human", "Constant"])? == 0 {
                    Technique::Human
                } else {
                    Technique::Constant
                }
            }
            "Clicker.CPS" => s.clicker.settings.cps = parse_int_pair(&v, p)?,
            "Clicker.MaxPerTick" => s.clicker.settings.max_per_tick = parse_int(&v, p)?,
            "Clicker.MissCooldown" => s.clicker.settings.miss_cooldown = parse_bool(&v, p)?,
            "Range" => s.range = parse_float(&v, p)?,
            "SwingMode" => s.swing = choose(&SWINGS)? as u8,
            "IgnoreOpenInventory" => s.ignore_inventory = parse_bool(&v, p)?,
            "OnlyInAir" => s.only_air = parse_bool(&v, p)?,
            "FOV" => s.fov = parse_float(&v, p)?,
            "Projectiles" => {
                let values = if let Some(a) = v.as_array() {
                    a.clone()
                } else {
                    vec![v.clone()]
                };
                s.projectiles = 0;
                for v in values {
                    let index = scalar_string(&v)
                        .and_then(|v| PROJECTILES.iter().position(|t| t.eq_ignore_ascii_case(&v)))
                        .ok_or_else(|| ConfigError::new(p, "unknown projectile"))?;
                    s.projectiles |= 1 << index;
                }
            }
            _ => return Err(ConfigError::new(p, "unknown ProjectilePuncher setting")),
        }
        Ok(())
    }
}
fn key(p: &str) -> String {
    format!(
        "liquidbounce.module.projectilePuncher.{}",
        p.split('.')
            .map(|p| {
                let mut c = p.chars();
                format!("{}{}", c.next().unwrap().to_lowercase(), c.as_str())
            })
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn choice(t: &Tree<'_>, name: &str, value: &str, tags: &[&str], path: &str) -> Value {
    let mut v = t.leaf(name, "CHOOSE", json!(value), &key(path), None);
    if t.format == JsonFormat::Interop {
        v["choices"] = json!(tags);
    }
    v
}
pub fn serialize_projectile_puncher(
    s: &ProjectilePuncherRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let b = s.clicker.settings;
    let mut values = t.common(ModuleKind::ProjectilePuncher, c);
    values.push(t.group(
        "Clicker",
        "CONFIGURABLE",
        vec![
            choice(
                &t,
                "Technique",
                if b.technique == Technique::Human {
                    "Human"
                } else {
                    "Constant"
                },
                &["Human", "Constant"],
                "Clicker.Technique",
            ),
            t.ranged(
                "CPS",
                "INT_RANGE",
                json!({"from":b.cps[0],"to":b.cps[1]}),
                json!({"from":1,"to":30}),
                "clicks",
                &key("Clicker.CPS"),
            ),
            t.ranged(
                "MaxPerTick",
                "INT",
                json!(b.max_per_tick),
                json!({"from":1,"to":5}),
                "clicks",
                &key("Clicker.MaxPerTick"),
            ),
            t.leaf(
                "MissCooldown",
                "BOOLEAN",
                json!(b.miss_cooldown),
                &key("Clicker.MissCooldown"),
                None,
            ),
        ],
    ));
    values.push(t.ranged(
        "Range",
        "FLOAT",
        float_json(s.range, "ProjectilePuncher.Range")?,
        json!({"from":3.,"to":6.}),
        "",
        &key("Range"),
    ));
    values.push(choice(
        &t,
        "SwingMode",
        SWINGS[s.swing as usize],
        &SWINGS,
        "SwingMode",
    ));
    values.push(t.leaf(
        "IgnoreOpenInventory",
        "BOOLEAN",
        json!(s.ignore_inventory),
        &key("IgnoreOpenInventory"),
        None,
    ));
    values.push(t.multi(
        "Projectiles",
        u32::from(s.projectiles),
        &PROJECTILES,
        &key("Projectiles"),
    ));
    values.push(t.leaf(
        "OnlyInAir",
        "BOOLEAN",
        json!(s.only_air),
        &key("OnlyInAir"),
        None,
    ));
    values.push(t.ranged(
        "FOV",
        "FLOAT",
        float_json(s.fov, "ProjectilePuncher.FOV")?,
        json!({"from":0.,"to":180.}),
        "",
        &key("FOV"),
    ));
    values.push(combat_rotations::serialize(
        &s.rotations,
        c,
        f,
        d,
        "liquidbounce.module.projectilePuncher.rotations",
    )?);
    Ok(t.group("ProjectilePuncher", "TOGGLEABLE", values))
}
pub(super) fn parse_projectile_puncher(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in [
        "Clicker",
        "Range",
        "SwingMode",
        "IgnoreOpenInventory",
        "Projectiles",
        "OnlyInAir",
        "FOV",
        "Rotations",
    ] {
        let Some(n) = root.get(name) else {
            continue;
        };
        if name == "Rotations" {
            let (c, e) = combat_rotations::parse(n, "ProjectilePuncher.Rotations");
            p.changes.extend(
                c.into_iter().map(|v| {
                    SettingChange::ProjectilePuncher(ProjectilePuncherSetting::Rotation(v))
                }),
            );
            p.issues.extend(e);
            continue;
        }
        if name == "Clicker" {
            match nodes(n, name, "ProjectilePuncher.Clicker") {
                Ok(n) => {
                    for name in ["Technique", "CPS", "MaxPerTick", "MissCooldown"] {
                        leaf(
                            p,
                            &n,
                            name,
                            &format!("ProjectilePuncher.Clicker.{name}"),
                            |v, _| {
                                Ok(SettingChange::ProjectilePuncher(
                                    ProjectilePuncherSetting::Value(
                                        format!("Clicker.{name}"),
                                        v.clone(),
                                    ),
                                ))
                            },
                        );
                    }
                }
                Err(e) => p.issues.push(e),
            }
            continue;
        }
        leaf(
            p,
            root,
            name,
            &format!("ProjectilePuncher.{name}"),
            |v, _| {
                Ok(SettingChange::ProjectilePuncher(
                    ProjectilePuncherSetting::Value(name.into(), v.clone()),
                ))
            },
        );
    }
}
