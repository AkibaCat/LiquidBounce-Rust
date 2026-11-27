// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::hole_esp::{HoleEsp, MODES};
#[derive(Clone, Debug, PartialEq)]
pub struct HoleEspSetting(pub String, pub Value);
impl HoleEspSetting {
    pub fn apply(self, s: &mut HoleEsp) {
        match self.0.as_str() {
            "Mode" => {
                s.mode = MODES
                    .iter()
                    .position(|m| Some(*m) == self.1.as_str())
                    .unwrap()
            }
            "Mode.Box.Outline" => s.box_outline = self.1.as_bool().unwrap(),
            "Mode.GlowingPlane.Outline" => s.plane_outline = self.1.as_bool().unwrap(),
            "Mode.GlowingPlane.GlowHeight" => s.glow_height = self.1.as_f64().unwrap() as f32,
            "HorizontalScanDistance" => s.horizontal = self.1.as_i64().unwrap() as i32,
            "VerticalScanDistance" => s.vertical = self.1.as_i64().unwrap() as i32,
            "DistanceFade" => s.distance_fade = self.1.as_f64().unwrap() as f32,
            p => {
                if let Some(i) = ["1x1Bedrock", "1x1", "1x2", "2x2"]
                    .iter()
                    .position(|n| *n == p)
                {
                    s.colors[i] = self.1.as_i64().unwrap() as i32;
                }
            }
        }
    }
}
pub fn hole_esp_key(path: &str) -> String {
    format!(
        "liquidbounce.module.holeESP.{}",
        path.split('.')
            .map(|p| p[..1].to_ascii_lowercase() + &p[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn metadata(v: &mut Value, path: &str, t: &Tree) {
    if t.format == JsonFormat::Interop {
        let k = hole_esp_key(path);
        v["key"] = json!(k);
        v["description"] = json!((t.describe)(&format!("{k}.description")));
    }
}
pub fn serialize_hole_esp(
    s: &HoleEsp,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::HoleEsp, common);
    let mut choices = Map::new();
    for (name, outline) in [("Box", s.box_outline), ("GlowingPlane", s.plane_outline)] {
        let mut leaves = vec![t.leaf(
            "Outline",
            "BOOLEAN",
            json!(outline),
            &hole_esp_key(&format!("Mode.{name}.Outline")),
            None,
        )];
        if name == "GlowingPlane" {
            leaves.push(t.ranged(
                "GlowHeight",
                "FLOAT",
                float_json(s.glow_height, "HoleESP.Mode.GlowingPlane.GlowHeight")?,
                json!({"from":0,"to":1}),
                "",
                &hole_esp_key("Mode.GlowingPlane.GlowHeight"),
            ));
        }
        let mut group = t.group(name, "CONFIGURABLE", leaves);
        metadata(&mut group, &format!("Mode.{name}"), &t);
        choices.insert(name.into(), group);
    }
    let mut mode = t.mode(MODES[s.mode], choices);
    metadata(&mut mode, "Mode", &t);
    values.push(mode);
    for (name, value) in [
        ("HorizontalScanDistance", s.horizontal),
        ("VerticalScanDistance", s.vertical),
    ] {
        values.push(t.ranged(
            name,
            "INT",
            json!(value),
            json!({"from":4,"to":128}),
            "",
            &hole_esp_key(name),
        ));
    }
    values.push(t.ranged(
        "DistanceFade",
        "FLOAT",
        float_json(s.distance_fade, "HoleESP.DistanceFade")?,
        json!({"from":0,"to":1}),
        "",
        &hole_esp_key("DistanceFade"),
    ));
    for (name, value) in ["1x1Bedrock", "1x1", "1x2", "2x2"]
        .into_iter()
        .zip(s.colors)
    {
        values.push(t.leaf(name, "COLOR", json!(value), &hole_esp_key(name), None));
    }
    Ok(t.group("HoleESP", "TOGGLEABLE", values))
}
fn color(v: &Value, p: &str) -> Result<Value, ConfigError> {
    if let Some(s) = v.as_str() {
        let h = s.strip_prefix('#').unwrap_or(s);
        if !matches!(h.len(), 6 | 8) {
            return Err(ConfigError::new(p, "expected 6/8 hex digits"));
        }
        i64::from_str_radix(h, 16)
            .map(|v| {
                json!(
                    v as i32
                        | if h.len() == 6 {
                            0xff000000u32 as i32
                        } else {
                            0
                        }
                )
            })
            .map_err(|_| ConfigError::new(p, "invalid ARGB color"))
    } else {
        parse_int(v, p).map(|v| json!(v))
    }
}
fn field(patch: &mut SettingsPatch, n: &Nodes, name: &str, path: &str, kind: &str) {
    leaf(patch, n, name, &format!("HoleESP.{path}"), |v, p| {
        let value = match kind {
            "BOOLEAN" => json!(parse_bool(v, p)?),
            "INT" => json!(parse_int(v, p)?),
            "FLOAT" => float_json(parse_float(v, p)?, p)?,
            _ => color(v, p)?,
        };
        Ok(SettingChange::HoleEsp(HoleEspSetting(path.into(), value)))
    });
}
pub(super) fn parse_hole_esp(patch: &mut SettingsPatch, root: &Nodes) {
    if let Some(m) = root.get("Mode") {
        if let Some(active) = m["active"].as_str().filter(|s| MODES.contains(s)) {
            patch.changes.push(SettingChange::HoleEsp(HoleEspSetting(
                "Mode".into(),
                json!(active),
            )));
        } else {
            patch
                .issues
                .push(ConfigError::new("HoleESP.Mode", "unknown mode"));
        }
        for mode in MODES {
            if let Some(v) = m.get("choices").and_then(|v| v.get(mode)) {
                match nodes(v, mode, &format!("HoleESP.Mode.{mode}")) {
                    Ok(n) => {
                        field(
                            patch,
                            &n,
                            "Outline",
                            &format!("Mode.{mode}.Outline"),
                            "BOOLEAN",
                        );
                        if mode == "GlowingPlane" {
                            field(
                                patch,
                                &n,
                                "GlowHeight",
                                "Mode.GlowingPlane.GlowHeight",
                                "FLOAT",
                            );
                        }
                    }
                    Err(e) => patch.issues.push(e),
                }
            }
        }
    }
    for (name, kind) in [
        ("HorizontalScanDistance", "INT"),
        ("VerticalScanDistance", "INT"),
        ("DistanceFade", "FLOAT"),
        ("1x1Bedrock", "COLOR"),
        ("1x1", "COLOR"),
        ("1x2", "COLOR"),
        ("2x2", "COLOR"),
    ] {
        field(patch, root, name, name, kind);
    }
}
