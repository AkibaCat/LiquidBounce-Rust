// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::xray::Settings;
#[derive(Clone, Debug, PartialEq)]
pub struct XRaySetting(pub String, pub Value);
impl XRaySetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        match self.0.as_str() {
            "FullBright" => s.full_bright = parse_bool(&self.1, &self.0)?,
            "ExposedOnly" => s.exposed_only = parse_bool(&self.1, &self.0)?,
            "BackgroundOpacity" => {
                s.background_opacity = self
                    .1
                    .as_i64()
                    .ok_or_else(|| ConfigError::new(&self.0, "expected integer"))?
                    .clamp(0, 255) as u8
            }
            "Blocks" => {
                let a = self
                    .1
                    .as_array()
                    .ok_or_else(|| ConfigError::new(&self.0, "expected block list"))?;
                let mut blocks = std::collections::BTreeSet::new();
                for v in a {
                    let id = scalar_string(v)
                        .ok_or_else(|| ConfigError::new(&self.0, "expected block identifier"))?;
                    blocks.insert(if id.contains(':') {
                        id
                    } else {
                        format!("minecraft:{id}")
                    });
                }
                s.blocks = blocks;
            }
            _ => return Err(ConfigError::new(&self.0, "unknown XRay setting")),
        }
        Ok(())
    }
}
pub fn serialize_xray(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::XRay, c);
    for (name, ty, value, range) in [
        ("FullBright", "BOOLEAN", json!(s.full_bright), None),
        ("ExposedOnly", "BOOLEAN", json!(s.exposed_only), None),
        (
            "BackgroundOpacity",
            "INT",
            json!(s.background_opacity),
            Some(json!({"from":0,"to":255})),
        ),
        ("Blocks", "REGISTRY_LIST", json!(s.blocks), None),
    ] {
        let key = format!(
            "liquidbounce.module.xRay.{}{}",
            name[..1].to_ascii_lowercase(),
            &name[1..]
        );
        let mut v = if let Some(range) = range {
            t.ranged(name, ty, value, range, "", &key)
        } else {
            t.leaf(name, ty, value, &key, None)
        };
        if f == JsonFormat::Interop && name == "Blocks" {
            v["registry"] = json!("block");
        }
        values.push(v);
    }
    Ok(t.group("XRay", "TOGGLEABLE", values))
}
pub(super) fn parse_xray(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["FullBright", "ExposedOnly", "BackgroundOpacity", "Blocks"] {
        leaf(patch, root, name, &format!("XRay.{name}"), |v, _| {
            let s = XRaySetting(name.into(), v.clone());
            s.apply(&mut Settings::default())?;
            Ok(SettingChange::XRay(s))
        });
    }
}
