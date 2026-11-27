// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::hit_fx::{PARTICLES, SOUNDS, Settings};
#[derive(Clone, Debug, PartialEq)]
pub struct HitFxSetting(pub String, pub Value);
impl HitFxSetting {
    pub fn apply(self, s: &mut Settings) {
        match self.0.as_str() {
            "Particle" => s.particles = self.1.as_u64().unwrap() as u32,
            "OtherSound" => s.other_sound = self.1.as_u64().unwrap() as u32,
            "SelfSound" => s.self_sound = self.1.as_u64().unwrap() as u32,
            "ParticleAmount" => {
                s.amount = [
                    self.1["from"].as_i64().unwrap() as i32,
                    self.1["to"].as_i64().unwrap() as i32,
                ]
            }
            _ => {}
        }
    }
}
pub fn serialize_hit_fx(
    s: &Settings,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::HitFx, c);
    let names = SOUNDS.iter().map(|v| v.0).collect::<Vec<_>>();
    let key = |p: &str| {
        format!(
            "liquidbounce.module.hitFX.{}",
            p[..1].to_ascii_lowercase() + &p[1..]
        )
    };
    values.push(t.multi("Particle", s.particles, &PARTICLES, &key("Particle")));
    values.push(t.ranged(
        "ParticleAmount",
        "INT_RANGE",
        json!({"from":s.amount[0],"to":s.amount[1]}),
        json!({"from":1,"to":20}),
        "",
        &key("ParticleAmount"),
    ));
    values.push(t.multi("OtherSound", s.other_sound, &names, &key("OtherSound")));
    values.push(t.multi("SelfSound", s.self_sound, &names, &key("SelfSound")));
    Ok(t.group("HitFX", "TOGGLEABLE", values))
}
pub(super) fn parse_hit_fx(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["Particle", "ParticleAmount", "OtherSound", "SelfSound"] {
        leaf(patch, root, name, &format!("HitFX.{name}"), |v, p| {
            let value = if name == "ParticleAmount" {
                let a = parse_int(&v["from"], p)?;
                let b = parse_int(&v["to"], p)?;
                if !(1..=20).contains(&a) || !(a..=20).contains(&b) {
                    return Err(ConfigError::new(p, "expected ordered range within 1..20"));
                }
                json!({"from":a,"to":b})
            } else {
                let names = SOUNDS.iter().map(|v| v.0).collect::<Vec<_>>();
                json!(parse_multi_mask(
                    v,
                    p,
                    if name == "Particle" {
                        &PARTICLES
                    } else {
                        &names
                    }
                )?)
            };
            Ok(SettingChange::HitFx(HitFxSetting(name.into(), value)))
        })
    }
}
