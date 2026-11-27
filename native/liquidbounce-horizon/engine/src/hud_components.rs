// SPDX-License-Identifier: GPL-3.0-or-later
//! Original Theme/HudComponentManager trees, backed by the upstream theme JSON files.
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
fn lower(name: &str) -> String {
    let mut c = name.chars();
    c.next()
        .map_or_else(String::new, |s| s.to_lowercase().to_string() + c.as_str())
}
fn upper(name: &str) -> String {
    let mut c = name.chars();
    c.next()
        .map_or_else(String::new, |s| s.to_uppercase().to_string() + c.as_str())
}
fn uuid() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        ^ u128::from(SEQ.fetch_add(1, Ordering::Relaxed)).wrapping_mul(0x9e3779b97f4a7c15);
    let b = (n & !(0xf000u128 << 64) & !(0xc000u128 << 48)) | (4u128 << 76) | (2u128 << 62);
    let h = format!("{b:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    )
}
pub fn leaf(name: &str, kind: &str, value: Value) -> Value {
    json!({"name":name,"valueType":kind,"value":value})
}
pub fn group(name: &str, kind: &str, values: Vec<Value>) -> Value {
    leaf(name, kind, json!(values))
}
fn choose(name: &str, value: Value, choices: Value) -> Value {
    let mut v = leaf(name, "CHOOSE", value);
    v["choices"] = choices;
    v
}
fn ranged(name: &str, kind: &str, value: Value, from: Value, to: Value, suffix: &str) -> Value {
    let mut v = leaf(name, kind, value);
    v["range"] = json!({"from":from,"to":to});
    v["suffix"] = json!(suffix);
    v
}
fn toggle(name: &str, enabled: bool, children: Vec<Value>) -> Value {
    let mut c = vec![leaf("Enabled", "BOOLEAN", json!(enabled))];
    c.extend(children);
    group(name, "TOGGLEABLE", c)
}
fn read_json(path: &Path) -> Result<Value, String> {
    let b = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let b = b.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&b);
    serde_json::from_slice(b).map_err(|e| format!("{}: {e}", path.display()))
}
fn hex_color(v: &str) -> Option<i32> {
    let s = v.strip_prefix('#')?;
    let c = u32::from_str_radix(s, 16).ok()?;
    Some(match s.len() {
        6 => 0xff000000 | c,
        8 => (c >> 8) | ((c & 255) << 24),
        _ => return None,
    } as i32)
}
fn source_node(source: &Value) -> Result<Value, String> {
    let name = source["name"].as_str().ok_or("value without name")?;
    let kind = source["type"].as_str().ok_or("value without type")?;
    let val = source["value"].clone();
    let children = || -> Result<Vec<Value>, String> {
        source["values"]
            .as_array()
            .into_iter()
            .flatten()
            .map(source_node)
            .collect()
    };
    let mut out = match kind {
        "CONFIGURABLE" => group(name, kind, children()?),
        "TOGGLEABLE" => toggle(name, val.as_bool().unwrap_or(false), children()?),
        "CHOICE" => {
            let mut choices = serde_json::Map::new();
            for choice in source["choices"]
                .as_array()
                .ok_or("CHOICE without choices")?
            {
                let n = choice["name"].as_str().ok_or("mode without name")?;
                let values: Result<Vec<_>, _> = choice["values"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(source_node)
                    .collect();
                choices.insert(n.into(), group(n, "CONFIGURABLE", values?));
            }
            let mut v = group(name, kind, vec![]);
            v["active"] = val;
            v["choices"] = Value::Object(choices);
            v
        }
        "COLOR" => leaf(
            name,
            kind,
            json!(
                val.as_i64()
                    .or_else(|| val.as_u64().map(|v| v as i64))
                    .unwrap_or(0) as i32
            ),
        ),
        "FILE" => leaf(
            name,
            kind,
            if val.as_str().is_some_and(|s| !s.trim().is_empty()) {
                val
            } else {
                Value::Null
            },
        ),
        "FLOAT" => leaf(name, kind, json!(val.as_f64().unwrap_or(0.) as f32)),
        "REGISTRY_LIST" => {
            let mut v = leaf(name, kind, val);
            v["innerValueType"] = source["innerValueType"].clone();
            v["registry"] = json!(if source["innerValueType"] == "BLOCK" {
                "block"
            } else {
                "item"
            });
            v
        }
        _ => leaf(name, kind, val),
    };
    if let Some(range) = source.get("range") {
        out["range"] = json!({"from":range["min"],"to":range["max"]});
        out["suffix"] = source.get("suffix").cloned().unwrap_or(json!(""));
    }
    for k in [
        "choices",
        "canBeNone",
        "isOrderSensitive",
        "dialogMode",
        "supportedExtensions",
    ] {
        if kind != "CHOICE" {
            if let Some(v) = source.get(k) {
                out[k] = v.clone();
            }
        }
    }
    Ok(out)
}
pub fn alignment(value: &Value) -> Value {
    group(
        "Alignment",
        "CONFIGURABLE",
        vec![
            choose(
                "Horizontal",
                value["horizontalAlignment"].clone(),
                json!(["Left", "Center", "CenterTranslated", "Right"]),
            ),
            ranged(
                "HorizontalOffset",
                "INT",
                value["horizontalOffset"].clone(),
                json!(-1000),
                json!(1000),
                "px",
            ),
            choose(
                "Vertical",
                value["verticalAlignment"].clone(),
                json!(["Top", "Center", "CenterTranslated", "Bottom"]),
            ),
            ranged(
                "VerticalOffset",
                "INT",
                value["verticalOffset"].clone(),
                json!(-1000),
                json!(1000),
                "px",
            ),
        ],
    )
}
pub fn find<'a>(tree: &'a Value, name: &str) -> Option<&'a Value> {
    tree["value"].as_array()?.iter().find(|v| v["name"] == name)
}
pub fn find_mut<'a>(tree: &'a mut Value, name: &str) -> Option<&'a mut Value> {
    tree["value"]
        .as_array_mut()?
        .iter_mut()
        .find(|v| v["name"] == name)
}
pub fn value<'a>(tree: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut n = tree;
    for p in path {
        n = find(n, p)?;
    }
    Some(&n["value"])
}
pub fn set(tree: &mut Value, path: &[&str], v: Value) -> Result<(), String> {
    let (head, tail) = path.split_first().ok_or("empty path")?;
    let n = find_mut(tree, head).ok_or("unknown value")?;
    if tail.is_empty() {
        n["value"] = v;
        Ok(())
    } else {
        set(n, tail, v)
    }
}
/// Deserialize known values sequentially; duplicate component names retain their instance order.
pub fn merge(target: &mut Value, patch: &Value) -> Result<(), String> {
    let kind = target["valueType"].as_str().unwrap_or("").to_owned();
    if matches!(kind.as_str(), "CONFIGURABLE" | "TOGGLEABLE" | "CHOICE") {
        if kind == "CHOICE" {
            if let Some(active) = patch["active"].as_str() {
                if target["choices"].get(active).is_some() {
                    target["active"] = json!(active);
                }
            }
            if let Some(choices) = patch["choices"].as_object() {
                for (n, p) in choices {
                    if let Some(t) = target["choices"].get_mut(n) {
                        merge(t, p)?;
                    }
                }
            }
        }
        if let Some(children) = patch["value"].as_array() {
            let mut counts = std::collections::BTreeMap::<String, usize>::new();
            for p in children {
                let Some(n) = p["name"].as_str() else {
                    continue;
                };
                let count = counts.entry(n.into()).or_default();
                if let Some(t) = target["value"]
                    .as_array_mut()
                    .and_then(|a| a.iter_mut().filter(|t| t["name"] == n).nth(*count))
                {
                    merge(t, p)?;
                }
                *count += 1;
            }
        }
        return Ok(());
    }
    let Some(v) = patch.get("value") else {
        return Ok(());
    };
    let parsed = match kind.as_str() {
        "BOOLEAN" => v.as_bool().map(|b| json!(b)).ok_or("expected boolean")?,
        "INT" | "COLOR" => json!(
            v.as_i64()
                .or_else(|| v.as_u64().map(|n| n as i64))
                .ok_or("expected int")? as i32
        ),
        "FLOAT" => {
            let f = v.as_f64().ok_or("expected float")? as f32;
            if !f.is_finite() {
                return Err("non-finite float".into());
            }
            json!(f)
        }
        "TEXT" => json!(v.as_str().ok_or("expected string")?),
        "FILE" => {
            if v.is_null() {
                Value::Null
            } else {
                json!(v.as_str().ok_or("expected path")?)
            }
        }
        "CHOOSE" => {
            let s = v.as_str().ok_or("expected choice")?;
            target["choices"]
                .as_array()
                .and_then(|a| {
                    a.iter()
                        .find(|c| c.as_str().is_some_and(|c| c.eq_ignore_ascii_case(s)))
                })
                .cloned()
                .unwrap_or_else(|| target["value"].clone())
        }
        _ => v.clone(),
    };
    target["value"] = parsed;
    Ok(())
}
pub fn plain(tree: &Value) -> Value {
    let kind = tree["valueType"].as_str().unwrap_or("");
    if kind == "CHOICE" {
        let active = tree["active"].as_str().unwrap_or("");
        let mut p = plain(&tree["choices"][active]);
        if !p.is_object() {
            p = json!({})
        }
        p["active"] = json!(active);
        return p;
    }
    if matches!(kind, "CONFIGURABLE" | "TOGGLEABLE") {
        let mut map = serde_json::Map::new();
        for c in tree["value"].as_array().into_iter().flatten() {
            if let Some(n) = c["name"].as_str() {
                map.insert(lower(n), plain(c));
            }
        }
        return Value::Object(map);
    }
    tree["value"].clone()
}
pub fn format_tree(
    mut tree: Value,
    key: &str,
    file: bool,
    describe: &dyn Fn(&str) -> String,
) -> Value {
    fn visit(v: &mut Value, parent: &str, file: bool, describe: &dyn Fn(&str) -> String) {
        let n = v["name"].as_str().unwrap_or("");
        let k = format!("{parent}.{}", lower(n));
        // Only configurable/group nodes contain child setting objects.  Some
        // leaves (REGISTRY_LIST, MULTI_CHOOSE, ... ) also store arrays under
        // `value`; walking those arrays would turn scalar entries into fake
        // settings and, in file mode, could discard their payload.
        let is_group = matches!(
            v["valueType"].as_str().unwrap_or_default(),
            "CONFIGURABLE" | "TOGGLEABLE"
        );
        if is_group {
            if let Some(children) = v["value"].as_array_mut() {
                for c in children.iter_mut() {
                    visit(c, &k, file, describe);
                }
                if !file {
                    children.retain(|c| c["name"] != "ZIndex");
                }
            }
        }
        if let Some(choices) = v["choices"].as_object_mut() {
            for c in choices.values_mut() {
                visit(c, &k, file, describe);
            }
        }
        if let Some(o) = v.as_object_mut() {
            if file {
                o.retain(|k, _| matches!(k.as_str(), "name" | "value" | "active" | "choices"));
            } else {
                o.insert("key".into(), json!(k));
                o.insert(
                    "description".into(),
                    json!(describe(&format!("{k}.description"))),
                );
            }
        }
    }
    visit(&mut tree, key, file, describe);
    tree
}
#[derive(Clone, Debug)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub description: String,
    pub singleton: bool,
    pub native: bool,
    pub tweaks: Vec<String>,
    pub settings: Value,
    pub defaults: Value,
}
impl Component {
    fn from_factory(source: &Value) -> Result<Self, String> {
        let name = source["name"]
            .as_str()
            .ok_or("component without name")?
            .to_string();
        let mut nodes = vec![
            leaf("Enabled", "BOOLEAN", source["enabled"].clone()),
            ranged("ZIndex", "INT", json!(0), json!(0), json!(i32::MAX), ""),
            alignment(&source["alignment"]),
        ];
        for v in source["values"].as_array().into_iter().flatten() {
            nodes.push(source_node(v)?);
        }
        let settings = group(&name, "TOGGLEABLE", nodes);
        Ok(Self {
            id: uuid(),
            name,
            description: source["description"].as_str().unwrap_or("").into(),
            singleton: source["singleton"].as_bool().unwrap_or(false),
            native: false,
            tweaks: source["tweaks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
            defaults: settings.clone(),
            settings,
        })
    }
    pub fn enabled(&self) -> bool {
        value(&self.settings, &["Enabled"])
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
    pub fn payload(&self) -> Value {
        let mut v = json!({"id":self.id,"name":self.name,"description":self.description,"settings":plain(&self.settings)});
        let a = &mut v["settings"]["alignment"];
        if a.is_object() {
            a["horizontalAlignment"] = a["horizontal"].take();
            a["verticalAlignment"] = a["vertical"].take();
        }
        if self.native {
            let size = value(&self.settings, &["Size"])
                .and_then(Value::as_i64)
                .unwrap_or(96) as f32
                * 2.;
            v["width"] = json!(size);
            v["height"] = json!(size);
        }
        v
    }
    pub fn z_index(&self) -> i32 {
        value(&self.settings, &["ZIndex"])
            .and_then(Value::as_i64)
            .unwrap_or(0) as i32
    }
    pub fn alignment(&self) -> Value {
        let p = self.payload();
        p["settings"]["alignment"].clone()
    }
}
#[derive(Clone, Debug)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub root: PathBuf,
    pub settings: Value,
    pub components: Vec<Component>,
}
impl Theme {
    pub fn load(root: &Path) -> Result<Self, String> {
        let md = read_json(&root.join("metadata.json"))?;
        let id = md["id"].as_str().ok_or("theme without id")?.to_owned();
        let mut colors = vec![];
        if let Some(map) = md["colors"].as_object() {
            for (n, v) in map {
                let c = hex_color(v.as_str().ok_or("theme color not string")?)
                    .ok_or("invalid theme color")?;
                colors.push(leaf(n, "COLOR", json!(c)));
            }
        }
        let mut nodes = vec![group("Colors", "CONFIGURABLE", colors)];
        for v in md["values"].as_array().into_iter().flatten() {
            nodes.push(source_node(v)?);
        }
        let mut components = vec![];
        for n in md["components"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            match read_json(
                &root
                    .join("components")
                    .join(format!("{}.json", n.to_ascii_lowercase())),
            )
            .and_then(|v| Component::from_factory(&v))
            {
                Ok(c) => components.push(c),
                Err(e) => eprintln!("[LiquidBounce] Failed to load component {n}: {e}"),
            }
        }
        Ok(Self {
            id: id.clone(),
            name: md["name"].as_str().unwrap_or(&id).into(),
            root: root.into(),
            settings: group(&upper(&id), "CONFIGURABLE", nodes),
            components,
        })
    }
    pub fn tree(&self) -> Value {
        let mut s = self.settings.clone();
        s["value"].as_array_mut().unwrap().push(group(
            "Components",
            "CONFIGURABLE",
            self.components.iter().map(|c| c.settings.clone()).collect(),
        ));
        s
    }
    fn apply(&mut self, patch: &Value) -> Result<(), String> {
        merge(&mut self.settings, patch)?;
        if let Some(c) = find(patch, "Components").and_then(|s| s["value"].as_array()) {
            let mut count = std::collections::BTreeMap::<String, usize>::new();
            for p in c {
                let Some(n) = p["name"].as_str() else {
                    continue;
                };
                let i = count.entry(n.into()).or_default();
                if self.components.iter().filter(|c| c.name == n).count() <= *i {
                    if let Some(factory) =
                        self.components.iter().find(|c| c.name == n && !c.singleton)
                    {
                        let mut c = factory.clone();
                        c.id = uuid();
                        c.settings = c.defaults.clone();
                        self.components.push(c);
                    }
                }
                if let Some(c) = self.components.iter_mut().filter(|c| c.name == n).nth(*i) {
                    merge(&mut c.settings, p)?;
                }
                *i += 1;
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct HudComponents {
    pub themes: Vec<Theme>,
    pub selected: String,
    pub native: Vec<Component>,
    pub generation: u64,
}
impl Default for HudComponents {
    fn default() -> Self {
        Self {
            themes: vec![],
            selected: "liquidbounce".into(),
            native: vec![minimap()],
            generation: 0,
        }
    }
}
impl HudComponents {
    pub fn load(&mut self, included: &Path, state_root: &Path) -> Result<(), String> {
        let mut themes = vec![];
        if let Ok(entries) = std::fs::read_dir(state_root.join("themes")) {
            for e in entries.flatten() {
                if !e.path().is_dir()
                    || e.file_name()
                        .to_string_lossy()
                        .eq_ignore_ascii_case("default")
                {
                    continue;
                }
                match Theme::load(&e.path()) {
                    Ok(t)
                        if !themes
                            .iter()
                            .any(|old: &Theme| old.id.eq_ignore_ascii_case(&t.id)) =>
                    {
                        themes.push(t)
                    }
                    Ok(_) => {}
                    Err(e) => eprintln!("[LiquidBounce] Failed to load theme: {e}"),
                }
            }
        }
        let theme = Theme::load(included)?;
        if !themes.iter().any(|t| t.id.eq_ignore_ascii_case(&theme.id)) {
            themes.push(theme);
        }
        self.themes = themes;
        self.changed();
        Ok(())
    }
    pub fn current(&self) -> Option<&Theme> {
        self.themes
            .iter()
            .find(|t| t.id.eq_ignore_ascii_case(&self.selected))
            .or_else(|| self.themes.iter().find(|t| t.id == "liquidbounce"))
            .or_else(|| self.themes.first())
    }
    pub fn changed(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
    pub fn themes_tree(&self) -> Value {
        group(
            "Themes",
            "CONFIGURABLE",
            self.themes.iter().map(Theme::tree).collect(),
        )
    }
    pub fn native_tree(&self) -> Value {
        group(
            "AdditionalComponents",
            "CONFIGURABLE",
            self.native.iter().map(|c| c.settings.clone()).collect(),
        )
    }
    pub fn apply_themes(&mut self, patch: &Value) -> Result<(), String> {
        for p in patch["value"].as_array().into_iter().flatten() {
            if let Some(t) = self.themes.iter_mut().find(|t| upper(&t.id) == p["name"]) {
                t.apply(p)?;
            }
        }
        self.changed();
        Ok(())
    }
    pub fn apply_native(&mut self, patch: &Value) -> Result<(), String> {
        for p in patch["value"].as_array().into_iter().flatten() {
            if let Some(t) = self.native.iter_mut().find(|t| t.name == p["name"]) {
                merge(&mut t.settings, p)?;
            }
        }
        self.changed();
        Ok(())
    }
    pub fn component(&self, id: &str) -> Option<&Component> {
        self.native
            .iter()
            .chain(self.themes.iter().flat_map(|t| &t.components))
            .find(|c| c.id == id)
    }
    pub fn component_mut(&mut self, id: &str) -> Option<&mut Component> {
        self.native
            .iter_mut()
            .chain(self.themes.iter_mut().flat_map(|t| &mut t.components))
            .find(|c| c.id == id)
    }
    pub fn payloads(&self, id: Option<&str>) -> Value {
        if id == Some("native") {
            return json!(
                self.native
                    .iter()
                    .map(Component::payload)
                    .collect::<Vec<_>>()
            );
        }
        let theme = if let Some(id) = id {
            self.themes.iter().find(|t| t.id == id)
        } else {
            self.current()
        };
        json!(
            theme
                .into_iter()
                .flat_map(|t| &t.components)
                .map(Component::payload)
                .collect::<Vec<_>>()
        )
    }
    pub fn events(&self) -> Vec<(String, Value)> {
        let mut e = vec![(
            "componentsUpdate".into(),
            json!({"source":"native","components":self.payloads(Some("native"))}),
        )];
        if let Some(t) = self.current() {
            e.push((
                "componentsUpdate".into(),
                json!({"source":"theme","themeId":t.id,"components":self.payloads(Some(&t.id))}),
            ));
        }
        e
    }
    pub fn catalog(&self, id: &str) -> Value {
        let Some(t) = self.themes.iter().find(|t| t.id == id) else {
            return json!([]);
        };
        let mut out:Vec<Value>=self.native.iter().map(|c|json!({"name":c.name,"description":c.description,"id":c.id,"singleton":true,"canAdd":!c.enabled()})).collect();
        let mut seen = std::collections::BTreeSet::new();
        for c in &t.components {
            if seen.insert(c.name.clone()) {
                out.push(json!({"name":c.name,"description":c.description,"id":c.id,"singleton":c.singleton,"canAdd":!c.singleton||!t.components.iter().any(|other|other.name==c.name&&other.enabled())}));
            }
        }
        json!(out)
    }
    pub fn add(&mut self, id: &str) -> Option<String> {
        if let Some(c) = self.native.iter_mut().find(|c| c.id == id) {
            if c.enabled() {
                return None;
            }
            set(&mut c.settings, &["Enabled"], json!(true)).ok()?;
            let id = c.id.clone();
            self.changed();
            return Some(id);
        }
        let t = self
            .themes
            .iter_mut()
            .find(|t| t.components.iter().any(|c| c.id == id))?;
        let source = t.components.iter().find(|c| c.id == id)?.clone();
        if source.singleton
            && t.components
                .iter()
                .any(|c| c.name == source.name && c.enabled())
        {
            return None;
        }
        let i = if let Some(i) = t
            .components
            .iter()
            .position(|c| c.name == source.name && !c.enabled())
        {
            if !source.singleton {
                t.components[i].settings = t.components[i].defaults.clone();
            }
            i
        } else {
            let mut c = source;
            c.id = uuid();
            c.settings = c.defaults.clone();
            t.components.push(c);
            t.components.len() - 1
        };
        set(&mut t.components[i].settings, &["Enabled"], json!(true)).ok()?;
        let id = t.components[i].id.clone();
        self.changed();
        Some(id)
    }
    pub fn bring_to_front(&mut self, id: &str) -> Option<i32> {
        let n = self
            .native
            .iter()
            .chain(self.current().into_iter().flat_map(|t| &t.components))
            .map(Component::z_index)
            .max()
            .unwrap_or(0)
            .wrapping_add(1);
        let c = self.component_mut(id)?;
        set(&mut c.settings, &["ZIndex"], json!(n)).ok()?;
        self.changed();
        Some(n)
    }
    pub fn tweak_mask(&self) -> u32 {
        self.native
            .iter()
            .chain(self.current().into_iter().flat_map(|t| &t.components))
            .filter(|c| c.enabled())
            .flat_map(|c| &c.tweaks)
            .fold(0, |mask, t| {
                mask | match t.as_str() {
                    "TWEAK_HOTBAR" => 1,
                    "DISABLE_CROSSHAIR" => 2,
                    "DISABLE_SCOREBOARD" => 4,
                    "DISABLE_STATUS_BAR" => 8,
                    "DISABLE_EXP_BAR" => 16,
                    "DISABLE_HELD_ITEM_TOOL_TIP" => 32,
                    "DISABLE_OVERLAY_MESSAGE" => 64,
                    "DISABLE_STATUS_EFFECT_OVERLAY" => 128,
                    "DISABLE_LOCATOR_BAR" => 256,
                    "DISABLE_SUBTITLE_OVERLAY" => 512,
                    _ => 0,
                }
            })
    }
}
fn minimap() -> Component {
    let align = json!({"horizontalAlignment":"Left","horizontalOffset":7,"verticalAlignment":"Top","verticalOffset":180});
    let extras = |n: &str| {
        toggle(
            n,
            false,
            vec![choose(
                "Placement",
                json!("TopLeft"),
                json!(["TopLeft", "TopRight", "BottomLeft", "BottomRight"]),
            )],
        )
    };
    let settings = toggle(
        "Minimap",
        false,
        vec![
            ranged("ZIndex", "INT", json!(0), json!(0), json!(i32::MAX), ""),
            alignment(&align),
            ranged("Size", "INT", json!(96), json!(1), json!(256), ""),
            ranged("ViewDistance", "FLOAT", json!(3.), json!(1.), json!(8.), ""),
            leaf("FixedDirection", "BOOLEAN", json!(false)),
            toggle(
                "Texture",
                true,
                vec![leaf("VertexColor", "COLOR", json!(-1))],
            ),
            toggle(
                "Entity",
                true,
                vec![
                    ranged("Scale", "FLOAT", json!(1.), json!(0.25), json!(4.), ""),
                    choose("OutOfBounds", json!("None"), json!(["None", "All"])),
                ],
            ),
            extras("Compass"),
            extras("Clock"),
        ],
    );
    Component {
        id: uuid(),
        name: "Minimap".into(),
        description: "Shows nearby terrain and entities.".into(),
        singleton: true,
        native: true,
        tweaks: vec![],
        defaults: settings.clone(),
        settings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tree_preserves_scalar_value_arrays() {
        let tree = group(
            "Root",
            "CONFIGURABLE",
            vec![leaf("Blocks", "REGISTRY_LIST", json!(["minecraft:stone"]))],
        );
        let formatted = format_tree(tree, "HUD", false, &|key| key.to_owned());
        assert_eq!(formatted["value"][0]["value"], json!(["minecraft:stone"]));
        assert_eq!(formatted["value"][0]["key"], "HUD.root.blocks");
    }

    #[test]
    fn merge_matches_duplicate_component_names_by_occurrence() {
        let mut target = group(
            "Components",
            "CONFIGURABLE",
            vec![
                leaf("Text", "BOOLEAN", json!(false)),
                leaf("Text", "BOOLEAN", json!(false)),
            ],
        );
        let patch = group(
            "Components",
            "CONFIGURABLE",
            vec![
                leaf("Text", "BOOLEAN", json!(true)),
                leaf("Text", "BOOLEAN", json!(true)),
            ],
        );
        merge(&mut target, &patch).expect("duplicate names merge");
        assert!(target["value"][0]["value"].as_bool().unwrap());
        assert!(target["value"][1]["value"].as_bool().unwrap());
    }
}
