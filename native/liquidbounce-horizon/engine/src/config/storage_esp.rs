// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::{
    breadcrumbs::{STYLES, specs},
    storage_esp::{StorageEsp, TYPES},
};
#[derive(Clone, Debug, PartialEq)]
pub struct StorageEspSetting(pub String, pub Value);
impl StorageEspSetting {
    pub fn apply(self, s: &mut StorageEsp) {
        let p = self.0.as_str();
        match p {
            "Mode" => s.glow = self.1 == "Glow",
            "Mode.Box.Outline" => s.outline = self.1.as_bool().unwrap(),
            "Mode.Glow.Style" => {
                s.style = std::iter::once("Vanilla")
                    .chain(STYLES)
                    .position(|v| self.1 == v)
                    .unwrap()
            }
            "RequiresChestStealer" => s.requires_chest_stealer = self.1.as_bool().unwrap(),
            "MergeAdjacent" => s.merge_adjacent = self.1.as_bool().unwrap(),
            "DistanceFade" => {
                let mut values: [f32; 4] = serde_json::from_value(self.1).unwrap();
                for value in &mut values {
                    *value = value.clamp(0., 512.);
                }
                for i in 1..4 {
                    values[i] = values[i].max(values[i - 1]);
                }
                s.fade = values;
            }
            _ => {
                if let Some(v) = p.strip_prefix("DistanceFade.") {
                    let i = ["NearStart", "NearEnd", "FarStart", "FarEnd"]
                        .iter()
                        .position(|n| *n == v)
                        .unwrap();
                    s.set_fade(i, self.1.as_f64().unwrap() as f32);
                    return;
                }
                for (i, name) in TYPES.iter().enumerate() {
                    if let Some(v) = p.strip_prefix(&format!("{name}.")) {
                        match v {
                            "Enabled" => s.categories[i].enabled = self.1.as_bool().unwrap(),
                            "Color" => s.categories[i].color = self.1.as_i64().unwrap() as i32,
                            "Tracers" => s.categories[i].tracers = self.1.as_bool().unwrap(),
                            _ => {}
                        }
                        return;
                    }
                }
                for (i, name) in STYLES.iter().enumerate() {
                    if let Some(v) = p.strip_prefix(&format!("Mode.Glow.Style.{name}.")) {
                        if specs(i).any(|x| x.name == v && x.color) {
                            s.styles[i].accent = self.1.as_i64().unwrap() as i32;
                        } else {
                            s.styles[i]
                                .values
                                .insert(v.into(), self.1.as_f64().unwrap() as f32);
                        }
                        return;
                    }
                }
            }
        }
    }
}
pub fn storage_esp_key(p: &str) -> String {
    format!(
        "liquidbounce.module.storageESP.{}",
        p.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn metadata(v: &mut Value, p: &str, t: &Tree) {
    if t.format == JsonFormat::Interop {
        let k = storage_esp_key(p);
        v["key"] = json!(k);
        v["description"] = json!((t.describe)(&format!("{k}.description")));
    }
}
fn choice(t: &Tree, name: &str, p: &str, active: &str, choices: Map<String, Value>) -> Value {
    let mut v = t.mode(active, choices);
    v["name"] = json!(name);
    metadata(&mut v, p, t);
    v
}
pub fn serialize_storage_esp(
    s: &StorageEsp,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::StorageEsp, common);
    let mut styles = Map::new();
    styles.insert("Vanilla".into(), t.group("Vanilla", "CONFIGURABLE", vec![]));
    for (i, name) in STYLES.iter().enumerate() {
        let mut fields = vec![];
        for spec in specs(i) {
            let p = format!("Mode.Glow.Style.{name}.{}", spec.name);
            fields.push(if spec.color {
                t.leaf(
                    spec.name,
                    "COLOR",
                    json!(s.styles[i].accent),
                    &storage_esp_key(&p),
                    None,
                )
            } else {
                t.ranged(
                    spec.name,
                    "FLOAT",
                    float_json(s.styles[i].get(spec.name), &p)?,
                    json!({"from":spec.from,"to":spec.to}),
                    spec.suffix,
                    &storage_esp_key(&p),
                )
            });
        }
        styles.insert((*name).into(), t.group(name, "CONFIGURABLE", fields));
    }
    let style = choice(
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
                    vec![t.leaf(
                        "Outline",
                        "BOOLEAN",
                        json!(s.outline),
                        &storage_esp_key("Mode.Box.Outline"),
                        None,
                    )],
                ),
            ),
            ("Glow".into(), t.group("Glow", "CONFIGURABLE", vec![style])),
        ]),
    ));
    for (i, name) in TYPES.iter().enumerate() {
        values.push(t.group(
            name,
            "TOGGLEABLE",
            vec![
                t.leaf(
                    "Enabled",
                    "BOOLEAN",
                    json!(s.categories[i].enabled),
                    &storage_esp_key(&format!("{name}.Enabled")),
                    None,
                ),
                t.leaf(
                    "Color",
                    "COLOR",
                    json!(s.categories[i].color),
                    &storage_esp_key(&format!("{name}.Color")),
                    None,
                ),
                t.leaf(
                    "Tracers",
                    "BOOLEAN",
                    json!(s.categories[i].tracers),
                    &storage_esp_key(&format!("{name}.Tracers")),
                    None,
                ),
            ],
        ));
    }
    for (name, value) in [
        ("RequiresChestStealer", s.requires_chest_stealer),
        ("MergeAdjacent", s.merge_adjacent),
    ] {
        values.push(t.leaf(name, "BOOLEAN", json!(value), &storage_esp_key(name), None));
    }
    let mut fields = vec![];
    for (i, name) in ["NearStart", "NearEnd", "FarStart", "FarEnd"]
        .iter()
        .enumerate()
    {
        let p = format!("DistanceFade.{name}");
        fields.push(t.ranged(
            name,
            "FLOAT",
            float_json(s.fade[i], &p)?,
            json!({"from":0,"to":512}),
            "",
            &storage_esp_key(&p),
        ));
    }
    values.push(t.group("DistanceFade", "CONFIGURABLE", fields));
    Ok(t.group("StorageESP", "TOGGLEABLE", values))
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
    leaf(patch, n, name, &format!("StorageESP.{path}"), |v, p| {
        let value = match kind {
            "BOOLEAN" => json!(parse_bool(v, p)?),
            "FLOAT" => float_json(parse_float(v, p)?, p)?,
            _ => color(v, p)?,
        };
        Ok(SettingChange::StorageEsp(StorageEspSetting(
            path.into(),
            value,
        )))
    });
}
fn group(
    patch: &mut SettingsPatch,
    n: &Nodes,
    name: &str,
    path: &str,
    f: impl FnOnce(&mut SettingsPatch, &Nodes),
) {
    if let Some(v) = n.get(name) {
        match nodes(v, name, &format!("StorageESP.{path}")) {
            Ok(c) => f(patch, &c),
            Err(e) => patch.issues.push(e),
        }
    }
}
fn modes(patch: &mut SettingsPatch, v: &Value, path: &str, names: &[&str]) {
    if let Some(v) = v["active"].as_str().filter(|v| names.contains(v)) {
        patch
            .changes
            .push(SettingChange::StorageEsp(StorageEspSetting(
                path.into(),
                json!(v),
            )));
    } else {
        patch.issues.push(ConfigError::new(
            &format!("StorageESP.{path}"),
            "unknown case-sensitive mode",
        ));
    }
}
pub(super) fn parse_storage_esp(patch: &mut SettingsPatch, root: &Nodes) {
    if let Some(m) = root.get("Mode") {
        modes(patch, m, "Mode", &["Box", "Glow"]);
        if let Some(v) = m["choices"].get("Box") {
            match nodes(v, "Box", "StorageESP.Mode.Box") {
                Ok(n) => field(patch, &n, "Outline", "Mode.Box.Outline", "BOOLEAN"),
                Err(e) => patch.issues.push(e),
            }
        }
        if let Some(v) = m["choices"].get("Glow") {
            match nodes(v, "Glow", "StorageESP.Mode.Glow") {
                Ok(n) => {
                    if let Some(style) = n.get("Style") {
                        let names = std::iter::once("Vanilla").chain(STYLES).collect::<Vec<_>>();
                        modes(patch, style, "Mode.Glow.Style", &names);
                        for (i, name) in STYLES.iter().enumerate() {
                            if let Some(v) = style["choices"].get(*name) {
                                match nodes(v, name, &format!("StorageESP.Mode.Glow.Style.{name}"))
                                {
                                    Ok(n) => {
                                        for spec in specs(i) {
                                            field(
                                                patch,
                                                &n,
                                                spec.name,
                                                &format!("Mode.Glow.Style.{name}.{}", spec.name),
                                                if spec.color { "COLOR" } else { "FLOAT" },
                                            );
                                        }
                                    }
                                    Err(e) => patch.issues.push(e),
                                }
                            }
                        }
                    }
                }
                Err(e) => patch.issues.push(e),
            }
        }
    }
    for name in TYPES {
        group(patch, root, name, name, |p, n| {
            for (field_name, kind) in [
                ("Enabled", "BOOLEAN"),
                ("Color", "COLOR"),
                ("Tracers", "BOOLEAN"),
            ] {
                field(p, n, field_name, &format!("{name}.{field_name}"), kind);
            }
        });
    }
    for name in ["RequiresChestStealer", "MergeAdjacent"] {
        field(patch, root, name, name, "BOOLEAN");
    }
    group(patch, root, "DistanceFade", "DistanceFade", |p, n| {
        let names = ["NearStart", "NearEnd", "FarStart", "FarEnd"];
        if names.iter().all(|name| n.contains_key(*name)) {
            let parsed = names.map(|name| {
                value_of(n[name], &format!("StorageESP.DistanceFade.{name}"))
                    .and_then(|v| parse_float(v, &format!("StorageESP.DistanceFade.{name}")))
            });
            let mut values = [0.; 4];
            let mut valid = true;
            for (i, value) in parsed.into_iter().enumerate() {
                match value {
                    Ok(v) => values[i] = v,
                    Err(e) => {
                        valid = false;
                        p.issues.push(e);
                    }
                }
            }
            if valid {
                p.changes.push(SettingChange::StorageEsp(StorageEspSetting(
                    "DistanceFade".into(),
                    json!(values),
                )));
            }
            return;
        }
        for name in ["NearStart", "NearEnd", "FarStart", "FarEnd"] {
            field(p, n, name, &format!("DistanceFade.{name}"), "FLOAT")
        }
    });
}
