// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleHud settings, including theme-provided component/value trees.
use super::*;
use crate::hud_runtime::HudRuntime;
#[derive(Clone, Debug, PartialEq)]
pub enum HudSetting {
    Blur(bool),
    Sigma(f32),
    Alpha([f32; 2]),
    Names(bool),
    Themes(Value),
    Native(Value),
    Fps(i32),
}
impl HudSetting {
    pub fn apply(self, s: &mut HudRuntime) -> Result<(), ConfigError> {
        match self {
            Self::Blur(v) => s.behavior.blur = v,
            Self::Sigma(v) => s.behavior.sigma = v,
            Self::Alpha(v) => s.behavior.alpha_blend_range = v,
            Self::Names(v) => s.names_changed(v),
            Self::Fps(v) => {
                if s.behavior.fps != v {
                    s.behavior.fps = v;
                    s.behavior.reopen();
                }
            }
            Self::Themes(v) => s
                .components
                .apply_themes(&v)
                .map_err(|e| ConfigError::new("HUD.Themes", e))?,
            Self::Native(v) => s
                .components
                .apply_native(&v)
                .map_err(|e| ConfigError::new("HUD.AdditionalComponents", e))?,
        }
        Ok(())
    }
}
pub fn serialize_hud(
    s: &HudRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut v = t.common(ModuleKind::Hud, c);
    v.push(t.group(
        "Blur",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.behavior.blur),
                "liquidbounce.module.hud.blur.enabled",
                None,
            ),
            t.ranged(
                "Sigma",
                "FLOAT",
                float_json(s.behavior.sigma, "HUD.Blur.Sigma")?,
                json!({"from":1.,"to":15.}),
                "",
                "liquidbounce.module.hud.blur.sigma",
            ),
            t.ranged(
                "AlphaBlendRange",
                "FLOAT_RANGE",
                float_range(
                    s.behavior.alpha_blend_range[0],
                    s.behavior.alpha_blend_range[1],
                    "HUD.Blur.AlphaBlendRange",
                )?,
                json!({"from":0.,"to":1.}),
                "",
                "liquidbounce.module.hud.blur.alphaBlendRange",
            ),
        ],
    ));
    v.push(t.leaf(
        "SpaceSeperatedNames",
        "BOOLEAN",
        json!(s.behavior.space_seperated_names),
        "liquidbounce.module.hud.spaceSeperatedNames",
        None,
    ));
    v.push(crate::hud_components::format_tree(
        s.components.themes_tree(),
        "liquidbounce.module.hud",
        f == JsonFormat::File,
        d,
    ));
    v.push(crate::hud_components::format_tree(
        s.components.native_tree(),
        "liquidbounce.module.hud",
        f == JsonFormat::File,
        d,
    ));
    if s.behavior.renderer_ready {
        v.push(t.group(
            "Renderer",
            "CONFIGURABLE",
            vec![t.ranged(
                "Fps",
                "INT",
                json!(s.behavior.fps),
                json!({"from":0,"to":s.refresh_rate}),
                "FPS",
                "liquidbounce.module.hud.renderer.fps",
            )],
        ));
    }
    Ok(t.group("HUD", "TOGGLEABLE", v))
}
pub(super) fn parse_hud(p: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(
        p,
        root,
        "SpaceSeperatedNames",
        "HUD.SpaceSeperatedNames",
        |v, at| parse_bool(v, at).map(|v| SettingChange::Hud(HudSetting::Names(v))),
    );
    for (group, names) in [
        ("Blur", vec!["Enabled", "Sigma", "AlphaBlendRange"]),
        ("Renderer", vec!["Fps"]),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("HUD.{group}")) {
                Ok(n) => {
                    for name in names {
                        let path = format!("HUD.{group}.{name}");
                        leaf(p, &n, name, &path, |v, at| {
                            Ok(SettingChange::Hud(match name {
                                "Enabled" => HudSetting::Blur(parse_bool(v, at)?),
                                "Sigma" => HudSetting::Sigma(parse_float(v, at)?),
                                "AlphaBlendRange" => HudSetting::Alpha(parse_float_pair(v, at)?),
                                "Fps" => HudSetting::Fps(parse_int(v, at)?),
                                _ => unreachable!(),
                            }))
                        })
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
    for group in ["Themes", "AdditionalComponents"] {
        if let Some(v) = root.get(group) {
            p.changes.push(SettingChange::Hud(if group == "Themes" {
                HudSetting::Themes((*v).clone())
            } else {
                HudSetting::Native((*v).clone())
            }));
        }
    }
}
