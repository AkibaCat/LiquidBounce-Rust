// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleAutoTool tree, including InventoryConstraints' legacy migration.
use super::*;
use liquidbounce_behaviors::auto_tool::{AutoTool, Filter, REQUIREMENTS, ToolSelector};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub enum AutoToolSetting {
    Selector(ToolSelector),
    Value(&'static str, Value),
}
impl AutoToolSetting {
    pub fn apply(
        self,
        module: &mut AutoTool,
        block_registry: &[String],
    ) -> Result<(), ConfigError> {
        let Self::Value(path, value) = self else {
            if let Self::Selector(selector) = self {
                module.tool_selector = selector;
            }
            return Ok(());
        };
        let full = format!("AutoTool.{path}");
        match path {
            "IgnoreDurability" => module.ignore_durability = parse_bool(&value, &full)?,
            "ConsiderInventory" => {
                // Lifecycle depends on inGame (its direct parent is a Mode) and is
                // dispatched by the runtime around this value mutation.
                module.consider_inventory = parse_bool(&value, &full)?;
            }
            "Slot" => module.slot = parse_int(&value, &full)?,
            "Filter" => module.filter = filter(&value, &full)?,
            "Blocks" => module.blocks = registry_set(&value, block_registry, &full)?,
            "SilkTouchHandler" => module.silk_touch_handler = parse_bool(&value, &full)?,
            "SilkTouchFilter" => module.silk_touch_filter = filter(&value, &full)?,
            "SilkTouchBlocks" => {
                module.silk_touch_blocks = registry_set(&value, block_registry, &full)?
            }
            "SwapBackDelay" => module.swap_back_delay = parse_int(&value, &full)?,
            "SwitchDelay" => module.switch_delay = parse_int(&value, &full)?,
            "RequireSneaking" => module.require_sneaking = parse_bool(&value, &full)?,
            "NotDuringCombat" => module.not_during_combat = parse_bool(&value, &full)?,
            "RequireNearBed" => module.require_near_bed = parse_bool(&value, &full)?,
            "BedDistance" => module.bed_distance = parse_float(&value, &full)?,
            "StartDelay" => module.constraints.start_delay = parse_int_pair(&value, &full)?,
            "ClickDelay" => module.constraints.click_delay = parse_int_pair(&value, &full)?,
            "CloseDelay" => module.constraints.close_delay = parse_int_pair(&value, &full)?,
            "MissChance" => module.constraints.miss_chance = parse_int_pair(&value, &full)?,
            "SlotMultiplier" => module.constraints.slot_multiplier = parse_float(&value, &full)?,
            "MillisecondDelays" => {
                module.constraints.millisecond_delays = parse_bool(&value, &full)?
            }
            "Requires" => {
                module.constraints.requirements = 0;
                let mut include = |value: &Value| -> Result<(), ConfigError> {
                    let tag = gson_string(value, &full)?;
                    if let Some(index) = REQUIREMENTS
                        .iter()
                        .position(|choice| java_tag_eq(choice, &tag))
                    {
                        module.constraints.requirements |= 1 << index;
                    }
                    Ok(())
                };
                match value {
                    Value::Array(values) => {
                        for value in values {
                            include(&value)?;
                        }
                    }
                    Value::String(_) | Value::Number(_) | Value::Bool(_) => include(&value)?,
                    _ => {}
                }
            }
            _ => return Err(ConfigError::new(&full, "unknown setting")),
        }
        Ok(())
    }
}
fn gson_string(value: &Value, path: &str) -> Result<String, ConfigError> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Array(values) if values.len() == 1 => gson_string(&values[0], path),
        _ => Err(ConfigError::new(path, "entry has no Gson string value")),
    }
}
fn java_tag_eq(a: &str, b: &str) -> bool {
    let fold = |c: char| {
        let mut it = c.to_uppercase();
        let first = it.next().unwrap_or(c);
        let up = if it.next().is_none() { first } else { c };
        up.to_lowercase().next().unwrap_or(up)
    };
    a.chars().map(fold).eq(b.chars().map(fold))
}
fn filter(value: &Value, path: &str) -> Result<Filter, ConfigError> {
    let tag = gson_string(value, path)?;
    if java_tag_eq(&tag, "Whitelist") {
        Ok(Filter::Whitelist)
    } else if java_tag_eq(&tag, "Blacklist") {
        Ok(Filter::Blacklist)
    } else {
        Err(ConfigError::new(path, "unknown filter"))
    }
}
fn registry_set(
    value: &Value,
    registry: &[String],
    path: &str,
) -> Result<BTreeSet<String>, ConfigError> {
    let resolve = |value: &Value| -> Result<String, ConfigError> {
        let text = match value {
            Value::String(value) => value.clone(),
            Value::Number(value) => value.to_string(),
            _ => {
                return Err(ConfigError::new(
                    path,
                    "registry entries must be string or number tokens",
                ));
            }
        };
        let id = match text.find(':') {
            None => format!("minecraft:{text}"),
            Some(0) => format!("minecraft{text}"),
            Some(_) => text,
        };
        Ok(if registry.contains(&id) {
            id
        } else {
            "minecraft:air".to_owned()
        })
    };
    match value {
        Value::Array(values) => values.iter().map(resolve).collect(),
        _ => Ok(BTreeSet::from([resolve(value)?])),
    }
}
fn sorted_blocks(values: &BTreeSet<String>) -> Value {
    let mut values: Vec<_> = values.iter().collect();
    values.sort_by(|a, b| {
        let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
        let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
        ap.cmp(bp).then_with(|| an.cmp(bn))
    });
    json!(values)
}
pub fn serialize_auto_tool(
    state: &AutoTool,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let key = "liquidbounce.module.autoTool";
    let boolean = |name: &str, value: bool, path: &str| {
        tree.leaf(
            name,
            "BOOLEAN",
            json!(value),
            &format!("{key}.{path}"),
            None,
        )
    };
    let choice = |name: &str, value: Filter, path: &str| {
        let mut leaf = tree.leaf(
            name,
            "CHOOSE",
            json!(value.tag()),
            &format!("{key}.{path}"),
            None,
        );
        if format == JsonFormat::Interop {
            leaf["choices"] = json!(["Whitelist", "Blacklist"]);
        }
        leaf
    };
    let blocks = |value: &BTreeSet<String>, path: &str| {
        let mut leaf = tree.leaf(
            "Blocks",
            "REGISTRY_LIST",
            sorted_blocks(value),
            &format!("{key}.{path}"),
            None,
        );
        if format == JsonFormat::Interop {
            leaf["registry"] = json!("block");
        }
        leaf
    };
    let mut constraints = Vec::new();
    for (name, value, range, suffix, path) in [
        (
            "StartDelay",
            state.constraints.start_delay,
            [0, 1000],
            "ms",
            "startDelay",
        ),
        (
            "ClickDelay",
            state.constraints.click_delay,
            [0, 1000],
            "ms",
            "clickDelay",
        ),
        (
            "CloseDelay",
            state.constraints.close_delay,
            [0, 1000],
            "ms",
            "closeDelay",
        ),
        (
            "MissChance",
            state.constraints.miss_chance,
            [0, 100],
            "%",
            "missChance",
        ),
    ] {
        constraints.push(tree.ranged(
            name,
            "INT_RANGE",
            json!({"from":value[0],"to":value[1]}),
            json!({"from":range[0],"to":range[1]}),
            suffix,
            &format!("{key}.toolSelector.dynamic.considerInventory.constraints.{path}"),
        ));
    }
    constraints.push(tree.ranged(
        "SlotMultiplier",
        "FLOAT",
        float_json(state.constraints.slot_multiplier, "AutoTool.SlotMultiplier")?,
        json!({"from":0.0,"to":1.0}),
        "x",
        &format!("{key}.toolSelector.dynamic.considerInventory.constraints.slotMultiplier"),
    ));
    if format == JsonFormat::File {
        constraints.push(boolean(
            "MillisecondDelays",
            state.constraints.millisecond_delays,
            "toolSelector.dynamic.considerInventory.constraints.millisecondDelays",
        ));
    }
    constraints.push(tree.multi(
        "Requires",
        u32::from(state.constraints.requirements),
        &REQUIREMENTS,
        &format!("{key}.toolSelector.dynamic.considerInventory.constraints.requires"),
    ));
    let dynamic = tree.group(
        "Dynamic",
        "CONFIGURABLE",
        vec![
            boolean(
                "IgnoreDurability",
                state.ignore_durability,
                "toolSelector.dynamic.ignoreDurability",
            ),
            tree.group(
                "ConsiderInventory",
                "TOGGLEABLE",
                vec![
                    boolean(
                        "Enabled",
                        state.consider_inventory,
                        "toolSelector.dynamic.considerInventory.enabled",
                    ),
                    tree.group("Constraints", "CONFIGURABLE", constraints),
                ],
            ),
        ],
    );
    let static_mode = tree.group(
        "Static",
        "CONFIGURABLE",
        vec![tree.ranged(
            "Slot",
            "INT",
            json!(state.slot),
            json!({"from":0,"to":8}),
            "",
            &format!("{key}.toolSelector.static.slot"),
        )],
    );
    let mut modes = Map::new();
    modes.insert("Dynamic".to_owned(), dynamic);
    modes.insert("Static".to_owned(), static_mode);
    let mut selector = tree.mode(state.tool_selector.tag(), modes);
    selector["name"] = json!("ToolSelector");
    let mut values = tree.common(ModuleKind::AutoTool, common);
    values.push(selector);
    values.push(choice("Filter", state.filter, "filter"));
    values.push(blocks(&state.blocks, "blocks"));
    values.push(tree.group(
        "SilkTouchHandler",
        "TOGGLEABLE",
        vec![
            boolean(
                "Enabled",
                state.silk_touch_handler,
                "silkTouchHandler.enabled",
            ),
            choice("Filter", state.silk_touch_filter, "silkTouchHandler.filter"),
            blocks(&state.silk_touch_blocks, "silkTouchHandler.blocks"),
        ],
    ));
    for (name, value, from, to, path) in [
        (
            "SwapBackDelay",
            state.swap_back_delay,
            1,
            100,
            "swapBackDelay",
        ),
        ("SwitchDelay", state.switch_delay, 0, 100, "switchDelay"),
    ] {
        values.push(tree.ranged(
            name,
            "INT",
            json!(value),
            json!({"from":from,"to":to}),
            "ticks",
            &format!("{key}.{path}"),
        ));
    }
    values.push(boolean(
        "RequireSneaking",
        state.require_sneaking,
        "requireSneaking",
    ));
    values.push(boolean(
        "NotDuringCombat",
        state.not_during_combat,
        "notDuringCombat",
    ));
    values.push(tree.group(
        "RequireNearBed",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", state.require_near_bed, "requireNearBed.enabled"),
            tree.ranged(
                "Distance",
                "FLOAT",
                float_json(state.bed_distance, "AutoTool.RequireNearBed.Distance")?,
                json!({"from":3.0,"to":50.0}),
                "",
                &format!("{key}.requireNearBed.distance"),
            ),
        ],
    ));
    Ok(tree.group("AutoTool", "TOGGLEABLE", values))
}
fn add(patch: &mut SettingsPatch, nodes: &Nodes<'_>, name: &str, path: &'static str) {
    if let Some(node) = nodes.get(name) {
        // JsonNull clears only multichoice. Missing value is a Kotlin null failure.
        let value = if path == "Requires" {
            node.get("value")
                .ok_or_else(|| ConfigError::new(path, "missing value"))
        } else {
            value_of(node, path)
        };
        match value {
            Ok(value) => patch
                .changes
                .push(SettingChange::AutoTool(AutoToolSetting::Value(
                    path,
                    value.clone(),
                ))),
            Err(error) => patch.issues.push(error),
        }
    }
}
fn group<'a>(
    patch: &mut SettingsPatch,
    parent: &Nodes<'a>,
    name: &str,
    path: &str,
) -> Option<Nodes<'a>> {
    let node = parent.get(name)?;
    match nodes(node, name, path) {
        Ok(nodes) => Some(nodes),
        Err(error) => {
            patch.issues.push(error);
            None
        }
    }
}
pub(super) fn parse_auto_tool(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(selector) = root.get("ToolSelector") {
        match selector.get("active").and_then(scalar_string).as_deref() {
            Some("Dynamic") => {
                patch
                    .changes
                    .push(SettingChange::AutoTool(AutoToolSetting::Selector(
                        ToolSelector::Dynamic,
                    )))
            }
            Some("Static") => {
                patch
                    .changes
                    .push(SettingChange::AutoTool(AutoToolSetting::Selector(
                        ToolSelector::Static,
                    )))
            }
            _ => patch.issues.push(ConfigError::new(
                "AutoTool.ToolSelector.active",
                "missing or unknown case-sensitive mode",
            )),
        }
        if let Some(choices) = selector.get("choices").and_then(Value::as_object) {
            if let Some(dynamic) = choices.get("Dynamic") {
                match nodes(dynamic, "Dynamic", "AutoTool.ToolSelector.Dynamic") {
                    Ok(dynamic) => {
                        add(patch, &dynamic, "IgnoreDurability", "IgnoreDurability");
                        if let Some(inventory) = group(
                            patch,
                            &dynamic,
                            "ConsiderInventory",
                            "AutoTool.ToolSelector.Dynamic.ConsiderInventory",
                        ) {
                            add(patch, &inventory, "Enabled", "ConsiderInventory");
                            if let Some(constraints_node) = inventory.get("Constraints") {
                                let mut migrated = (*constraints_node).clone();
                                let result = prepare_constraints(&mut migrated);
                                match result.and_then(|()| {
                                    nodes(&migrated, "Constraints", "AutoTool.Constraints")
                                }) {
                                    Ok(constraints) => {
                                        for name in [
                                            "StartDelay",
                                            "ClickDelay",
                                            "CloseDelay",
                                            "MissChance",
                                            "SlotMultiplier",
                                            "MillisecondDelays",
                                            "Requires",
                                        ] {
                                            add(patch, &constraints, name, name);
                                        }
                                    }
                                    Err(error) => patch.issues.push(error),
                                }
                            }
                        }
                    }
                    Err(error) => patch.issues.push(error),
                }
            } else {
                patch.issues.push(ConfigError::new(
                    "AutoTool.ToolSelector.Dynamic",
                    "choice not found",
                ));
            }
            if let Some(static_mode) = choices.get("Static") {
                match nodes(static_mode, "Static", "AutoTool.ToolSelector.Static") {
                    Ok(nodes) => add(patch, &nodes, "Slot", "Slot"),
                    Err(error) => patch.issues.push(error),
                }
            } else {
                patch.issues.push(ConfigError::new(
                    "AutoTool.ToolSelector.Static",
                    "choice not found",
                ));
            }
        } else {
            patch.issues.push(ConfigError::new(
                "AutoTool.ToolSelector.choices",
                "expected choices object",
            ));
        }
    }
    add(patch, root, "Filter", "Filter");
    add(patch, root, "Blocks", "Blocks");
    if let Some(silk) = group(patch, root, "SilkTouchHandler", "AutoTool.SilkTouchHandler") {
        add(patch, &silk, "Enabled", "SilkTouchHandler");
        add(patch, &silk, "Filter", "SilkTouchFilter");
        add(patch, &silk, "Blocks", "SilkTouchBlocks");
    }
    if root.contains_key("SwapBackDelay") {
        add(patch, root, "SwapBackDelay", "SwapBackDelay");
    } else {
        add(patch, root, "SwapPreviousDelay", "SwapBackDelay");
    }
    for name in ["SwitchDelay", "RequireSneaking", "NotDuringCombat"] {
        add(patch, root, name, name);
    }
    if let Some(bed) = group(patch, root, "RequireNearBed", "AutoTool.RequireNearBed") {
        add(patch, &bed, "Enabled", "RequireNearBed");
        add(patch, &bed, "Distance", "BedDistance");
    }
}
fn prepare_constraints(group: &mut Value) -> Result<(), ConfigError> {
    let has_type = group.get("valueType").is_some();
    let values = group
        .get_mut("value")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| ConfigError::new("AutoTool.Constraints", "expected value array"))?;
    for value in values.iter() {
        if !value.is_object() {
            return Err(ConfigError::new(
                "AutoTool.Constraints",
                "expected child object",
            ));
        }
    }
    if has_type || values.iter().any(|v| v["name"] == "MillisecondDelays") {
        return Ok(());
    }
    for value in values {
        if matches!(
            value["name"].as_str(),
            Some("StartDelay" | "ClickDelay" | "CloseDelay")
        ) {
            let range = value
                .get_mut("value")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| {
                    ConfigError::new("AutoTool.Constraints", "legacy delay must be range object")
                })?;
            // Gson mutates from before reading to, but this preparation copy
            // remains isolated from the caller's JSON object.
            for name in ["from", "to"] {
                let number = parse_int(
                    range.get(name).ok_or_else(|| {
                        ConfigError::new("AutoTool.Constraints", "missing range member")
                    })?,
                    "AutoTool.Constraints",
                )?;
                range.insert(name.to_owned(), json!(number.wrapping_mul(50)));
            }
        }
    }
    Ok(())
}
