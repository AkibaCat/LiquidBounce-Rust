// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::better_tab::{BetterTab, Filter, Pattern, Sorting, tag_eq};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BetterTabSetting {
    Sorting(Sorting),
    Bool(&'static str, bool),
    Int(&'static str, i32),
    Color(usize, i32),
    Mask(&'static str, u8),
    Names(&'static str, Vec<Option<Pattern>>),
}
impl BetterTabSetting {
    pub fn apply(self, s: &mut BetterTab) {
        match self {
            Self::Sorting(v) => s.sorting = v,
            Self::Color(i, v) => s.colors[i].color = v,
            Self::Bool(p, v) => match p {
                "ShowGameMode" => s.show_game_mode = v,
                "Highlight.Enabled" => s.highlight = v,
                "Highlight.Self.Enabled" => s.colors[0].enabled = v,
                "Highlight.Friends.Enabled" => s.colors[1].enabled = v,
                "Highlight.Others.Enabled" => s.colors[2].enabled = v,
                "AccurateLatency.Enabled" => s.accurate_latency = v,
                "AccurateLatency.AppendMSSuffix" => s.append_ms_suffix = v,
                "PlayerHider.Enabled" => s.player_hider = v,
                _ => {}
            },
            Self::Int(p, v) => {
                if p == "Limits.TabSize" {
                    s.tab_size = v
                } else {
                    s.column_height = v
                }
            }
            Self::Mask(p, v) => match p {
                "Visibility" => s.visibility = v,
                "Highlight.Others.Filter.FilterBy" => s.others_filter.filter_by = v,
                _ => s.hider_filter.filter_by = v,
            },
            Self::Names(p, v) => {
                if p == "Highlight.Others.Filter.Names" {
                    s.others_filter.names = v
                } else {
                    s.hider_filter.names = v
                }
            }
        }
    }
}
fn bt_key(path: &str) -> String {
    let p = path
        .split('.')
        .map(|s| {
            let mut c = s.chars();
            c.next()
                .map(|f| f.to_lowercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(".");
    format!("liquidbounce.module.betterTab.{p}")
}
fn bt_multi(tree: &Tree<'_>, name: &str, path: &str, bits: u8, names: &[&str]) -> Value {
    let selected = names
        .iter()
        .enumerate()
        .filter(|(i, _)| bits & (1 << i) != 0)
        .map(|(_, v)| *v)
        .collect::<Vec<_>>();
    let mut v = tree.leaf(name, "MULTI_CHOOSE", json!(selected), &bt_key(path), None);
    if tree.format == JsonFormat::Interop {
        v["choices"] = json!(names);
        v["canBeNone"] = json!(true);
        v["isOrderSensitive"] = json!(false);
    }
    v
}
fn bt_bool(tree: &Tree<'_>, name: &str, path: &str, v: bool) -> Value {
    tree.leaf(name, "BOOLEAN", json!(v), &bt_key(path), None)
}
fn bt_filter(tree: &Tree<'_>, path: &str, s: &Filter) -> Value {
    let mut names = tree.leaf(
        "Names",
        "MUTABLE_LIST",
        json!(
            s.names
                .iter()
                .map(|p| p.as_ref().map(Pattern::pattern))
                .collect::<Vec<_>>()
        ),
        &bt_key(&format!("{path}.Names")),
        None,
    );
    if tree.format == JsonFormat::Interop {
        names["innerValueType"] = json!("TEXT");
    }
    tree.group(
        "Filter",
        "CONFIGURABLE",
        vec![
            bt_multi(
                tree,
                "FilterBy",
                &format!("{path}.FilterBy"),
                s.filter_by,
                &["DisplayName", "PlayerName"],
            ),
            names,
        ],
    )
}
pub fn serialize_better_tab(
    s: &BetterTab,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut v = tree.common(ModuleKind::BetterTab, common);
    let mut sorting = tree.leaf(
        "Sorting",
        "CHOOSE",
        json!(s.sorting.tag()),
        &bt_key("Sorting"),
        None,
    );
    if format == JsonFormat::Interop {
        sorting["choices"] = json!(Sorting::ALL.map(Sorting::tag));
    }
    v.extend([
        sorting,
        bt_multi(
            &tree,
            "Visibility",
            "Visibility",
            s.visibility,
            &["Header", "Footer", "NameOnly"],
        ),
        bt_bool(&tree, "ShowGameMode", "ShowGameMode", s.show_game_mode),
    ]);
    v.push(tree.group(
        "Limits",
        "CONFIGURABLE",
        vec![
            tree.ranged(
                "TabSize",
                "INT",
                json!(s.tab_size),
                json!({"from":1,"to":1000}),
                "",
                &bt_key("Limits.TabSize"),
            ),
            tree.ranged(
                "ColumnHeight",
                "INT",
                json!(s.column_height),
                json!({"from":1,"to":100}),
                "",
                &bt_key("Limits.ColumnHeight"),
            ),
        ],
    ));
    let mut highlights = vec![bt_bool(&tree, "Enabled", "Highlight.Enabled", s.highlight)];
    for (i, name) in ["Self", "Friends", "Others"].iter().enumerate() {
        let path = format!("Highlight.{name}");
        let mut children = vec![
            bt_bool(
                &tree,
                "Enabled",
                &format!("{path}.Enabled"),
                s.colors[i].enabled,
            ),
            tree.leaf(
                "Color",
                "COLOR",
                json!(s.colors[i].color),
                &bt_key(&format!("{path}.Color")),
                None,
            ),
        ];
        if i == 2 {
            children.push(bt_filter(
                &tree,
                "Highlight.Others.Filter",
                &s.others_filter,
            ));
        }
        highlights.push(tree.group(name, "TOGGLEABLE", children));
    }
    v.push(tree.group("Highlight", "TOGGLEABLE", highlights));
    v.push(tree.group(
        "AccurateLatency",
        "TOGGLEABLE",
        vec![
            bt_bool(
                &tree,
                "Enabled",
                "AccurateLatency.Enabled",
                s.accurate_latency,
            ),
            bt_bool(
                &tree,
                "AppendMSSuffix",
                "AccurateLatency.AppendMSSuffix",
                s.append_ms_suffix,
            ),
        ],
    ));
    v.push(tree.group(
        "PlayerHider",
        "TOGGLEABLE",
        vec![
            bt_bool(&tree, "Enabled", "PlayerHider.Enabled", s.player_hider),
            bt_filter(&tree, "PlayerHider.Filter", &s.hider_filter),
        ],
    ));
    Ok(tree.group("BetterTab", "TOGGLEABLE", v))
}
fn bt_color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    match v {
        Value::Number(n) => {
            decimal_int(&n.to_string()).ok_or_else(|| ConfigError::new(p, "invalid ARGB number"))
        }
        Value::String(s) => {
            let h = s.strip_prefix('#').unwrap_or(s);
            if !matches!(h.len(), 6 | 8) {
                return Err(ConfigError::new(p, "expected 6/8 digit hex"));
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
        _ => Err(ConfigError::new(p, "invalid color")),
    }
}
fn bt_names(v: &Value, p: &str) -> Result<Vec<Option<Pattern>>, ConfigError> {
    fn one(v: &Value, p: &str) -> Result<Option<Pattern>, ConfigError> {
        match v {
            Value::Null => Ok(None),
            Value::String(s) => Pattern::new(s)
                .map(Some)
                .map_err(|e| ConfigError::new(p, e.to_string())),
            _ => Err(ConfigError::new(p, "Regex input must be string or null")),
        }
    }
    match v {
        Value::Array(a) => a.iter().map(|v| one(v, p)).collect(),
        v => one(v, p).map(|v| vec![v]),
    }
}
pub(super) fn parse_better_tab(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    bt_parse_group(patch, root, "");
}
fn bt_parse_group(patch: &mut SettingsPatch, root: &Nodes<'_>, prefix: &str) {
    const BOOLS: [&str; 8] = [
        "ShowGameMode",
        "Highlight.Enabled",
        "Highlight.Self.Enabled",
        "Highlight.Friends.Enabled",
        "Highlight.Others.Enabled",
        "AccurateLatency.Enabled",
        "AccurateLatency.AppendMSSuffix",
        "PlayerHider.Enabled",
    ];
    let order: &[&str] = match prefix {
        "" => &[
            "Sorting",
            "Visibility",
            "ShowGameMode",
            "Limits",
            "Highlight",
            "AccurateLatency",
            "PlayerHider",
        ],
        "Limits" => &["TabSize", "ColumnHeight"],
        "Highlight" => &["Enabled", "Self", "Friends", "Others"],
        "Highlight.Self" | "Highlight.Friends" => &["Enabled", "Color"],
        "Highlight.Others" => &["Enabled", "Color", "Filter"],
        "AccurateLatency" => &["Enabled", "AppendMSSuffix"],
        "PlayerHider" => &["Enabled", "Filter"],
        _ => &["FilterBy", "Names"],
    };
    for name in order {
        let Some(node) = root.get(*name) else {
            continue;
        };
        let path = if prefix.is_empty() {
            (*name).to_owned()
        } else {
            format!("{prefix}.{name}")
        };
        let error_path = format!("BetterTab.{path}");
        if matches!(
            *name,
            "Limits"
                | "Highlight"
                | "AccurateLatency"
                | "PlayerHider"
                | "Self"
                | "Friends"
                | "Others"
                | "Filter"
        ) {
            match nodes(node, name, &error_path) {
                Ok(children) => bt_parse_group(patch, &children, &path),
                Err(e) => patch.issues.push(e),
            }
            continue;
        }
        let value = if *name == "Names" {
            node.get("value")
                .ok_or_else(|| ConfigError::new(&error_path, "missing value"))
        } else {
            value_of(node, &error_path)
        };
        let result = value.and_then(|v| {
            if let Some(p) = BOOLS.into_iter().find(|p| *p == path) {
                return parse_bool(v, &error_path).map(|v| BetterTabSetting::Bool(p, v));
            }
            match path.as_str() {
                "Sorting" => scalar_string(v)
                    .and_then(|v| Sorting::parse(&v))
                    .map(BetterTabSetting::Sorting)
                    .ok_or_else(|| ConfigError::new(&error_path, "unknown sorting choice")),
                "Limits.TabSize" => {
                    parse_int(v, &error_path).map(|v| BetterTabSetting::Int("Limits.TabSize", v))
                }
                "Limits.ColumnHeight" => parse_int(v, &error_path)
                    .map(|v| BetterTabSetting::Int("Limits.ColumnHeight", v)),
                "Highlight.Self.Color" | "Highlight.Friends.Color" | "Highlight.Others.Color" => {
                    bt_color(v, &error_path).map(|v| {
                        BetterTabSetting::Color(
                            if path.contains("Self") {
                                0
                            } else if path.contains("Friends") {
                                1
                            } else {
                                2
                            },
                            v,
                        )
                    })
                }
                "Highlight.Others.Filter.Names" => bt_names(v, &error_path)
                    .map(|v| BetterTabSetting::Names("Highlight.Others.Filter.Names", v)),
                "PlayerHider.Filter.Names" => bt_names(v, &error_path)
                    .map(|v| BetterTabSetting::Names("PlayerHider.Filter.Names", v)),
                "Visibility"
                | "Highlight.Others.Filter.FilterBy"
                | "PlayerHider.Filter.FilterBy" => {
                    let (p, names): (&'static str, &[&str]) = if path == "Visibility" {
                        ("Visibility", &["Header", "Footer", "NameOnly"])
                    } else if path.starts_with("Highlight") {
                        (
                            "Highlight.Others.Filter.FilterBy",
                            &["DisplayName", "PlayerName"],
                        )
                    } else {
                        (
                            "PlayerHider.Filter.FilterBy",
                            &["DisplayName", "PlayerName"],
                        )
                    };
                    let mut bits = 0;
                    patch
                        .changes
                        .push(SettingChange::BetterTab(BetterTabSetting::Mask(p, 0)));
                    let entries = match v {
                        Value::Array(a) => a.iter().collect::<Vec<_>>(),
                        _ => vec![v],
                    };
                    for v in entries {
                        let tag = scalar_string(v).ok_or_else(|| {
                            ConfigError::new(&error_path, "invalid Gson string entry")
                        })?;
                        for (i, name) in names.iter().enumerate() {
                            if tag_eq(&tag, name) {
                                bits |= 1 << i;
                            }
                        }
                        patch
                            .changes
                            .push(SettingChange::BetterTab(BetterTabSetting::Mask(p, bits)));
                    }
                    Ok(BetterTabSetting::Mask(p, bits))
                }
                _ => Err(ConfigError::new(
                    &error_path,
                    "unknown declared BetterTab leaf",
                )),
            }
        });
        match result {
            Ok(v) => patch.changes.push(SettingChange::BetterTab(v)),
            Err(e) => patch.issues.push(e),
        }
    }
}
