// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete original BlockWalk registry-list tree.
use super::*;
use liquidbounce_behaviors::block_walk::BlockWalk;

#[derive(Clone, Debug, PartialEq)]
pub enum BlockWalkSetting {
    Blocks(Value),
}
impl BlockWalkSetting {
    pub fn apply(self, module: &mut BlockWalk, registry: &[String]) -> Result<(), ConfigError> {
        match self {
            Self::Blocks(value) => module
                .deserialize_blocks(&value, registry)
                .map_err(|error| ConfigError::new("BlockWalk.Blocks", error)),
        }
    }
}
pub fn serialize_block_walk(
    state: &BlockWalk,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::BlockWalk, common);
    let mut blocks = tree.leaf(
        "Blocks",
        "REGISTRY_LIST",
        json!(state.sorted_block_ids()),
        "liquidbounce.module.blockWalk.blocks",
        None,
    );
    if format == JsonFormat::Interop {
        blocks["registry"] = json!("block");
    }
    values.push(blocks);
    Ok(tree.group("BlockWalk", "TOGGLEABLE", values))
}
pub(super) fn parse_block_walk(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Blocks", "BlockWalk.Blocks", |value, _| {
        Ok(SettingChange::BlockWalk(BlockWalkSetting::Blocks(
            value.clone(),
        )))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_tree_roundtrips_both_formats_and_failed_import_is_transactional() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        let registry = ["minecraft:air", "minecraft:cobweb", "minecraft:snow"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let source = BlockWalk::default();
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let value = serialize_block_walk(&source, &common, format, &str::to_owned).unwrap();
            let blocks = value["value"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["name"] == "Blocks")
                .unwrap();
            if format == JsonFormat::Interop {
                assert_eq!(blocks["registry"], "block");
            }
            let patch = parse_settings(ModuleKind::BlockWalk, &value).unwrap();
            assert!(patch.issues.is_empty());
            let mut restored = BlockWalk::default();
            restored.blocks.clear();
            for change in patch.changes {
                if let SettingChange::BlockWalk(value) = change {
                    value.apply(&mut restored, &registry).unwrap();
                }
            }
            assert_eq!(restored, source);
        }
        let mut state = source.clone();
        assert!(
            BlockWalkSetting::Blocks(json!(["snow", false]))
                .apply(&mut state, &registry)
                .is_err()
        );
        assert_eq!(state, source);
    }
}
