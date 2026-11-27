// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete original SmartEat settings and declaration order, ae0eb396.
use super::*;
use liquidbounce_behaviors::smart_eat::SmartEat;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SmartEatSetting {
    SwapBackDelay(i32),
    PreferGappleHealthThreshold(f32),
    PreferNotchAppleHealthThreshold(f32),
    PreferHealthPotHealthThreshold(f32),
    CombatPauseTime(i32),
    NotDuringCombat(bool),
    SilentOffhand(bool),
    RenderSlot(bool),
    Offset(i32),
    AutoEat(bool),
    MinHunger(i32),
}
impl SmartEatSetting {
    /// The runtime invokes child onDisabled around enabled-value transitions
    /// only while inGame and the module parent is enabled. Keep lifecycle out
    /// of this value adapter, as a file can also be loaded at the main menu.
    pub fn apply(self, module: &mut SmartEat) {
        match self {
            Self::SwapBackDelay(value) => module.swap_back_delay = value,
            Self::PreferGappleHealthThreshold(value) => module.prefer_gapple_health = value,
            Self::PreferNotchAppleHealthThreshold(value) => module.prefer_notch_health = value,
            Self::PreferHealthPotHealthThreshold(value) => module.prefer_health_pot_health = value,
            Self::CombatPauseTime(value) => module.combat_pause_time = value,
            Self::NotDuringCombat(value) => module.not_during_combat = value,
            Self::SilentOffhand(value) => module.silent_offhand = value,
            Self::RenderSlot(value) => module.render_slot = value,
            Self::Offset(value) => module.render_offset = value,
            Self::AutoEat(value) => module.auto_eat = value,
            Self::MinHunger(value) => module.min_hunger = value,
        }
    }
}
pub fn serialize_smart_eat(
    state: &SmartEat,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let key = "liquidbounce.module.smartEat";
    let boolean = |name: &str, value: bool, path: &str| {
        tree.leaf(
            name,
            "BOOLEAN",
            json!(value),
            &format!("{key}.{path}"),
            None,
        )
    };
    let int = |name: &str, value: i32, from: i32, to: i32, suffix: &str, path: &str| {
        tree.ranged(
            name,
            "INT",
            json!(value),
            json!({"from":from,"to":to}),
            suffix,
            &format!("{key}.{path}"),
        )
    };
    let mut values = tree.common(ModuleKind::SmartEat, common);
    values.push(int(
        "SwapBackDelay",
        state.swap_back_delay,
        1,
        20,
        "",
        "swapBackDelay",
    ));
    for (name, value, path) in [
        (
            "PreferGappleHealthThreshold",
            state.prefer_gapple_health,
            "preferGappleHealthThreshold",
        ),
        (
            "PreferNotchAppleHealthThreshold",
            state.prefer_notch_health,
            "preferNotchAppleHealthThreshold",
        ),
        (
            "PreferHealthPotHealthThreshold",
            state.prefer_health_pot_health,
            "preferHealthPotHealthThreshold",
        ),
    ] {
        values.push(tree.ranged(
            name,
            "FLOAT",
            float_json(value, &format!("SmartEat.{name}"))?,
            json!({"from":0.0,"to":20.0}),
            "",
            &format!("{key}.{path}"),
        ));
    }
    values.push(int(
        "CombatPauseTime",
        state.combat_pause_time,
        0,
        40,
        "ticks",
        "combatPauseTime",
    ));
    values.push(boolean(
        "NotDuringCombat",
        state.not_during_combat,
        "notDuringCombat",
    ));
    values.push(tree.group(
        "SilentOffhand",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", state.silent_offhand, "silentOffhand.enabled"),
            tree.group(
                "RenderSlot",
                "TOGGLEABLE",
                vec![
                    boolean(
                        "Enabled",
                        state.render_slot,
                        "silentOffhand.renderSlot.enabled",
                    ),
                    int(
                        "Offset",
                        state.render_offset,
                        30,
                        70,
                        "",
                        "silentOffhand.renderSlot.offset",
                    ),
                ],
            ),
        ],
    ));
    values.push(tree.group(
        "AutoEat",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", state.auto_eat, "autoEat.enabled"),
            int(
                "MinHunger",
                state.min_hunger,
                0,
                20,
                "",
                "autoEat.minHunger",
            ),
        ],
    ));
    Ok(tree.group("SmartEat", "TOGGLEABLE", values))
}
pub(super) fn parse_smart_eat(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    use SmartEatSetting as S;
    leaf(
        patch,
        root,
        "SwapBackDelay",
        "SmartEat.SwapBackDelay",
        |v, p| parse_int(v, p).map(|v| SettingChange::SmartEat(S::SwapBackDelay(v))),
    );
    for (name, ctor) in [
        (
            "PreferGappleHealthThreshold",
            S::PreferGappleHealthThreshold as fn(f32) -> S,
        ),
        (
            "PreferNotchAppleHealthThreshold",
            S::PreferNotchAppleHealthThreshold,
        ),
        (
            "PreferHealthPotHealthThreshold",
            S::PreferHealthPotHealthThreshold,
        ),
    ] {
        leaf(patch, root, name, &format!("SmartEat.{name}"), |v, p| {
            parse_float(v, p).map(|v| SettingChange::SmartEat(ctor(v)))
        });
    }
    leaf(
        patch,
        root,
        "CombatPauseTime",
        "SmartEat.CombatPauseTime",
        |v, p| parse_int(v, p).map(|v| SettingChange::SmartEat(S::CombatPauseTime(v))),
    );
    leaf(
        patch,
        root,
        "NotDuringCombat",
        "SmartEat.NotDuringCombat",
        |v, p| parse_bool(v, p).map(|v| SettingChange::SmartEat(S::NotDuringCombat(v))),
    );
    if let Some(node) = root.get("SilentOffhand") {
        match nodes(node, "SilentOffhand", "SmartEat.SilentOffhand") {
            Err(error) => patch.issues.push(error),
            Ok(group) => {
                leaf(
                    patch,
                    &group,
                    "Enabled",
                    "SmartEat.SilentOffhand.Enabled",
                    |v, p| parse_bool(v, p).map(|v| SettingChange::SmartEat(S::SilentOffhand(v))),
                );
                if let Some(node) = group.get("RenderSlot") {
                    match nodes(node, "RenderSlot", "SmartEat.SilentOffhand.RenderSlot") {
                        Err(error) => patch.issues.push(error),
                        Ok(group) => {
                            leaf(
                                patch,
                                &group,
                                "Enabled",
                                "SmartEat.SilentOffhand.RenderSlot.Enabled",
                                |v, p| {
                                    parse_bool(v, p)
                                        .map(|v| SettingChange::SmartEat(S::RenderSlot(v)))
                                },
                            );
                            leaf(
                                patch,
                                &group,
                                "Offset",
                                "SmartEat.SilentOffhand.RenderSlot.Offset",
                                |v, p| {
                                    parse_int(v, p).map(|v| SettingChange::SmartEat(S::Offset(v)))
                                },
                            );
                        }
                    }
                }
            }
        }
    }
    if let Some(node) = root.get("AutoEat") {
        match nodes(node, "AutoEat", "SmartEat.AutoEat") {
            Err(error) => patch.issues.push(error),
            Ok(group) => {
                leaf(
                    patch,
                    &group,
                    "Enabled",
                    "SmartEat.AutoEat.Enabled",
                    |v, p| parse_bool(v, p).map(|v| SettingChange::SmartEat(S::AutoEat(v))),
                );
                leaf(
                    patch,
                    &group,
                    "MinHunger",
                    "SmartEat.AutoEat.MinHunger",
                    |v, p| parse_int(v, p).map(|v| SettingChange::SmartEat(S::MinHunger(v))),
                );
            }
        }
    }
}
