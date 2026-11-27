// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::auto_mob_heal::{Kind, Settings};
#[derive(Clone, Debug, PartialEq)]
pub struct AutoMobHealSetting(pub String, pub Value);
impl AutoMobHealSetting {
    pub fn apply(self, s: &mut Settings) -> Result<(), ConfigError> {
        let Self(path, v) = self;
        let p = format!("AutoMobHeal.{path}");
        match path.as_str() {
            "Range" => s.range = parse_float(&v, &p)?,
            "Delay" => {
                let delay = parse_int_pair(&v, &p)?;
                if delay[0] > delay[1] { return Err(ConfigError::new(&p, "empty integer range")); }
                s.delay = delay;
            }
            "SlotResetDelay" => s.slot_reset_delay = parse_int(&v, &p)?,
            "SwingMode" => {
                let tag = scalar_string(&v).ok_or_else(|| ConfigError::new(&p, "expected swing choice"))?;
                s.swing_mode = ["DoNotHide", "HideForClient"].into_iter().find(|t| t.eq_ignore_ascii_case(&tag))
                    .ok_or_else(|| ConfigError::new(&p, "unknown interaction swing choice"))?.into();
            }
            "Wolf.Armor.Enabled" => s.wolf_armor = parse_bool(&v, &p)?,
            "Wolf.Armor.Equip" => s.wolf_equip = parse_bool(&v, &p)?,
            "Wolf.Armor.RepairThreshold" => s.repair_threshold = parse_float(&v, &p)?,
            "Wolf.Armor.OnlyWhenSitting" => s.only_when_sitting = parse_bool(&v, &p)?,
            "HorseFamily.AllowGoldenCarrot" => s.allow_golden_carrot = parse_bool(&v, &p)?,
            "HorseFamily.AllowGoldenApple" => s.allow_golden_apple = parse_bool(&v, &p)?,
            "HorseFamily.AllowEnchantedGoldenApple" => s.allow_enchanted_golden_apple = parse_bool(&v, &p)?,
            "Nautilus.AllowBucketFood" => s.allow_bucket_food = parse_bool(&v, &p)?,
            _ => {
                let Some((group, name)) = path.split_once('.') else { return Err(ConfigError::new(&p, "unknown setting")); };
                let kind = Kind::from_tag(group).ok_or_else(|| ConfigError::new(&p, "unknown target"))?;
                let target = &mut s.targets[kind as usize];
                match name {
                    "Enabled" => target.enabled = parse_bool(&v, &p)?,
                    "HealthThreshold" => target.health_threshold = parse_float(&v, &p)?,
                    "NoWaste" if kind.food() => target.no_waste = parse_bool(&v, &p)?,
                    _ => return Err(ConfigError::new(&p, "unknown target setting")),
                }
            }
        }
        Ok(())
    }
}
fn key(path: &str) -> String {
    format!("liquidbounce.module.autoMobHeal.{}", path.split('.').map(|p| {
        let mut chars = p.chars();
        format!("{}{}", chars.next().unwrap().to_lowercase(), chars.as_str())
    }).collect::<Vec<_>>().join("."))
}
pub fn serialize_auto_mob_heal(s: &Settings, c: &CommonSettings, f: JsonFormat, d: &dyn Fn(&str) -> String) -> Result<Value, ConfigError> {
    let t = Tree { format: f, describe: d };
    let boolean = |name: &str, value: bool, path: &str| t.leaf(name, "BOOLEAN", json!(value), &key(path), None);
    let percent = |name: &str, value: f32, path: &str| t.ranged(name, "FLOAT", json!(value), json!({"from":1.,"to":100.}), "%", &key(path));
    let mut values = t.common(ModuleKind::AutoMobHeal, c);
    values.push(t.ranged("Range", "FLOAT", float_json(s.range, "AutoMobHeal.Range")?, json!({"from":1.,"to":6.}), "", &key("Range")));
    values.push(t.ranged("Delay", "INT_RANGE", json!({"from":s.delay[0],"to":s.delay[1]}), json!({"from":0,"to":40}), "ticks", &key("Delay")));
    values.push(t.ranged("SlotResetDelay", "INT", json!(s.slot_reset_delay), json!({"from":0,"to":20}), "ticks", &key("SlotResetDelay")));
    let mut swing = t.leaf("SwingMode", "CHOOSE", json!(s.swing_mode), &key("SwingMode"), None);
    if f == JsonFormat::Interop { swing["choices"] = json!(["DoNotHide", "HideForClient"]); }
    values.push(swing);
    for kind in Kind::ALL {
        let name = kind.tag();
        let target = &s.targets[kind as usize];
        let mut leaves = vec![boolean("Enabled", target.enabled, name), percent("HealthThreshold", target.health_threshold, &format!("{name}.HealthThreshold"))];
        if kind.food() { leaves.push(boolean("NoWaste", target.no_waste, &format!("{name}.NoWaste"))); }
        match kind {
            Kind::Wolf => leaves.push(t.group("Armor", "TOGGLEABLE", vec![
                boolean("Enabled", s.wolf_armor, "Wolf.Armor"),
                boolean("Equip", s.wolf_equip, "Wolf.Armor.Equip"),
                percent("RepairThreshold", s.repair_threshold, "Wolf.Armor.RepairThreshold"),
                boolean("OnlyWhenSitting", s.only_when_sitting, "Wolf.Armor.OnlyWhenSitting"),
            ])),
            Kind::HorseFamily => leaves.extend([
                boolean("AllowGoldenCarrot", s.allow_golden_carrot, "HorseFamily.AllowGoldenCarrot"),
                boolean("AllowGoldenApple", s.allow_golden_apple, "HorseFamily.AllowGoldenApple"),
                boolean("AllowEnchantedGoldenApple", s.allow_enchanted_golden_apple, "HorseFamily.AllowEnchantedGoldenApple"),
            ]),
            Kind::Nautilus => leaves.push(boolean("AllowBucketFood", s.allow_bucket_food, "Nautilus.AllowBucketFood")),
            _ => {}
        }
        values.push(t.group(name, "TOGGLEABLE", leaves));
    }
    Ok(t.group("AutoMobHeal", "TOGGLEABLE", values))
}
fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
    leaf(p, n, name, &format!("AutoMobHeal.{path}"), |v, _| {
        let setting = AutoMobHealSetting(path.into(), v.clone());
        setting.clone().apply(&mut Settings::default())?;
        Ok(SettingChange::AutoMobHeal(setting))
    });
}
pub(super) fn parse_auto_mob_heal(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["Range", "Delay", "SlotResetDelay", "SwingMode"] { add(p, root, name, name); }
    for kind in Kind::ALL {
        let group = kind.tag();
        let Some(v) = root.get(group) else { continue; };
        match nodes(v, group, &format!("AutoMobHeal.{group}")) {
            Ok(n) => {
                for name in ["Enabled", "HealthThreshold"] { add(p, &n, name, &format!("{group}.{name}")); }
                if kind.food() { add(p, &n, "NoWaste", &format!("{group}.NoWaste")); }
                if kind == Kind::HorseFamily {
                    for name in ["AllowGoldenCarrot", "AllowGoldenApple", "AllowEnchantedGoldenApple"] { add(p, &n, name, &format!("{group}.{name}")); }
                }
                if kind == Kind::Nautilus { add(p, &n, "AllowBucketFood", "Nautilus.AllowBucketFood"); }
                if kind == Kind::Wolf {
                    if let Some(v) = n.get("Armor") {
                        match nodes(v, "Armor", "AutoMobHeal.Wolf.Armor") {
                            Ok(n) => for name in ["Enabled", "Equip", "RepairThreshold", "OnlyWhenSitting"] { add(p, &n, name, &format!("Wolf.Armor.{name}")); },
                            Err(e) => p.issues.push(e),
                        }
                    }
                }
            }
            Err(e) => p.issues.push(e),
        }
    }
}
