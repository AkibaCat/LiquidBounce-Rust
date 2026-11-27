// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleCustomAmbience, world effects and actual mixin branches, LiquidBounce ae0eb396.
use crate::{easing::Easing, target_renderer::Node};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub const WEATHER: [&str; 5] = ["NoChange", "Sunny", "Rainy", "Snowy", "Thunder"];
pub const TIME: [&str; 7] = [
    "NoChange", "Dawn", "Day", "Noon", "Dusk", "Night", "MidNight",
];
pub const PARTICLE: [&str; 11] = [
    "Orbiz",
    "Star",
    "Dollar",
    "Crown",
    "Heart",
    "Lightning",
    "Line",
    "Point",
    "Rhombus",
    "Snowflake",
    "Spark",
];
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
fn group(name: &'static str, children: Vec<Node>) -> Node {
    let mut n = leaf(name, "CONFIGURABLE", Value::Null);
    n.children = children;
    n
}
fn toggle(name: &'static str, enabled: bool, children: Vec<Node>) -> Node {
    let mut n = group(name, vec![leaf("Enabled", "BOOLEAN", json!(enabled))]);
    n.kind = "TOGGLEABLE";
    n.children.extend(children);
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
    choices: impl IntoIterator<Item = &'static str>,
) -> Node {
    let mut n = choice(
        name,
        default,
        choices.into_iter().map(|n| group(n, vec![])).collect(),
    );
    n.kind = "CHOOSE";
    n
}
fn range(name: &'static str, kind: &'static str, default: Value, a: f32, b: f32) -> Node {
    let mut n = leaf(name, kind, default);
    n.range = Some([a, b]);
    n
}
pub fn schema() -> Node {
    let blue = json!(0xff4677ffu32 as i32);
    let mist = json!(0xc94677ffu32 as i32);
    let mut life = range("Life", "INT_RANGE", json!({"from":20,"to":30}), 1., 512.);
    life.suffix = "anim/life";
    let mut spawn = range("SpawnTime", "INT", json!(25), 1., 200.);
    spawn.suffix = "ms";
    group(
        "CustomAmbience",
        vec![
            choose("Weather", "Snowy", WEATHER),
            choose("Time", "Night", TIME),
            toggle(
                "ModifyPrecipitation",
                true,
                vec![range("Gradient", "FLOAT", json!(0.7), 0.1, 1.)],
            ),
            toggle(
                "Fog",
                true,
                vec![
                    leaf("DisableWorldFog", "BOOLEAN", json!(false)),
                    toggle(
                        "FogColorOverride",
                        true,
                        vec![leaf("Color", "COLOR", mist.clone())],
                    ),
                    leaf("BackgroundColor", "COLOR", mist),
                    range(
                        "Environmental",
                        "FLOAT_RANGE",
                        json!({"from":5.,"to":40.}),
                        -16.,
                        2048.,
                    ),
                    range(
                        "RenderDistance",
                        "FLOAT_RANGE",
                        json!({"from":230.,"to":256.}),
                        0.,
                        1024.,
                    ),
                    range("SkyEnd", "FLOAT", json!(40.), 0., 1024.),
                    range("CloudEnd", "FLOAT", json!(40.), 0., 4096.),
                ],
            ),
            toggle(
                "CustomLightmap",
                true,
                vec![choice(
                    "Mode",
                    "SingleColor",
                    vec![
                        group("SingleColor", vec![leaf("Color", "COLOR", blue.clone())]),
                        group(
                            "Custom",
                            vec![
                                leaf("BlockLightTint", "COLOR", json!(0x00ffffff)),
                                leaf("SkyLightColor", "COLOR", json!(0x004677ff)),
                                leaf("AmbientColor", "COLOR", json!(0x004677ff)),
                                leaf("NightVisionColor", "COLOR", json!(0x00ffffff)),
                            ],
                        ),
                    ],
                )],
            ),
            toggle(
                "SkyColor",
                false,
                vec![leaf("Color", "COLOR", blue.clone())],
            ),
            toggle(
                "WorldParticles",
                false,
                vec![choice(
                    "Mode",
                    "Simple",
                    vec![group(
                        "Simple",
                        vec![
                            range("Radius", "INT_RANGE", json!({"from":8,"to":16}), 1., 256.),
                            range("Count", "INT", json!(256), 32., 1024.),
                            range("Size", "FLOAT", json!(1.), 0.1, 10.),
                            life,
                            spawn,
                            choose(
                                "AnimCurve",
                                "QuadInOut",
                                Easing::ALL.into_iter().map(Easing::tag),
                            ),
                            leaf("CanBeCovered", "BOOLEAN", json!(true)),
                            toggle(
                                "YMotion",
                                false,
                                vec![
                                    range("Motion", "FLOAT", json!(2.), -10., 10.),
                                    choose("AnimBy", "Progress", ["Progress", "Age"]),
                                ],
                            ),
                            toggle(
                                "Wander",
                                false,
                                vec![
                                    range("Distance", "FLOAT", json!(0.5), 0.05, 4.),
                                    range("Speed", "FLOAT", json!(1.), 0.1, 5.),
                                ],
                            ),
                            choose("Particle", "Spark", PARTICLE),
                            group("Color", vec![leaf("Color", "COLOR", blue)]),
                        ],
                    )],
                )],
            ),
        ],
    )
}
#[derive(Clone, Debug, PartialEq)]
pub struct Particle {
    pub position: [f64; 3],
    pub expiration: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CustomAmbience {
    pub values: BTreeMap<String, Value>,
    pub coords: Vec<Particle>,
    pub tick: i32,
    pub last_spawn: i64,
    pub rng: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CustomAmbienceSetting(pub String, pub Value);
impl Default for CustomAmbience {
    fn default() -> Self {
        fn walk(n: &Node, p: &str, v: &mut BTreeMap<String, Value>) {
            if !n.default.is_null() {
                v.insert(p.into(), n.default.clone());
            }
            for c in &n.children {
                walk(
                    c,
                    &if p.is_empty() {
                        c.name.into()
                    } else {
                        format!("{p}.{}", c.name)
                    },
                    v,
                );
            }
            for c in &n.choices {
                if n.kind == "CHOICE" {
                    walk(c, &format!("{p}.{}", c.name), v);
                }
            }
        }
        let mut values = BTreeMap::new();
        walk(&schema(), "", &mut values);
        Self {
            values,
            coords: vec![],
            tick: 0,
            last_spawn: 0,
            rng: 0xa2785d1e9b036c4f,
        }
    }
}
impl CustomAmbienceSetting {
    pub fn apply(self, s: &mut CustomAmbience) {
        if s.values.get(&self.0).is_none_or(|v| v == &self.1) {
            return;
        }
        if self.0 == "WorldParticles.Mode"
            || self.0 == "WorldParticles.Mode.Simple.Life"
            || (self.0 == "WorldParticles.Enabled" && self.1 == false)
        {
            s.clear();
        }
        s.values.insert(self.0, self.1);
    }
}
const SIMPLE: &str = "WorldParticles.Mode.Simple";
pub fn rgba(argb: i32) -> [f32; 4] {
    let [a, r, g, b] = (argb as u32).to_be_bytes();
    [r, g, b, a].map(|v| v as f32 / 255.)
}
pub fn vec3_hash(position: [f64; 3]) -> i32 {
    position.into_iter().enumerate().fold(0i32, |h, (i, x)| {
        let bits = if x.is_nan() {
            0x7ff8000000000000
        } else {
            x.to_bits()
        };
        let v = (bits ^ (bits >> 32)) as i32;
        if i == 0 {
            v
        } else {
            h.wrapping_mul(31).wrapping_add(v)
        }
    })
}
const SINES: &[u8; 262144] = include_bytes!("../vendor/minecraft-math/sine-table.bin");
fn fast_trig(x: f64, cos: bool) -> f32 {
    let index =
        ((x * 10430.378350470453 + if cos { 16384. } else { 0. }) as i64 & 65535) as usize * 4;
    f32::from_le_bytes(SINES[index..index + 4].try_into().unwrap())
}
impl CustomAmbience {
    pub fn value(&self, p: &str) -> Value {
        self.values.get(p).cloned().unwrap_or_default()
    }
    pub fn text(&self, p: &str) -> &str {
        self.values.get(p).and_then(Value::as_str).unwrap_or("")
    }
    pub fn number(&self, p: &str) -> f32 {
        self.values.get(p).and_then(Value::as_f64).unwrap_or(0.) as f32
    }
    pub fn boolean(&self, p: &str) -> bool {
        self.values.get(p).and_then(Value::as_bool).unwrap_or(false)
    }
    pub fn color(&self, p: &str) -> [f32; 4] {
        rgba(self.value(p).as_i64().unwrap_or(0) as i32)
    }
    fn prop(&self, p: &str) -> String {
        format!("{SIMPLE}.{p}")
    }
    pub fn clear(&mut self) {
        self.coords.clear();
    }
    pub fn game_tick(&mut self) {
        if self.boolean("WorldParticles.Enabled") {
            self.tick = self.tick.wrapping_add(1);
            self.coords.retain(|p| p.expiration > self.tick);
        }
    }
    pub fn time(&self, original: i64, overworld: bool) -> i64 {
        if !overworld {
            return original;
        }
        match self.text("Time") {
            "Dawn" => 23041,
            "Day" => 1000,
            "Noon" => 6000,
            "Dusk" => 12610,
            "Night" => 13000,
            "MidNight" => 18000,
            _ => original,
        }
    }
    pub fn weather(&self, original: [f32; 2]) -> [f32; 2] {
        match self.text("Weather") {
            "Sunny" => [0., 0.],
            "Rainy" => [1., 0.],
            "Snowy" => [0.9, 0.],
            "Thunder" => [1., 1.],
            _ => original,
        }
    }
    pub fn precipitation(&self, original: f32) -> f32 {
        let intensity = if self.text("Weather") == "Snowy" {
            0.
        } else {
            original
        };
        if intensity != 0. && self.boolean("ModifyPrecipitation.Enabled") {
            self.number("ModifyPrecipitation.Gradient")
        } else {
            intensity
        }
    }
    pub fn settings(&self) -> Value {
        let fog=self.boolean("Fog.Enabled").then(||json!({"color":self.boolean("Fog.FogColorOverride.Enabled").then(||self.color("Fog.FogColorOverride.Color")),"background":self.color("Fog.BackgroundColor"),"disable":self.boolean("Fog.DisableWorldFog"),"environmental":self.value("Fog.Environmental"),"render_distance":self.value("Fog.RenderDistance"),"sky_end":self.number("Fog.SkyEnd"),"cloud_end":self.number("Fog.CloudEnd")}));
        let lightmap = self.boolean("CustomLightmap.Enabled").then(|| {
            let mode = self.text("CustomLightmap.Mode");
            if mode == "SingleColor" {
                json!({"single":self.color("CustomLightmap.Mode.SingleColor.Color")})
            } else {
                let mut v = json!({});
                for (p, k) in [
                    ("BlockLightTint", "block"),
                    ("SkyLightColor", "sky"),
                    ("AmbientColor", "ambient"),
                    ("NightVisionColor", "night"),
                ] {
                    let c = self.color(&format!("CustomLightmap.Mode.Custom.{p}"));
                    if c[3] != 0. {
                        v[k] = json!([c[0], c[1], c[2]]);
                    }
                }
                v
            }
        });
        json!({"weather":self.text("Weather"),"time":self.text("Time"),"precipitation":self.boolean("ModifyPrecipitation.Enabled").then(||self.number("ModifyPrecipitation.Gradient")),"fog":fog,"lightmap":lightmap,"sky_color":self.boolean("SkyColor.Enabled").then(||self.color("SkyColor.Color"))})
    }
    fn random(&mut self) -> f64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        (x >> 11) as f64 / 9007199254740992.
    }
    pub fn spawn_with(&mut self, now: i64, player: [f64; 3], random: [f64; 3]) -> bool {
        if self.coords.len() >= self.number(&self.prop("Count")) as usize {
            return false;
        }
        self.last_spawn = now;
        let radius = self.value(&self.prop("Radius"));
        let min = radius["from"].as_f64().unwrap_or(8.);
        let max = radius["to"].as_f64().unwrap_or(16.);
        if min >= max {
            return false;
        }
        let u = -1. + 2. * random[0];
        let theta = std::f64::consts::TAU * random[1];
        let s = (1. - u * u).sqrt();
        let direction = [
            s * f64::from(fast_trig(theta, true)),
            u,
            s * f64::from(fast_trig(theta, false)),
        ];
        let min = min.powi(3);
        let max = max.powi(3);
        let distance = (min + random[2] * (max - min)).cbrt();
        let position = std::array::from_fn(|i| distance.mul_add(direction[i], player[i]));
        let ttl = self.value(&self.prop("Life"))["to"].as_i64().unwrap_or(30) as i32;
        self.coords.push(Particle {
            position,
            expiration: self.tick.wrapping_add(ttl),
        });
        true
    }
    pub fn frame(&mut self, input: &Value) -> Value {
        let mut output = self.settings();
        output["groups"] = json!([]);
        let mut images = vec![];
        if !self.boolean("WorldParticles.Enabled") {
            output["images"] = json!(images);
            return output;
        }
        let Ok(player) = serde_json::from_value::<[f64; 3]>(input["player"].clone()) else {
            output["images"] = json!(images);
            return output;
        };
        let now = input["now"].as_i64().unwrap_or(0);
        if now.wrapping_sub(self.last_spawn) >= self.number(&self.prop("SpawnTime")) as i64
            && self.coords.len() < self.number(&self.prop("Count")) as usize
        {
            let random = std::array::from_fn(|_| self.random());
            self.spawn_with(now, player, random);
        }
        let partial = input["partial"].as_f64().unwrap_or(0.) as f32;
        let right =
            serde_json::from_value::<[f32; 3]>(input["right"].clone()).unwrap_or([1., 0., 0.]);
        let up = serde_json::from_value::<[f32; 3]>(input["up"].clone()).unwrap_or([0., 1., 0.]);
        let life = self.value(&self.prop("Life"));
        let animation = life["from"].as_f64().unwrap_or(20.) as f32;
        let lifetime = life["to"].as_i64().unwrap_or(30) as i32;
        let ease = Easing::ALL
            .into_iter()
            .find(|v| v.tag() == self.text(&self.prop("AnimCurve")))
            .unwrap_or(Easing::QuadInOut);
        for particle in &self.coords {
            let age =
                lifetime.wrapping_sub(particle.expiration.wrapping_sub(self.tick)) as f32 + partial;
            let progress = ease.transform(age / animation).clamp(0., 1.);
            let mut position = particle.position;
            if self.boolean(&self.prop("YMotion.Enabled")) {
                let factor = if self.text(&self.prop("YMotion.AnimBy")) == "Age" {
                    age
                } else {
                    progress
                };
                position[1] += f64::from(self.number(&self.prop("YMotion.Motion")) * factor);
            }
            if self.boolean(&self.prop("Wander.Enabled")) {
                let phase = (vec3_hash(particle.position) & 255) as f32;
                let angle = age * 0.05 * self.number(&self.prop("Wander.Speed")) + phase;
                let distance = self.number(&self.prop("Wander.Distance"));
                position[0] += f64::from((f64::from(angle).sin() as f32) * distance);
                position[1] += f64::from((f64::from(angle * 0.7).sin() as f32) * distance * 0.5);
                position[2] += f64::from((f64::from(angle * 1.3).cos() as f32) * distance);
            }
            let half = self.number(&self.prop("Size")) * progress * 0.5;
            let corners = [[-half, half], [-half, -half], [half, -half], [half, half]].map(|p| {
                std::array::from_fn::<_, 3, _>(|i| {
                    position[i] + f64::from(right[i] * p[0] + up[i] * p[1])
                })
            });
            images.push(json!({"corners":corners,"color":self.color(&self.prop("Color.Color")),"texture":PARTICLE.iter().position(|v|*v==self.text(&self.prop("Particle"))).unwrap_or(10),"depth":self.boolean(&self.prop("CanBeCovered")),"age":age,"progress":progress}));
        }
        let texture = format!("builtin:{}", self.text(&self.prop("Particle")));
        let quads = images
            .iter()
            .map(|image| {
                let mut q = image.clone();
                q["texture"] = json!(0);
                q
            })
            .collect::<Vec<_>>();
        output["groups"] = json!([{ "slot":0,"textures":[texture.clone(),texture],"allow_load":[true,true],"depth":self.boolean(&self.prop("CanBeCovered")),"quads":quads }]);
        output["images"] = json!(images);
        output
    }
}
