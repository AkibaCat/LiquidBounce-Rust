// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::middle_click_action::{MODES, MiddleClickAction, Mode};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MiddleClickActionSetting {
    Mode(Mode),
    PickUpRange(f32),
    SlotResetDelay(i32),
    StopOnSubmit([f32; 2]),
}
impl MiddleClickActionSetting {
    pub fn apply(self, s: &mut MiddleClickAction) {
        match self {
            Self::Mode(v) => s.set_mode(v),
            Self::PickUpRange(v) => s.pick_up_range = v,
            Self::SlotResetDelay(v) => s.slot_reset_delay = v,
            Self::StopOnSubmit(v) => s.stop_on_submit = v,
        }
    }
}
pub fn serialize_middle_click_action(
    s: &MiddleClickAction,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::MiddleClickAction, common);
    let mut choices = Map::new();
    choices.insert(
        MODES[0].into(),
        tree.group(
            MODES[0],
            "CONFIGURABLE",
            vec![tree.ranged(
                "PickUpRange",
                "FLOAT",
                float_json(
                    s.pick_up_range,
                    "MiddleClickAction.Mode.FriendClicker.PickUpRange",
                )?,
                json!({"from":1.0,"to":100.0}),
                "",
                "liquidbounce.module.middleClickAction.mode.friendClicker.pickUpRange",
            )],
        ),
    );
    choices.insert(
        MODES[1].into(),
        tree.group(
            MODES[1],
            "CONFIGURABLE",
            vec![
                tree.ranged(
                    "SlotResetDelay",
                    "INT",
                    json!(s.slot_reset_delay),
                    json!({"from":0,"to":10}),
                    "ticks",
                    "liquidbounce.module.middleClickAction.mode.pearl.slotResetDelay",
                ),
                tree.ranged(
                    "StopOnSubmit",
                    "FLOAT_RANGE",
                    float_range(
                        s.stop_on_submit[0],
                        s.stop_on_submit[1],
                        "MiddleClickAction.Mode.Pearl.StopOnSubmit",
                    )?,
                    json!({"from":60.0,"to":90.0}),
                    "Pitch",
                    "liquidbounce.module.middleClickAction.mode.pearl.stopOnSubmit",
                ),
            ],
        ),
    );
    values.push(tree.mode(s.mode.tag(), choices));
    Ok(tree.group("MiddleClickAction", "TOGGLEABLE", values))
}
pub(super) fn parse_middle_click_action(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Mode") else {
        return;
    };
    match mode
        .get("active")
        .and_then(scalar_string)
        .and_then(|v| Mode::parse(&v))
    {
        Some(v) => patch.changes.push(SettingChange::MiddleClickAction(
            MiddleClickActionSetting::Mode(v),
        )),
        None => patch.issues.push(ConfigError::new(
            "MiddleClickAction.Mode.active",
            "unknown mode",
        )),
    }
    if let Err(e) = nodes(mode, "Mode", "MiddleClickAction.Mode") {
        patch.issues.push(e);
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch.issues.push(ConfigError::new(
            "MiddleClickAction.Mode.choices",
            "expected choices object",
        ));
        return;
    };
    for tag in MODES {
        let path = format!("MiddleClickAction.Mode.{tag}");
        match choices.get(tag) {
            Some(v) => match nodes(v, tag, &path) {
                Ok(v) if tag == MODES[0] => leaf(
                    patch,
                    &v,
                    "PickUpRange",
                    &format!("{path}.PickUpRange"),
                    |v, p| {
                        parse_float(v, p).map(|v| {
                            SettingChange::MiddleClickAction(MiddleClickActionSetting::PickUpRange(
                                v,
                            ))
                        })
                    },
                ),
                Ok(v) => {
                    leaf(
                        patch,
                        &v,
                        "SlotResetDelay",
                        &format!("{path}.SlotResetDelay"),
                        |v, p| {
                            parse_int(v, p).map(|v| {
                                SettingChange::MiddleClickAction(
                                    MiddleClickActionSetting::SlotResetDelay(v),
                                )
                            })
                        },
                    );
                    leaf(
                        patch,
                        &v,
                        "StopOnSubmit",
                        &format!("{path}.StopOnSubmit"),
                        |v, p| {
                            parse_float_pair(v, p).map(|v| {
                                SettingChange::MiddleClickAction(
                                    MiddleClickActionSetting::StopOnSubmit(v),
                                )
                            })
                        },
                    );
                }
                Err(e) => patch.issues.push(e),
            },
            None => patch
                .issues
                .push(ConfigError::new(&path, "choice not found")),
        }
    }
}
