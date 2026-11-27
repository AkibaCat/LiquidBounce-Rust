// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::old_regen::{CONDITIONS, OldRegen, PACKET_TYPES, packet_type};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OldRegenSetting {
    HealthEnabled(bool),
    Health(i32),
    Increment(i32),
    BadEffects(bool),
    Fire(bool),
    MaximumSpeed(i32),
    Timer(f32),
    Conditions(u32),
    PacketType(u8),
}
impl OldRegenSetting {
    pub fn apply(self, module: &mut OldRegen) {
        match self {
            Self::HealthEnabled(v) => module.health_enabled = v,
            Self::Health(v) => module.health = v,
            Self::Increment(v) => module.increment = v,
            Self::BadEffects(v) => module.bad_effects = v,
            Self::Fire(v) => module.fire = v,
            Self::MaximumSpeed(v) => module.maximum_speed = v,
            Self::Timer(v) => module.timer = v,
            Self::Conditions(v) => module.conditions = v,
            Self::PacketType(v) => module.packet_type = v,
        }
    }
}
pub fn serialize_old_regen(
    module: &OldRegen,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::OldRegen, common);
    let mut health = tree.group(
        "Health",
        "TOGGLEABLE",
        vec![
            tree.leaf(
                "Enabled",
                "BOOLEAN",
                json!(module.health_enabled),
                "liquidbounce.module.oldRegen.health.enabled",
                None,
            ),
            tree.ranged(
                "Health",
                "INT",
                json!(module.health),
                json!({"from":0,"to":20}),
                "",
                "liquidbounce.module.oldRegen.health.health",
            ),
            tree.ranged(
                "Increment",
                "INT",
                json!(module.increment),
                json!({"from":0,"to":60}),
                "",
                "liquidbounce.module.oldRegen.health.increment",
            ),
        ],
    );
    if format == JsonFormat::Interop {
        health["key"] = json!("liquidbounce.module.oldRegen.health");
        health["description"] = json!(describe("liquidbounce.module.oldRegen.health.description"));
    }
    values.push(health);
    for (name, key, v) in [
        ("BadEffects", "badEffects", module.bad_effects),
        ("Fire", "fire", module.fire),
    ] {
        values.push(tree.leaf(
            name,
            "BOOLEAN",
            json!(v),
            &format!("liquidbounce.module.oldRegen.{key}"),
            None,
        ));
    }
    values.push(tree.ranged(
        "MaximumSpeed",
        "INT",
        json!(module.maximum_speed),
        json!({"from":5,"to":200}),
        "",
        "liquidbounce.module.oldRegen.maximumSpeed",
    ));
    values.push(tree.ranged(
        "Timer",
        "FLOAT",
        float_json(module.timer, "OldRegen.Timer")?,
        json!({"from":0.1,"to":10.0}),
        "",
        "liquidbounce.module.oldRegen.timer",
    ));
    values.push(tree.multi(
        "Conditions",
        module.conditions,
        &CONDITIONS,
        "liquidbounce.module.oldRegen.conditions",
    ));
    let mut kind = tree.leaf(
        "PacketType",
        "CHOOSE",
        json!(PACKET_TYPES[module.packet_type as usize]),
        "liquidbounce.module.oldRegen.packetType",
        None,
    );
    if format == JsonFormat::Interop {
        kind["choices"] = json!(PACKET_TYPES);
    }
    values.push(kind);
    Ok(tree.group("OldRegen", "TOGGLEABLE", values))
}
pub(super) fn parse_old_regen(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(node) = root.get("Health") {
        match nodes(node, "Health", "OldRegen.Health") {
            Ok(children) => {
                leaf(
                    patch,
                    &children,
                    "Enabled",
                    "OldRegen.Health.Enabled",
                    |v, p| {
                        parse_bool(v, p)
                            .map(|v| SettingChange::OldRegen(OldRegenSetting::HealthEnabled(v)))
                    },
                );
                leaf(
                    patch,
                    &children,
                    "Health",
                    "OldRegen.Health.Health",
                    |v, p| {
                        parse_int(v, p).map(|v| SettingChange::OldRegen(OldRegenSetting::Health(v)))
                    },
                );
                leaf(
                    patch,
                    &children,
                    "Increment",
                    "OldRegen.Health.Increment",
                    |v, p| {
                        parse_int(v, p)
                            .map(|v| SettingChange::OldRegen(OldRegenSetting::Increment(v)))
                    },
                );
            }
            Err(error) => patch.issues.push(error),
        }
    }
    leaf(patch, root, "BadEffects", "OldRegen.BadEffects", |v, p| {
        parse_bool(v, p).map(|v| SettingChange::OldRegen(OldRegenSetting::BadEffects(v)))
    });
    leaf(patch, root, "Fire", "OldRegen.Fire", |v, p| {
        parse_bool(v, p).map(|v| SettingChange::OldRegen(OldRegenSetting::Fire(v)))
    });
    leaf(
        patch,
        root,
        "MaximumSpeed",
        "OldRegen.MaximumSpeed",
        |v, p| parse_int(v, p).map(|v| SettingChange::OldRegen(OldRegenSetting::MaximumSpeed(v))),
    );
    leaf(patch, root, "Timer", "OldRegen.Timer", |v, p| {
        parse_float(v, p).map(|v| SettingChange::OldRegen(OldRegenSetting::Timer(v)))
    });
    leaf(patch, root, "Conditions", "OldRegen.Conditions", |v, p| {
        parse_multi_mask(v, p, &CONDITIONS)
            .map(|v| SettingChange::OldRegen(OldRegenSetting::Conditions(v)))
    });
    leaf(patch, root, "PacketType", "OldRegen.PacketType", |v, p| {
        scalar_string(v)
            .and_then(|v| packet_type(&v))
            .map(|v| SettingChange::OldRegen(OldRegenSetting::PacketType(v)))
            .ok_or_else(|| ConfigError::new(p, "unknown packet type"))
    });
}
