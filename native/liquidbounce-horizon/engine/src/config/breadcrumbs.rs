// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::breadcrumbs::{Breadcrumbs, STYLES, specs};
#[derive(Clone, Debug, PartialEq)]
pub enum BreadcrumbsSetting {
    OnlyOwn(bool),
    Color(i32),
    Rainbow(bool),
    Height(f32),
    Temporary(bool),
    Alive(i32),
    Fade(bool),
    Shader(bool),
    Style(usize),
    StyleFloat(usize, String, f32),
    StyleColor(usize, i32),
}
impl BreadcrumbsSetting {
    pub fn apply(self, s: &mut Breadcrumbs) {
        match self {
            Self::OnlyOwn(v) => s.only_own = v,
            Self::Color(v) => s.color = v,
            Self::Rainbow(v) => s.rainbow = v,
            Self::Height(v) => s.height = v,
            Self::Temporary(v) => s.temporary = v,
            Self::Alive(v) => s.alive = v,
            Self::Fade(v) => s.fade = v,
            Self::Shader(v) => s.shader = v,
            Self::Style(v) => s.style = v,
            Self::StyleFloat(i, n, v) => {
                s.styles[i].values.insert(n, v);
            }
            Self::StyleColor(i, v) => s.styles[i].accent = v,
        }
    }
}
fn key(path: &str) -> String {
    let mut out = String::from("liquidbounce.module.breadcrumbs");
    for name in path.split('.') {
        if name.is_empty() {
            continue;
        }
        out.push('.');
        let mut chars = name.chars();
        if let Some(c) = chars.next() {
            out.extend(c.to_lowercase());
            out.extend(chars);
        }
    }
    out
}
fn mode(t: &Tree, name: &str, path: &str, active: &str, choices: Map<String, Value>) -> Value {
    let mut v = t.mode(active, choices);
    v["name"] = json!(name);
    if t.format == JsonFormat::Interop {
        v["key"] = json!(key(path));
        v["description"] = json!((t.describe)(&format!("{}.description", key(path))));
    }
    v
}
pub fn serialize_breadcrumbs(
    s: &Breadcrumbs,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::Breadcrumbs, common);
    for (name, kind, v) in [
        ("OnlyOwn", "BOOLEAN", json!(s.only_own)),
        ("Color", "COLOR", json!(s.color)),
        ("Rainbow", "BOOLEAN", json!(s.rainbow)),
    ] {
        values.push(t.leaf(name, kind, v, &key(name), None));
    }
    values.push(t.ranged(
        "Height",
        "FLOAT",
        float_json(s.height, "Breadcrumbs.Height")?,
        json!({"from":0,"to":2}),
        "",
        &key("Height"),
    ));
    values.push(t.group(
        "Temporary",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.temporary),
                &key("Temporary.Enabled"),
                None,
            ),
            t.ranged(
                "Alive",
                "INT",
                json!(s.alive),
                json!({"from":10,"to":10000}),
                "ms",
                &key("Temporary.Alive"),
            ),
            t.leaf(
                "Fade",
                "BOOLEAN",
                json!(s.fade),
                &key("Temporary.Fade"),
                None,
            ),
        ],
    ));
    let mut styles = Map::new();
    for (i, name) in STYLES.iter().enumerate() {
        let mut fields = vec![];
        for spec in specs(i) {
            let path = format!("Render.Shader.Style.{name}.{}", spec.name);
            let k = key(&path);
            fields.push(if spec.color {
                t.leaf(spec.name, "COLOR", json!(s.styles[i].accent), &k, None)
            } else {
                t.ranged(
                    spec.name,
                    "FLOAT",
                    float_json(s.styles[i].get(spec.name), &path)?,
                    json!({"from":spec.from,"to":spec.to}),
                    spec.suffix,
                    &k,
                )
            });
        }
        styles.insert((*name).into(), t.group(name, "CONFIGURABLE", fields));
    }
    let style = mode(&t, "Style", "Render.Shader.Style", STYLES[s.style], styles);
    let choices = Map::from_iter([
        ("Plain".into(), t.group("Plain", "CONFIGURABLE", vec![])),
        (
            "Shader".into(),
            t.group("Shader", "CONFIGURABLE", vec![style]),
        ),
    ]);
    values.push(mode(
        &t,
        "Render",
        "Render",
        if s.shader { "Shader" } else { "Plain" },
        choices,
    ));
    Ok(t.group("Breadcrumbs", "TOGGLEABLE", values))
}
fn change(v: BreadcrumbsSetting) -> SettingChange {
    SettingChange::Breadcrumbs(v)
}
fn color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    match v {
        Value::Number(n) => {
            decimal_int(&n.to_string()).ok_or_else(|| ConfigError::new(p, "expected ARGB integer"))
        }
        Value::String(s) => {
            let h = s.strip_prefix('#').unwrap_or(s);
            if !matches!(h.len(), 6 | 8) {
                return Err(ConfigError::new(p, "expected 6/8 hex digits"));
            }
            i64::from_str_radix(h, 16)
                .map(|v| {
                    if h.len() == 6 {
                        v as i32 | 0xff000000u32 as i32
                    } else {
                        v as i32
                    }
                })
                .map_err(|_| ConfigError::new(p, "invalid hex"))
        }
        _ => Err(ConfigError::new(p, "expected ARGB color")),
    }
}
fn choices(
    patch: &mut SettingsPatch,
    v: &Value,
    name: &str,
    path: &str,
    names: &[&str],
    mut select: impl FnMut(usize) -> BreadcrumbsSetting,
    mut parse: impl FnMut(&mut SettingsPatch, usize, &Nodes<'_>),
) {
    match v
        .get("active")
        .and_then(scalar_string)
        .and_then(|s| names.iter().position(|n| *n == s))
    {
        Some(i) => patch.changes.push(change(select(i))),
        None => patch.issues.push(ConfigError::new(
            &format!("{path}.active"),
            "missing or unknown case-sensitive mode",
        )),
    }
    let Some(children) = v.get("choices").and_then(Value::as_object) else {
        patch
            .issues
            .push(ConfigError::new(path, "expected choices object"));
        return;
    };
    for (i, n) in names.iter().enumerate() {
        let p = format!("{path}.{n}");
        match children.get(*n) {
            Some(v) => match nodes(v, n, &p) {
                Ok(n) => parse(patch, i, &n),
                Err(e) => patch.issues.push(e),
            },
            None => patch.issues.push(ConfigError::new(&p, "choice not found")),
        }
    }
    if let Err(e) = nodes(v, name, path) {
        patch.issues.push(e);
    }
}
pub(super) fn parse_breadcrumbs(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for (name, set) in [
        (
            "OnlyOwn",
            BreadcrumbsSetting::OnlyOwn as fn(bool) -> BreadcrumbsSetting,
        ),
        ("Rainbow", BreadcrumbsSetting::Rainbow),
    ] {
        leaf(patch, root, name, &format!("Breadcrumbs.{name}"), |v, p| {
            parse_bool(v, p).map(|v| change(set(v)))
        });
    }
    leaf(patch, root, "Color", "Breadcrumbs.Color", |v, p| {
        color(v, p).map(|v| change(BreadcrumbsSetting::Color(v)))
    });
    leaf(patch, root, "Height", "Breadcrumbs.Height", |v, p| {
        parse_float(v, p).map(|v| change(BreadcrumbsSetting::Height(v)))
    });
    if let Some(v) = root.get("Temporary") {
        match nodes(v, "Temporary", "Breadcrumbs.Temporary") {
            Ok(n) => {
                leaf(
                    patch,
                    &n,
                    "Enabled",
                    "Breadcrumbs.Temporary.Enabled",
                    |v, p| parse_bool(v, p).map(|v| change(BreadcrumbsSetting::Temporary(v))),
                );
                leaf(patch, &n, "Alive", "Breadcrumbs.Temporary.Alive", |v, p| {
                    parse_int(v, p).map(|v| change(BreadcrumbsSetting::Alive(v)))
                });
                leaf(patch, &n, "Fade", "Breadcrumbs.Temporary.Fade", |v, p| {
                    parse_bool(v, p).map(|v| change(BreadcrumbsSetting::Fade(v)))
                });
            }
            Err(e) => patch.issues.push(e),
        }
    }
    if let Some(v) = root.get("Render") {
        choices(
            patch,
            v,
            "Render",
            "Breadcrumbs.Render",
            &["Plain", "Shader"],
            |i| BreadcrumbsSetting::Shader(i == 1),
            |patch, i, n| {
                if i != 1 {
                    return;
                }
                if let Some(style) = n.get("Style") {
                    choices(
                        patch,
                        style,
                        "Style",
                        "Breadcrumbs.Render.Shader.Style",
                        &STYLES,
                        BreadcrumbsSetting::Style,
                        |patch, i, n| {
                            for spec in specs(i) {
                                let p = format!(
                                    "Breadcrumbs.Render.Shader.Style.{}.{}",
                                    STYLES[i], spec.name
                                );
                                leaf(patch, n, spec.name, &p, |v, p| {
                                    if spec.color {
                                        color(v, p)
                                            .map(|v| change(BreadcrumbsSetting::StyleColor(i, v)))
                                    } else {
                                        parse_float(v, p).map(|v| {
                                            change(BreadcrumbsSetting::StyleFloat(
                                                i,
                                                spec.name.into(),
                                                v,
                                            ))
                                        })
                                    }
                                });
                            }
                        },
                    );
                }
            },
        );
    }
}
