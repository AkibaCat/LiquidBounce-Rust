// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    block_esp::{ColorMode, Settings},
    breadcrumbs::{STYLES, specs},
};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq)]
pub struct BlockEspSetting(pub String, pub Value);
fn key(p: &str) -> String {
    format!(
        "liquidbounce.module.blockESP.{}",
        p.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn error(p: &str, message: &str) -> ConfigError {
    ConfigError::new(&format!("BlockESP.{p}"), message)
}
fn text(v: &Value, p: &str) -> Result<String, ConfigError> {
    scalar_string(v).ok_or_else(|| error(p, "expected string"))
}
fn color(v: &Value, p: &str) -> Result<u32, ConfigError> {
    if let Some(v) = v.as_str() {
        let h = v.strip_prefix('#').unwrap_or(v);
        if !matches!(h.len(), 6 | 8) {
            return Err(error(p, "expected 6/8 hex digits"));
        }
        u32::from_str_radix(h, 16)
            .map(|v| v | if h.len() == 6 { 0xff000000 } else { 0 })
            .map_err(|_| error(p, "invalid ARGB color"))
    } else {
        v.as_i64()
            .map(|v| v as u32)
            .or_else(|| v.as_u64().map(|v| v as u32))
            .ok_or_else(|| error(p, "expected ARGB integer"))
    }
}
impl BlockEspSetting {
    pub fn apply(&self, s: &mut Settings, registry: &[String]) -> Result<(), ConfigError> {
        let p = self.0.as_str();
        let v = &self.1;
        let path = format!("BlockESP.{p}");
        match p {
            "Mode" => {
                s.glow = match text(v, p)?.as_str() {
                    "Box" => false,
                    "Glow" => true,
                    _ => return Err(error(p, "unknown case-sensitive mode")),
                }
            }
            "Mode.Box.Outline" => s.outline = parse_bool(v, &path)?,
            "Mode.Glow.Style" => {
                s.style = std::iter::once("Vanilla")
                    .chain(STYLES)
                    .position(|n| v == n)
                    .ok_or_else(|| error(p, "unknown case-sensitive style"))?
            }
            "Targets" => {
                let values = match v {
                    Value::Array(a) => a.as_slice(),
                    _ => std::slice::from_ref(v),
                };
                let mut result = BTreeSet::new();
                for v in values {
                    let raw = text(v, p)?;
                    let id = match raw.split_once(':') {
                        None => format!("minecraft:{raw}"),
                        Some(("", n)) => format!("minecraft:{n}"),
                        Some(_) => raw,
                    };
                    result.insert(if registry.is_empty() || registry.contains(&id) {
                        id
                    } else {
                        "minecraft:air".into()
                    });
                }
                s.targets = result;
            }
            "ColorMode" => {
                s.color_mode = ColorMode::parse(&text(v, p)?)
                    .ok_or_else(|| error(p, "unknown case-sensitive color mode"))?
            }
            "ColorMode.Static.Color" => s.color = color(v, p)?,
            "MergeAdjacent" => s.merge_adjacent = parse_bool(v, &path)?,
            "DistanceFade" => {
                let mut values: [f32; 4] = serde_json::from_value(v.clone())
                    .map_err(|_| error(p, "expected four distances"))?;
                for v in &mut values {
                    if !v.is_finite() {
                        return Err(error(p, "expected finite distance"));
                    }
                    *v = v.clamp(0., 512.);
                }
                for i in 1..4 {
                    values[i] = values[i].max(values[i - 1]);
                }
                s.fade = values;
            }
            _ => {
                if let Some(n) = p.strip_prefix("DistanceFade.") {
                    let i = ["NearStart", "NearEnd", "FarStart", "FarEnd"]
                        .iter()
                        .position(|v| *v == n)
                        .ok_or_else(|| error(p, "unknown distance field"))?;
                    let value = parse_float(v, &path)?;
                    if !value.is_finite() {
                        return Err(error(p, "expected finite distance"));
                    }
                    s.set_fade(i, value);
                    return Ok(());
                }
                for (i, name) in STYLES.iter().enumerate() {
                    if let Some(n) = p.strip_prefix(&format!("Mode.Glow.Style.{name}.")) {
                        let spec = specs(i)
                            .find(|x| x.name == n)
                            .ok_or_else(|| error(p, "unknown style parameter"))?;
                        if spec.color {
                            s.styles[i].accent = color(v, p)? as i32;
                        } else {
                            let value = parse_float(v, &path)?;
                            if !value.is_finite() {
                                return Err(error(p, "expected finite style parameter"));
                            }
                            s.styles[i]
                                .values
                                .insert(n.into(), value.clamp(spec.from, spec.to));
                        }
                        return Ok(());
                    }
                }
                return Err(error(p, "unknown setting"));
            }
        }
        Ok(())
    }
}
fn meta(mut v: Value, p: &str, t: &Tree) -> Value {
    if t.format == JsonFormat::Interop {
        v["key"] = json!(key(p));
        v["description"] = json!((t.describe)(&format!("{}.description", key(p))));
    }
    v
}
fn choice(t: &Tree, name: &str, p: &str, active: &str, choices: Map<String, Value>) -> Value {
    let mut v = t.mode(active, choices);
    v["name"] = json!(name);
    meta(v, p, t)
}
pub fn serialize_block_esp(
    s: &Settings,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::BlockEsp, common);
    let leaf = |n: &str, ty: &str, v: Value, p: &str| t.leaf(n, ty, v, &key(p), None);
    let mut styles = Map::new();
    styles.insert("Vanilla".into(), t.group("Vanilla", "CONFIGURABLE", vec![]));
    for (i, name) in STYLES.iter().enumerate() {
        let mut fields = Vec::new();
        for spec in specs(i) {
            let p = format!("Mode.Glow.Style.{name}.{}", spec.name);
            fields.push(if spec.color {
                leaf(spec.name, "COLOR", json!(s.styles[i].accent), &p)
            } else {
                t.ranged(
                    spec.name,
                    "FLOAT",
                    float_json(s.styles[i].get(spec.name), &p)?,
                    json!({"from":spec.from,"to":spec.to}),
                    spec.suffix,
                    &key(&p),
                )
            });
        }
        styles.insert((*name).into(), t.group(name, "CONFIGURABLE", fields));
    }
    let style = choice(&t, "Style", "Mode.Glow.Style", s.style_name(), styles);
    values.push(choice(
        &t,
        "Mode",
        "Mode",
        if s.glow { "Glow" } else { "Box" },
        Map::from_iter([
            (
                "Box".into(),
                t.group(
                    "Box",
                    "CONFIGURABLE",
                    vec![leaf(
                        "Outline",
                        "BOOLEAN",
                        json!(s.outline),
                        "Mode.Box.Outline",
                    )],
                ),
            ),
            ("Glow".into(), t.group("Glow", "CONFIGURABLE", vec![style])),
        ]),
    ));
    let mut targets = leaf("Targets", "REGISTRY_LIST", json!(s.targets), "Targets");
    if format == JsonFormat::Interop {
        targets["registry"] = json!("block");
    }
    values.push(targets);
    values.push(choice(
        &t,
        "ColorMode",
        "ColorMode",
        s.color_mode.tag(),
        Map::from_iter([
            (
                "MapColor".into(),
                t.group("MapColor", "CONFIGURABLE", vec![]),
            ),
            (
                "Static".into(),
                t.group(
                    "Static",
                    "CONFIGURABLE",
                    vec![leaf(
                        "Color",
                        "COLOR",
                        json!(s.color as i32),
                        "ColorMode.Static.Color",
                    )],
                ),
            ),
            ("Rainbow".into(), t.group("Rainbow", "CONFIGURABLE", vec![])),
        ]),
    ));
    let fields = ["NearStart", "NearEnd", "FarStart", "FarEnd"]
        .into_iter()
        .enumerate()
        .map(|(i, n)| {
            let p = format!("DistanceFade.{n}");
            Ok(t.ranged(
                n,
                "FLOAT",
                float_json(s.fade[i], &p)?,
                json!({"from":0,"to":512}),
                "",
                &key(&p),
            ))
        })
        .collect::<Result<Vec<_>, ConfigError>>()?;
    values.push(meta(
        t.group("DistanceFade", "CONFIGURABLE", fields),
        "DistanceFade",
        &t,
    ));
    values.push(leaf(
        "MergeAdjacent",
        "BOOLEAN",
        json!(s.merge_adjacent),
        "MergeAdjacent",
    ));
    Ok(t.group("BlockESP", "TOGGLEABLE", values))
}
fn add(patch: &mut SettingsPatch, n: &Nodes, name: &str, path: &str) {
    leaf(patch, n, name, &format!("BlockESP.{path}"), |v, _| {
        let setting = BlockEspSetting(path.into(), v.clone());
        setting.apply(&mut Settings::default(), &[])?;
        Ok(SettingChange::BlockEsp(setting))
    });
}
fn mode(patch: &mut SettingsPatch, v: &Value, p: &str) {
    let Some(active) = v["active"].as_str() else {
        patch.issues.push(error(p, "expected active mode"));
        return;
    };
    let setting = BlockEspSetting(p.into(), json!(active));
    match setting.apply(&mut Settings::default(), &[]) {
        Ok(()) => patch.changes.push(SettingChange::BlockEsp(setting)),
        Err(e) => patch.issues.push(e),
    }
}
fn group(
    patch: &mut SettingsPatch,
    v: &Value,
    name: &str,
    p: &str,
    f: impl FnOnce(&mut SettingsPatch, &Nodes),
) {
    match nodes(v, name, &format!("BlockESP.{p}")) {
        Ok(n) => f(patch, &n),
        Err(e) => patch.issues.push(e),
    }
}
pub(super) fn parse_block_esp(patch: &mut SettingsPatch, root: &Nodes) {
    if let Some(m) = root.get("Mode") {
        mode(patch, m, "Mode");
        if let Some(v) = m["choices"].get("Box") {
            group(patch, v, "Box", "Mode.Box", |p, n| {
                add(p, n, "Outline", "Mode.Box.Outline")
            });
        }
        if let Some(v) = m["choices"].get("Glow") {
            group(patch, v, "Glow", "Mode.Glow", |p, n| {
                if let Some(s) = n.get("Style") {
                    mode(p, s, "Mode.Glow.Style");
                    for (i, name) in STYLES.iter().enumerate() {
                        if let Some(v) = s["choices"].get(*name) {
                            group(p, v, name, &format!("Mode.Glow.Style.{name}"), |p, n| {
                                for spec in specs(i) {
                                    add(
                                        p,
                                        n,
                                        spec.name,
                                        &format!("Mode.Glow.Style.{name}.{}", spec.name),
                                    );
                                }
                            });
                        }
                    }
                }
            });
        }
    }
    if let Some(m) = root.get("ColorMode") {
        mode(patch, m, "ColorMode");
        if let Some(v) = m["choices"].get("Static") {
            group(patch, v, "Static", "ColorMode.Static", |p, n| {
                add(p, n, "Color", "ColorMode.Static.Color")
            });
        }
    }
    for n in ["Targets", "MergeAdjacent"] {
        add(patch, root, n, n);
    }
    if let Some(v) = root.get("DistanceFade") {
        group(patch, v, "DistanceFade", "DistanceFade", |p, n| {
            let names = ["NearStart", "NearEnd", "FarStart", "FarEnd"];
            if names.iter().all(|v| n.contains_key(*v)) {
                let values = names
                    .into_iter()
                    .map(|name| {
                        value_of(n[name], &format!("BlockESP.DistanceFade.{name}"))
                            .and_then(|v| parse_float(v, &format!("BlockESP.DistanceFade.{name}")))
                    })
                    .collect::<Result<Vec<_>, ConfigError>>();
                match values {
                    Ok(v) => {
                        let setting = BlockEspSetting("DistanceFade".into(), json!(v));
                        match setting.apply(&mut Settings::default(), &[]) {
                            Ok(()) => p.changes.push(SettingChange::BlockEsp(setting)),
                            Err(e) => p.issues.push(e),
                        }
                    }
                    Err(e) => p.issues.push(e),
                }
                return;
            }
            for name in names {
                add(p, n, name, &format!("DistanceFade.{name}"));
            }
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_inactive_styles_targets_and_color_modes_roundtrip() {
        let mut expected = Settings::default();
        expected.glow = true;
        expected.style = 7;
        expected.color_mode = ColorMode::Static;
        expected.color = 0x8099aabb;
        expected.fade = [2., 8., 64., 128.];
        expected.merge_adjacent = true;
        expected.targets = ["minecraft:red_bed".into(), "minecraft:dragon_egg".into()]
            .into_iter()
            .collect();
        expected.styles[2].values.insert("CoreMix".into(), 0.25);
        let common = CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Toggle"}),
        };
        for f in [JsonFormat::File, JsonFormat::Interop] {
            let value = serialize_block_esp(&expected, &common, f, &str::to_owned).unwrap();
            let patch = parse_settings(ModuleKind::BlockEsp, &value).unwrap();
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            let mut actual = Settings::default();
            for change in patch.changes {
                if let SettingChange::BlockEsp(s) = change {
                    s.apply(&mut actual, &[]).unwrap();
                }
            }
            assert_eq!(actual, expected);
        }
    }
    #[test]
    fn bad_mode_isolated_registry_fallback_and_fade_callbacks() {
        let mut s = Settings::default();
        assert!(
            BlockEspSetting("ColorMode".into(), json!("mapcolor"))
                .apply(&mut s, &[])
                .is_err()
        );
        BlockEspSetting("Targets".into(), json!(["red_bed", "missing"]))
            .apply(&mut s, &["minecraft:red_bed".into()])
            .unwrap();
        assert_eq!(
            s.targets,
            ["minecraft:red_bed".into(), "minecraft:air".into()]
                .into_iter()
                .collect()
        );
        BlockEspSetting("DistanceFade".into(), json!([8., 4., 3., 2.]))
            .apply(&mut s, &[])
            .unwrap();
        assert_eq!(s.fade, [8., 8., 8., 8.]);
        BlockEspSetting("DistanceFade.NearStart".into(), json!(20))
            .apply(&mut s, &[])
            .unwrap();
        assert_eq!(s.fade[0], 8.);
    }
}
