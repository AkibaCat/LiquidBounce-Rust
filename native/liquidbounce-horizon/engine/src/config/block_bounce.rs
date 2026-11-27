// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete original BlockBounce settings tree.
use super::*;
use liquidbounce_behaviors::block_bounce::BlockBounce;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockBounceSetting(pub f32);
impl BlockBounceSetting {
    pub fn apply(self, state: &mut BlockBounce) {
        state.motion = self.0;
    }
}
pub fn serialize_block_bounce(
    state: &BlockBounce,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::BlockBounce, common);
    values.push(tree.ranged(
        "Motion",
        "FLOAT",
        float_json(state.motion, "BlockBounce.Motion")?,
        json!({"from":0.2,"to":2.0}),
        "",
        "liquidbounce.module.blockBounce.motion",
    ));
    Ok(tree.group("BlockBounce", "TOGGLEABLE", values))
}
pub(super) fn parse_block_bounce(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "Motion",
        "BlockBounce.Motion",
        |value, path| {
            parse_float(value, path)
                .map(|value| SettingChange::BlockBounce(BlockBounceSetting(value)))
        },
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        }
    }
    #[test]
    fn original_motion_metadata_and_unclamped_finite_imports_roundtrip() {
        for motion in [-5.0, 0.0, 0.42, 2.0, 50.0] {
            let state = BlockBounce { motion };
            for format in [JsonFormat::Interop, JsonFormat::File] {
                let value =
                    serialize_block_bounce(&state, &common(), format, &str::to_owned).unwrap();
                let leaf = value["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["name"] == "Motion")
                    .unwrap();
                if format == JsonFormat::Interop {
                    assert_eq!(leaf["valueType"], "FLOAT");
                    assert_eq!(leaf["range"], json!({"from":0.2,"to":2.0}));
                }
                let patch = parse_settings(ModuleKind::BlockBounce, &value).unwrap();
                assert!(patch.issues.is_empty());
                let mut restored = BlockBounce::default();
                for change in patch.changes {
                    if let SettingChange::BlockBounce(value) = change {
                        value.apply(&mut restored);
                    }
                }
                assert_eq!(restored, state);
            }
        }
        assert!(
            serialize_block_bounce(
                &BlockBounce {
                    motion: f32::INFINITY
                },
                &common(),
                JsonFormat::File,
                &str::to_owned
            )
            .is_err()
        );
    }
    #[test]
    fn invalid_motion_is_reported_without_rejecting_common_settings() {
        let value = json!({"name":"BlockBounce","value":[{"name":"Enabled","value":true},
            {"name":"Motion","value":"bad"}]});
        let patch = parse_settings(ModuleKind::BlockBounce, &value).unwrap();
        assert_eq!(patch.issues.len(), 1);
        assert!(!patch.changes.is_empty());
    }
}
