// SPDX-License-Identifier: GPL-3.0-or-later
// PotionFX ae0eb396: three independent toggle groups and original textured quads.
use crate::{easing::Easing, target_renderer::Node};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub const MAIN: [&str; 4] = ["Dashed", "Solid", "Runes", "Atlas"];
pub const SECOND: [&str; 4] = ["Cracked", "Neuron", "Hexagon", "Stardust"];
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
fn range(name: &'static str, kind: &'static str, default: Value, a: f32, b: f32) -> Node {
    let mut n = leaf(name, kind, default);
    n.range = Some([a, b]);
    n
}
fn group(name: &'static str, children: Vec<Node>) -> Node {
    let mut n = leaf(name, "CONFIGURABLE", Value::Null);
    n.children = children;
    n
}
fn toggle(name: &'static str, enabled: bool, mut children: Vec<Node>) -> Node {
    children.insert(0, leaf("Enabled", "BOOLEAN", json!(enabled)));
    let mut n = group(name, children);
    n.kind = "TOGGLEABLE";
    n
}
fn choose(name: &'static str, default: &'static str, names: &[&'static str]) -> Node {
    let mut n = leaf(name, "CHOOSE", json!(default));
    n.choices = names.iter().map(|&n| group(n, vec![])).collect();
    n
}
fn source(secondary: bool) -> Node {
    let names = if secondary { &SECOND } else { &MAIN };
    let mut n = leaf("Source", "CHOICE", json!("Builtin"));
    n.choices = vec![
        group("Builtin", vec![choose("Preset", names[0], names)]),
        group("Custom", vec![leaf("File", "FILE", json!(""))]),
    ];
    n
}
fn rotation(default: f32) -> Node {
    range("RotationSpeed", "FLOAT", json!(default), -10., 10.)
}
fn extra(default: f32) -> Node {
    range("ExtraRadius", "FLOAT", json!(default), 0., 10.)
}
fn radius(default: f32) -> Node {
    range("Radius", "FLOAT", json!(default), 0.1, 10.)
}
fn flash() -> Node {
    toggle(
        "Flash",
        true,
        vec![range("AnimTime", "INT", json!(4), 1., 20.), radius(2.)],
    )
}
pub fn schema() -> Node {
    group(
        "PotionFX",
        vec![
            toggle(
                "Players",
                true,
                vec![
                    group("MainEffect", vec![radius(1.), rotation(4.), source(false)]),
                    toggle(
                        "SecondEffect",
                        false,
                        vec![rotation(4.), extra(0.), source(true)],
                    ),
                    leaf("CanBeCovered", "BOOLEAN", json!(true)),
                ],
            ),
            toggle(
                "SplashPotion",
                true,
                vec![
                    group(
                        "MainEffect",
                        vec![
                            range(
                                "Lifetime",
                                "INT_RANGE",
                                json!({"from":20,"to":40}),
                                1.,
                                1000.,
                            ),
                            radius(3.),
                            rotation(3.),
                            range("FadeStart", "FLOAT", json!(0.9), 0.01, 1.),
                            source(false),
                        ],
                    ),
                    group(
                        "SecondEffects",
                        vec![
                            flash(),
                            toggle(
                                "Effect",
                                false,
                                vec![
                                    range("AnimAcceleration", "FLOAT", json!(1.25), 0.1, 2.),
                                    rotation(3.),
                                    extra(0.),
                                    source(true),
                                ],
                            ),
                        ],
                    ),
                    leaf("CanBeCovered", "BOOLEAN", json!(true)),
                ],
            ),
            toggle(
                "LingeringPotion",
                false,
                vec![
                    group(
                        "MainEffect",
                        vec![extra(0.375), rotation(1.), source(false)],
                    ),
                    group(
                        "SecondEffects",
                        vec![
                            flash(),
                            toggle("Effect", false, vec![rotation(1.), extra(0.), source(true)]),
                        ],
                    ),
                    leaf("CanBeCovered", "BOOLEAN", json!(true)),
                ],
            ),
        ],
    )
}
#[derive(Clone, Debug, PartialEq)]
pub struct PotionFX {
    pub values: BTreeMap<String, Value>,
    pub tick: i32,
    pub splashes: Vec<Splash>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PotionFXSetting(pub String, pub Value);
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Splash {
    pub color: i32,
    pub position: [f64; 3],
    pub expiration: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub position: [f64; 3],
    pub old: [f64; 3],
    pub age: i32,
    pub color: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Cloud {
    pub position: [f64; 3],
    pub age: i32,
    pub radius: f32,
    pub color: i32,
}
impl Player {
    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            position: serde_json::from_value(v["position"].clone()).ok()?,
            old: serde_json::from_value(v["old"].clone()).ok()?,
            age: v["age"].as_i64()? as i32,
            color: v["color"].as_i64()? as i32,
        })
    }
}
impl Cloud {
    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            position: serde_json::from_value(v["position"].clone()).ok()?,
            age: v["age"].as_i64()? as i32,
            radius: v["radius"].as_f64()? as f32,
            color: v["color"].as_i64()? as i32,
        })
    }
}
impl Default for PotionFX {
    fn default() -> Self {
        fn walk(n: &Node, p: &str, out: &mut BTreeMap<String, Value>) {
            if !n.default.is_null() {
                out.insert(p.into(), n.default.clone());
            }
            for c in n.children.iter().chain(&n.choices) {
                let q = if p.is_empty() {
                    c.name.into()
                } else {
                    format!("{p}.{}", c.name)
                };
                walk(c, &q, out);
            }
        }
        let mut values = BTreeMap::new();
        walk(&schema(), "", &mut values);
        Self {
            values,
            tick: 0,
            splashes: vec![],
        }
    }
}
impl PotionFXSetting {
    pub fn apply(self, s: &mut PotionFX) {
        if self.0 == "SplashPotion.Enabled" && self.1 == false && s.value(&self.0) != &self.1 {
            s.splashes.clear();
        }
        s.values.insert(self.0, self.1);
    }
}
impl PotionFX {
    pub fn value(&self, p: &str) -> &Value {
        &self.values[p]
    }
    pub fn text(&self, p: &str) -> &str {
        self.value(p).as_str().unwrap()
    }
    pub fn number(&self, p: &str) -> f32 {
        self.value(p).as_f64().unwrap() as f32
    }
    pub fn boolean(&self, p: &str) -> bool {
        self.value(p).as_bool().unwrap()
    }
    pub fn clear(&mut self) {
        self.splashes.clear();
    }
    pub fn game_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.splashes.retain(|s| s.expiration > self.tick);
    }
    pub fn splash(&mut self, color: i32, position: [f64; 3]) {
        if self.boolean("SplashPotion.Enabled") {
            let life = self.value("SplashPotion.MainEffect.Lifetime")["to"]
                .as_i64()
                .unwrap() as i32;
            self.splashes.push(Splash {
                color,
                position,
                expiration: self.tick.wrapping_add(life),
            });
        }
    }
    pub fn texture(&self, path: &str) -> String {
        if self.text(&format!("{path}.Source")) == "Custom" {
            self.text(&format!("{path}.Source.Custom.File")).into()
        } else {
            format!(
                "builtin:{}",
                self.text(&format!("{path}.Source.Builtin.Preset"))
            )
        }
    }
    pub fn geometry(
        &self,
        partial: f32,
        players: &[Player],
        clouds: &[Cloud],
        right: [f32; 3],
        up: [f32; 3],
    ) -> Value {
        let mut groups = vec![];
        for (index, name, main, second) in [
            (0, "Players", "Players.MainEffect", "Players.SecondEffect"),
            (
                1,
                "SplashPotion",
                "SplashPotion.MainEffect",
                "SplashPotion.SecondEffects.Effect",
            ),
            (
                2,
                "LingeringPotion",
                "LingeringPotion.MainEffect",
                "LingeringPotion.SecondEffects.Effect",
            ),
        ] {
            if !self.boolean(&format!("{name}.Enabled")) {
                continue;
            }
            let textures = [self.texture(main), self.texture(second)];
            let enabled = self.boolean(&format!("{second}.Enabled"));
            let depth = self.boolean(&format!("{name}.CanBeCovered"));
            let mut quads = vec![];
            if index == 0 {
                for player in players {
                    let at = std::array::from_fn(|i| {
                        player.old[i]
                            + (player.position[i] - player.old[i]) * f64::from(partial)
                            + if i == 1 { 0.01 } else { 0. }
                    });
                    let age = player.age as f32 + partial;
                    let r = self.number(&format!("{main}.Radius"));
                    horizontal(
                        &mut quads,
                        at,
                        r,
                        age * self.number(&format!("{main}.RotationSpeed")),
                        player.color,
                        0,
                    );
                    if enabled {
                        horizontal(
                            &mut quads,
                            at,
                            r + self.number(&format!("{second}.ExtraRadius")),
                            age * self.number(&format!("{second}.RotationSpeed")),
                            player.color,
                            1,
                        );
                    }
                }
            } else if index == 1 {
                let lifetime = self.value("SplashPotion.MainEffect.Lifetime");
                let a = lifetime["from"].as_i64().unwrap() as f32;
                let b = lifetime["to"].as_i64().unwrap() as f32;
                let flash = "SplashPotion.SecondEffects.Flash";
                for splash in &self.splashes {
                    let remain = splash.expiration.wrapping_sub(self.tick) as f32;
                    let age = b - remain + partial;
                    let progress = Easing::ExponentialOut.transform(age / a).clamp(0., 1.);
                    let glow = Easing::QuadInOut
                        .transform(age / self.number(&format!("{flash}.AnimTime")))
                        .clamp(0., 1.);
                    let fade = ((remain - partial)
                        / (b * (1. - self.number("SplashPotion.MainEffect.FadeStart"))))
                    .clamp(0., 1.);
                    let alpha = (((splash.color as u32 >> 24) as f32 * fade) as u32).min(255);
                    let color = ((splash.color as u32 & 0xffffff) | (alpha << 24)) as i32;
                    let mut at = splash.position;
                    at[1] += 0.01;
                    let r = self.number(&format!("{main}.Radius"));
                    horizontal(
                        &mut quads,
                        at,
                        r * progress,
                        age * self.number(&format!("{main}.RotationSpeed")),
                        color,
                        0,
                    );
                    if enabled {
                        let mut second_at = at;
                        second_at[1] -= 0.005;
                        horizontal(
                            &mut quads,
                            second_at,
                            (r + self.number(&format!("{second}.ExtraRadius")))
                                * (progress * self.number(&format!("{second}.AnimAcceleration")))
                                    .clamp(0., 1.),
                            age * self.number(&format!("{second}.RotationSpeed")),
                            color,
                            1,
                        );
                    }
                    if self.boolean(&format!("{flash}.Enabled")) {
                        billboard(
                            &mut quads,
                            at,
                            self.number(&format!("{flash}.Radius")) * glow,
                            splash.color,
                            right,
                            up,
                        );
                    }
                }
            } else {
                let flash = "LingeringPotion.SecondEffects.Flash";
                let anim = self.number(&format!("{flash}.AnimTime"));
                for cloud in clouds {
                    let age = cloud.age as f32 + partial;
                    let glow = Easing::QuadInOut
                        .transform((cloud.age.min(anim as i32 * 2) as f32 + partial) / anim)
                        .clamp(0., 1.);
                    let mut at = cloud.position;
                    at[1] += 0.01;
                    let r = cloud.radius + self.number(&format!("{main}.ExtraRadius"));
                    horizontal(
                        &mut quads,
                        at,
                        r,
                        age * self.number(&format!("{main}.RotationSpeed")),
                        cloud.color,
                        0,
                    );
                    if enabled {
                        let mut at = at;
                        at[1] -= 0.005;
                        horizontal(
                            &mut quads,
                            at,
                            r + self.number(&format!("{second}.ExtraRadius")),
                            age * self.number(&format!("{second}.RotationSpeed")),
                            cloud.color,
                            1,
                        );
                    }
                    if self.boolean(&format!("{flash}.Enabled")) {
                        billboard(
                            &mut quads,
                            at,
                            self.number(&format!("{flash}.Radius")) * glow,
                            cloud.color,
                            right,
                            up,
                        );
                    }
                }
            }
            groups.push(json!({"slot":index,"textures":textures,"allow_load":[true,enabled],"depth":depth,"quads":quads}));
        }
        json!({"groups":groups})
    }
}
fn quad(out: &mut Vec<Value>, corners: [[f64; 3]; 4], color: i32, texture: usize) {
    let [a, r, g, b] = (color as u32).to_be_bytes();
    let color = [
        r as f32 / 255.,
        g as f32 / 255.,
        b as f32 / 255.,
        a as f32 / 255.,
    ];
    out.push(json!({"corners":corners,"color":color,"texture":texture}));
}
fn horizontal(
    out: &mut Vec<Value>,
    at: [f64; 3],
    radius: f32,
    rotation: f32,
    color: i32,
    texture: usize,
) {
    let rad = rotation * std::f32::consts::PI / 180.;
    let (s, c) = (f64::from(rad).sin() as f32, f64::from(rad).cos() as f32);
    let corners = [
        [-radius, radius],
        [-radius, -radius],
        [radius, -radius],
        [radius, radius],
    ]
    .map(|[x, y]| {
        let rx = c * x - s * y;
        let ry = s * x + c * y;
        [at[0] + f64::from(rx), at[1], at[2] - f64::from(ry)]
    });
    quad(out, corners, color, texture);
}
fn billboard(
    out: &mut Vec<Value>,
    at: [f64; 3],
    radius: f32,
    color: i32,
    right: [f32; 3],
    up: [f32; 3],
) {
    let corners = [
        [-radius, radius],
        [-radius, -radius],
        [radius, -radius],
        [radius, radius],
    ]
    .map(|[x, y]| std::array::from_fn(|i| at[i] + f64::from(right[i] * x + up[i] * y)));
    quad(out, corners, color, 2);
}
