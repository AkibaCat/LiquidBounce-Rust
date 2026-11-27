// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared TargetRenderer, LiquidBounce ae0eb396, CCBlueX 2015-2026.
use serde_json::{Value, json};
use std::collections::BTreeMap;
#[path = "tracers_curve.rs"]
mod curve;
pub const MODES: [&str; 8] = [
    "Legacy",
    "Circle",
    "Image",
    "GlowingCircle",
    "Ghost",
    "Hearts",
    "Text2D",
    "Arrow",
];
pub const HEIGHT_MODES: [&str; 5] = ["Feet", "Top", "Relative", "Health", "Animated"];
const BLUE: i32 = 0xff4677ffu32 as i32;
const DEFAULT_COLOR: i32 = 0x644677ff;
const SINES: &[u8; 262144] = include_bytes!("../vendor/minecraft-math/sine-table.bin");
fn fast_sin(x: f32) -> f32 {
    let i = ((f64::from(x) * 10430.378350470453) as i64 & 65535) as usize * 4;
    f32::from_le_bytes(SINES[i..i + 4].try_into().unwrap())
}
fn fast_cos(x: f32) -> f32 {
    let i = ((f64::from(x) * 10430.378350470453 + 16384.) as i64 & 65535) as usize * 4;
    f32::from_le_bytes(SINES[i..i + 4].try_into().unwrap())
}

