// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::chest_stealer_runtime::ChestStealerRuntime;
use liquidbounce_behaviors::{
    chest_stealer::{SELECTIONS, STARTS},
    no_rotate_set::NoRotateSet,
};
#[derive(Clone, Debug, PartialEq)]
pub enum ChestStealerSetting {
    Inventory(ChestCleanerSetting),
    Rotation(NoRotateSetSetting),
    Value(&'static str, Value),
}
const PATHS: &[&str] = &[
    "SelectionMode",
    "SelectionMode.Distance.StartItem",
    "SelectionMode.Distance.RandomFactor",
    "SelectionMode.Index.Order",
    "MoveMode",
    "QuickSwaps",
    "OnFull",
    "Aura.Enabled",
    "Aura.Range",
    "Aura.WallRange",
    "Aura.Delay",
    "Aura.SwingMode",
    "Aura.NotDuringCombat",
    "Aura.TrackManualInteractions",
    "Aura.PauseOn",
    "Aura.AwaitContainer.Enabled",
    "Aura.AwaitContainer.Timeout",
    "Aura.AwaitContainer.MaxRetries",
    "Aura.ValidStorageBlocks",
    "SilentScreen.Enabled",
    "SilentScreen.UnlockCursor",
    "SilentScreen.DrawInventoryTag.Enabled",
    "SilentScreen.DrawInventoryTag.Background",
    "SilentScreen.DrawInventoryTag.Background.Rect.Color",
    "SilentScreen.DrawInventoryTag.Background.Rect.OutlineColor",
    "SilentScreen.DrawInventoryTag.Background.Rect.Margin",
    "SilentScreen.DrawInventoryTag.Background.Rect.Radius",
    "SilentScreen.DrawInventoryTag.Background.Rect.Gradient",
    "SilentScreen.DrawInventoryTag.Background.Rect.Glow",
    "SilentScreen.DrawInventoryTag.Scale",
    "SilentScreen.DrawInventoryTag.RenderOffset",
    "SilentScreen.DrawInventoryTag.ShowTitle",
];
fn remap(mut e: ConfigError) -> ConfigError {
    e.path = e.path.replace("ChestCleaner", "ChestStealer").replace(
        "NoRotateSet.Mode.ResetRotation.Rotations",
        "ChestStealer.Aura.Rotations",
    );
    e
}
fn choice(v: &Value, choices: &[&str], p: &str) -> Result<usize, ConfigError> {
    let tag = scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected choice"))?;
    choices
        .iter()
        .position(|s| s.eq_ignore_ascii_case(&tag))
        .ok_or_else(|| ConfigError::new(p, "unknown choice"))
}
fn color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    match v {
        Value::Number(n) => {
            decimal_int(&n.to_string()).ok_or_else(|| ConfigError::new(p, "invalid ARGB"))
        }
        Value::String(s) => {
            let h = s.strip_prefix('#').unwrap_or(s);
            if !matches!(h.len(), 6 | 8) {
                return Err(ConfigError::new(p, "expected 6/8 hex digits"));
            }
            let n =
                i64::from_str_radix(h, 16).map_err(|_| ConfigError::new(p, "invalid hex color"))?;
            Ok(n as i32
                | if h.len() == 6 {
                    0xff00_0000u32 as i32
                } else {
                    0
                })
        }
        _ => Err(ConfigError::new(p, "expected ARGB color")),
    }
}
impl ChestStealerSetting {
    pub fn apply(
        self,
        s: &mut ChestStealerRuntime,
        items: &[String],
        blocks: &[String],
    ) -> Result<(), ConfigError> {
        match self {
            Self::Inventory(v) => {
                v.apply(&mut s.settings.inventory, items).map_err(remap)?;
                s.settings.inventory.constraints.requirements &= 31;
            }
            Self::Rotation(v) => {
                let mut proxy = NoRotateSet {
                    rotations: std::mem::take(&mut s.rotations),
                    ..Default::default()
                };
                v.apply(&mut proxy);
                s.rotations = proxy.rotations;
            }
            Self::Value(path, v) => {
                let p = format!("ChestStealer.{path}");
                let c = &mut s.settings;
                match path {
                    "SelectionMode" => c.selection = choice(&v, &SELECTIONS, &p)?,
                    "SelectionMode.Distance.StartItem" => c.start = choice(&v, &STARTS, &p)?,
                    "SelectionMode.Distance.RandomFactor" => c.factor = parse_float_pair(&v, &p)?,
                    "SelectionMode.Index.Order" => {
                        c.descending = choice(&v, &["Ascending", "Descending"], &p)? == 1
                    }
                    "MoveMode" => c.drag = choice(&v, &["QuickMove", "DragAndDrop"], &p)? == 1,
                    "QuickSwaps" => c.quick_swaps = parse_bool(&v, &p)?,
                    "OnFull" => c.throw_full = choice(&v, &["None", "Throw"], &p)? == 1,
                    "Aura.Enabled" => {
                        c.aura.enabled = parse_bool(&v, &p)?;
                    }
                    "Aura.Range" => c.aura.range = parse_float(&v, &p)?,
                    "Aura.WallRange" => c.aura.walls = parse_float(&v, &p)?.min(c.aura.range),
                    "Aura.Delay" => c.aura.delay = parse_int(&v, &p)?,
                    "Aura.SwingMode" => {
                        c.aura.hide_swing = choice(&v, &["DoNotHide", "HideForClient"], &p)? == 1
                    }
                    "Aura.NotDuringCombat" => c.aura.not_combat = parse_bool(&v, &p)?,
                    "Aura.TrackManualInteractions" => c.aura.track_manual = parse_bool(&v, &p)?,
                    "Aura.PauseOn" => {
                        c.aura.pause_on = 0;
                        let entries = match &v {
                            Value::Array(a) => a.clone(),
                            _ => vec![v],
                        };
                        for v in entries {
                            if let Ok(i) = choice(&v, &["Combat", "UsingItem"], &p) {
                                c.aura.pause_on |= 1 << i;
                            }
                        }
                    }
                    "Aura.AwaitContainer.Enabled" => c.aura.await_container = parse_bool(&v, &p)?,
                    "Aura.AwaitContainer.Timeout" => c.aura.timeout = parse_int(&v, &p)?,
                    "Aura.AwaitContainer.MaxRetries" => c.aura.max_retries = parse_int(&v, &p)?,
                    "Aura.ValidStorageBlocks" => {
                        s.blocks_initialized = true;
                        c.aura.blocks.clear();
                        let entries = match &v {
                            Value::Array(a) => a.clone(),
                            _ => vec![v],
                        };
                        for v in entries {
                            let raw = match v {
                                Value::String(s) => s,
                                Value::Number(n) => n.to_string(),
                                _ => return Err(ConfigError::new(&p, "expected registry entry")),
                            };
                            let id = if raw.starts_with(':') {
                                format!("minecraft{raw}")
                            } else if raw.contains(':') {
                                raw
                            } else {
                                format!("minecraft:{raw}")
                            };
                            c.aura.blocks.insert(if blocks.contains(&id) {
                                id
                            } else {
                                "minecraft:air".into()
                            });
                        }
                    }
                    "SilentScreen.Enabled" => {
                        c.silent.enabled = parse_bool(&v, &p)?;
                        if !c.silent.enabled {
                            s.should_hide = false;
                            s.last_block = None;
                        }
                    }
                    "SilentScreen.UnlockCursor" => c.silent.unlock = parse_bool(&v, &p)?,
                    "SilentScreen.DrawInventoryTag.Enabled" => c.silent.draw = parse_bool(&v, &p)?,
                    "SilentScreen.DrawInventoryTag.Background" => {
                        c.silent.texture = choice(&v, &["Rect", "Texture"], &p)? == 1
                    }
                    "SilentScreen.DrawInventoryTag.Background.Rect.Color" => {
                        c.silent.color = color(&v, &p)?
                    }
                    "SilentScreen.DrawInventoryTag.Background.Rect.OutlineColor" => {
                        c.silent.outline = color(&v, &p)?
                    }
                    "SilentScreen.DrawInventoryTag.Background.Rect.Margin" => {
                        c.silent.margin = parse_float(&v, &p)?
                    }
                    "SilentScreen.DrawInventoryTag.Background.Rect.Radius" => {
                        c.silent.radius = parse_float(&v, &p)?
                    }
                    "SilentScreen.DrawInventoryTag.Background.Rect.Gradient" => {
                        c.silent.gradient = parse_float(&v, &p)?
                    }
                    "SilentScreen.DrawInventoryTag.Background.Rect.Glow" => {
                        c.silent.glow = parse_float(&v, &p)?
                    }
                    "SilentScreen.DrawInventoryTag.Scale" => c.silent.scale = parse_float(&v, &p)?,
                    "SilentScreen.DrawInventoryTag.ShowTitle" => {
                        c.silent.title = parse_bool(&v, &p)?
                    }
                    "SilentScreen.DrawInventoryTag.RenderOffset" => {
                        for (i, key) in ["x", "y", "z"].into_iter().enumerate() {
                            c.silent.offset[i] = match &v[key] {
                                Value::Number(n) => n.to_string().parse().ok(),
                                Value::String(s) => s.parse().ok(),
                                _ => None,
                            }
                            .ok_or_else(|| ConfigError::new(&p, "invalid vector component"))?;
                        }
                    }
                    _ => return Err(ConfigError::new(&p, "unknown setting")),
                }
                if path == "Aura.Enabled" && !s.settings.aura.enabled {
                    s.clear_aura();
                }
            }
        }
        Ok(())
    }
}
fn key(path: &str) -> String {
    let mut s = "liquidbounce.module.chestStealer".to_owned();
    for part in path.split('.') {
        s.push('.');
        let mut c = part.chars();
        if let Some(f) = c.next() {
            s.extend(f.to_lowercase());
            s.extend(c);
        }
    }
    s
}
fn rename(v: &mut Value) {
    match v {
        Value::String(s) => {
            *s = s
                .replace("ChestCleaner", "ChestStealer")
                .replace("chestCleaner", "chestStealer")
                .replace(
                    "liquidbounce.module.noRotateSet.mode.resetRotation.rotations",
                    "liquidbounce.module.chestStealer.aura.rotations",
                )
        }
        Value::Array(a) => a.iter_mut().for_each(rename),
        Value::Object(o) => o.values_mut().for_each(rename),
        _ => {}
    }
}
pub fn serialize_chest_stealer(
    s: &ChestStealerRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let mut tree = serialize_chest_cleaner(&s.settings.inventory, c, f, &|k| {
        d(&k.replace("chestCleaner", "chestStealer"))
    })
    .map_err(remap)?;
    rename(&mut tree);
    let values = tree["value"].as_array_mut().unwrap();
    values.retain(|v| v["name"] != "Filter" && v["name"] != "Items");
    if let Some(requires) = values
        .iter_mut()
        .find(|v| v["name"] == "Constraints")
        .and_then(|v| v["value"].as_array_mut())
        .and_then(|v| v.iter_mut().find(|v| v["name"] == "Requires"))
    {
        if f == JsonFormat::Interop {
            requires["choices"] = json!([
                "NoMovement",
                "NoRotation",
                "NotUsingItem",
                "NotBreaking",
                "NotDuringCombat"
            ]);
        }
        if let Some(a) = requires["value"].as_array_mut() {
            a.retain(|v| v != "InventoryOpen");
        }
    }
    let t = Tree {
        format: f,
        describe: d,
    };
    let leaf =
        |name: &str, path: &str, kind: &str, v: Value| t.leaf(name, kind, v, &key(path), None);
    let choose = |name: &str, path: &str, value: &str, choices: &[&str]| {
        let mut v = leaf(name, path, "CHOOSE", json!(value));
        if f == JsonFormat::Interop {
            v["choices"] = json!(choices);
        }
        v
    };
    let ranged = |name: &str, path: &str, kind: &str, v: Value, from: f64, to: f64, unit: &str| {
        t.ranged(
            name,
            kind,
            v,
            json!({"from":from,"to":to}),
            unit,
            &key(path),
        )
    };
    let cfg = &s.settings;
    let mut selection = t.group("SelectionMode", "CHOICE", vec![]);
    selection["active"] = json!(SELECTIONS[cfg.selection]);
    selection["choices"] = json!({"Value":t.group("Value","CONFIGURABLE",vec![]),"Distance":t.group("Distance","CONFIGURABLE",vec![choose("StartItem","SelectionMode.Distance.StartItem",STARTS[cfg.start],&STARTS),ranged("RandomFactor","SelectionMode.Distance.RandomFactor","FLOAT_RANGE",json!({"from":cfg.factor[0],"to":cfg.factor[1]}),0.25,2.,"x")]),"Index":t.group("Index","CONFIGURABLE",vec![choose("Order","SelectionMode.Index.Order",if cfg.descending{"Descending"}else{"Ascending"},&["Ascending","Descending"])]),"Random":t.group("Random","CONFIGURABLE",vec![])});
    values.push(selection);
    values.push(choose(
        "MoveMode",
        "MoveMode",
        if cfg.drag { "DragAndDrop" } else { "QuickMove" },
        &["QuickMove", "DragAndDrop"],
    ));
    values.push(leaf(
        "QuickSwaps",
        "QuickSwaps",
        "BOOLEAN",
        json!(cfg.quick_swaps),
    ));
    values.push(choose(
        "OnFull",
        "OnFull",
        if cfg.throw_full { "Throw" } else { "None" },
        &["None", "Throw"],
    ));
    let mut rotation = serialize_no_rotate_set(
        &NoRotateSet {
            rotations: s.rotations.clone(),
            ..Default::default()
        },
        c,
        f,
        &|k| {
            d(&k.replace(
                "liquidbounce.module.noRotateSet.mode.resetRotation.rotations",
                "liquidbounce.module.chestStealer.aura.rotations",
            ))
        },
    )?;
    let mut rotation = rotation["value"]
        .as_array_mut()
        .and_then(|v| v.iter_mut().find(|v| v["name"] == "Mode"))
        .and_then(|v| v["choices"]["ResetRotation"]["value"].as_array_mut())
        .and_then(|v| v.first_mut())
        .map(Value::take)
        .ok_or_else(|| {
            ConfigError::new(
                "ChestStealer.Aura.Rotations",
                "shared rotation tree changed",
            )
        })?;
    rename(&mut rotation);
    let a = &cfg.aura;
    let mut blocks = leaf(
        "ValidStorageBlocks",
        "Aura.ValidStorageBlocks",
        "REGISTRY_LIST",
        json!(a.blocks),
    );
    if f == JsonFormat::Interop {
        blocks["registry"] = json!("block");
    }
    values.push(t.group(
        "Aura",
        "TOGGLEABLE",
        vec![
            leaf("Enabled", "Aura.Enabled", "BOOLEAN", json!(a.enabled)),
            ranged(
                "Range",
                "Aura.Range",
                "FLOAT",
                float_json(a.range, "ChestStealer.Aura.Range")?,
                1.,
                6.,
                "",
            ),
            ranged(
                "WallRange",
                "Aura.WallRange",
                "FLOAT",
                float_json(a.walls, "ChestStealer.Aura.WallRange")?,
                0.,
                6.,
                "",
            ),
            ranged(
                "Delay",
                "Aura.Delay",
                "INT",
                json!(a.delay),
                1.,
                80.,
                "ticks",
            ),
            choose(
                "SwingMode",
                "Aura.SwingMode",
                if a.hide_swing {
                    "HideForClient"
                } else {
                    "DoNotHide"
                },
                &["DoNotHide", "HideForClient"],
            ),
            leaf(
                "NotDuringCombat",
                "Aura.NotDuringCombat",
                "BOOLEAN",
                json!(a.not_combat),
            ),
            leaf(
                "TrackManualInteractions",
                "Aura.TrackManualInteractions",
                "BOOLEAN",
                json!(a.track_manual),
            ),
            t.multi(
                "PauseOn",
                u32::from(a.pause_on),
                &["Combat", "UsingItem"],
                &key("Aura.PauseOn"),
            ),
            blocks,
            t.group(
                "AwaitContainer",
                "TOGGLEABLE",
                vec![
                    leaf(
                        "Enabled",
                        "Aura.AwaitContainer.Enabled",
                        "BOOLEAN",
                        json!(a.await_container),
                    ),
                    ranged(
                        "Timeout",
                        "Aura.AwaitContainer.Timeout",
                        "INT",
                        json!(a.timeout),
                        1.,
                        80.,
                        "ticks",
                    ),
                    ranged(
                        "MaxRetries",
                        "Aura.AwaitContainer.MaxRetries",
                        "INT",
                        json!(a.max_retries),
                        1.,
                        10.,
                        "",
                    ),
                ],
            ),
            rotation,
        ],
    ));
    let p = &cfg.silent;
    let mut background = t.group("Background", "CHOICE", vec![]);
    background["active"] = json!(if p.texture { "Texture" } else { "Rect" });
    background["choices"] = json!({"Rect":t.group("Rect","CONFIGURABLE",vec![leaf("Color","SilentScreen.DrawInventoryTag.Background.Rect.Color","COLOR",json!(p.color)),leaf("OutlineColor","SilentScreen.DrawInventoryTag.Background.Rect.OutlineColor","COLOR",json!(p.outline)),ranged("Margin","SilentScreen.DrawInventoryTag.Background.Rect.Margin","FLOAT",float_json(p.margin,"ChestStealer.SilentScreen.Margin")?,0.,100.,""),ranged("Radius","SilentScreen.DrawInventoryTag.Background.Rect.Radius","FLOAT",float_json(p.radius,"ChestStealer.SilentScreen.Radius")?,0.,16.,""),ranged("Gradient","SilentScreen.DrawInventoryTag.Background.Rect.Gradient","FLOAT",float_json(p.gradient,"ChestStealer.SilentScreen.Gradient")?,0.,1.,""),ranged("Glow","SilentScreen.DrawInventoryTag.Background.Rect.Glow","FLOAT",float_json(p.glow,"ChestStealer.SilentScreen.Glow")?,0.,1.,"")]),"Texture":t.group("Texture","CONFIGURABLE",vec![])});
    values.push(t.group(
        "SilentScreen",
        "TOGGLEABLE",
        vec![
            leaf(
                "Enabled",
                "SilentScreen.Enabled",
                "BOOLEAN",
                json!(p.enabled),
            ),
            leaf(
                "UnlockCursor",
                "SilentScreen.UnlockCursor",
                "BOOLEAN",
                json!(p.unlock),
            ),
            t.group(
                "DrawInventoryTag",
                "TOGGLEABLE",
                vec![
                    leaf(
                        "Enabled",
                        "SilentScreen.DrawInventoryTag.Enabled",
                        "BOOLEAN",
                        json!(p.draw),
                    ),
                    background,
                    ranged(
                        "Scale",
                        "SilentScreen.DrawInventoryTag.Scale",
                        "FLOAT",
                        float_json(p.scale, "ChestStealer.SilentScreen.Scale")?,
                        0.25,
                        4.,
                        "",
                    ),
                    leaf(
                        "RenderOffset",
                        "SilentScreen.DrawInventoryTag.RenderOffset",
                        "VECTOR3_D",
                        json!({"x":p.offset[0],"y":p.offset[1],"z":p.offset[2]}),
                    ),
                    leaf(
                        "ShowTitle",
                        "SilentScreen.DrawInventoryTag.ShowTitle",
                        "BOOLEAN",
                        json!(p.title),
                    ),
                ],
            ),
        ],
    ));
    Ok(tree)
}
pub(super) fn parse_chest_stealer(p: &mut SettingsPatch, root: &Nodes<'_>) {
    let mut proxy = SettingsPatch {
        module: ModuleKind::ChestCleaner,
        changes: vec![],
        issues: vec![],
    };
    let mut filtered = root.clone();
    filtered.remove("Items");
    filtered.remove("Filter");
    chest_cleaner::parse_chest_cleaner(&mut proxy, &filtered);
    for c in proxy.changes {
        if let SettingChange::ChestCleaner(v) = c {
            p.changes
                .push(SettingChange::ChestStealer(ChestStealerSetting::Inventory(
                    v,
                )));
        }
    }
    p.issues.extend(proxy.issues.into_iter().map(remap));
    fn walk(p: &mut SettingsPatch, n: &Nodes<'_>, prefix: &str) {
        for (name, node) in n {
            let path = if prefix.is_empty() {
                name.to_owned()
            } else {
                format!("{prefix}.{name}")
            };
            if let Some(path) = PATHS.iter().copied().find(|v| *v == path) {
                if let Some(active) = node.get("active") {
                    p.changes
                        .push(SettingChange::ChestStealer(ChestStealerSetting::Value(
                            path,
                            active.clone(),
                        )));
                } else if let Some(v) = node.get("value") {
                    p.changes
                        .push(SettingChange::ChestStealer(ChestStealerSetting::Value(
                            path,
                            v.clone(),
                        )));
                }
            }
            if let Some(choices) = node.get("choices").and_then(Value::as_object) {
                for (tag, choice) in choices {
                    if let Ok(children) = nodes(choice, tag, &path) {
                        walk(p, &children, &format!("{path}.{tag}"));
                    }
                }
            }
            if node["value"].is_array()
                && let Ok(children) = nodes(node, name, &path)
            {
                walk(p, &children, &path);
            }
        }
    }
    walk(p, root, "");
    if let Some(aura) = root
        .get("Aura")
        .and_then(|v| nodes(v, "Aura", "ChestStealer.Aura").ok())
        && let Some(v) = aura.get("Rotations")
    {
        let proxy = json!({"name":"NoRotateSet","value":[{"name":"Mode","active":"ResetRotation","value":[],"choices":{"SilentAccept":{"name":"SilentAccept","value":[]},"ResetRotation":{"name":"ResetRotation","value":[v]}}}]});
        let root = nodes(&proxy, "NoRotateSet", "NoRotateSet").unwrap();
        let mut parsed = SettingsPatch {
            module: ModuleKind::NoRotateSet,
            changes: vec![],
            issues: vec![],
        };
        no_rotate_set::parse_no_rotate_set(&mut parsed, &root);
        for c in parsed.changes {
            if let SettingChange::NoRotateSet(v) = c
                && !matches!(v, NoRotateSetSetting::Mode(_))
            {
                p.changes
                    .push(SettingChange::ChestStealer(ChestStealerSetting::Rotation(
                        v,
                    )));
            }
        }
        p.issues.extend(parsed.issues.into_iter().map(remap));
    }
}
