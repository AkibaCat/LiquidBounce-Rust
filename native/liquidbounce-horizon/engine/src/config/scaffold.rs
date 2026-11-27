// SPDX-License-Identifier: GPL-3.0-or-later
//! Source setting tree, with full inactive choice serialization/import.
use super::*;
use crate::{auto_walk_runtime::AutoWalkSettings, scaffold_runtime::ScaffoldRuntime};
use liquidbounce_behaviors::no_rotate_set::NoRotateSet;
use std::{collections::BTreeMap, sync::LazyLock};
pub static SCHEMA: LazyLock<Vec<Value>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("scaffold_schema.json")).expect("owned Scaffold schema")
});
pub fn defaults() -> BTreeMap<String, Value> {
    fn add(nodes: &[Value], prefix: &str, out: &mut BTreeMap<String, Value>) {
        for n in nodes {
            let name = n["name"].as_str().expect("schema name");
            let p = join(prefix, name);
            if let Some(d) = n.get("default") {
                out.insert(p.clone(), d.clone());
            }
            if let Some(c) = n["children"].as_array() {
                add(c, &p, out);
            }
            if let Some(c) = n["modes"].as_array() {
                add(c, &p, out);
            }
        }
    }
    let mut r = BTreeMap::new();
    add(&SCHEMA, "", &mut r);
    r
}
fn join(p: &str, n: &str) -> String {
    if p.is_empty() {
        n.into()
    } else {
        format!("{p}.{n}")
    }
}
fn key(path: &str) -> String {
    let mut k = "liquidbounce.module.scaffold".to_owned();
    for p in path.split('.') {
        k.push('.');
        let mut c = p.chars();
        if let Some(a) = c.next() {
            k.extend(a.to_lowercase());
            k.extend(c);
        }
    }
    k
}
#[derive(Clone, Debug, PartialEq)]
pub enum ScaffoldSetting {
    Value(String, Value),
    Rotation(NoRotateSetSetting),
    Render(placement_esp::PlacementSetting),
}
#[derive(Clone, Debug, PartialEq)]
pub struct ClutchSetting(pub f32);
impl ScaffoldSetting {
    pub fn apply(self, r: &mut ScaffoldRuntime, blocks: &[String]) -> Result<(), ConfigError> {
        match self {
            Self::Rotation(v) => {
                let mut p = NoRotateSet {
                    rotations: std::mem::take(&mut r.rotations),
                    ..Default::default()
                };
                v.apply(&mut p);
                r.rotations = p.rotations;
            }
            Self::Render(v) => v.apply(&mut r.render),
            Self::Value(path, v) => {
                let spec = find(&SCHEMA, "", &path).ok_or_else(|| {
                    ConfigError::new(&format!("Scaffold.{path}"), "unknown Scaffold setting")
                })?;
                let value = validate(spec, &v, &format!("Scaffold.{path}"), blocks)?;
                let previous = r.settings.get(&path).cloned();
                if path == "Technique.Normal.Eagle.Enabled"
                    && previous != Some(json!(true))
                    && value == true
                {
                    r.eagle.placed = 0;
                }
                if path == "Technique.Normal.HeadHitter.Enabled"
                    && previous != Some(json!(true))
                    && value == true
                {
                    r.head_cooldown = 0;
                }
                if path == "Strafe.Enabled" && previous != Some(json!(true)) && value == true {
                    r.move_ticks = 0;
                }
                if path == "Strafe.Enabled"
                    && previous == Some(json!(true))
                    && value == false
                    && r.flag("Strafe.Hypixel")
                {
                    r.strafe_disable_pending = true;
                }
                if path == "Prediction.Enabled" && value == false {
                    r.prediction_reset_pending = true;
                }
                r.settings.insert(path, value);
            }
        }
        Ok(())
    }
}
fn find<'a>(nodes: &'a [Value], prefix: &str, path: &str) -> Option<&'a Value> {
    for n in nodes {
        let p = join(prefix, n["name"].as_str()?);
        if p == path && n.get("default").is_some() {
            return Some(n);
        }
        for c in ["children", "modes"] {
            if let Some(a) = n[c].as_array() {
                if let Some(s) = find(a, &p, path) {
                    return Some(s);
                }
            }
        }
    }
    None
}
fn validate(n: &Value, v: &Value, p: &str, blocks: &[String]) -> Result<Value, ConfigError> {
    Ok(match n["kind"].as_str().unwrap_or("") {
        "BOOLEAN" => json!(parse_bool(v, p)?),
        "INT" => json!(parse_int(v, p)?),
        "FLOAT" => json!(parse_float(v, p)?),
        "INT_RANGE" => {
            let a = parse_int_pair(v, p)?;
            if a[0] > a[1] {
                return Err(ConfigError::new(p, "empty integer range"));
            }
            json!(a)
        }
        "FLOAT_RANGE" => {
            let a = parse_float_pair(v, p)?;
            if a[0] > a[1] {
                return Err(ConfigError::new(p, "empty float range"));
            }
            json!(a)
        }
        "CHOICE" | "CHOOSE" => {
            let s = scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected choice"))?;
            let alias = match s.as_str() {
                "Karhu" => "Karhu-2024-05-10",
                "Vulcan" => "Vulcan-2.8.8",
                "Hypixel" if p == "Scaffold.Tower" => "Hypixel-2026-07-12",
                _ => &s,
            };
            let opts = n["options"]
                .as_array()
                .or_else(|| n["modes"].as_array())
                .expect("schema choices");
            let chosen = opts
                .iter()
                .find_map(|a| {
                    let t = a.as_str().or_else(|| a["name"].as_str())?;
                    t.eq_ignore_ascii_case(alias).then_some(t)
                })
                .ok_or_else(|| ConfigError::new(p, "unknown choice"))?;
            json!(chosen)
        }
        "MULTI_CHOOSE" => {
            let values = v
                .as_array()
                .ok_or_else(|| ConfigError::new(p, "expected choices array"))?;
            let opts = n["options"].as_array().expect("schema options");
            let mut selected = Vec::new();
            for v in values {
                let s = scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected choice"))?;
                let c = opts
                    .iter()
                    .find(|c| c.as_str().is_some_and(|c| c.eq_ignore_ascii_case(&s)))
                    .ok_or_else(|| ConfigError::new(p, "unknown choice"))?;
                if !selected.contains(c) {
                    selected.push(c.clone());
                }
            }
            if selected.is_empty() && n["canBeNone"] == false {
                return Err(ConfigError::new(p, "choice cannot be empty"));
            }
            json!(selected)
        }
        "BLOCK" => {
            let a = v
                .as_array()
                .ok_or_else(|| ConfigError::new(p, "expected blocks array"))?;
            let mut r = Vec::new();
            for v in a {
                let s = scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected block id"))?;
                let s = if s.contains(':') {
                    s
                } else {
                    format!("minecraft:{s}")
                };
                if !blocks.is_empty() && !blocks.contains(&s) {
                    return Err(ConfigError::new(p, "unknown block id"));
                }
                if !r.contains(&s) {
                    r.push(s)
                }
            }
            r.sort_by(|a, b| {
                let (an, ap) = a.split_once(':').unwrap();
                let (bn, bp) = b.split_once(':').unwrap();
                ap.cmp(bp).then(an.cmp(bn))
            });
            json!(r)
        }
        _ => return Err(ConfigError::new(p, "invalid Scaffold schema type")),
    })
}
fn remap_keys(v: &mut Value, old: &str, new: &str) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(k)) = o.get_mut("key") {
                *k = k.replace(old, new)
            }
            for v in o.values_mut() {
                remap_keys(v, old, new)
            }
        }
        Value::Array(a) => {
            for v in a {
                remap_keys(v, old, new)
            }
        }
        _ => {}
    }
}
fn render_node(
    t: &Tree,
    n: &Value,
    prefix: &str,
    r: &ScaffoldRuntime,
    c: &CommonSettings,
) -> Result<Value, ConfigError> {
    let name = n["name"].as_str().unwrap();
    let path = join(prefix, name);
    let kind = n["kind"].as_str().unwrap();
    if kind == "PLACEMENT" {
        return placement_esp::placement_tree(t, &r.render, "scaffold", "Render");
    }
    let mut children = Vec::new();
    if let Some(a) = n["children"].as_array() {
        for n in a {
            children.push(render_node(t, n, &path, r, c)?);
        }
    }
    if kind == "ROTATIONS" {
        let proxy = AutoWalkSettings {
            rotations: r.rotations.clone(),
            ..Default::default()
        };
        let d = |k: &str| {
            (t.describe)(&k.replace(
                "liquidbounce.module.autoWalk.rotations",
                "liquidbounce.module.scaffold.rotations",
            ))
        };
        let mut tree = serialize_auto_walk(&proxy, c, t.format, &d)?;
        let mut rot = tree["value"]
            .as_array_mut()
            .and_then(|a| a.iter_mut().find(|v| v["name"] == "Rotations"))
            .map(Value::take)
            .ok_or_else(|| ConfigError::new("Scaffold.Rotations", "shared rotations changed"))?;
        remap_keys(
            &mut rot,
            "liquidbounce.module.autoWalk.rotations",
            "liquidbounce.module.scaffold.rotations",
        );
        rot["value"].as_array_mut().unwrap().extend(children);
        return Ok(rot);
    }
    if kind == "CHOICE" {
        let mut value = t.group(name, "CHOICE", vec![]);
        value["active"] = r.settings[&path].clone();
        let mut modes = serde_json::Map::new();
        for mode in n["modes"].as_array().unwrap() {
            modes.insert(
                mode["name"].as_str().unwrap().into(),
                render_node(t, mode, &path, r, c)?,
            );
        }
        value["choices"] = Value::Object(modes);
        return Ok(value);
    }
    if matches!(kind, "CONFIGURABLE" | "TOGGLEABLE") {
        return Ok(t.group(name, kind, children));
    }
    let mut v = r.settings[&path].clone();
    if matches!(kind, "FLOAT_RANGE" | "INT_RANGE") {
        v = json!({"from":v[0],"to":v[1]});
    }
    let mut value = if let Some(range) = n.get("range") {
        t.ranged(
            name,
            kind,
            v,
            range.clone(),
            n["suffix"].as_str().unwrap_or(""),
            &key(&path),
        )
    } else {
        t.leaf(name, kind, v, &key(&path), None)
    };
    if t.format == JsonFormat::Interop {
        if let Some(o) = n.get("options") {
            value["choices"] = o.clone();
        }
        if kind == "MULTI_CHOOSE" {
            value["canBeNone"] = n["canBeNone"].clone();
        }
    }
    Ok(value)
}
pub fn serialize_scaffold(
    r: &ScaffoldRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut v = t.common(ModuleKind::Scaffold, c);
    for n in SCHEMA.iter() {
        v.push(render_node(&t, n, "", r, c)?)
    }
    Ok(t.group("Scaffold", "TOGGLEABLE", v))
}
pub fn serialize_clutch(
    min: f32,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut v = t.common(ModuleKind::Clutch, c);
    v.push(t.ranged(
        "MinFallDistance",
        "FLOAT",
        float_json(min, "Clutch.MinFallDistance")?,
        json!({"from":2.,"to":50.}),
        "",
        "liquidbounce.module.clutch.minFallDistance",
    ));
    Ok(t.group("Clutch", "TOGGLEABLE", v))
}
pub(super) fn parse_clutch(p: &mut SettingsPatch, n: &Nodes<'_>) {
    leaf(p, n, "MinFallDistance", "Clutch.MinFallDistance", |v, k| {
        parse_float(v, k).map(|v| SettingChange::Clutch(ClutchSetting(v)))
    });
}
fn parse_nodes(p: &mut SettingsPatch, nodeset: &Nodes<'_>, schema: &[Value], prefix: &str) {
    for s in schema {
        let name = s["name"].as_str().unwrap();
        let path = join(prefix, name);
        let Some(v) = nodeset.get(name) else { continue };
        let kind = s["kind"].as_str().unwrap();
        if kind == "PLACEMENT" {
            placement_esp::parse_placement(p, nodeset, "Render", "Scaffold", |v| {
                SettingChange::Scaffold(ScaffoldSetting::Render(v))
            });
            continue;
        }
        if kind == "CHOICE" {
            if let Some(v) = v.get("active") {
                push(p, &path, v, s);
            }
            for m in s["modes"].as_array().unwrap() {
                let mode = m["name"].as_str().unwrap();
                let alias = match mode {
                    "Karhu-2024-05-10" => "Karhu",
                    "Vulcan-2.8.8" => "Vulcan",
                    "Hypixel-2026-07-12" => "Hypixel",
                    _ => mode,
                };
                if let Some(v) = v["choices"].get(mode).or_else(|| v["choices"].get(alias)) {
                    match nodes(
                        v,
                        v["name"].as_str().unwrap_or(mode),
                        &format!("Scaffold.{path}.{mode}"),
                    ) {
                        Ok(n) => parse_nodes(
                            p,
                            &n,
                            m["children"].as_array().unwrap(),
                            &join(&path, mode),
                        ),
                        Err(e) => p.issues.push(e),
                    }
                }
            }
            continue;
        }
        if kind == "ROTATIONS" {
            let proxy = json!({"name":"AutoWalk","value":[v]});
            let n = nodes(&proxy, "AutoWalk", "AutoWalk").unwrap();
            let mut out = SettingsPatch {
                module: ModuleKind::AutoWalk,
                changes: vec![],
                issues: vec![],
            };
            auto_walk::parse_auto_walk(&mut out, &n);
            for change in out.changes {
                if let SettingChange::AutoWalk(AutoWalkSetting::Rotation(v)) = change {
                    p.changes
                        .push(SettingChange::Scaffold(ScaffoldSetting::Rotation(v)));
                }
            }
            p.issues.extend(out.issues.into_iter().map(|mut e| {
                e.path = e.path.replace("AutoWalk.Rotations", "Scaffold.Rotations");
                e
            }));
        }
        if let Some(children) = s["children"].as_array() {
            match nodes(v, name, &format!("Scaffold.{path}")) {
                Ok(n) => parse_nodes(p, &n, children, &path),
                Err(e) => p.issues.push(e),
            }
            continue;
        }
        match value_of(v, &format!("Scaffold.{path}")) {
            Ok(v) => push(p, &path, v, s),
            Err(e) => p.issues.push(e),
        }
    }
}
fn push(p: &mut SettingsPatch, path: &str, v: &Value, spec: &Value) {
    match validate(spec, v, &format!("Scaffold.{path}"), &[]) {
        Ok(v) => p
            .changes
            .push(SettingChange::Scaffold(ScaffoldSetting::Value(
                path.into(),
                v,
            ))),
        Err(e) => p.issues.push(e),
    }
}
pub(super) fn parse_scaffold(p: &mut SettingsPatch, n: &Nodes<'_>) {
    parse_nodes(p, n, &SCHEMA, "")
}
