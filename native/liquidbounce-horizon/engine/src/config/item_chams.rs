// SPDX-License-Identifier: GPL-3.0-or-later
//! All source ItemChams values, including disabled effect branches.
use super::*;
use liquidbounce_behaviors::item_chams::{ItemChams, Kind, SPECS};
#[derive(Clone, Debug, PartialEq)]
pub struct ItemChamsSetting(pub String, pub Value);
impl ItemChamsSetting {
    pub fn apply(self, target: &mut ItemChams) {
        target.values.insert(self.0, self.1);
    }
}
fn key(path: &str) -> String {
    format!(
        "liquidbounce.module.itemChams.{}",
        path.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
fn groups(prefix: &str) -> Vec<&'static str> {
    let mut names = vec![];
    for spec in SPECS {
        if let Some(rest) = spec.path.strip_prefix(prefix) {
            let name = rest.split('.').next().unwrap();
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}
pub fn serialize_item_chams(
    s: &ItemChams,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    fn build(t: &Tree, s: &ItemChams, p: &str, name: &str) -> Result<Value, ConfigError> {
        let mut v = if let Some(spec) = SPECS.iter().find(|sp| sp.path == p) {
            let value = s
                .values
                .get(p)
                .ok_or_else(|| ConfigError::new(p, "missing ItemChams value"))?;
            match spec.kind {
                Kind::Bool=>t.leaf(name,"BOOLEAN",value.clone(),&key(p),None),
                Kind::Color=>t.leaf(name,"COLOR",value.clone(),&key(p),None),
                Kind::Int(a,b)=>t.ranged(name,"INT",value.clone(),json!({"from":a,"to":b}),"",&key(p)),
                Kind::Float(a,b,suffix)=>t.ranged(name,"FLOAT",float_json(value.as_f64().unwrap()as f32,p)?,json!({"from":a,"to":b}),suffix,&key(p)),
                Kind::Vec2=>t.leaf(name,"VECTOR2_F",json!({"x":float_json(value[0].as_f64().unwrap()as f32,p)?,"y":float_json(value[1].as_f64().unwrap()as f32,p)?}),&key(p),None),
                Kind::Choice(choices)=>{let mut v=t.leaf(name,"CHOOSE",value.clone(),&key(p),None);if t.format==JsonFormat::Interop {v["choices"]=json!(choices);}v},
            }
        } else {
            let prefix = format!("{p}.");
            let fields = groups(&prefix)
                .into_iter()
                .map(|n| build(t, s, &format!("{prefix}{n}"), n))
                .collect::<Result<Vec<_>, _>>()?;
            t.group(name, "TOGGLEABLE", fields)
        };
        if t.format == JsonFormat::Interop {
            v["key"] = json!(key(p));
            v["description"] = json!((t.describe)(&format!("{}.description", key(p))));
        }
        Ok(v)
    }
    let t = Tree { format, describe };
    let mut children = t.common(ModuleKind::ItemChams, common);
    for name in groups("") {
        children.push(build(&t, s, name, name)?);
    }
    Ok(t.group("ItemChams", "TOGGLEABLE", children))
}
fn parsed(
    spec: &liquidbounce_behaviors::item_chams::Spec,
    v: &Value,
    p: &str,
) -> Result<Value, ConfigError> {
    Ok(match spec.kind {
        Kind::Bool => json!(parse_bool(v, p)?),
        Kind::Int(_, _) => json!(parse_int(v, p)?),
        Kind::Float(_, _, _) => json!(parse_float(v, p)?),
        Kind::Color => {
            if let Some(s) = v.as_str() {
                let h = s.strip_prefix('#').unwrap_or(s);
                if matches!(h.len(), 6 | 8) {
                    let color = u32::from_str_radix(h, 16)
                        .map_err(|_| ConfigError::new(p, "invalid ARGB color"))?;
                    json!((color | if h.len() == 6 { 0xff000000 } else { 0 }) as i32)
                } else {
                    json!(parse_int(v, p)?)
                }
            } else {
                json!(parse_int(v, p)?)
            }
        }
        Kind::Vec2 => {
            let pair = if let Some(a) = v.as_array().filter(|a| a.len() == 2) {
                [&a[0], &a[1]]
            } else {
                [
                    v.get("x").ok_or_else(|| ConfigError::new(p, "missing x"))?,
                    v.get("y").ok_or_else(|| ConfigError::new(p, "missing y"))?,
                ]
            };
            json!([parse_float(pair[0], p)?, parse_float(pair[1], p)?])
        }
        Kind::Choice(choices) => {
            let value = scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected choice"))?;
            if !choices.contains(&value.as_str()) {
                return Err(ConfigError::new(p, "unknown choice"));
            }
            json!(value)
        }
    })
}
pub(super) fn parse_item_chams(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    fn walk(patch: &mut SettingsPatch, root: &Nodes<'_>, prefix: &str) {
        for name in groups(prefix) {
            let p = format!("{prefix}{name}");
            let full = format!("ItemChams.{p}");
            if let Some(spec) = SPECS.iter().find(|sp| sp.path == p) {
                leaf(patch, root, name, &full, |v, path| {
                    parsed(spec, v, path)
                        .map(|v| SettingChange::ItemChams(ItemChamsSetting(p.clone(), v)))
                });
            } else if let Some(node) = root.get(name) {
                match nodes(node, name, &full) {
                    Ok(n) => walk(patch, &n, &format!("{p}.")),
                    Err(e) => patch.issues.push(e),
                }
            }
        }
    }
    walk(patch, root, "");
}