#[derive(Clone, Debug)]
pub struct Node {
    pub name: &'static str,
    pub kind: &'static str,
    pub default: Value,
    pub range: Option<[f32; 2]>,
    pub suffix: &'static str,
    pub children: Vec<Node>,
    pub choices: Vec<Node>,
}
fn leaf(name: &'static str, kind: &'static str, value: Value) -> Node {
    Node {
        name,
        kind,
        default: value,
        range: None,
        suffix: "",
        children: vec![],
        choices: vec![],
    }
}
fn float(name: &'static str, v: f32, a: f32, b: f32) -> Node {
    let mut n = leaf(name, "FLOAT", json!(v));
    n.range = Some([a, b]);
    n
}
fn int(name: &'static str, v: i32, a: i32, b: i32) -> Node {
    let mut n = leaf(name, "INT", json!(v));
    n.range = Some([a as f32, b as f32]);
    n
}
fn color(name: &'static str, v: i32) -> Node {
    leaf(name, "COLOR", json!(v))
}
fn boolean(name: &'static str, v: bool) -> Node {
    leaf(name, "BOOLEAN", json!(v))
}
fn group(name: &'static str, children: Vec<Node>) -> Node {
    let mut n = leaf(name, "CONFIGURABLE", Value::Null);
    n.children = children;
    n
}
fn choice(name: &'static str, selected: &str, choices: Vec<Node>) -> Node {
    let mut n = leaf(name, "CHOICE", json!(selected));
    n.choices = choices;
    n
}
fn height() -> Node {
    choice(
        "HeightMode",
        "Feet",
        vec![
            group("Feet", vec![float("Offset", 0., -1., 1.)]),
            group("Top", vec![float("Offset", 0., -1., 1.)]),
            group("Relative", vec![float("Height", 0.5, -0.5, 1.5)]),
            group("Health", vec![]),
            group(
                "Animated",
                vec![
                    float("Speed", 0.18, 0.01, 1.),
                    float("HeightMultiplier", 0.4, 0.1, 1.),
                    float("HeightOffset", 1.3, 0., 2.),
                    float("GlowOffset", -1., -3.1, 3.1),
                ],
            ),
        ],
    )
}
pub fn schema() -> Node {
    let mut period = int("Period", 1000, 10, 20000);
    period.suffix = "ms";
    let mut speed = float("Speed", 35., -360., 360.);
    speed.suffix = "deg/s";
    let mut file = leaf("File", "FILE", json!(""));
    file.suffix = "png,jpg";
    let mut preset = choice(
        "Preset",
        "Marker1",
        vec![group("Marker1", vec![]), group("Marker2", vec![])],
    );
    preset.kind = "CHOOSE";
    let mut n = group(
        "TargetRendering",
        vec![
            boolean("Enabled", true),
            choice(
                "Mode",
                "GlowingCircle",
                vec![
                    group(
                        "Legacy",
                        vec![
                            float("Size", 0.5, 0.1, 2.),
                            float("Height", 0.1, 0.02, 2.),
                            color("Color", DEFAULT_COLOR),
                            float("ExtraYOffset", 0.1, 0., 1.),
                        ],
                    ),
                    group(
                        "Circle",
                        vec![
                            float("Radius", 0.85, 0.1, 2.),
                            float("InnerRadius", 0., 0., 2.),
                            height(),
                            color("OuterColor", DEFAULT_COLOR),
                            color("InnerColor", DEFAULT_COLOR),
                            color("Color", 0xff007cffu32 as i32),
                        ],
                    ),
                    group(
                        "Image",
                        vec![
                            choice(
                                "Source",
                                "Custom",
                                vec![group("Custom", vec![file]), group("Builtin", vec![preset])],
                            ),
                            leaf("Scale", "VECTOR2_F", json!({"x":1.0,"y":1.0})),
                            color("ColorModulator", -1),
                            group(
                                "Rotate",
                                vec![
                                    period,
                                    boolean("Symmetric", true),
                                    leaf(
                                        "Curve",
                                        "CURVE",
                                        json!([{"x":0.0,"y":0.0},{"x":1.0,"y":0.0}]),
                                    ),
                                ],
                            ),
                            height(),
                        ],
                    ),
                    group(
                        "GlowingCircle",
                        vec![
                            float("Radius", 0.85, 0.1, 2.),
                            height(),
                            color("OuterColor", DEFAULT_COLOR),
                            color("GlowColor", BLUE & 0xffffff),
                            float("GlowHeight", 0.3, -1., 1.),
                            color("Color", 0xff007cffu32 as i32),
                        ],
                    ),
                    group(
                        "Ghost",
                        vec![
                            color("Color", 0xff0000ffu32 as i32),
                            float("Size", 0.5, 0.4, 0.7),
                            int("Length", 25, 15, 40),
                        ],
                    ),
                    group(
                        "Hearts",
                        vec![
                            color("Color", 0xb4ffffffu32 as i32),
                            boolean("DynamicCount", true),
                            int("HeartCount", 10, 1, 32),
                            float("YOffset", 0.1, -1., 3.),
                            float("Size", 0.15, 0.05, 1.),
                            group(
                                "Orbit",
                                vec![
                                    float("Radius", 0.5, 0.1, 1.),
                                    speed,
                                    float("SqueezeStrength", 0.25, 0., 1.),
                                    int("SqueezeSpeed", 2, 1, 4),
                                ],
                            ),
                            boolean("CanBeCovered", false),
                        ],
                    ),
                    group(
                        "Text2D",
                        vec![
                            float("Scale", 1., 0.01, 10.),
                            boolean("Shadow", true),
                            color("Color", 0xffff0000u32 as i32),
                            leaf("Text", "MUTABLE_LIST", json!(["TARGET"])),
                            height(),
                        ],
                    ),
                    group(
                        "Arrow",
                        vec![
                            color("Color", 0xffff0000u32 as i32),
                            color("OutlineColor", 0),
                            float("Size", 1.5, 0.5, 20.),
                        ],
                    ),
                ],
            ),
        ],
    );
    n.kind = "TOGGLEABLE";
    n
}
#[derive(Clone, Debug, PartialEq)]
pub struct TargetRendererSettings {
    pub values: BTreeMap<String, Value>,
}
impl Default for TargetRendererSettings {
    fn default() -> Self {
        fn walk(n: &Node, p: &str, out: &mut BTreeMap<String, Value>) {
            if !n.default.is_null() {
                out.insert(p.into(), n.default.clone());
            }
            for c in &n.children {
                walk(
                    c,
                    &if p.is_empty() {
                        c.name.into()
                    } else {
                        format!("{p}.{}", c.name)
                    },
                    out,
                );
            }
            for c in &n.choices {
                walk(c, &format!("{p}.{}", c.name), out);
            }
        }
        let mut values = BTreeMap::new();
        walk(&schema(), "", &mut values);
        Self { values }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct TargetRendererSetting(pub String, pub Value);
impl TargetRendererSetting {
    pub fn apply(self, s: &mut TargetRendererSettings) {
        let mut value = self.1;
        if self.0 == "Mode.Circle.InnerRadius" {
            if let Some(v) = value.as_f64() {
                value = json!(v.min(s.number("Mode.Circle.Radius")));
            }
        }
        s.values.insert(self.0, value);
    }
}
impl TargetRendererSettings {
    pub fn value(&self, p: &str) -> &Value {
        self.values.get(p).unwrap_or(&Value::Null)
    }
    pub fn number(&self, p: &str) -> f64 {
        let value = self.value(p);
        let number = value.as_f64().unwrap_or(0.);
        if value.is_i64() || value.is_u64() {
            number
        } else {
            f64::from(number as f32)
        }
    }
    pub fn boolean(&self, p: &str) -> bool {
        self.value(p).as_bool().unwrap_or(false)
    }
    pub fn text(&self, p: &str) -> &str {
        self.value(p).as_str().unwrap_or("")
    }
    pub fn color(&self, p: &str) -> i32 {
        self.value(p).as_i64().unwrap_or(0) as i32
    }
    pub fn mode(&self) -> &str {
        self.text("Mode")
    }
    pub fn height(&self, p: &str, e: &Entity, partial: f32, glow: bool) -> f64 {
        let n = |leaf: &str| self.number(&format!("{p}.{leaf}"));
        match self.text(p) {
            "Feet" => n("Feet.Offset"),
            "Top" => f64::from(e.height) + n("Top.Offset"),
            "Relative" => n("Relative.Height") * f64::from(e.height),
            "Health" => {
                if e.living {
                    f64::from(e.health / e.max_health) * f64::from(e.height)
                } else {
                    0.
                }
            }
            "Animated" => {
                let t = (e.tick_count as f32 + partial) * n("Animated.Speed") as f32
                    + if glow {
                        n("Animated.GlowOffset") as f32
                    } else {
                        0.
                    };
                f64::from(t).sin() * n("Animated.HeightMultiplier") + n("Animated.HeightOffset")
            }
            _ => 0.,
        }
    }
    pub fn rotation(&self, elapsed: u64) -> f32 {
        let p = self.number("Mode.Image.Rotate.Period").max(1.) as u64;
        let mut x = (elapsed
            % if self.boolean("Mode.Image.Rotate.Symmetric") {
                p * 2
            } else {
                p
            }) as f32
            / p as f32;
        if x > 1. {
            x = 2. - x;
        }
        let points = self
            .value("Mode.Image.Rotate.Curve")
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| Some([v["x"].as_f64()? as f32, v["y"].as_f64()? as f32]))
            .collect::<Vec<_>>();
        curve::sample(&points, x)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Entity {
    pub id: i32,
    pub position: [f64; 3],
    pub interpolated: [f64; 3],
    pub height: f32,
    pub health: f32,
    pub max_health: f32,
    pub absorption: f32,
    pub hurt_time: i32,
    pub tick_count: i32,
    pub living: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub partial: f32,
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
}
#[derive(Clone, Debug)]
pub struct Quad {
    pub corners: [[f64; 3]; 4],
    pub colors: [[f32; 4]; 4],
    pub uv: [[f32; 2]; 4],
    pub shape: u32,
    pub inner: f32,
    pub inner_color: [f32; 4],
    pub texture: String,
    pub depth: bool,
}
#[derive(Clone, Debug)]
pub struct Overlay {
    pub position: [f64; 3],
    pub text: Vec<String>,
    pub scale: f32,
    pub shadow: bool,
    pub color: i32,
    pub outline: i32,
    pub arrow: bool,
}
#[derive(Clone, Default, Debug)]
pub struct Frame {
    pub quads: Vec<Quad>,
    pub overlay: Vec<Overlay>,
}
fn rgba(c: i32) -> [f32; 4] {
    let c = c as u32;
    [
        (c >> 16 & 255) as f32 / 255.,
        (c >> 8 & 255) as f32 / 255.,
        (c & 255) as f32 / 255.,
        (c >> 24) as f32 / 255.,
    ]
}
fn plus(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
fn quad(corners: [[f64; 3]; 4], color: i32, shape: u32) -> Quad {
    Quad {
        corners,
        colors: [rgba(color); 4],
        uv: [[0., 0.], [0., 1.], [1., 1.], [1., 0.]],
        shape,
        inner: 0.,
        inner_color: rgba(color),
        texture: String::new(),
        depth: false,
    }
}
fn horizontal(at: [f64; 3], radius: f64, color: i32, shape: u32) -> Quad {
    quad(
        [
            [-radius, 0., -radius],
            [-radius, 0., radius],
            [radius, 0., radius],
            [radius, 0., -radius],
        ]
        .map(|v| plus(at, v)),
        color,
        shape,
    )
}
fn plane(at: [f64; 3], xy: [[f64; 2]; 4], right: [f32; 3], up: [f32; 3]) -> [[f64; 3]; 4] {
    xy.map(|v| {
        std::array::from_fn(|i| at[i] + v[0] * f64::from(right[i]) + v[1] * f64::from(up[i]))
    })
}
#[derive(Clone, Debug)]
pub struct TargetRendererRuntime {
    target: i32,
    last: u64,
    flash: f32,
    squeeze: f32,
    size: f32,
    layout: Vec<[f32; 2]>,
    random: u64,
}
impl Default for TargetRendererRuntime {
    fn default() -> Self {
        Self {
            target: -1,
            last: 0,
            flash: 0.,
            squeeze: 0.,
            size: 0.,
            layout: vec![],
            random: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0x9e3779b97f4a7c15, |d| d.as_nanos() as u64 | 1),
        }
    }
}
impl TargetRendererRuntime {
    fn random(&mut self) -> f32 {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 7;
        self.random ^= self.random << 17;
        (self.random >> 40) as f32 / (1u32 << 24) as f32
    }
    pub fn render(
        &mut self,
        s: &TargetRendererSettings,
        e: &Entity,
        c: &Camera,
        now_ms: u64,
        elapsed_ms: u64,
    ) -> Frame {
        let mut f = Frame::default();
        if !s.boolean("Enabled") {
            return f;
        }
        let p = format!("Mode.{}", s.mode());
        let n = |v: &str| s.number(&format!("{p}.{v}"));
        let color = |v: &str| s.color(&format!("{p}.{v}"));
        let hp = format!("{p}.HeightMode");
        let h = s.height(&hp, e, c.partial, false);
        let at = plus(e.interpolated, [0., h, 0.]);
        match s.mode() {
            "Legacy" => {
                let r = n("Size");
                let pos = plus(
                    e.interpolated,
                    [0., f64::from(e.height) + n("ExtraYOffset"), 0.],
                );
                let h = n("Height");
                let b = [
                    [-r, 0., -r],
                    [r, 0., -r],
                    [r, 0., r],
                    [-r, 0., r],
                    [-r, h, -r],
                    [r, h, -r],
                    [r, h, r],
                    [-r, h, r],
                ]
                .map(|v| plus(pos, v));
                for face in [
                    [0, 1, 2, 3],
                    [4, 7, 6, 5],
                    [0, 4, 5, 1],
                    [1, 5, 6, 2],
                    [2, 6, 7, 3],
                    [3, 7, 4, 0],
                ] {
                    f.quads.push(quad(face.map(|i| b[i]), color("Color"), 0));
                }
            }
            "Circle" => {
                let mut q = horizontal(at, n("Radius"), color("OuterColor"), 2);
                q.inner = (n("InnerRadius") / n("Radius")).clamp(0., 1.) as f32;
                q.inner_color = rgba(color("InnerColor"));
                f.quads.push(q);
                f.quads.push(horizontal(at, n("Radius"), color("Color"), 3));
            }
            "GlowingCircle" => {
                let radius = n("Radius");
                let glow = if s.text(&hp) == "Animated" {
                    s.height(&hp, e, c.partial, true) - h
                } else {
                    n("GlowHeight")
                };
                if glow.abs() > 1e-5 {
                    let count = 40;
                    for i in 0..count {
                        let a = i as f32 * std::f32::consts::TAU / count as f32;
                        let b = (i + 1) as f32 * std::f32::consts::TAU / count as f32;
                        let at_angle = |a: f32, y: f64| {
                            plus(
                                at,
                                [
                                    f64::from(fast_cos(a) * radius as f32),
                                    y,
                                    f64::from(fast_sin(a) * radius as f32),
                                ],
                            )
                        };
                        let mut q = quad(
                            [
                                at_angle(a, 0.),
                                at_angle(a, glow),
                                at_angle(b, glow),
                                at_angle(b, 0.),
                            ],
                            color("OuterColor"),
                            0,
                        );
                        q.colors[1] = rgba(color("GlowColor"));
                        q.colors[2] = q.colors[1];
                        f.quads.push(q);
                    }
                } else {
                    let mut q = horizontal(at, radius, color("OuterColor"), 2);
                    q.inner = 1.;
                    q.inner_color = rgba(color("GlowColor"));
                    f.quads.push(q);
                }
                f.quads.push(horizontal(at, radius, color("OuterColor"), 2));
                f.quads.push(horizontal(at, radius, color("Color"), 3));
            }
            "Image" => {
                let source = s.text("Mode.Image.Source");
                let texture = if source == "Builtin" {
                    format!("builtin:{}", s.text("Mode.Image.Source.Builtin.Preset"))
                } else {
                    s.text("Mode.Image.Source.Custom.File").into()
                };
                let scale = s.value("Mode.Image.Scale");
                let x = f64::from(scale["x"].as_f64().unwrap_or(1.) as f32);
                let y = f64::from(scale["y"].as_f64().unwrap_or(1.) as f32);
                let a = f64::from(s.rotation(elapsed_ms).to_radians());
                let xy = [[-0.5, -0.5], [-0.5, 0.5], [0.5, 0.5], [0.5, -0.5]].map(|v| {
                    let vx = v[0] * x;
                    let vy = v[1] * y;
                    [vx * a.cos() - vy * a.sin(), vx * a.sin() + vy * a.cos()]
                });
                let mut q = quad(plane(at, xy, c.right, c.up), color("ColorModulator"), 1);
                q.texture = texture;
                f.quads.push(q);
            }
            "Ghost" => {
                let size = n("Size");
                let length = n("Length").max(0.) as usize;
                let base = plus(e.interpolated, [0.2 - size / 2., 1.25 - size / 2., 0.]);
                let yaw = f64::from(-c.yaw.to_radians());
                let pitch = f64::from(c.pitch.to_radians());
                let right = [yaw.cos() as f32, 0., -yaw.sin() as f32];
                let up = [
                    (yaw.sin() * pitch.sin()) as f32,
                    pitch.cos() as f32,
                    (yaw.cos() * pitch.sin()) as f32,
                ];
                for stream in 0..3 {
                    for i in 0..length {
                        let angle = f64::from(0.15f32)
                            * (elapsed_ms as f64 - i as f64 * (10. + length as f64 * 0.2))
                            / 30.;
                        let sn = angle.sin() * 0.67;
                        let cs = angle.cos() * 0.67;
                        let offset = match stream {
                            0 => [sn, cs, -cs],
                            1 => [-sn, sn, -cs],
                            _ => [-sn, -sn, cs],
                        };
                        let corners = plane(
                            plus(base, offset),
                            [
                                [-size / 2., size],
                                [-size / 2., 0.],
                                [size / 2., 0.],
                                [size / 2., size],
                            ],
                            right,
                            up,
                        );
                        let alpha = ((color("Color") as u32 >> 24) as i32 - i as i32 * 15)
                            .clamp(0, (color("Color") as u32 >> 24) as i32);
                        let mut q = quad(corners, color("Color") & 0xffffff | (alpha << 24), 1);
                        q.texture = "builtin:Glow".into();
                        q.depth = true;
                        f.quads.push(q);
                    }
                }
            }
            "Hearts" => self.hearts(s, e, now_ms, &mut f),
            "Text2D" | "Arrow" => {
                let arrow = s.mode() == "Arrow";
                f.overlay.push(Overlay {
                    position: if arrow {
                        plus(e.interpolated, [0., f64::from(e.height), 0.])
                    } else {
                        at
                    },
                    text: s
                        .value("Mode.Text2D.Text")
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect(),
                    scale: if arrow {
                        n("Size") as f32
                    } else {
                        n("Scale") as f32
                    },
                    shadow: s.boolean("Mode.Text2D.Shadow"),
                    color: color("Color"),
                    outline: color("OutlineColor"),
                    arrow,
                });
            }
            _ => {}
        }
        f
    }
    fn hearts(&mut self, s: &TargetRendererSettings, e: &Entity, now: u64, f: &mut Frame) {
        if !e.living {
            return;
        }
        let n = |v: &str| s.number(&format!("Mode.Hearts.{v}")) as f32;
        let mut slots = vec![];
        let mut add = |amount: f32, absorption: bool| {
            let hearts = amount.max(0.) / 2.;
            for _ in 0..hearts as usize {
                slots.push((1., absorption));
            }
            let partial = hearts - hearts as usize as f32;
            if partial > 0. {
                slots.push((partial, absorption));
            }
        };
        add(
            if s.boolean("Mode.Hearts.DynamicCount") {
                e.health
            } else {
                n("HeartCount") * 2.
            },
            false,
        );
        add(e.absorption, true);
        let mut delta = if self.last != 0 {
            ((now.saturating_sub(self.last)) as f32 / 1000.).min(0.25)
        } else {
            0.
        };
        self.last = now;
        let size = n("Size");
        if self.target != e.id {
            self.target = e.id;
            self.flash = 0.;
            self.squeeze = 0.;
            delta = 0.;
            self.layout.clear();
        }
        if self.size != size {
            self.size = size;
            self.layout.clear();
        }
        self.flash = if (8..=10).contains(&e.hurt_time) {
            1.
        } else {
            (self.flash - delta * 3.5).max(0.)
        };
        let speed = n("Orbit.SqueezeSpeed") as i32;
        self.squeeze += if ((5 + speed)..=10).contains(&e.hurt_time) {
            (delta * n("Orbit.SqueezeStrength") * 5.).max(0.)
        } else if ((1 + speed)..=(4 + speed)).contains(&e.hurt_time) {
            -(delta * n("Orbit.SqueezeStrength") * 5.).min(self.squeeze)
        } else {
            -self.squeeze
        };
        let mut attempts = slots.len().saturating_mul(24).max(64);
        while self.layout.len() < slots.len() && attempts > 0 {
            attempts -= 1;
            let candidate = [self.random() * 360., self.random()];
            if self.layout.iter().all(|a| {
                let d = (a[0] - candidate[0]).abs();
                d.min(360. - d) >= size * 115. || (a[1] - candidate[1]).abs() >= size * 2.
            }) {
                self.layout.push(candidate);
            }
        }
        while self.layout.len() < slots.len() {
            let a = [self.random() * 360., self.random()];
            self.layout.push(a);
        }
        for (i, (fill, absorption)) in slots.into_iter().enumerate() {
            let layout = self.layout[i];
            let a = (f64::from(layout[0]) + now as f64 / 1000. * f64::from(n("Orbit.Speed")))
                .to_radians();
            let radius = n("Orbit.Radius");
            let distance = f64::from((radius - self.squeeze).clamp(0.05, radius));
            let offset = [
                a.cos() * distance,
                f64::from(n("YOffset")) + f64::from(e.height) * f64::from(layout[1]),
                a.sin() * distance,
            ];
            let pos = plus(e.interpolated, offset);
            let yaw = (-offset[0]).atan2(-offset[2]);
            let right = [yaw.cos() as f32, 0., -yaw.sin() as f32];
            let up = [0., 1., 0.];
            let base = s.color("Mode.Hearts.Color");
            let mut color = rgba(if absorption {
                (base & 0xff000000u32 as i32) | 0xffd648
            } else {
                base
            });
            for (channel, target) in color[..3].iter_mut().zip([1., 0., 0.]) {
                let base = *channel * 255.;
                *channel = self
                    .flash
                    .mul_add(target * 255. - base, base)
                    .clamp(0., 255.) as i32 as f32
                    / 255.;
            }
            for (fill, alpha) in [
                (1., (color[3] * 255. * 0.25) as i32 as f32 / 255.),
                (fill, color[3]),
            ] {
                if fill <= 0. {
                    continue;
                }
                let hw = f64::from(size * 1.0938363);
                let y = f64::from(size);
                let right_x = -hw + 2. * hw * f64::from(fill);
                let mut q = quad(
                    plane(
                        pos,
                        [[-hw, -y], [-hw, y], [right_x, y], [right_x, -y]],
                        right,
                        up,
                    ),
                    base,
                    4,
                );
                q.colors = [[color[0], color[1], color[2], alpha]; 4];
                q.uv = [[0., 0.], [0., 1.], [fill, 1.], [fill, 0.]];
                q.depth = s.boolean("Mode.Hearts.CanBeCovered");
                f.quads.push(q);
            }
        }
    }
}
impl Frame {
    pub fn json(&self) -> Value {
        json!({"quads":self.quads.iter().map(|q|json!({"corners":q.corners,"colors":q.colors,"uv":q.uv,"shape":q.shape,"inner":q.inner,"inner_color":q.inner_color,"texture":q.texture,"depth":q.depth})).collect::<Vec<_>>(),"overlay":self.overlay.iter().map(|o|json!({"position":o.position,"text":o.text,"scale":o.scale,"shadow":o.shadow,"color":o.color,"outline":o.outline,"arrow":o.arrow})).collect::<Vec<_>>()})
    }
}
