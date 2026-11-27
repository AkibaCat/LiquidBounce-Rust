// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::particles::{Settings, TEXTURES};
#[derive(Clone, Debug, PartialEq)]
pub struct ParticlesSetting(pub String, pub Value);
impl ParticlesSetting {
    pub fn apply(self, s: &mut Settings) {
        match self.0.as_str() {
            "Size" => s.size = self.1.as_f64().unwrap() as f32,
            "Count" => {
                s.count = [
                    self.1["from"].as_i64().unwrap() as i32,
                    self.1["to"].as_i64().unwrap() as i32,
                ]
            }
            "RandomParticleRotation" => s.rotate = self.1.as_bool().unwrap(),
            "Physical.Motion" => s.motion = self.1.as_f64().unwrap() as f32,
            "Physical.BounceX" => s.bounce[0] = self.1.as_f64().unwrap() as f32,
            "Physical.BounceY" => s.bounce[1] = self.1.as_f64().unwrap() as f32,
            "Physical.BounceZ" => s.bounce[2] = self.1.as_f64().unwrap() as f32,
            "Physical.Drag" => s.drag = self.1.as_f64().unwrap() as f32,
            "Physical.GravityFactor" => s.gravity = self.1.as_f64().unwrap() as f32,
            "Color" => s.color = self.1.as_i64().unwrap() as i32,
            "Particle" => s.textures = self.1.as_u64().unwrap() as u32,
            _ => {}
        }
    }
}
pub fn particles_key(path: &str) -> String {
    format!(
        "liquidbounce.module.particles.{}",
        path.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
pub fn serialize_particles(
    s: &Settings,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::Particles, c);
    values.push(t.ranged(
        "Size",
        "FLOAT",
        float_json(s.size, "Particles.Size")?,
        json!({"from":0.5,"to":2.}),
        "",
        &particles_key("Size"),
    ));
    values.push(t.ranged(
        "Count",
        "INT_RANGE",
        json!({"from":s.count[0],"to":s.count[1]}),
        json!({"from":2,"to":30}),
        "particles",
        &particles_key("Count"),
    ));
    values.push(t.leaf(
        "RandomParticleRotation",
        "BOOLEAN",
        json!(s.rotate),
        &particles_key("RandomParticleRotation"),
        None,
    ));
    let mut leaves = vec![];
    for (name, value, min, max) in [
        ("Motion", s.motion, 1., 30.),
        ("BounceX", s.bounce[0], 0., 1.),
        ("BounceY", s.bounce[1], 0., 1.),
        ("BounceZ", s.bounce[2], 0., 1.),
        ("Drag", s.drag, 0., 1.),
        ("GravityFactor", s.gravity, 0., 1.),
    ] {
        let path = format!("Physical.{name}");
        leaves.push(t.ranged(
            name,
            "FLOAT",
            float_json(value, &format!("Particles.{path}"))?,
            json!({"from":min,"to":max}),
            "",
            &particles_key(&path),
        ));
    }
    let mut physical = t.group("Physical", "CONFIGURABLE", leaves);
    if format == JsonFormat::Interop {
        physical["key"] = json!(particles_key("Physical"));
        physical["description"] = json!(describe(&format!(
            "{}.description",
            particles_key("Physical")
        )));
    }
    values.push(physical);
    values.push(t.leaf(
        "Color",
        "COLOR",
        json!(s.color),
        &particles_key("Color"),
        None,
    ));
    let mut particle = t.multi(
        "Particle",
        s.textures,
        &TEXTURES,
        &particles_key("Particle"),
    );
    if format == JsonFormat::Interop {
        particle["canBeNone"] = json!(false);
    }
    values.push(particle);
    Ok(t.group("Particles", "TOGGLEABLE", values))
}
fn add(patch: &mut SettingsPatch, n: &Nodes, name: &str, path: &str) {
    leaf(patch, n, name, &format!("Particles.{path}"), |v, p| {
        let value = match path {
            "Count" => {
                let a = parse_int(&v["from"], p)?;
                let b = parse_int(&v["to"], p)?;
                if !(2..=30).contains(&a) || !(a..=30).contains(&b) {
                    return Err(ConfigError::new(p, "expected ordered range in 2..30"));
                }
                json!({"from":a,"to":b})
            }
            "RandomParticleRotation" => json!(parse_bool(v, p)?),
            "Color" => {
                if let Some(h) = v.as_str() {
                    let h = h.strip_prefix('#').unwrap_or(h);
                    if !matches!(h.len(), 6 | 8) {
                        return Err(ConfigError::new(p, "expected 6/8 hex digits"));
                    }
                    json!(
                        (i64::from_str_radix(h, 16)
                            .map_err(|_| ConfigError::new(p, "invalid ARGB color"))?
                            as i32)
                            | if h.len() == 6 {
                                0xff000000u32 as i32
                            } else {
                                0
                            }
                    )
                } else {
                    json!(parse_int(v, p)?)
                }
            }
            "Particle" => {
                let mask = parse_multi_mask(v, p, &TEXTURES)?;
                if mask == 0 {
                    return Err(ConfigError::new(
                        p,
                        "at least one particle texture required",
                    ));
                }
                json!(mask)
            }
            _ => {
                let value = parse_float(v, p)?;
                let (lo, hi) = match path {
                    "Size" => (0.5, 2.),
                    "Physical.Motion" => (1., 30.),
                    _ => (0., 1.),
                };
                if value < lo || value > hi {
                    return Err(ConfigError::new(p, "value outside setting range"));
                }
                float_json(value, p)?
            }
        };
        Ok(SettingChange::Particles(ParticlesSetting(
            path.into(),
            value,
        )))
    });
}
pub(super) fn parse_particles(patch: &mut SettingsPatch, root: &Nodes) {
    for name in [
        "Size",
        "Count",
        "RandomParticleRotation",
        "Color",
        "Particle",
    ] {
        add(patch, root, name, name);
    }
    if let Some(v) = root.get("Physical") {
        match nodes(v, "Physical", "Particles.Physical") {
            Ok(n) => {
                for name in [
                    "Motion",
                    "BounceX",
                    "BounceY",
                    "BounceZ",
                    "Drag",
                    "GravityFactor",
                ] {
                    add(patch, &n, name, &format!("Physical.{name}"));
                }
            }
            Err(e) => patch.issues.push(e),
        }
    }
}
