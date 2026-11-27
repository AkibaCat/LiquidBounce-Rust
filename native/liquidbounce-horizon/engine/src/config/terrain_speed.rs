// SPDX-License-Identifier: GPL-3.0-or-later
//! All TerrainSpeed toggle groups, both climb modes and raw imported floats.
use super::*;
use liquidbounce_behaviors::terrain_speed::{ClimbMode, TerrainSpeed};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TerrainSpeedSetting {
    FastClimbEnabled(bool),
    ClimbMode(ClimbMode),
    ClimbMotion(f32),
    IceEnabled(bool),
    IceSlipperiness(f32),
    IceMotionEnabled(bool),
    IceMotion(f32),
    WaterEnabled(bool),
    AutoSwim(bool),
    BaseHorizontal(f32),
    BaseVertical(f32),
    SprintEnabled(bool),
    SprintHorizontal(f32),
    SprintVertical(f32),
}
impl TerrainSpeedSetting {
    pub fn apply(self, state: &mut TerrainSpeed) {
        match self {
            Self::FastClimbEnabled(v) => state.fast_climb.enabled = v,
            Self::ClimbMode(v) => state.fast_climb.mode = v,
            Self::ClimbMotion(v) => state.fast_climb.motion = v,
            Self::IceEnabled(v) => state.ice_speed.enabled = v,
            Self::IceSlipperiness(v) => state.ice_speed.slipperiness = v,
            Self::IceMotionEnabled(v) => state.ice_speed.motion_enabled = v,
            Self::IceMotion(v) => state.ice_speed.horizontal_motion = v,
            Self::WaterEnabled(v) => state.water_speed.enabled = v,
            Self::AutoSwim(v) => state.water_speed.auto_swim = v,
            Self::BaseHorizontal(v) => state.water_speed.base_horizontal = v,
            Self::BaseVertical(v) => state.water_speed.base_vertical = v,
            Self::SprintEnabled(v) => state.water_speed.sprint_enabled = v,
            Self::SprintHorizontal(v) => state.water_speed.sprint_horizontal = v,
            Self::SprintVertical(v) => state.water_speed.sprint_vertical = v,
        }
    }
}
pub fn serialize_terrain_speed(
    state: &TerrainSpeed,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let key = |suffix: &str| format!("liquidbounce.module.terrainSpeed.{suffix}");
    let boolean = |name: &str, suffix: &str, value: bool| {
        tree.leaf(name, "BOOLEAN", json!(value), &key(suffix), None)
    };
    let float =
        |name: &str, suffix: &str, value: f32, min: f32, max: f32| -> Result<Value, ConfigError> {
            Ok(tree.ranged(
                name,
                "FLOAT",
                float_json(value, &format!("TerrainSpeed.{suffix}"))?,
                json!({"from":min,"to":max}),
                "",
                &key(suffix),
            ))
        };
    let mut choices = Map::new();
    choices.insert(
        "Motion".into(),
        tree.group(
            "Motion",
            "CONFIGURABLE",
            vec![float(
                "Motion",
                "fastClimb.mode.motion.motion",
                state.fast_climb.motion,
                0.1,
                0.5,
            )?],
        ),
    );
    choices.insert("Clip".into(), tree.group("Clip", "CONFIGURABLE", vec![]));
    let climb = tree.group(
        "FastClimb",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", "fastClimb.enabled", state.fast_climb.enabled),
            tree.mode(state.fast_climb.mode.name(), choices),
        ],
    );
    let ice = tree.group(
        "IceSpeed",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", "iceSpeed.enabled", state.ice_speed.enabled),
            float(
                "Slipperiness",
                "iceSpeed.slipperiness",
                state.ice_speed.slipperiness,
                0.3,
                1.,
            )?,
            tree.group(
                "Motion",
                "TOGGLEABLE",
                vec![
                    boolean(
                        "Enabled",
                        "iceSpeed.motion.enabled",
                        state.ice_speed.motion_enabled,
                    ),
                    float(
                        "Motion",
                        "iceSpeed.motion.motion",
                        state.ice_speed.horizontal_motion,
                        0.2,
                        1.5,
                    )?,
                ],
            ),
        ],
    );
    let water = tree.group(
        "WaterSpeed",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", "waterSpeed.enabled", state.water_speed.enabled),
            boolean(
                "AutoSwim",
                "waterSpeed.autoSwim",
                state.water_speed.auto_swim,
            ),
            tree.group(
                "BaseSpeed",
                "CONFIGURABLE",
                vec![
                    float(
                        "Horizontal",
                        "waterSpeed.baseSpeed.horizontal",
                        state.water_speed.base_horizontal,
                        0.1,
                        10.,
                    )?,
                    float(
                        "Vertical",
                        "waterSpeed.baseSpeed.vertical",
                        state.water_speed.base_vertical,
                        0.1,
                        10.,
                    )?,
                ],
            ),
            tree.group(
                "SprintSpeed",
                "TOGGLEABLE",
                vec![
                    boolean(
                        "Enabled",
                        "waterSpeed.sprintSpeed.enabled",
                        state.water_speed.sprint_enabled,
                    ),
                    float(
                        "Horizontal",
                        "waterSpeed.sprintSpeed.horizontal",
                        state.water_speed.sprint_horizontal,
                        0.1,
                        10.,
                    )?,
                    float(
                        "Vertical",
                        "waterSpeed.sprintSpeed.vertical",
                        state.water_speed.sprint_vertical,
                        0.1,
                        10.,
                    )?,
                ],
            ),
        ],
    );
    let mut values = tree.common(ModuleKind::TerrainSpeed, common);
    values.extend([climb, ice, water]);
    Ok(tree.group("TerrainSpeed", "TOGGLEABLE", values))
}
fn child(
    patch: &mut SettingsPatch,
    root: &Nodes<'_>,
    name: &str,
    path: &str,
    parse: impl FnOnce(&mut SettingsPatch, &Nodes<'_>),
) {
    if let Some(value) = root.get(name) {
        match nodes(value, name, path) {
            Ok(root) => parse(patch, &root),
            Err(e) => patch.issues.push(e),
        }
    }
}
fn boolean(
    patch: &mut SettingsPatch,
    root: &Nodes<'_>,
    name: &str,
    path: &str,
    setting: fn(bool) -> TerrainSpeedSetting,
) {
    leaf(patch, root, name, path, |v, p| {
        parse_bool(v, p).map(|v| SettingChange::TerrainSpeed(setting(v)))
    });
}
fn float(
    patch: &mut SettingsPatch,
    root: &Nodes<'_>,
    name: &str,
    path: &str,
    setting: fn(f32) -> TerrainSpeedSetting,
) {
    leaf(patch, root, name, path, |v, p| {
        parse_float(v, p).map(|v| SettingChange::TerrainSpeed(setting(v)))
    });
}
pub(super) fn parse_terrain_speed(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    use TerrainSpeedSetting as S;
    child(
        patch,
        root,
        "FastClimb",
        "TerrainSpeed.FastClimb",
        |patch, root| {
            boolean(
                patch,
                root,
                "Enabled",
                "TerrainSpeed.FastClimb.Enabled",
                S::FastClimbEnabled,
            );
            if let Some(mode) = root.get("Mode") {
                match mode.get("active").and_then(scalar_string).as_deref() {
                    Some("Motion") => patch
                        .changes
                        .push(SettingChange::TerrainSpeed(S::ClimbMode(ClimbMode::Motion))),
                    Some("Clip") => patch
                        .changes
                        .push(SettingChange::TerrainSpeed(S::ClimbMode(ClimbMode::Clip))),
                    _ => patch.issues.push(ConfigError::new(
                        "TerrainSpeed.FastClimb.Mode.active",
                        "missing or unknown case-sensitive mode",
                    )),
                }
                if let Some(choices) = mode.get("choices").and_then(Value::as_object) {
                    if let Some(motion) = choices.get("Motion") {
                        match nodes(motion, "Motion", "TerrainSpeed.FastClimb.Mode.Motion") {
                            Ok(root) => float(
                                patch,
                                &root,
                                "Motion",
                                "TerrainSpeed.FastClimb.Mode.Motion.Motion",
                                S::ClimbMotion,
                            ),
                            Err(e) => patch.issues.push(e),
                        }
                    } else {
                        patch.issues.push(ConfigError::new(
                            "TerrainSpeed.FastClimb.Mode.Motion", "choice not found"));
                    }
                    if let Some(clip) = choices.get("Clip")
                        && let Err(e) = nodes(clip, "Clip", "TerrainSpeed.FastClimb.Mode.Clip")
                    {
                        patch.issues.push(e);
                    }
                    if !choices.contains_key("Clip") {
                        patch.issues.push(ConfigError::new(
                            "TerrainSpeed.FastClimb.Mode.Clip", "choice not found"));
                    }
                } else {
                    patch.issues.push(ConfigError::new(
                        "TerrainSpeed.FastClimb.Mode.choices",
                        "expected choices object",
                    ));
                }
                if let Err(error) = nodes(mode, "Mode", "TerrainSpeed.FastClimb.Mode") {
                    patch.issues.push(error);
                }
            }
        },
    );
    child(
        patch,
        root,
        "IceSpeed",
        "TerrainSpeed.IceSpeed",
        |patch, root| {
            boolean(
                patch,
                root,
                "Enabled",
                "TerrainSpeed.IceSpeed.Enabled",
                S::IceEnabled,
            );
            float(
                patch,
                root,
                "Slipperiness",
                "TerrainSpeed.IceSpeed.Slipperiness",
                S::IceSlipperiness,
            );
            child(
                patch,
                root,
                "Motion",
                "TerrainSpeed.IceSpeed.Motion",
                |patch, root| {
                    boolean(
                        patch,
                        root,
                        "Enabled",
                        "TerrainSpeed.IceSpeed.Motion.Enabled",
                        S::IceMotionEnabled,
                    );
                    float(
                        patch,
                        root,
                        "Motion",
                        "TerrainSpeed.IceSpeed.Motion.Motion",
                        S::IceMotion,
                    );
                },
            );
        },
    );
    child(
        patch,
        root,
        "WaterSpeed",
        "TerrainSpeed.WaterSpeed",
        |patch, root| {
            boolean(
                patch,
                root,
                "Enabled",
                "TerrainSpeed.WaterSpeed.Enabled",
                S::WaterEnabled,
            );
            boolean(
                patch,
                root,
                "AutoSwim",
                "TerrainSpeed.WaterSpeed.AutoSwim",
                S::AutoSwim,
            );
            child(
                patch,
                root,
                "BaseSpeed",
                "TerrainSpeed.WaterSpeed.BaseSpeed",
                |patch, root| {
                    float(
                        patch,
                        root,
                        "Horizontal",
                        "TerrainSpeed.WaterSpeed.BaseSpeed.Horizontal",
                        S::BaseHorizontal,
                    );
                    float(
                        patch,
                        root,
                        "Vertical",
                        "TerrainSpeed.WaterSpeed.BaseSpeed.Vertical",
                        S::BaseVertical,
                    );
                },
            );
            child(
                patch,
                root,
                "SprintSpeed",
                "TerrainSpeed.WaterSpeed.SprintSpeed",
                |patch, root| {
                    boolean(
                        patch,
                        root,
                        "Enabled",
                        "TerrainSpeed.WaterSpeed.SprintSpeed.Enabled",
                        S::SprintEnabled,
                    );
                    float(
                        patch,
                        root,
                        "Horizontal",
                        "TerrainSpeed.WaterSpeed.SprintSpeed.Horizontal",
                        S::SprintHorizontal,
                    );
                    float(
                        patch,
                        root,
                        "Vertical",
                        "TerrainSpeed.WaterSpeed.SprintSpeed.Vertical",
                        S::SprintVertical,
                    );
                },
            );
        },
    );
}
