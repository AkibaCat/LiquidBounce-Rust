// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual GlobalSettingsInventory values, consumed by the shared manager.
use super::*;
use liquidbounce_behaviors::inventory_manager::Manager;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlobalInventorySetting {
    ResponseTime(i32),
    LockAfterRejections(i32),
}
impl GlobalInventorySetting {
    pub fn name(self) -> &'static str {
        match self {
            Self::ResponseTime(_) => "ResponseTime",
            Self::LockAfterRejections(_) => "LockAfterRejections",
        }
    }
    pub fn apply(self, manager: &mut Manager) {
        // Gson's Value serializer writes IntValue.inner directly. Ranges are
        // editor metadata and do not clamp imported values.
        match self {
            Self::ResponseTime(value) => manager.response_time = value,
            Self::LockAfterRejections(value) => manager.lock_after_rejections = value,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlobalInventoryPatch {
    pub changes: Vec<GlobalInventorySetting>,
    pub issues: Vec<ConfigError>,
}

/// Returns the Inventory child group to append to the Settings root.
pub fn serialize_global_inventory(
    manager: &Manager,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Value {
    let tree = Tree { format, describe };
    tree.group(
        "Inventory",
        "CONFIGURABLE",
        vec![
            tree.ranged(
                "ResponseTime",
                "INT",
                json!(manager.response_time),
                json!([50, 2000]),
                "ms",
                "liquidbounce.settings.inventory.responseTime",
            ),
            tree.ranged(
                "LockAfterRejections",
                "INT",
                json!(manager.lock_after_rejections),
                json!([1, 10]),
                "",
                "liquidbounce.settings.inventory.lockAfterRejections",
            ),
        ],
    )
}

pub fn parse_global_inventory(value: &Value) -> Result<GlobalInventoryPatch, ConfigError> {
    let root = nodes(value, "Settings", "Settings")?;
    let mut patch = GlobalInventoryPatch::default();
    let Some(node) = root.get("Inventory") else {
        return Ok(patch);
    };
    let children = match nodes(node, "Inventory", "Settings.Inventory") {
        Ok(children) => children,
        Err(error) => {
            patch.issues.push(error);
            return Ok(patch);
        }
    };
    for name in ["ResponseTime", "LockAfterRejections"] {
        let Some(node) = children.get(name) else {
            continue;
        };
        let path = format!("Settings.Inventory.{name}");
        let value = node
            .get("value")
            .ok_or_else(|| ConfigError::new(&path, "missing value"))
            .and_then(|value| parse_int(value, &path));
        match value {
            Ok(value) => patch.changes.push(if name == "ResponseTime" {
                GlobalInventorySetting::ResponseTime(value)
            } else {
                GlobalInventorySetting::LockAfterRejections(value)
            }),
            Err(error) => patch.issues.push(error),
        }
    }
    Ok(patch)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_inventory_defaults_editor_metadata_and_file_roundtrip() {
        let mut manager = Manager::default();
        let group = serialize_global_inventory(&manager, JsonFormat::Interop, &str::to_owned);
        assert_eq!(group["name"], "Inventory");
        assert_eq!(group["value"][0]["name"], "ResponseTime");
        assert_eq!(group["value"][0]["value"], 400);
        assert_eq!(group["value"][0]["range"], json!([50, 2000]));
        assert_eq!(group["value"][0]["suffix"], "ms");
        assert_eq!(group["value"][1]["value"], 3);
        assert_eq!(group["value"][1]["range"], json!([1, 10]));
        GlobalInventorySetting::ResponseTime(123).apply(&mut manager);
        GlobalInventorySetting::LockAfterRejections(2).apply(&mut manager);
        let group = serialize_global_inventory(&manager, JsonFormat::File, &|_| panic!());
        assert!(group["value"][0].get("range").is_none());
        let patch = parse_global_inventory(&json!({"name":"Settings","value":[group]})).unwrap();
        assert!(patch.issues.is_empty());
        let mut restored = Manager::default();
        for change in patch.changes {
            change.apply(&mut restored);
        }
        assert_eq!(
            (restored.response_time, restored.lock_after_rejections),
            (123, 2)
        );
    }
    #[test]
    fn bad_leaf_does_not_skip_sibling_and_imports_keep_raw_gson_int() {
        let patch = parse_global_inventory(&json!({"name":"Settings","value":[
        {"name":"Inventory","value":[
            {"name":"ResponseTime","value":null},
            {"name":"LockAfterRejections","value":"-5"}
        ]}]}))
        .unwrap();
        assert_eq!(patch.issues.len(), 1);
        let mut manager = Manager::default();
        for change in patch.changes {
            change.apply(&mut manager);
        }
        assert_eq!(manager.response_time, 400);
        assert_eq!(manager.lock_after_rejections, -5);
        let patch = parse_global_inventory(&json!({"name":"Settings","value":[
        {"name":"Inventory","value":[
            {"name":"ResponseTime","value":2147483648u64},
            {"name":"ResponseTime","value":100},
            {"name":"LockAfterRejections","value":7}
        ]}]}))
        .unwrap();
        for change in patch.changes {
            change.apply(&mut manager);
        }
        assert_eq!(manager.response_time, i32::MIN);
        assert_eq!(manager.lock_after_rejections, 7);
    }
    #[test]
    fn configured_response_time_is_used_by_actual_pending_timers() {
        let mut manager = Manager::default();
        GlobalInventorySetting::ResponseTime(73).apply(&mut manager);
        manager.may_open_container(10);
        manager.item_used(15);
        assert!(manager.container_open_pending(82));
        assert!(!manager.container_open_pending(83));
        assert!(manager.use_item_pending(87));
        assert!(!manager.use_item_pending(88));
        assert!(
            parse_global_inventory(&json!({"name":"Settings","value":[
            {"name":"Inventory","value":null}]}))
            .unwrap()
            .issues
            .len()
                == 1
        );
        assert!(
            parse_global_inventory(&json!({"name":"Settings","value":[
            {"name":"inventory","value":[]}]}))
            .unwrap()
            .changes
            .is_empty()
        );
    }
}
