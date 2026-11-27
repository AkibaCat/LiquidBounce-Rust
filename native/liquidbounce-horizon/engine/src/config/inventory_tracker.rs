// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InventoryTrackerSetting(pub bool);
impl InventoryTrackerSetting {
    pub fn apply(&self, runtime: &mut crate::inventory_tracker_runtime::InventoryTrackerRuntime) {
        runtime.save_players(self.0);
    }
}
pub fn serialize_inventory_tracker(
    save_players: bool,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::InventoryTracker, common);
    values.push(tree.leaf(
        "SavePlayers",
        "BOOLEAN",
        json!(save_players),
        "liquidbounce.module.inventoryTracker.savePlayers",
        None,
    ));
    Ok(tree.group("InventoryTracker", "TOGGLEABLE", values))
}
pub(super) fn parse_inventory_tracker(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "SavePlayers",
        "InventoryTracker.SavePlayers",
        |value, path| {
            parse_bool(value, path)
                .map(|value| SettingChange::InventoryTracker(InventoryTrackerSetting(value)))
        },
    );
}

#[cfg(test)]
mod inventory_tracker_config_tests {
    use super::*;
    #[test]
    fn save_players_roundtrips_in_file_and_interop_trees() {
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let value = serialize_inventory_tracker(
                true,
                &CommonSettings {
                    enabled: false,
                    hidden: false,
                    key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Toggle"}),
                },
                format,
                &str::to_owned,
            )
            .unwrap();
            let patch = parse_settings(ModuleKind::InventoryTracker, &value).unwrap();
            assert!(patch.issues.is_empty());
            let mut runtime = crate::inventory_tracker_runtime::InventoryTrackerRuntime::default();
            for change in patch.changes {
                if let SettingChange::InventoryTracker(change) = change {
                    change.apply(&mut runtime);
                }
            }
            assert!(runtime.model.save_players);
            assert!(runtime.pending_saved_clear);
        }
    }
    #[test]
    fn malformed_save_players_cannot_overwrite_a_valid_setting() {
        let value =
            json!({"name":"InventoryTracker","value":[{"name":"SavePlayers","value":{"invalid":true}}]});
        let patch = parse_settings(ModuleKind::InventoryTracker, &value).unwrap();
        assert_eq!(patch.issues.len(), 1);
        assert!(
            !patch
                .changes
                .iter()
                .any(|change| matches!(change, SettingChange::InventoryTracker(_)))
        );
        // JsonPrimitive.getAsBoolean follows Boolean.parseBoolean for strings.
        let value = json!({"name":"InventoryTracker","value":[{"name":"SavePlayers","value":"malformed"}]});
        let patch = parse_settings(ModuleKind::InventoryTracker, &value).unwrap();
        assert!(patch.issues.is_empty());
        assert!(patch.changes.iter().any(|change| matches!(change, SettingChange::InventoryTracker(InventoryTrackerSetting(false)))));
    }
}
