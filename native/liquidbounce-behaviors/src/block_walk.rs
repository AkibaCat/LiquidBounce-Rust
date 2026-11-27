// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete pinned ModuleBlockWalk and its registry-list value semantics.
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockWalk {
    pub blocks: BTreeSet<String>,
}
impl Default for BlockWalk {
    fn default() -> Self {
        Self {
            blocks: ["minecraft:cobweb", "minecraft:snow"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }
    }
}
impl BlockWalk {
    pub fn full_block(&self, running: bool, shape_changes_suppressed: bool, block: &str) -> bool {
        running && !shape_changes_suppressed && self.blocks.contains(block)
    }
    /// ListValue resolves the complete replacement before clearing the old
    /// sorted set. Invalid later entries therefore retain the original value.
    pub fn deserialize_blocks(&mut self, value: &Value, registry: &[String]) -> Result<(), String> {
        let resolve = |value: &Value| -> Result<String, String> {
            let token = match value {
                Value::String(token) => token.clone(),
                Value::Number(token) => token.to_string(),
                _ => return Err("block registry entries require string or number tokens".into()),
            };
            let id = match token.find(':') {
                None => format!("minecraft:{token}"),
                Some(0) => format!("minecraft{token}"),
                Some(_) => token,
            };
            Ok(if registry.contains(&id) {
                id
            } else {
                "minecraft:air".to_owned()
            })
        };
        self.blocks = match value {
            Value::Array(values) => values.iter().map(resolve).collect::<Result<_, _>>()?,
            _ => BTreeSet::from([resolve(value)?]),
        };
        Ok(())
    }
    /// Identifier.compareTo compares path first, then namespace.
    pub fn sorted_block_ids(&self) -> Vec<&str> {
        let mut ids: Vec<_> = self.blocks.iter().map(String::as_str).collect();
        ids.sort_by(|a, b| {
            let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
            let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
            ap.cmp(bp).then_with(|| an.cmp(bn))
        });
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn default_blocks_and_original_event_gates() {
        let state = BlockWalk::default();
        assert_eq!(
            state.sorted_block_ids(),
            ["minecraft:cobweb", "minecraft:snow"]
        );
        for running in [false, true] {
            for suppressed in [false, true] {
                assert_eq!(
                    state.full_block(running, suppressed, "minecraft:snow"),
                    running && !suppressed
                );
                assert!(!state.full_block(running, suppressed, "minecraft:stone"));
            }
        }
    }
    #[test]
    fn registry_import_replaces_atomically_and_sorts_by_identifier() {
        let registry = ["minecraft:air", "minecraft:snow", "mod:a", "minecraft:z"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let mut state = BlockWalk::default();
        for bad in [
            json!(["snow", false]),
            json!(["snow", null]),
            json!(["snow", {}]),
            json!([["snow"]]),
        ] {
            let before = state.clone();
            assert!(state.deserialize_blocks(&bad, &registry).is_err());
            assert_eq!(state, before);
        }
        state
            .deserialize_blocks(
                &json!(["snow", ":snow", "missing", 17, "mod:a", "minecraft:z"]),
                &registry,
            )
            .unwrap();
        assert_eq!(
            state.sorted_block_ids(),
            ["mod:a", "minecraft:air", "minecraft:snow", "minecraft:z"]
        );
        state.deserialize_blocks(&json!([]), &registry).unwrap();
        assert!(state.blocks.is_empty());
        state.deserialize_blocks(&json!("snow"), &registry).unwrap();
        assert_eq!(state.sorted_block_ids(), ["minecraft:snow"]);
    }
}
