// SPDX-License-Identifier: GPL-3.0-or-later
//! NoEntityInteract's four declared values. NoBlockInteract has common values only.
use super::*;
use liquidbounce_behaviors::no_entity_interact::{Filter, NoEntityInteract};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub enum NoEntitySetting {
    EntityTypeFilter(Filter),
    EntityTypes(Value),
    HoldingItemFilter(Filter),
    HoldingItems(Value),
}

impl NoEntitySetting {
    pub fn apply(
        self,
        module: &mut NoEntityInteract,
        entity_types: &[String],
        items: &[String],
    ) -> Result<(), ConfigError> {
        match self {
            Self::EntityTypeFilter(value) => module.entity_type_filter = value,
            Self::HoldingItemFilter(value) => module.holding_item_filter = value,
            Self::EntityTypes(value) => {
                module.entity_types = registry_set(
                    &value,
                    entity_types,
                    "minecraft:pig",
                    "NoEntityInteract.EntityTypes",
                )?
            }
            Self::HoldingItems(value) => {
                module.holding_items = registry_set(
                    &value,
                    items,
                    "minecraft:air",
                    "NoEntityInteract.HoldingItems",
                )?
            }
        }
        Ok(())
    }
}

fn registry_set(
    value: &Value,
    registry: &[String],
    default: &str,
    path: &str,
) -> Result<BTreeSet<String>, ConfigError> {
    // Both source registries are DefaultedMappedRegistry: invalid and unknown
    // identifiers resolve to PIG/AIR, including Identifier.tryParse returning null.
    let resolve = |value: &Value| {
        let text = match value {
            Value::String(text) => text.clone(),
            Value::Number(number) => number.to_string(),
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
            default.to_owned()
        })
    };
    match value {
        Value::Array(values) => values.iter().map(resolve).collect(),
        _ => Ok(BTreeSet::from([resolve(value)?])),
    }
}

fn ordered_set(values: &BTreeSet<String>) -> Value {
    let mut values: Vec<_> = values.iter().collect();
    values.sort_by(|a, b| {
        let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
        let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
        ap.cmp(bp).then_with(|| an.cmp(bn))
    });
    json!(values)
}

pub fn serialize_no_entity_interact(
    state: &NoEntityInteract,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::NoEntityInteract, common);
    for (name, filter, selected, registry, key, list_key) in [
        (
            "EntityTypeFilter",
            state.entity_type_filter,
            &state.entity_types,
            "entity_type",
            "liquidbounce.module.noEntityInteract.entityTypeFilter",
            "liquidbounce.module.noEntityInteract.entityTypes",
        ),
        (
            "HoldingItemFilter",
            state.holding_item_filter,
            &state.holding_items,
            "item",
            "liquidbounce.module.noEntityInteract.holdingItemFilter",
            "liquidbounce.module.noEntityInteract.holdingItems",
        ),
    ] {
        let mut value = tree.leaf(
            name,
            "CHOOSE",
            json!(match filter {
                Filter::Whitelist => "Whitelist",
                Filter::Blacklist => "Blacklist",
            }),
            key,
            None,
        );
        if format == JsonFormat::Interop {
            value["choices"] = json!(["Whitelist", "Blacklist"]);
        }
        values.push(value);
        let mut list = tree.leaf(
            if registry == "item" {
                "HoldingItems"
            } else {
                "EntityTypes"
            },
            "REGISTRY_LIST",
            ordered_set(selected),
            list_key,
            None,
        );
        if format == JsonFormat::Interop {
            list["registry"] = json!(registry);
        }
        values.push(list);
    }
    Ok(tree.group("NoEntityInteract", "TOGGLEABLE", values))
}

pub(super) fn parse_no_entity_interact(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in [
        "EntityTypeFilter",
        "EntityTypes",
        "HoldingItemFilter",
        "HoldingItems",
    ] {
        leaf(
            patch,
            root,
            name,
            &format!("NoEntityInteract.{name}"),
            |value, path| {
                let setting = match name {
                    "EntityTypes" => NoEntitySetting::EntityTypes(value.clone()),
                    "HoldingItems" => NoEntitySetting::HoldingItems(value.clone()),
                    _ => {
                        let value = value
                            .as_str()
                            .ok_or_else(|| ConfigError::new(path, "expected enum string"))?;
                        let filter = if value.eq_ignore_ascii_case("Whitelist") {
                            Filter::Whitelist
                        } else if value.eq_ignore_ascii_case("Blacklist") {
                            Filter::Blacklist
                        } else {
                            return Err(ConfigError::new(path, "unknown filter"));
                        };
                        if name == "EntityTypeFilter" {
                            NoEntitySetting::EntityTypeFilter(filter)
                        } else {
                            NoEntitySetting::HoldingItemFilter(filter)
                        }
                    }
                };
                Ok(SettingChange::NoEntityInteract(setting))
            },
        );
    }
}
