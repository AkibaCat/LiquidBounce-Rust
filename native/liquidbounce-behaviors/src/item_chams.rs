// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleItemChams and exact source uniform packing, LiquidBounce ae0eb396.
use serde_json::{json, Value};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Bool,
    Int(i32, i32),
    Float(f32, f32, &'static str),
    Color,
    Vec2,
    Choice(&'static [&'static str]),
}
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub path: &'static str,
    pub kind: Kind,
    pub default: &'static str,
}
pub const SPECS: &[Spec] = &[
    Spec {
        path: "Lightmap.Enabled",
        kind: Kind::Bool,
        default: "true",
    },
    Spec {
        path: "Lightmap.BlendColor",
        kind: Kind::Color,
        default: "-1174388481",
    },
    Spec {
        path: "Lightmap.Alpha",
        kind: Kind::Int(1, 255),
        default: "95",
    },
    Spec {
        path: "Lightmap.GlowColor",
        kind: Kind::Color,
        default: "251674879",
    },
    Spec {
        path: "Lightmap.Layers",
        kind: Kind::Int(1, 10),
        default: "3",
    },
    Spec {
        path: "Lightmap.LayerSize",
        kind: Kind::Float(1., 5., ""),
        default: "1.91",
    },
    Spec {
        path: "Lightmap.Falloff",
        kind: Kind::Float(0., 20., ""),
        default: "6.83",
    },
    Spec {
        path: "Shield.Enabled",
        kind: Kind::Bool,
        default: "true",
    },
    Spec {
        path: "Shield.TintMode",
        kind: Kind::Choice(&["Override", "Multiply"]),
        default: "\"Multiply\"",
    },
    Spec {
        path: "Shield.Tint",
        kind: Kind::Color,
        default: "-1",
    },
    Spec {
        path: "Effects.Enabled",
        kind: Kind::Bool,
        default: "false",
    },
    Spec {
        path: "Effects.Flow.Enabled",
        kind: Kind::Bool,
        default: "true",
    },
    Spec {
        path: "Effects.Flow.Color",
        kind: Kind::Color,
        default: "-8544257",
    },
    Spec {
        path: "Effects.Flow.Speed",
        kind: Kind::Float(0., 5., ""),
        default: "0.6",
    },
    Spec {
        path: "Effects.Flow.Scale",
        kind: Kind::Float(1., 64., ""),
        default: "12",
    },
    Spec {
        path: "Effects.Flow.Sharpness",
        kind: Kind::Float(1., 64., ""),
        default: "6",
    },
    Spec {
        path: "Effects.Flow.Angle",
        kind: Kind::Float(0., 360., "°"),
        default: "45",
    },
    Spec {
        path: "Effects.Flow.Intensity",
        kind: Kind::Float(0., 4., ""),
        default: "0.8",
    },
    Spec {
        path: "Effects.Trail.Enabled",
        kind: Kind::Bool,
        default: "true",
    },
    Spec {
        path: "Effects.Trail.Color",
        kind: Kind::Color,
        default: "-12158977",
    },
    Spec {
        path: "Effects.Trail.Weight",
        kind: Kind::Float(0., 4., ""),
        default: "0.7",
    },
    Spec {
        path: "Effects.Trail.Decay",
        kind: Kind::Float(0., 1., ""),
        default: "0.86",
    },
    Spec {
        path: "Effects.Trail.Drift",
        kind: Kind::Vec2,
        default: "[2,0]",
    },
    Spec {
        path: "Effects.Glow.Enabled",
        kind: Kind::Bool,
        default: "false",
    },
    Spec {
        path: "Effects.Glow.Color",
        kind: Kind::Color,
        default: "-8544257",
    },
    Spec {
        path: "Effects.Glow.Radius",
        kind: Kind::Float(1., 32., "px"),
        default: "6",
    },
    Spec {
        path: "Effects.Glow.Thickness",
        kind: Kind::Float(0.2, 8., ""),
        default: "1.5",
    },
    Spec {
        path: "Effects.Glow.Intensity",
        kind: Kind::Float(0., 4., ""),
        default: "0.9",
    },
    Spec {
        path: "Effects.Chromatic.Enabled",
        kind: Kind::Bool,
        default: "false",
    },
    Spec {
        path: "Effects.Chromatic.Strength",
        kind: Kind::Float(0., 16., "px"),
        default: "2",
    },
    Spec {
        path: "Effects.Chromatic.Ghost",
        kind: Kind::Bool,
        default: "false",
    },
    Spec {
        path: "Effects.Scanlines.Enabled",
        kind: Kind::Bool,
        default: "false",
    },
    Spec {
        path: "Effects.Scanlines.Color",
        kind: Kind::Color,
        default: "-8544257",
    },
    Spec {
        path: "Effects.Scanlines.Spacing",
        kind: Kind::Float(2., 256., "px"),
        default: "40",
    },
    Spec {
        path: "Effects.Scanlines.Thickness",
        kind: Kind::Float(0.02, 1., ""),
        default: "0.4",
    },
    Spec {
        path: "Effects.Scanlines.Speed",
        kind: Kind::Float(-5., 5., ""),
        default: "0.3",
    },
    Spec {
        path: "Effects.Scanlines.Angle",
        kind: Kind::Float(0., 360., "°"),
        default: "90",
    },
    Spec {
        path: "Effects.Scanlines.Intensity",
        kind: Kind::Float(0., 4., ""),
        default: "0.5",
    },
    Spec {
        path: "Effects.Recolor.Enabled",
        kind: Kind::Bool,
        default: "false",
    },
    Spec {
        path: "Effects.Recolor.HueSpeed",
        kind: Kind::Float(-2., 2., ""),
        default: "0.2",
    },
    Spec {
        path: "Effects.Recolor.HueMix",
        kind: Kind::Float(0., 1., ""),
        default: "1",
    },
    Spec {
        path: "Effects.Recolor.Saturation",
        kind: Kind::Float(0., 3., ""),
        default: "1.4",
    },
    Spec {
        path: "Effects.Recolor.Brightness",
        kind: Kind::Float(0., 3., ""),
        default: "1.2",
    },
    Spec {
        path: "Effects.Recolor.PulseSpeed",
        kind: Kind::Float(0., 12., ""),
        default: "2",
    },
    Spec {
        path: "Effects.Recolor.PulseAmount",
        kind: Kind::Float(0., 1., ""),
        default: "0.15",
    },
];
#[derive(Clone, Debug, PartialEq)]
pub struct ItemChams {
    pub values: BTreeMap<String, Value>,
}
impl Default for ItemChams {
    fn default() -> Self {
        Self {
            values: SPECS
                .iter()
                .map(|s| (s.path.into(), serde_json::from_str(s.default).unwrap()))
                .collect(),
        }
    }
}
impl ItemChams {
    pub fn clear(&mut self) {
        // Disabling clears the native render pass; user settings stay configured.
    }
    pub fn flag(&self, p: &str) -> bool {
        self.values[p].as_bool().unwrap()
    }
    pub fn number(&self, p: &str) -> f32 {
        self.values[p].as_f64().unwrap() as f32
    }
    pub fn color(&self, p: &str) -> u32 {
        self.values[p].as_i64().unwrap() as u32
    }
    pub fn shield_tint(&self, original: u32) -> u32 {
        if !self.flag("Shield.Enabled") {
            return original;
        }
        let tint = self.color("Shield.Tint");
        if self.values["Shield.TintMode"] == "Override" {
            return tint;
        }
        let a = original.to_be_bytes();
        let b = tint.to_be_bytes();
        u32::from_be_bytes(std::array::from_fn(|i| {
            (u32::from(a[i]) * u32::from(b[i]) / 255) as u8
        }))
    }
    pub fn render_settings(&self, time: f32) -> Value {
        let rgba = |p: &str| {
            let [a, r, g, b] = self.color(p).to_be_bytes();
            [r, g, b, a].map(|v| f32::from(v) / 255.)
        };
        let mut light = [0u32; 20];
        light[1] = (self.number("Lightmap.Alpha") / 255.).to_bits();
        for (i, v) in rgba("Lightmap.BlendColor").into_iter().enumerate() {
            light[4 + i] = v.to_bits();
        }
        light[8] = self.number("Lightmap.LayerSize").to_bits();
        for (i, v) in rgba("Lightmap.GlowColor").into_iter().enumerate() {
            light[12 + i] = v.to_bits();
        }
        light[16] = self.number("Lightmap.Falloff").to_bits();
        light[17] = self.number("Lightmap.Layers") as u32;
        let mut effects = [0u32; 52];
        effects[0] = time.to_bits();
        for (i, n) in ["Flow", "Trail", "Glow", "Chromatic", "Scanlines"]
            .iter()
            .enumerate()
        {
            if self.flag(&format!("Effects.{n}.Enabled")) {
                effects[1] |= 1 << i;
            }
        }
        for (start, p) in [(4, "Flow"), (16, "Trail"), (24, "Glow"), (32, "Scanlines")] {
            for (i, v) in rgba(&format!("Effects.{p}.Color")).into_iter().enumerate() {
                effects[start + i] = v.to_bits();
            }
        }
        for (index, path) in [
            (8, "Flow.Speed"),
            (9, "Flow.Scale"),
            (10, "Flow.Sharpness"),
            (11, "Flow.Intensity"),
            (20, "Trail.Weight"),
            (21, "Trail.Decay"),
            (28, "Glow.Radius"),
            (29, "Glow.Thickness"),
            (30, "Glow.Intensity"),
            (36, "Scanlines.Spacing"),
            (37, "Scanlines.Thickness"),
            (38, "Scanlines.Speed"),
            (39, "Scanlines.Intensity"),
            (42, "Chromatic.Strength"),
            (45, "Recolor.HueMix"),
            (46, "Recolor.Saturation"),
            (47, "Recolor.Brightness"),
            (48, "Recolor.PulseSpeed"),
            (49, "Recolor.PulseAmount"),
        ] {
            effects[index] = self.number(&format!("Effects.{path}")).to_bits();
        }
        for (index, name) in [(12, "Flow"), (40, "Scanlines")] {
            let angle = self.number(&format!("Effects.{name}.Angle")).to_radians();
            effects[index] = angle.cos().to_bits();
            effects[index + 1] = angle.sin().to_bits();
        }
        let drift = self.values["Effects.Trail.Drift"].as_array().unwrap();
        effects[22] = (drift[0].as_f64().unwrap() as f32).to_bits();
        effects[23] = (drift[1].as_f64().unwrap() as f32).to_bits();
        effects[43] = u32::from(self.flag("Effects.Chromatic.Ghost"));
        effects[44] = (time * self.number("Effects.Recolor.HueSpeed"))
            .rem_euclid(1.)
            .to_bits();
        json!({"lightmap":self.flag("Lightmap.Enabled"),"effects":self.flag("Effects.Enabled"),"recolor":self.flag("Effects.Recolor.Enabled"),"lightUniform":light.to_vec(),"effectUniform":effects.to_vec(),"shield":self.flag("Shield.Enabled"),"shieldOverride":self.values["Shield.TintMode"]=="Override","shieldColor":self.color("Shield.Tint")})
    }
}
