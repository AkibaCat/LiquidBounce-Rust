// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleJumpEffect, JumpEffectMode and both modes, LiquidBounce ae0eb396, CCBlueX 2015-2026.
use crate::{
    breadcrumbs::{Effect, STYLES, StyleSettings, specs},
    easing::Easing,
    target_renderer::Node,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub const MODES: [&str; 2] = ["Simple", "Image"];
pub const PRESETS: [&str; 2] = ["LiquidBounce", "LiquidBounceWithLogo"];
fn leaf(name: &'static str, kind: &'static str, default: Value) -> Node {
    Node {
        name,
        kind,
        default,
        range: None,
        suffix: "",
        children: vec![],
        choices: vec![],
    }
}
fn ranged(
    name: &'static str,
    kind: &'static str,
    default: Value,
    a: f32,
    b: f32,
    suffix: &'static str,
) -> Node {
    let mut n = leaf(name, kind, default);
    n.range = Some([a, b]);
    n.suffix = suffix;
    n
}
fn group(name: &'static str, children: Vec<Node>) -> Node {
    let mut n = leaf(name, "CONFIGURABLE", Value::Null);
    n.children = children;
    n
}
fn choice(name: &'static str, default: &'static str, choices: Vec<Node>) -> Node {
    let mut n = leaf(name, "CHOICE", json!(default));
    n.choices = choices;
    n
}
fn choose(
    name: &'static str,
    default: &'static str,
    names: impl Iterator<Item = &'static str>,
) -> Node {
    let mut n = choice(name, default, names.map(|n| group(n, vec![])).collect());
    n.kind = "CHOOSE";
    n
}
fn colors() -> Node {
    group(
        "Colors",
        vec![
            leaf("InnerColor", "COLOR", json!(0x324677ffi32)),
            leaf("OuterColor", "COLOR", json!(0x964677ffu32 as i32)),
        ],
    )
}
fn render() -> Node {
    let styles = STYLES
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            group(
                name,
                specs(i)
                    .map(|s| {
                        if s.color {
                            leaf(s.name, "COLOR", json!(-1))
                        } else {
                            ranged(s.name, "FLOAT", json!(s.default), s.from, s.to, s.suffix)
                        }
                    })
                    .collect(),
            )
        })
        .collect();
    choice(
        "Render",
        "Plain",
        vec![
            group("Plain", vec![]),
            group("Shader", vec![choice("Style", "Smooth", styles)]),
        ],
    )
}
pub fn schema() -> Node {
    let common = || {
        vec![
            ranged(
                "EndRadius",
                "FLOAT_RANGE",
                json!({"from":0.15,"to":0.8}),
                0.,
                3.,
                "",
            ),
            choose(
                "AnimCurve",
                "QuadOut",
                Easing::ALL.into_iter().map(Easing::tag),
            ),
            ranged("HueOffsetAnim", "INT", json!(-63), -360., 360., ""),
            ranged(
                "Lifetime",
                "INT_RANGE",
                json!({"from":15,"to":15}),
                1.,
                120.,
                "anim/life",
            ),
            leaf("CanBeCovered", "BOOLEAN", json!(false)),
        ]
    };
    let mut simple = common();
    simple.push(colors());
    simple.push(render());
    let mut image = common();
    image.push(ranged("RotationSpeed", "INT", json!(10), -360., 360., ""));
    image.push(choice(
        "Source",
        "Builtin",
        vec![
            group(
                "Builtin",
                vec![choose("Preset", PRESETS[0], PRESETS.into_iter())],
            ),
            group("Custom", vec![leaf("File", "FILE", json!(""))]),
        ],
    ));
    image.push(colors());
    group(
        "JumpEffect",
        vec![choice(
            "Mode",
            "Simple",
            vec![group("Simple", simple), group("Image", image)],
        )],
    )
}
#[derive(Clone, Debug, PartialEq)]
pub struct JumpEffect {
    pub values: BTreeMap<String, Value>,
    pub tick: i32,
    pub circles: Vec<Circle>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    pub position: [f64; 3],
    pub expiration: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct JumpEffectSetting(pub String, pub Value);
impl Default for JumpEffect {
    fn default() -> Self {
        fn walk(n: &Node, p: &str, out: &mut BTreeMap<String, Value>) {
            if !n.default.is_null() {
                out.insert(p.into(), n.default.clone());
            }
            for c in n.children.iter().chain(&n.choices) {
                let p = if p.is_empty() {
                    c.name.into()
                } else {
                    format!("{p}.{}", c.name)
                };
                walk(c, &p, out);
            }
        }
        let mut values = BTreeMap::new();
        walk(&schema(), "", &mut values);
        Self {
            values,
            tick: 0,
            circles: vec![],
        }
    }
}
impl JumpEffectSetting {
    pub fn apply(self, s: &mut JumpEffect) {
        if self.0 == "Mode" && s.values.get("Mode") != Some(&self.1) {
            s.clear();
        }
        s.values.insert(self.0, self.1);
    }
}
fn rgba(c: i32) -> [f32; 4] {
    let [a, r, g, b] = (c as u32).to_be_bytes();
    [r, g, b, a].map(|v| f32::from(v) / 255.)
}
fn interpolate(a: i32, b: i32, t: f32) -> [f32; 4] {
    let channel = |shift: u32| {
        let a = ((a as u32 >> shift) & 255) as f32;
        let b = ((b as u32 >> shift) & 255) as f32;
        t.mul_add(b - a, a) as i32 as f32 / 255.
    };
    [channel(16), channel(8), channel(0), channel(24)]
}
pub fn shift_hue(color: i32, shift: i32) -> i32 {
    if shift == 0 {
        return color;
    }
    let [a, r, g, b] = (color as u32).to_be_bytes();
    let (r, g, b) = (i32::from(r), i32::from(g), i32::from(b));
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let saturation = if max == 0 {
        0.
    } else {
        (max - min) as f32 / max as f32
    };
    let brightness = max as f32 / 255.;
    let mut hue = 0.;
    if saturation != 0. {
        let d = (max - min) as f32;
        let rc = (max - r) as f32 / d;
        let gc = (max - g) as f32 / d;
        let bc = (max - b) as f32 / d;
        hue = if r == max {
            bc - gc
        } else if g == max {
            2. + rc - bc
        } else {
            4. + gc - rc
        };
        hue /= 6.;
        if hue < 0. {
            hue += 1.;
        }
    }
    hue = (hue + shift as f32 / 360.) % 1.;
    let mut rgb = [brightness; 3];
    if saturation != 0. {
        let h = (hue - hue.floor()) * 6.;
        let f = h - h.floor();
        let p = brightness * (1. - saturation);
        let q = brightness * (1. - saturation * f);
        let t = brightness * (1. - saturation * (1. - f));
        rgb = match h as i32 {
            0 => [brightness, t, p],
            1 => [q, brightness, p],
            2 => [p, brightness, t],
            3 => [p, q, brightness],
            4 => [t, p, brightness],
            _ => [brightness, p, q],
        };
    }
    let c = rgb.map(|v| (v * 255. + 0.5) as u32);
    ((u32::from(a) << 24) | (c[0] << 16) | (c[1] << 8) | c[2]) as i32
}
impl JumpEffect {
    pub fn value(&self, p: &str) -> &Value {
        &self.values[p]
    }
    pub fn text(&self, p: &str) -> &str {
        self.value(p).as_str().unwrap()
    }
    pub fn number(&self, p: &str) -> f32 {
        self.value(p).as_f64().unwrap() as f32
    }
    pub fn clear(&mut self) {
        self.circles.clear();
    }
    pub fn active(&self) -> &str {
        self.text("Mode")
    }
    fn path(&self, name: &str) -> String {
        format!("Mode.{}.{name}", self.active())
    }
    pub fn game_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.circles.retain(|c| c.expiration > self.tick);
    }
    pub fn jump(&mut self, position: [f64; 3]) {
        let life = self.value(&self.path("Lifetime"))["to"].as_i64().unwrap() as i32;
        self.circles.push(Circle {
            position,
            expiration: self.tick.wrapping_add(life),
        });
    }
    pub fn effect(&self) -> Option<Effect> {
        if self.active() != "Simple" || self.text("Mode.Simple.Render") != "Shader" {
            return None;
        }
        let name = self.text("Mode.Simple.Render.Shader.Style");
        let i = STYLES.iter().position(|n| *n == name).unwrap();
        let mut s = StyleSettings::new(i);
        for spec in specs(i) {
            let p = format!("Mode.Simple.Render.Shader.Style.{name}.{}", spec.name);
            if spec.color {
                s.accent = self.value(&p).as_i64().unwrap() as i32;
            } else {
                s.values.insert(spec.name.into(), self.number(&p));
            }
        }
        Some(s.effect(i))
    }
    pub fn animate_color(&self, color: i32, age: f32) -> i32 {
        let life = self.value(&self.path("Lifetime"));
        let first = life["from"].as_i64().unwrap() as f32;
        let last = life["to"].as_i64().unwrap() as f32;
        let fade = if last > first {
            ((age - first) / (last - first)).clamp(0., 1.)
        } else {
            (age / last).clamp(0., 1.)
        };
        let a = (((color as u32 >> 24) as f32) * (1. - fade)) as u32;
        let faded = (color as u32 & 0xffffff) | (a << 24);
        shift_hue(
            faded as i32,
            (self.number(&self.path("HueOffsetAnim")) * (age / last).clamp(0., 1.)) as i32,
        )
    }
    pub fn geometry(&self, partial: f32) -> Value {
        let base = self.path("");
        let p = |n: &str| format!("{base}{n}");
        let image = self.active() == "Image";
        let effect = self.effect();
        let radius = self.value(&p("EndRadius"));
        let inner = radius["from"].as_f64().unwrap() as f32;
        let outer = radius["to"].as_f64().unwrap() as f32;
        let life = self.value(&p("Lifetime"));
        let first = life["from"].as_i64().unwrap() as f32;
        let last = life["to"].as_i64().unwrap() as i32;
        let curve = Easing::ALL
            .into_iter()
            .find(|e| e.tag() == self.text(&p("AnimCurve")))
            .unwrap();
        let depth = self.value(&p("CanBeCovered")).as_bool().unwrap();
        let texture = if image {
            if self.text(&p("Source")) == "Builtin" {
                format!("builtin:{}", self.text(&p("Source.Builtin.Preset")))
            } else {
                self.text(&p("Source.Custom.File")).into()
            }
        } else {
            "builtin:White".into()
        };
        let mut quads = vec![];
        for c in &self.circles {
            let age = last.wrapping_sub(c.expiration.wrapping_sub(self.tick)) as f32 + partial;
            let progress = curve.transform(age / first).clamp(0., 1.);
            let outer = outer * progress;
            let inner = inner * progress;
            let get = |n: &str| {
                let mut color = self.value(&p(n)).as_i64().unwrap() as i32;
                if effect.is_some() {
                    color = (color as u32 | 0xff000000) as i32;
                }
                self.animate_color(color, age)
            };
            let (outer_color, inner_color) = (get("Colors.OuterColor"), get("Colors.InnerColor"));
            if outer <= 0. || (outer_color as u32 >> 24 == 0 && inner_color as u32 >> 24 == 0) {
                continue;
            }
            let at = |x: f32, z: f32| {
                [
                    c.position[0] + f64::from(x),
                    c.position[1] + 0.01,
                    c.position[2] + f64::from(z),
                ]
            };
            if !image {
                let colors = [rgba(outer_color); 4];
                quads.push(json!({"corners":[at(-outer,-outer),at(-outer,outer),at(outer,outer),at(outer,-outer)],"colors":colors,"uv":[[0.,0.],[0.,1.],[1.,1.],[1.,0.]],"inner_color":rgba(inner_color),"inner":(inner/outer).clamp(0.,1.),"shape":2}));
                continue;
            }
            let angle = (self.number(&p("RotationSpeed")) * age) * std::f32::consts::PI / 180.;
            let (sin, cos) = angle.sin_cos();
            let corner = |x: f32, y: f32| at(cos * x - sin * y, sin * x + cos * y);
            let ratio = (inner / outer).clamp(0., 1.).max(0.8);
            let color = |x: f32, y: f32| {
                let d = ((x * x + y * y).sqrt() / outer).clamp(0., 1.);
                let t = if d <= ratio {
                    0.
                } else {
                    ((d - ratio) / (1. - ratio)).clamp(0., 1.)
                };
                interpolate(inner_color, outer_color, t)
            };
            let step = outer * 2. / 16.;
            for row in 0..16 {
                for col in 0..16 {
                    let x1 = -outer + col as f32 * step;
                    let x2 = x1 + step;
                    let y1 = -outer + row as f32 * step;
                    let y2 = y1 + step;
                    let (u1, u2, v1, v2) = (
                        col as f32 / 16.,
                        (col + 1) as f32 / 16.,
                        row as f32 / 16.,
                        (row + 1) as f32 / 16.,
                    );
                    quads.push(json!({"corners":[corner(x1,y2),corner(x1,y1),corner(x2,y1),corner(x2,y2)],"colors":[color(x1,y2),color(x1,y1),color(x2,y1),color(x2,y2)],"uv":[[u1,v2],[u1,v1],[u2,v1],[u2,v2]],"inner_color":[0.,0.,0.,0.],"inner":0.,"shape":1}));
                }
            }
        }
        let effect=effect.map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity}));
        json!({"quads":quads,"effect":effect,"depth":depth,"texture":texture})
    }
}
