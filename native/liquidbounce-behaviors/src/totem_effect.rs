// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleTotemEffect, both modes and standing WireframePlayer, LiquidBounce ae0eb396.
use crate::{
    breadcrumbs::{Effect, STYLES, StyleSettings, specs},
    easing::Easing,
    target_renderer::Node,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub const MODES: [&str; 2] = ["Shockwave", "Soul"];
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
fn ranged(name: &'static str, kind: &'static str, default: Value, a: f32, b: f32) -> Node {
    let mut n = leaf(name, kind, default);
    n.range = Some([a, b]);
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
            leaf("SyncColors", "BOOLEAN", json!(false)),
            leaf("InnerColor", "COLOR", json!(0xff2cff00u32 as i32)),
            leaf("OuterColor", "COLOR", json!(0xffafff00u32 as i32)),
        ],
    )
}
fn render() -> Node {
    choice(
        "Render",
        "Plain",
        vec![
            group("Plain", vec![]),
            group(
                "Shader",
                vec![choice(
                    "Style",
                    "Smooth",
                    STYLES
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
                                            let mut n = ranged(
                                                s.name,
                                                "FLOAT",
                                                json!(s.default),
                                                s.from,
                                                s.to,
                                            );
                                            n.suffix = s.suffix;
                                            n
                                        }
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                )],
            ),
        ],
    )
}
pub fn schema() -> Node {
    let common = || {
        vec![
            ranged("Lifetime", "INT", json!(20), 1., 200.),
            ranged("Fade", "FLOAT", json!(0.7), 0., 1.),
            choose(
                "AnimCurve",
                "ExponentialOut",
                Easing::ALL.into_iter().map(Easing::tag),
            ),
            leaf("CanBeCovered", "BOOLEAN", json!(false)),
        ]
    };
    let mut shock = common();
    shock.push(ranged(
        "radius",
        "FLOAT_RANGE",
        json!({"from":2.,"to":2.2}),
        0.1,
        10.,
    ));
    shock.push(colors());
    shock.push(render());
    let mut soul = common();
    soul.push(leaf("Halo", "BOOLEAN", json!(false)));
    let mut motion = group(
        "YMotion",
        vec![
            leaf("Enabled", "BOOLEAN", json!(true)),
            ranged("SoulYMotion", "FLOAT", json!(0.75), 0.1, 10.),
            choose("AnimBy", "Age", ["Progress", "Age"].into_iter()),
        ],
    );
    motion.kind = "TOGGLEABLE";
    soul.push(motion);
    soul.push(colors());
    group(
        "TotemEffect",
        vec![choice(
            "Mode",
            MODES[0],
            vec![group(MODES[0], shock), group(MODES[1], soul)],
        )],
    )
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snapshot {
    pub position: [f64; 3],
    pub x_rot: f32,
    pub y_rot: f32,
    pub height: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pop {
    pub snapshot: Snapshot,
    pub expiration: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TotemEffect {
    pub values: BTreeMap<String, Value>,
    pub tick: i32,
    pub entities: Vec<Pop>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TotemEffectSetting(pub String, pub Value);
impl Default for TotemEffect {
    fn default() -> Self {
        fn walk(n: &Node, p: &str, out: &mut BTreeMap<String, Value>) {
            if !n.default.is_null() {
                out.insert(p.into(), n.default.clone());
            }
            for c in n.children.iter().chain(&n.choices) {
                let path = if p.is_empty() {
                    c.name.into()
                } else {
                    format!("{p}.{}", c.name)
                };
                walk(c, &path, out);
            }
        }
        let mut values = BTreeMap::new();
        walk(&schema(), "", &mut values);
        Self {
            values,
            tick: 0,
            entities: vec![],
        }
    }
}
impl TotemEffectSetting {
    pub fn apply(self, s: &mut TotemEffect) {
        if (self.0 == "Mode" || self.0.ends_with(".Lifetime"))
            && s.values.get(&self.0) != Some(&self.1)
        {
            s.clear();
        }
        s.values.insert(self.0, self.1);
    }
}
fn rgba(c: i32, alpha: i32) -> [f32; 4] {
    let [_, r, g, b] = (c as u32).to_be_bytes();
    [
        r as f32 / 255.,
        g as f32 / 255.,
        b as f32 / 255.,
        alpha as f32 / 255.,
    ]
}
const SINES: &[u8; 262144] = include_bytes!("../vendor/minecraft-math/sine-table.bin");
fn fast_sin(x: f32) -> f32 {
    let i = ((f64::from(x) * 10430.378350470453) as i64 & 65535) as usize * 4;
    f32::from_le_bytes(SINES[i..i + 4].try_into().unwrap())
}
fn fast_cos(x: f32) -> f32 {
    let i = ((f64::from(x) * 10430.378350470453 + 16384.) as i64 & 65535) as usize * 4;
    f32::from_le_bytes(SINES[i..i + 4].try_into().unwrap())
}
fn angle(i: usize, n: usize) -> f32 {
    i as f32 * std::f32::consts::TAU / n as f32
}
#[derive(Default)]
struct Mesh {
    quads: Vec<Value>,
    lines: Vec<Value>,
    order: Vec<[usize; 2]>,
}
impl Mesh {
    fn quad(&mut self, corners: [[f64; 3]; 4], color: [f32; 4]) {
        self.order.push([0, self.quads.len()]);
        let colors = [color; 4];
        self.quads.push(json!({"corners":corners,"colors":colors,"uv":[[0.,0.],[0.,1.],[1.,1.],[1.,0.]],"inner_color":color,"inner":0.,"shape":0}));
    }
    fn line(&mut self, ends: [[f64; 3]; 2], color: [f32; 4]) {
        self.order.push([1, self.lines.len()]);
        let colors = [color; 2];
        self.lines.push(json!({"ends":ends,"colors":colors}));
    }
}
impl TotemEffect {
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
    pub fn active(&self) -> &str {
        self.text("Mode")
    }
    pub fn clear(&mut self) {
        self.entities.clear();
    }
    fn path(&self, n: &str) -> String {
        format!("Mode.{}.{n}", self.active())
    }
    pub fn game_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.entities.retain(|e| e.expiration > self.tick);
    }
    pub fn pop(&mut self, snapshot: Snapshot) {
        let lifetime = self.number(&self.path("Lifetime")) as i32;
        self.entities.push(Pop {
            snapshot,
            expiration: self.tick.wrapping_add(lifetime),
        });
    }
    pub fn effect(&self) -> Option<Effect> {
        if self.active() != MODES[0] || self.text("Mode.Shockwave.Render") != "Shader" {
            return None;
        }
        let name = self.text("Mode.Shockwave.Render.Shader.Style");
        let i = STYLES.iter().position(|n| *n == name).unwrap();
        let mut s = StyleSettings::new(i);
        for spec in specs(i) {
            let p = format!("Mode.Shockwave.Render.Shader.Style.{name}.{}", spec.name);
            if spec.color {
                s.accent = self.value(&p).as_i64().unwrap() as i32;
            } else {
                s.values.insert(spec.name.into(), self.number(&p));
            }
        }
        Some(s.effect(i))
    }
    pub fn geometry(&self, partial: f32) -> Value {
        let p = |n: &str| self.path(n);
        let lifetime = self.number(&p("Lifetime"));
        let fade_start = self.number(&p("Fade"));
        let curve = Easing::ALL
            .into_iter()
            .find(|e| e.tag() == self.text(&p("AnimCurve")))
            .unwrap();
        let mut mesh = Mesh::default();
        for pop in &self.entities {
            let age = (lifetime as i32).wrapping_sub(pop.expiration.wrapping_sub(self.tick)) as f32
                + partial;
            let progress = curve.transform(age / lifetime).clamp(0., 1.);
            let fade = if progress >= fade_start {
                (progress - fade_start) / (1. - fade_start).max(0.001)
            } else {
                0.
            };
            let alpha = ((1. - fade).clamp(0., 1.) * 255.) as i32;
            let inner = rgba(
                self.value(&p("Colors.InnerColor")).as_i64().unwrap() as i32,
                alpha,
            );
            let outer = if self.boolean(&p("Colors.SyncColors")) {
                inner
            } else {
                rgba(
                    self.value(&p("Colors.OuterColor")).as_i64().unwrap() as i32,
                    alpha,
                )
            };
            if self.active() == MODES[0] {
                let radius = self.value(&p("radius"));
                let inside = radius["from"].as_f64().unwrap() as f32 * progress;
                let outside = radius["to"].as_f64().unwrap() as f32 * progress;
                if outside <= 0. {
                    continue;
                }
                let at = |x: f32, z: f32| {
                    [
                        pop.snapshot.position[0] + f64::from(x),
                        pop.snapshot.position[1] + f64::from(pop.snapshot.height) / 2.,
                        pop.snapshot.position[2] + f64::from(z),
                    ]
                };
                mesh.order.push([0, mesh.quads.len()]);
                let colors = [outer; 4];
                mesh.quads.push(json!({"corners":[at(-outside,-outside),at(-outside,outside),at(outside,outside),at(outside,-outside)],"colors":colors,"uv":[[0.,0.],[0.,1.],[1.,1.],[1.,0.]],"inner_color":inner,"inner":(inside/outside).clamp(0.,1.),"shape":2}));
            } else if alpha > 0 {
                let anim = if self.boolean(&p("YMotion.Enabled")) {
                    if self.text(&p("YMotion.AnimBy")) == "Age" {
                        f64::from(age)
                    } else {
                        f64::from(progress)
                    }
                } else {
                    0.
                };
                let mut at = pop.snapshot.position;
                at[1] += anim * f64::from(self.number(&p("YMotion.SoulYMotion")));
                at[2] -= 0.1;
                wireframe(
                    &mut mesh,
                    at,
                    pop.snapshot.x_rot,
                    pop.snapshot.y_rot,
                    inner,
                    outer,
                );
                if self.boolean(&p("Halo")) {
                    halo(&mut mesh, [at[0], at[1] + 2.1, at[2] + 0.125], inner);
                }
            }
        }
        let effect = self.effect().map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity}));
        json!({"quads":mesh.quads,"lines":mesh.lines,"order":mesh.order,"effect":effect,"depth":self.boolean(&p("CanBeCovered"))})
    }
}
fn wireframe(
    mesh: &mut Mesh,
    at: [f64; 3],
    pitch: f32,
    yaw: f32,
    fill: [f32; 4],
    outline: [f32; 4],
) {
    let mut yaw = yaw % 360.;
    if yaw >= 180. {
        yaw -= 360.;
    }
    if yaw < -180. {
        yaw += 360.;
    }
    let y = -yaw * (std::f32::consts::PI / 180.);
    let (sy, cy) = (f64::from(y).sin() as f32, f64::from(y).cos() as f32);
    let x = pitch * (std::f32::consts::PI / 180.);
    let (sx, cx) = (f64::from(x).sin() as f32, f64::from(x).cos() as f32);
    for (index, (min, max)) in [
        ([-0.125, 0., 0.], [0., 0.375, 0.125]),
        ([0., 0., 0.], [0.125, 0.375, 0.125]),
        ([-0.125, 0.375, 0.], [0.125, 0.75, 0.125]),
        ([-0.25, 0.375, 0.], [-0.125, 0.75, 0.125]),
        ([0.125, 0.375, 0.], [0.25, 0.75, 0.125]),
        ([-0.125, 0.75, -0.0625], [0.125, 1., 0.1875]),
    ]
    .into_iter()
    .enumerate()
    {
        let pivot = [(min[0] + max[0]) * 0.5, min[1], (min[2] + max[2]) * 0.5];
        let points: [[f64; 3]; 8] = std::array::from_fn(|i| {
            let mut p: [f32; 3] = std::array::from_fn(|axis| {
                if i & (1 << axis) == 0 {
                    min[axis]
                } else {
                    max[axis]
                }
            });
            if index == 5 {
                let dy = p[1] - pivot[1];
                let dz = p[2] - pivot[2];
                p[1] = pivot[1] + cx * dy - sx * dz;
                p[2] = pivot[2] + sx * dy + cx * dz;
            }
            let p = p.map(|x| x * 1.9);
            [
                at[0] + f64::from(cy * p[0] + sy * p[2]),
                at[1] + f64::from(p[1]),
                at[2] + f64::from(-sy * p[0] + cy * p[2]),
            ]
        });
        for face in [
            [0, 1, 5, 4],
            [2, 6, 7, 3],
            [0, 2, 3, 1],
            [4, 5, 7, 6],
            [0, 4, 6, 2],
            [1, 3, 7, 5],
        ] {
            mesh.quad(face.map(|i| points[i]), fill);
        }
        for edge in [
            [0, 1],
            [1, 3],
            [3, 2],
            [2, 0],
            [4, 5],
            [5, 7],
            [7, 6],
            [6, 4],
            [0, 4],
            [1, 5],
            [2, 6],
            [3, 7],
        ] {
            mesh.line(edge.map(|i| points[i]), outline);
        }
    }
}
fn halo(mesh: &mut Mesh, at: [f64; 3], color: [f32; 4]) {
    for outer in 0..144 {
        let a = angle(outer, 144);
        let b = angle(outer + 1, 144);
        for inner in 0..12 {
            let u = angle(inner, 12);
            let v = angle(inner + 1, 12);
            let y = 0.0375 * fast_sin(u);
            let ny = 0.0375 * fast_sin(v);
            let off = 0.0375 * fast_cos(u);
            let noff = 0.0375 * fast_cos(v);
            let p = [
                [(0.3 + off) * fast_sin(b), y, (0.3 + off) * fast_cos(b)],
                [(0.3 + off) * fast_sin(a), y, (0.3 + off) * fast_cos(a)],
                [(0.3 + noff) * fast_sin(a), ny, (0.3 + noff) * fast_cos(a)],
                [(0.3 + noff) * fast_sin(b), ny, (0.3 + noff) * fast_cos(b)],
            ];
            mesh.quad(
                p.map(|p| std::array::from_fn(|i| at[i] + f64::from(p[i]))),
                color,
            );
        }
    }
}
