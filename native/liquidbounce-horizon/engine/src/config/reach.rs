// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleReach/RangeValueGroup declaration order and setting-time range clamp.
use super::*;
use liquidbounce_behaviors::reach::Reach;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReachSetting {
    RangeIncrease(f32),
    ThroughWallsRange(f32),
    BlockRangeIncrease(f32),
}

impl ReachSetting {
    pub fn apply(self, module: &mut Reach, raw_entity_attribute: Option<f64>) {
        match self {
            Self::RangeIncrease(value) => module.range_increase = value,
            Self::ThroughWallsRange(value) => {
                // Value.set skips onChange for the same boxed Float, with
                // equal NaNs and distinct signed zero.
                if !float_equal(module.through_walls_range, value) {
                    module.change_through_walls_range(value, raw_entity_attribute);
                }
            }
            Self::BlockRangeIncrease(value) => module.block_range_increase = value,
        }
    }
}

pub fn serialize_reach(
    state: &Reach,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::Reach, common);
    values.push(tree.group(
        "Entity",
        "CONFIGURABLE",
        vec![
            tree.ranged(
                "RangeIncrease",
                "FLOAT",
                float_json(state.range_increase, "Reach.Entity.RangeIncrease")?,
                json!({"from":0.0,"to":5.0}),
                "blocks",
                "liquidbounce.module.reach.entity.rangeIncrease",
            ),
            tree.ranged(
                "ThroughWallsRange",
                "FLOAT",
                float_json(state.through_walls_range, "Reach.Entity.ThroughWallsRange")?,
                json!({"from":0.0,"to":8.0}),
                "blocks",
                "liquidbounce.module.reach.entity.throughWallsRange",
            ),
        ],
    ));
    values.push(tree.ranged(
        "BlockRangeIncrease",
        "FLOAT",
        float_json(state.block_range_increase, "Reach.BlockRangeIncrease")?,
        json!({"from":0.0,"to":64.0}),
        "",
        "liquidbounce.module.reach.blockRangeIncrease",
    ));
    Ok(tree.group("Reach", "TOGGLEABLE", values))
}

pub(super) fn parse_reach(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(entity) = root.get("Entity") {
        match nodes(entity, "Entity", "Reach.Entity") {
            Ok(entity) => {
                leaf(
                    patch,
                    &entity,
                    "RangeIncrease",
                    "Reach.Entity.RangeIncrease",
                    |value, path| {
                        parse_float(value, path)
                            .map(|value| SettingChange::Reach(ReachSetting::RangeIncrease(value)))
                    },
                );
                leaf(
                    patch,
                    &entity,
                    "ThroughWallsRange",
                    "Reach.Entity.ThroughWallsRange",
                    |value, path| {
                        parse_float(value, path).map(|value| {
                            SettingChange::Reach(ReachSetting::ThroughWallsRange(value))
                        })
                    },
                );
            }
            Err(error) => patch.issues.push(error),
        }
    }
    leaf(
        patch,
        root,
        "BlockRangeIncrease",
        "Reach.BlockRangeIncrease",
        |value, path| {
            parse_float(value, path)
                .map(|value| SettingChange::Reach(ReachSetting::BlockRangeIncrease(value)))
        },
    );
}
