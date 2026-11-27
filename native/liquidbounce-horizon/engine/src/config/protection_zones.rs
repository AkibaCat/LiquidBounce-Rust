// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::protection_zones::ProtectionZones;
#[derive(Clone, Debug, PartialEq)]
pub enum ProtectionZonesSetting {
    ProtectionBlocks(Vec<String>),
    RadiusX(i32),
    RadiusZ(i32),
    RadiusY(i32),
    RenderLimit(i32),
    HoldBlockToRender(bool),
    SnapY(bool),
    ZoneFill(i32),
    ZoneOutline(i32),
    CenterZoneOutline(i32),
    IndicatorOutline(i32),
    IndicatorFill(i32),
}
impl ProtectionZonesSetting {
    pub fn apply(self, s: &mut ProtectionZones) {
        match self {
            Self::ProtectionBlocks(v) => s.protection_blocks = v,
            Self::RadiusX(v) => s.radius_x = v,
            Self::RadiusZ(v) => s.radius_z = v,
            Self::RadiusY(v) => s.radius_y = v,
            Self::RenderLimit(v) => s.render_limit = v,
            Self::HoldBlockToRender(v) => s.hold_block_to_render = v,
            Self::SnapY(v) => s.snap_y = v,
            Self::ZoneFill(v) => s.zone_fill = v,
            Self::ZoneOutline(v) => s.zone_outline = v,
            Self::CenterZoneOutline(v) => s.center_zone_outline = v,
            Self::IndicatorOutline(v) => s.indicator_outline = v,
            Self::IndicatorFill(v) => s.indicator_fill = v,
        }
    }
}
fn key(path: &str) -> String {
    let mut k = "liquidbounce.module.protectionZones".to_owned();
    for p in path.split('.') {
        let mut c = p.chars();
        k.push('.');
        if let Some(first) = c.next() {
            k.extend(first.to_lowercase());
            k.extend(c);
        }
    }
    k
}
fn group(t: &Tree, name: &str, values: Vec<Value>) -> Value {
    t.group(name, "CONFIGURABLE", values)
}
pub fn serialize_protection_zones(
    s: &ProtectionZones,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::ProtectionZones, common);
    let mut blocks = t.leaf(
        "ProtectionBlocks",
        "REGISTRY_LIST",
        json!(s.protection_blocks),
        &key("ProtectionBlocks"),
        None,
    );
    if format == JsonFormat::Interop {
        blocks["innerValueType"] = json!("BLOCK");
        blocks["registry"] = json!("block");
    }
    values.push(blocks);
    values.push(group(
        &t,
        "ProtectionRadius",
        [
            ("RadiusX", s.radius_x, 256),
            ("RadiusZ", s.radius_z, 256),
            ("RadiusY", s.radius_y, 383),
        ]
        .map(|(n, v, max)| {
            t.ranged(
                n,
                "INT",
                json!(v),
                json!({"from":1,"to":max}),
                "blocks",
                &key(&format!("ProtectionRadius.{n}")),
            )
        })
        .into(),
    ));
    values.push(group(
        &t,
        "PlacementIndicator",
        vec![t.leaf(
            "SnapToY",
            "BOOLEAN",
            json!(s.snap_y),
            &key("PlacementIndicator.SnapToY"),
            None,
        )],
    ));
    let protection = group(
        &t,
        "ProtectionColors",
        [
            ("ZoneFill", s.zone_fill),
            ("ZoneOutline", s.zone_outline),
            ("CenterZoneOutline", s.center_zone_outline),
        ]
        .map(|(n, v)| {
            t.leaf(
                n,
                "COLOR",
                json!(v),
                &key(&format!("Renderer.ProtectionColors.{n}")),
                None,
            )
        })
        .into(),
    );
    let indicator = group(
        &t,
        "IndicatorColors",
        [
            ("IndicatorOutline", s.indicator_outline),
            ("IndicatorFill", s.indicator_fill),
        ]
        .map(|(n, v)| {
            t.leaf(
                n,
                "COLOR",
                json!(v),
                &key(&format!("Renderer.IndicatorColors.{n}")),
                None,
            )
        })
        .into(),
    );
    values.push(group(
        &t,
        "Renderer",
        vec![
            t.ranged(
                "RenderLimit",
                "INT",
                json!(s.render_limit),
                json!({"from":3,"to":50}),
                "zones",
                &key("Renderer.RenderLimit"),
            ),
            t.leaf(
                "HoldBlockToRender",
                "BOOLEAN",
                json!(s.hold_block_to_render),
                &key("Renderer.HoldBlockToRender"),
                None,
            ),
            protection,
            indicator,
        ],
    ));
    Ok(t.group("ProtectionZones", "TOGGLEABLE", values))
}
fn change(v: ProtectionZonesSetting) -> SettingChange {
    SettingChange::ProtectionZones(v)
}
fn color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    match v {
        Value::Number(n) => {
            decimal_int(&n.to_string()).ok_or_else(|| ConfigError::new(p, "expected ARGB integer"))
        }
        Value::String(s) => {
            let h = s.strip_prefix('#').unwrap_or(s);
            if !matches!(h.len(), 6 | 8) {
                return Err(ConfigError::new(p, "expected 6/8 hex digits"));
            }
            i64::from_str_radix(h, 16)
                .map(|v| {
                    if h.len() == 6 {
                        v as i32 | 0xff000000u32 as i32
                    } else {
                        v as i32
                    }
                })
                .map_err(|_| ConfigError::new(p, "invalid hex color"))
        }
        _ => Err(ConfigError::new(p, "expected ARGB color")),
    }
}
pub(super) fn parse_protection_zones(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        patch,
        root,
        "ProtectionBlocks",
        "ProtectionZones.ProtectionBlocks",
        |v, p| {
            let Some(values) = v.as_array() else {
                return Err(ConfigError::new(p, "expected block registry list"));
            };
            let mut blocks = Vec::new();
            for value in values {
                let Some(block) = value.as_str() else {
                    return Err(ConfigError::new(p, "expected block identifier"));
                };
                let id = if block.contains(':') {
                    block.to_owned()
                } else {
                    format!("minecraft:{block}")
                };
                if !blocks.contains(&id) {
                    blocks.push(id);
                }
            }
            Ok(change(ProtectionZonesSetting::ProtectionBlocks(blocks)))
        },
    );
    if let Some(n) = root.get("ProtectionRadius") {
        match nodes(n, "ProtectionRadius", "ProtectionZones.ProtectionRadius") {
            Ok(g) => {
                for (name, set) in [
                    (
                        "RadiusX",
                        ProtectionZonesSetting::RadiusX as fn(i32) -> ProtectionZonesSetting,
                    ),
                    ("RadiusZ", ProtectionZonesSetting::RadiusZ),
                    ("RadiusY", ProtectionZonesSetting::RadiusY),
                ] {
                    leaf(
                        patch,
                        &g,
                        name,
                        &format!("ProtectionZones.ProtectionRadius.{name}"),
                        |v, p| parse_int(v, p).map(|v| change(set(v))),
                    );
                }
            }
            Err(e) => patch.issues.push(e),
        }
    }
    if let Some(n) = root.get("PlacementIndicator") {
        match nodes(
            n,
            "PlacementIndicator",
            "ProtectionZones.PlacementIndicator",
        ) {
            Ok(g) => leaf(
                patch,
                &g,
                "SnapToY",
                "ProtectionZones.PlacementIndicator.SnapToY",
                |v, p| parse_bool(v, p).map(|v| change(ProtectionZonesSetting::SnapY(v))),
            ),
            Err(e) => patch.issues.push(e),
        }
    }
    if let Some(n) = root.get("Renderer") {
        match nodes(n, "Renderer", "ProtectionZones.Renderer") {
            Ok(g) => {
                leaf(
                    patch,
                    &g,
                    "RenderLimit",
                    "ProtectionZones.Renderer.RenderLimit",
                    |v, p| parse_int(v, p).map(|v| change(ProtectionZonesSetting::RenderLimit(v))),
                );
                leaf(
                    patch,
                    &g,
                    "HoldBlockToRender",
                    "ProtectionZones.Renderer.HoldBlockToRender",
                    |v, p| {
                        parse_bool(v, p)
                            .map(|v| change(ProtectionZonesSetting::HoldBlockToRender(v)))
                    },
                );
                for (group_name, settings) in [
                    (
                        "ProtectionColors",
                        vec![
                            (
                                "ZoneFill",
                                ProtectionZonesSetting::ZoneFill
                                    as fn(i32) -> ProtectionZonesSetting,
                            ),
                            ("ZoneOutline", ProtectionZonesSetting::ZoneOutline),
                            (
                                "CenterZoneOutline",
                                ProtectionZonesSetting::CenterZoneOutline,
                            ),
                        ],
                    ),
                    (
                        "IndicatorColors",
                        vec![
                            (
                                "IndicatorOutline",
                                ProtectionZonesSetting::IndicatorOutline
                                    as fn(i32) -> ProtectionZonesSetting,
                            ),
                            ("IndicatorFill", ProtectionZonesSetting::IndicatorFill),
                        ],
                    ),
                ] {
                    if let Some(n) = g.get(group_name) {
                        match nodes(
                            n,
                            group_name,
                            &format!("ProtectionZones.Renderer.{group_name}"),
                        ) {
                            Ok(colors) => {
                                for (name, set) in settings {
                                    leaf(
                                        patch,
                                        &colors,
                                        name,
                                        &format!("ProtectionZones.Renderer.{group_name}.{name}"),
                                        |v, p| color(v, p).map(|v| change(set(v))),
                                    );
                                }
                            }
                            Err(e) => patch.issues.push(e),
                        }
                    }
                }
            }
            Err(e) => patch.issues.push(e),
        }
    }
}
