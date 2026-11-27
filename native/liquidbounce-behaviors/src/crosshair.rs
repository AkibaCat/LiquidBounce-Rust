// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned ModuleCrosshair, Circle/CS2, cooldown priority and hit flash.
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub const MODES: [&str; 2] = ["Circle", "CS2"];
pub const SOURCES: [&str; 3] = ["Charge", "Item", "Attack"];
#[derive(Clone, Debug)]
pub struct Field {
    pub path: String,
    pub kind: &'static str,
    pub default: Value,
    pub range: Option<[f32; 2]>,
    pub suffix: &'static str,
}
pub fn fields() -> Vec<Field> {
    let mut out = vec![];
    let mut add = |p: &str, k, d, r, suffix| {
        out.push(Field {
            path: p.into(),
            kind: k,
            default: d,
            range: r,
            suffix,
        })
    };
    add("Mode", "MODE", json!("Circle"), None, "");
    for mode in MODES {
        let p = format!("Mode.{mode}");
        add(
            &format!("{p}.ShowInThirdPerson"),
            "BOOLEAN",
            json!(true),
            None,
            "",
        );
        if mode == "Circle" {
            add(
                &format!("{p}.Radius.Range"),
                "FLOAT_RANGE",
                json!({"from":3.,"to":5.}),
                Some([0., 25.]),
                "",
            );
            add(
                &format!("{p}.Radius.DynamicRadiusMultiplier"),
                "FLOAT",
                json!(1.),
                Some([0., 5.]),
                "",
            );
        } else {
            for (n, d, r) in [
                ("Length", 5., [1., 20.]),
                ("Thickness", 1., [0.5, 5.]),
                ("Gap", 2., [0., 10.]),
                ("DynamicMultiplier", 1., [0., 10.]),
            ] {
                add(
                    &format!("{p}.Crosshair.{n}"),
                    "FLOAT",
                    json!(d),
                    Some(r),
                    "",
                );
            }
        }
        add(&format!("{p}.Color.Sync"), "BOOLEAN", json!(true), None, "");
        add(
            &format!("{p}.Color.FirstColor"),
            "COLOR",
            json!(0xff4677ffu32 as i32),
            None,
            "",
        );
        add(
            &format!("{p}.Color.SecondColor"),
            "COLOR",
            json!(mix(0xff4677ffu32 as i32, -1, 0.5)),
            None,
            "",
        );
        add(
            &format!("{p}.Color.SpinSpeed"),
            "FLOAT",
            json!(4.),
            Some([-10., 10.]),
            "",
        );
    }
    add("CooldownBar.Enabled", "BOOLEAN", json!(false), None, "");
    add(
        "CooldownBar.Sources",
        "MULTI_CHOOSE",
        json!(SOURCES),
        None,
        "",
    );
    for (n, d, r) in [
        ("Width", 20., [4., 80.]),
        ("Height", 2., [0.5, 8.]),
        ("Offset", 8., [0., 40.]),
    ] {
        add(&format!("CooldownBar.{n}"), "FLOAT", json!(d), Some(r), "");
    }
    add(
        "CooldownBar.ReadyFlash.Enabled",
        "BOOLEAN",
        json!(true),
        None,
        "",
    );
    add("HitFlash.Enabled", "BOOLEAN", json!(false), None, "");
    for (p, d) in [
        ("CooldownBar.Color", -1),
        ("CooldownBar.Background", 0x78000000),
        ("CooldownBar.ReadyFlash.Color", 0xff4677ffu32 as i32),
        ("HitFlash.Color", 0xffff2828u32 as i32),
    ] {
        add(p, "COLOR", json!(d), None, "");
    }
    add(
        "CooldownBar.ReadyFlash.Duration",
        "INT",
        json!(250),
        Some([50., 1000.]),
        "ms",
    );
    add(
        "HitFlash.Duration",
        "INT",
        json!(300),
        Some([50., 1000.]),
        "ms",
    );
    out
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    target: Option<String>,
    hurt: i32,
    hit_at: i64,
    last_source: Option<usize>,
    ready_at: i64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Crosshair {
    pub values: BTreeMap<String, Value>,
    pub state: State,
}
impl Default for Crosshair {
    fn default() -> Self {
        Self {
            values: fields().into_iter().map(|f| (f.path, f.default)).collect(),
            state: State::default(),
        }
    }
}
pub fn mix(a: i32, b: i32, t: f32) -> i32 {
    let a = (a as u32).to_be_bytes();
    let b = (b as u32).to_be_bytes();
    u32::from_be_bytes(std::array::from_fn(|i| {
        t.mul_add(b[i] as f32 - a[i] as f32, a[i] as f32)
            .clamp(0., 255.) as u8
    })) as i32
}
fn sin(a: f32) -> f32 {
    const TABLE: &[u8] = include_bytes!("../vendor/minecraft-math/sine-table.bin");
    let i = ((a * 10430.378) as i32 & 65535) as usize * 4;
    f32::from_le_bytes(TABLE[i..i + 4].try_into().unwrap())
}
impl Crosshair {
    pub fn f(&self, p: &str) -> f32 {
        self.values[p].as_f64().unwrap() as f32
    }
    pub fn b(&self, p: &str) -> bool {
        self.values[p].as_bool().unwrap()
    }
    pub fn c(&self, p: &str) -> i32 {
        self.values[p].as_i64().unwrap() as i32
    }
    pub fn mode(&self) -> &str {
        self.values["Mode"].as_str().unwrap()
    }
    pub fn set(&mut self, path: String, value: Value) {
        if value.as_bool() == Some(false) {
            if path == "HitFlash.Enabled" {
                self.state.target = None;
                self.state.hit_at = 0;
            }
            if path == "CooldownBar.Enabled" {
                self.state.last_source = None;
                self.state.ready_at = 0;
            }
        }
        self.values.insert(path, value);
    }
    pub fn reset(&mut self) {
        self.state = State::default();
    }
    pub fn tick(&mut self, now: i64, target: Option<String>, hurt: i32, running: bool) {
        if !running {
            self.reset();
            return;
        }
        if self.b("HitFlash.Enabled") {
            if target == self.state.target && hurt > self.state.hurt {
                self.state.hit_at = now;
            }
            self.state.target = target;
            self.state.hurt = hurt;
        } else {
            self.state.target = None;
            self.state.hit_at = 0;
        }
        if !self.b("CooldownBar.Enabled") {
            self.state.last_source = None;
            self.state.ready_at = 0;
        }
    }
    fn color(&self, path: &str, angle: f32, wall: i64, now: i64) -> i32 {
        let a = self.c(&format!("{path}.FirstColor"));
        let b = if self.b(&format!("{path}.Sync")) {
            a
        } else {
            self.c(&format!("{path}.SecondColor"))
        };
        let speed = self.f(&format!("{path}.SpinSpeed"));
        let offset = if speed == 0. {
            0.
        } else {
            ((wall as f64 / 10000. * speed as f64) % 1.) * std::f32::consts::TAU as f64
        };
        let base = mix(a, b, sin((angle as f64 + offset) as f32) * 0.5 + 0.5);
        if self.b("HitFlash.Enabled") {
            mix(
                base,
                self.c("HitFlash.Color"),
                (1. - now.saturating_sub(self.state.hit_at) as f32 / self.f("HitFlash.Duration"))
                    .clamp(0., 1.),
            )
        } else {
            base
        }
    }
    pub fn render(&mut self, v: &Value) -> Value {
        if v["hiddenScreen"].as_bool() == Some(true) {
            return json!({"quads":[]});
        }
        let now = v["now"].as_i64().unwrap_or(0);
        let wall = v["wall"].as_i64().unwrap_or(now);
        let (w, h) = (
            v["width"].as_f64().unwrap_or(0.) as f32,
            v["height"].as_f64().unwrap_or(0.) as f32,
        );
        let attack = v["attack"].as_f64().unwrap_or(1.) as f32;
        let mut quads = vec![];
        let mut circle = Value::Null;
        let m = format!("Mode.{}", self.mode());
        if v["firstPerson"].as_bool() != Some(false) || self.b(&format!("{m}.ShowInThirdPerson")) {
            let (x, y) = (w / 2.002, h / 2.0025);
            let color = format!("{m}.Color");
            if self.mode() == "Circle" {
                let mult = self.f(&format!("{m}.Radius.DynamicRadiusMultiplier"));
                let multiplier = if mult.abs() < 1e-5 {
                    0.
                } else {
                    mult * (1. - attack)
                };
                let range = &self.values[&format!("{m}.Radius.Range")];
                let lut = (0..256)
                    .map(|i| self.color(&color, i as f32 / 256. * std::f32::consts::TAU, wall, now))
                    .collect::<Vec<_>>();
                circle = json!({"center":[x,y],"inner":range["from"].as_f64().unwrap()as f32+multiplier,"outer":range["to"].as_f64().unwrap()as f32+multiplier,"lut":lut});
            } else {
                let l = self.f(&format!("{m}.Crosshair.Length"));
                let t = self.f(&format!("{m}.Crosshair.Thickness")) / 2.;
                let mult = self.f(&format!("{m}.Crosshair.DynamicMultiplier"));
                let g = self.f(&format!("{m}.Crosshair.Gap"))
                    + if mult.abs() < 1e-5 {
                        0.
                    } else {
                        mult * (1. - attack)
                    };
                let c = self.color(&color, 0., wall, now);
                for [a, b, d, e] in [
                    [-t, -g - l, t, -g],
                    [-t, g, t, g + l],
                    [-g - l, -t, -g, t],
                    [g, -t, g + l, t],
                ] {
                    quads.push(json!([x + a, y + b, x + d, y + e, c]));
                }
            }
        }
        if self.b("CooldownBar.Enabled") {
            let progress = [
                v["charge"].as_f64().map(|x| x as f32),
                v["item"].as_f64().map(|x| x as f32),
                Some(attack),
            ];
            if self
                .state
                .last_source
                .is_some_and(|i| progress[i] == Some(1.))
            {
                self.state.ready_at = now;
            }
            let selected = self.values["CooldownBar.Sources"].as_array().unwrap();
            self.state.last_source = (0..3).find(|&i| {
                selected.iter().any(|v| v.as_str() == Some(SOURCES[i]))
                    && progress[i].is_some_and(|p| p < 1.)
            });
            let flash = if self.b("CooldownBar.ReadyFlash.Enabled") {
                (1. - now.saturating_sub(self.state.ready_at) as f32
                    / self.f("CooldownBar.ReadyFlash.Duration"))
                .clamp(0., 1.)
            } else {
                0.
            };
            let p = self.state.last_source.and_then(|i| progress[i]);
            if p.is_some() || flash > 0. {
                let width = self.f("CooldownBar.Width");
                let x = w / 2. - width / 2.;
                let y = h / 2. + self.f("CooldownBar.Offset");
                let height = self.f("CooldownBar.Height");
                let color = mix(
                    self.c("CooldownBar.Color"),
                    self.c("CooldownBar.ReadyFlash.Color"),
                    flash,
                );
                quads.push(json!([
                    x,
                    y,
                    x + width,
                    y + height,
                    self.c("CooldownBar.Background")
                ]));
                quads.push(json!([
                    x,
                    y,
                    x + width * p.unwrap_or(1.),
                    y + height,
                    color
                ]));
            }
        } else {
            self.state.last_source = None;
            self.state.ready_at = 0;
        }
        json!({"quads":quads,"circle":circle})
    }
}
