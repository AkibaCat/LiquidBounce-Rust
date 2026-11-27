// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    breadcrumbs::{STYLES, specs},
    item_esp::{ItemEsp, Mode},
};
#[derive(Clone, Debug, PartialEq)]
pub struct ItemEspSetting(pub String, pub Value);
impl ItemEspSetting {
    pub fn apply(&self, s: &mut ItemEsp, items: &[String]) -> Result<(), ConfigError> {
        let p = format!("ItemESP.{}", self.0);
        let v = &self.1;
        match self.0.as_str() {
            "Filter" => {
                s.whitelist = scalar_string(v)
                    .and_then(|v| {
                        ["Blacklist", "Whitelist"]
                            .into_iter()
                            .position(|n| n.eq_ignore_ascii_case(&v))
                    })
                    .map(|i| i == 1)
                    .ok_or_else(|| ConfigError::new(&p, "unknown filter"))?
            }
            "Items" => {
                let entries = match v {
                    Value::Array(a) => a.clone(),
                    _ => vec![v.clone()],
                };
                let mut result = std::collections::BTreeSet::new();
                for entry in entries {
                    let raw = match entry {
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        _ => return Err(ConfigError::new(&p, "expected item registry primitive")),
                    };
                    let id = if raw.starts_with(':') {
                        format!("minecraft{raw}")
                    } else if !raw.contains(':') {
                        format!("minecraft:{raw}")
                    } else {
                        raw
                    };
                    result.insert(if items.contains(&id) {
                        id
                    } else {
                        "minecraft:air".into()
                    });
                }
                s.items = result;
            }
            "MaximumDistance" => s.maximum_distance = parse_float(v, &p)?,
            "Tracers" => s.tracers = parse_bool(v, &p)?,
            "ShowArrows.Enabled" => s.arrows = parse_bool(v, &p)?,
            "ShowArrows.RegularArrows" => s.regular_arrows = parse_bool(v, &p)?,
            "ShowArrows.SpectralArrows" => s.spectral_arrows = parse_bool(v, &p)?,
            "ShowArrows.ArrowsWithEffects" => s.arrows_with_effects = parse_bool(v, &p)?,
            "ShowTridents" => s.tridents = parse_bool(v, &p)?,
            "Mode" => {
                let mode = scalar_string(v)
                    .and_then(|v| Mode::ALL.into_iter().find(|m| m.tag() == v))
                    .ok_or_else(|| ConfigError::new(&p, "unknown case-sensitive mode"))?;
                if s.mode != mode {
                    s.clear();
                    s.mode = mode;
                }
            }
            "Mode.Box.MergeIntersecting" => s.merge_intersecting = parse_bool(v, &p)?,
            "Mode.Legacy2D.Scale" => s.scale = parse_float(v, &p)?,
            "Mode.Legacy2D.YOffset" => s.y_offset = parse_float(v, &p)?,
            "Mode.Legacy2D.BackgroundAlpha" => s.background_alpha = parse_int(v, &p)?,
            "ColorMode" => {
                s.rainbow = match scalar_string(v).as_deref() {
                    Some("Static") => false,
                    Some("Rainbow") => true,
                    _ => return Err(ConfigError::new(&p, "unknown case-sensitive color mode")),
                }
            }
            "ColorMode.Static.Color" => s.color = parse_color(v, &p)?,
            "Mode.Glow.Style" => {
                s.style = scalar_string(v)
                    .and_then(|v| {
                        std::iter::once("Vanilla")
                            .chain(STYLES)
                            .position(|n| n == v)
                    })
                    .ok_or_else(|| ConfigError::new(&p, "unknown case-sensitive style"))?
            }
            _ => {
                let mut matched = false;
                for (i, name) in STYLES.iter().enumerate() {
                    if let Some(leaf) = self.0.strip_prefix(&format!("Mode.Glow.Style.{name}.")) {
                        let spec = specs(i)
                            .find(|spec| spec.name == leaf)
                            .ok_or_else(|| ConfigError::new(&p, "unknown style setting"))?;
                        if spec.color {
                            s.styles[i].accent = parse_color(v, &p)?;
                        } else {
                            s.styles[i].values.insert(leaf.into(), parse_float(v, &p)?);
                        }
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    return Err(ConfigError::new(&p, "unknown ItemESP setting"));
                }
            }
        }
        Ok(())
    }
}
fn parse_color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    if let Some(s) = v.as_str() {
        let h = s.strip_prefix('#').unwrap_or(s);
        if !matches!(h.len(), 6 | 8) {
            return Err(ConfigError::new(p, "expected 6/8 hex digits"));
        }
        i64::from_str_radix(h, 16)
            .map(|v| {
                v as i32
                    | if h.len() == 6 {
                        0xff000000u32 as i32
                    } else {
                        0
                    }
            })
            .map_err(|_| ConfigError::new(p, "invalid ARGB color"))
    } else {
        parse_int(v, p)
    }
}
pub fn item_esp_key(p: &str) -> String {
    format!(
        "liquidbounce.module.itemEsp.{}",
        p.split('.')
            .map(|v| v[..1].to_ascii_lowercase() + &v[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn choose(t: &Tree, name: &str, path: &str, active: &str, choices: Map<String, Value>) -> Value {
    let mut v = t.mode(active, choices);
    v["name"] = json!(name);
    if t.format == JsonFormat::Interop {
        let key = item_esp_key(path);
        v["key"] = json!(key);
        v["description"] = json!((t.describe)(&format!("{key}.description")));
    }
    v
}
pub fn serialize_item_esp(
    s: &ItemEsp,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::ItemEsp, c);
    let mut filter = t.leaf(
        "Filter",
        "CHOOSE",
        json!(if s.whitelist {
            "Whitelist"
        } else {
            "Blacklist"
        }),
        &item_esp_key("Filter"),
        None,
    );
    if format == JsonFormat::Interop {
        filter["choices"] = json!(["Blacklist", "Whitelist"]);
    }
    values.push(filter);
    let mut sorted_items = s.items.iter().collect::<Vec<_>>();
    sorted_items.sort_by(|a, b| {
        let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
        let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
        ap.cmp(bp).then_with(|| an.cmp(bn))
    });
    let mut items = t.leaf(
        "Items",
        "REGISTRY_LIST",
        json!(sorted_items),
        &item_esp_key("Items"),
        None,
    );
    if format == JsonFormat::Interop {
        items["registry"] = json!("item");
    }
    values.push(items);
    values.push(t.ranged(
        "MaximumDistance",
        "FLOAT",
        float_json(s.maximum_distance, "MaximumDistance")?,
        json!({"from":1,"to":512}),
        "",
        &item_esp_key("MaximumDistance"),
    ));
    values.push(t.leaf(
        "Tracers",
        "BOOLEAN",
        json!(s.tracers),
        &item_esp_key("Tracers"),
        None,
    ));
    values.push(
        t.group(
            "ShowArrows",
            "TOGGLEABLE",
            [
                ("Enabled", s.arrows),
                ("RegularArrows", s.regular_arrows),
                ("SpectralArrows", s.spectral_arrows),
                ("ArrowsWithEffects", s.arrows_with_effects),
            ]
            .into_iter()
            .map(|(n, v)| {
                t.leaf(
                    n,
                    "BOOLEAN",
                    json!(v),
                    &item_esp_key(&format!("ShowArrows.{n}")),
                    None,
                )
            })
            .collect(),
        ),
    );
    values.push(t.leaf(
        "ShowTridents",
        "BOOLEAN",
        json!(s.tridents),
        &item_esp_key("ShowTridents"),
        None,
    ));
    let mut styles = Map::new();
    styles.insert("Vanilla".into(), t.group("Vanilla", "CONFIGURABLE", vec![]));
    for (i, name) in STYLES.iter().enumerate() {
        let mut fields = vec![];
        for spec in specs(i) {
            let path = format!("Mode.Glow.Style.{name}.{}", spec.name);
            fields.push(if spec.color {
                t.leaf(
                    spec.name,
                    "COLOR",
                    json!(s.styles[i].accent),
                    &item_esp_key(&path),
                    None,
                )
            } else {
                t.ranged(
                    spec.name,
                    "FLOAT",
                    float_json(s.styles[i].get(spec.name), &path)?,
                    json!({"from":spec.from,"to":spec.to}),
                    spec.suffix,
                    &item_esp_key(&path),
                )
            });
        }
        styles.insert((*name).into(), t.group(name, "CONFIGURABLE", fields));
    }
    let style = choose(
        &t,
        "Style",
        "Mode.Glow.Style",
        if s.style == 0 {
            "Vanilla"
        } else {
            STYLES[s.style - 1]
        },
        styles,
    );
    let legacy = vec![
        t.ranged(
            "Scale",
            "FLOAT",
            float_json(s.scale, "Scale")?,
            json!({"from":0.02,"to":0.3}),
            "",
            &item_esp_key("Mode.Legacy2D.Scale"),
        ),
        t.ranged(
            "YOffset",
            "FLOAT",
            float_json(s.y_offset, "YOffset")?,
            json!({"from":-1,"to":1}),
            "",
            &item_esp_key("Mode.Legacy2D.YOffset"),
        ),
        t.ranged(
            "BackgroundAlpha",
            "INT",
            json!(s.background_alpha),
            json!({"from":0,"to":255}),
            "",
            &item_esp_key("Mode.Legacy2D.BackgroundAlpha"),
        ),
    ];
    values.push(choose(
        &t,
        "Mode",
        "Mode",
        s.mode.tag(),
        Map::from_iter([
            ("Glow".into(), t.group("Glow", "CONFIGURABLE", vec![style])),
            (
                "Box".into(),
                t.group(
                    "Box",
                    "CONFIGURABLE",
                    vec![t.leaf(
                        "MergeIntersecting",
                        "BOOLEAN",
                        json!(s.merge_intersecting),
                        &item_esp_key("Mode.Box.MergeIntersecting"),
                        None,
                    )],
                ),
            ),
            (
                "Legacy2D".into(),
                t.group("Legacy2D", "CONFIGURABLE", legacy),
            ),
        ]),
    ));
    values.push(choose(
        &t,
        "ColorMode",
        "ColorMode",
        if s.rainbow { "Rainbow" } else { "Static" },
        Map::from_iter([
            (
                "Static".into(),
                t.group(
                    "Static",
                    "CONFIGURABLE",
                    vec![t.leaf(
                        "Color",
                        "COLOR",
                        json!(s.color),
                        &item_esp_key("ColorMode.Static.Color"),
                        None,
                    )],
                ),
            ),
            ("Rainbow".into(), t.group("Rainbow", "CONFIGURABLE", vec![])),
        ]),
    ));
    Ok(t.group("ItemESP", "TOGGLEABLE", values))
}
fn setting(patch: &mut SettingsPatch, n: &Nodes, name: &str, path: &str) {
    leaf(patch, n, name, &format!("ItemESP.{path}"), |v, _| {
        let setting = ItemEspSetting(path.into(), v.clone());
        setting.apply(&mut ItemEsp::default(), &[])?;
        Ok(SettingChange::ItemEsp(setting))
    });
}
fn child<'a>(
    patch: &mut SettingsPatch,
    v: Option<&&'a Value>,
    name: &str,
    path: &str,
) -> Option<Nodes<'a>> {
    let v = *v?;
    match nodes(v, name, path) {
        Ok(v) => Some(v),
        Err(e) => {
            patch.issues.push(e);
            None
        }
    }
}
pub(super) fn parse_item_esp(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for n in [
        "Filter",
        "Items",
        "MaximumDistance",
        "Tracers",
        "ShowTridents",
    ] {
        setting(patch, root, n, n);
    }
    if let Some(n) = child(
        patch,
        root.get("ShowArrows"),
        "ShowArrows",
        "ItemESP.ShowArrows",
    ) {
        for name in [
            "Enabled",
            "RegularArrows",
            "SpectralArrows",
            "ArrowsWithEffects",
        ] {
            setting(patch, &n, name, &format!("ShowArrows.{name}"));
        }
    }
    for (name, modes) in [
        ("Mode", &["Glow", "Box", "Legacy2D"][..]),
        ("ColorMode", &["Static", "Rainbow"][..]),
    ] {
        let Some(v) = root.get(name) else { continue };
        let active = v.get("active");
        match active {
            Some(v) => {
                let s = ItemEspSetting(name.into(), v.clone());
                match s.apply(&mut ItemEsp::default(), &[]) {
                    Ok(()) => patch.changes.push(SettingChange::ItemEsp(s)),
                    Err(e) => patch.issues.push(e),
                }
            }
            None => patch.issues.push(ConfigError::new(
                &format!("ItemESP.{name}.active"),
                "missing active mode",
            )),
        }
        let Some(choices) = v.get("choices").and_then(Value::as_object) else {
            patch.issues.push(ConfigError::new(name, "missing choices"));
            continue;
        };
        for &mode in modes {
            let Some(v) = choices.get(mode) else {
                patch.issues.push(ConfigError::new(mode, "missing choice"));
                continue;
            };
            let n = match nodes(v, mode, mode) {
                Ok(n) => n,
                Err(e) => {
                    patch.issues.push(e);
                    continue;
                }
            };
            match (name, mode) {
                ("Mode", "Box") => {
                    setting(patch, &n, "MergeIntersecting", "Mode.Box.MergeIntersecting")
                }
                ("Mode", "Legacy2D") => {
                    for name in ["Scale", "YOffset", "BackgroundAlpha"] {
                        setting(patch, &n, name, &format!("Mode.Legacy2D.{name}"));
                    }
                }
                ("ColorMode", "Static") => setting(patch, &n, "Color", "ColorMode.Static.Color"),
                ("Mode", "Glow") => {
                    if let Some(v) = n.get("Style") {
                        if let Some(active) = v.get("active") {
                            let s = ItemEspSetting("Mode.Glow.Style".into(), active.clone());
                            match s.apply(&mut ItemEsp::default(), &[]) {
                                Ok(()) => patch.changes.push(SettingChange::ItemEsp(s)),
                                Err(e) => patch.issues.push(e),
                            }
                        } else {
                            patch.issues.push(ConfigError::new(
                                "ItemESP.Mode.Glow.Style.active",
                                "missing active style",
                            ));
                        }
                        if let Some(c) = v.get("choices").and_then(Value::as_object) {
                            for (i, style) in STYLES.iter().enumerate() {
                                let Some(v) = c.get(*style) else {
                                    patch
                                        .issues
                                        .push(ConfigError::new(style, "missing style choice"));
                                    continue;
                                };
                                let n = match nodes(v, style, style) {
                                    Ok(n) => n,
                                    Err(e) => {
                                        patch.issues.push(e);
                                        continue;
                                    }
                                };
                                for spec in specs(i) {
                                    setting(
                                        patch,
                                        &n,
                                        spec.name,
                                        &format!("Mode.Glow.Style.{style}.{}", spec.name),
                                    );
                                }
                            }
                        } else {
                            patch
                                .issues
                                .push(ConfigError::new("Style", "missing choices"));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
