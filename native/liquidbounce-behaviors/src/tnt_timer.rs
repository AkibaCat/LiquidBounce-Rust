// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleTNTTimer/GameTimeUnit at LiquidBounce ae0eb396.
use crate::{target_renderer::Node, tracers::curve};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub const SHOW: [&str; 2] = ["InView", "Offscreen"];
fn node(name: &'static str, kind: &'static str, default: Value) -> Node {
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
pub fn schema() -> Node {
    let mut root = node("TNTTimer", "CONFIGURABLE", Value::Null);
    root.children.push(node("ESP", "BOOLEAN", json!(true)));
    let mut timer = node("ShowTimer", "TOGGLEABLE", Value::Null);
    timer.children = vec![
        node("Enabled", "BOOLEAN", json!(false)),
        node("Scale", "CURVE", json!([{"x":0.,"y":1.},{"x":200.,"y":1.}])),
        {
            let mut n = node("RenderY", "FLOAT", json!(1.));
            n.range = Some([-2., 2.]);
            n
        },
        node("OwnerName", "BOOLEAN", json!(true)),
        {
            let mut n = node("TimeUnit", "CHOOSE", json!("Ticks"));
            n.choices = ["Ticks", "Seconds"]
                .into_iter()
                .map(|n| node(n, "CONFIGURABLE", Value::Null))
                .collect();
            n
        },
        {
            let mut n = node("Show", "MULTI_CHOOSE", json!(SHOW));
            n.choices = SHOW
                .into_iter()
                .map(|n| node(n, "CONFIGURABLE", Value::Null))
                .collect();
            n
        },
    ];
    root.children.push(timer);
    root
}
#[derive(Clone, Debug, PartialEq)]
pub struct TNTTimer {
    pub values: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TNTTimerSetting(pub String, pub Value);
impl Default for TNTTimer {
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
        }
        let mut values = BTreeMap::new();
        walk(&schema(), "", &mut values);
        Self { values }
    }
}
impl TNTTimerSetting {
    pub fn apply(self, s: &mut TNTTimer) {
        if s.values.contains_key(&self.0) {
            s.values.insert(self.0, self.1);
        }
    }
}
impl TNTTimer {
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
    pub fn scale(&self, distance: f32) -> f32 {
        let points = self
            .value("ShowTimer.Scale")
            .as_array()
            .into_iter()
            .flatten()
            .map(|v| {
                [
                    v["x"].as_f64().unwrap_or(0.) as f32,
                    v["y"].as_f64().unwrap_or(1.) as f32,
                ]
            })
            .collect::<Vec<_>>();
        curve::sample(&points, distance)
    }
    pub fn time(&self, fuse: i32) -> String {
        if self.text("ShowTimer.TimeUnit") == "Seconds" {
            let text = format!("{:.1}", f64::from(fuse) * 0.05);
            format!("{}s", text.strip_suffix(".0").unwrap_or(&text))
        } else {
            fuse.to_string()
        }
    }
    pub fn glow(fuse: i32) -> u32 {
        let denominator = 12i32.wrapping_add(fuse);
        let red = (255. * (1. + 0.5 * (2400. / f64::from(denominator)).sin())).floor() as i32;
        0xff000000 | ((red.clamp(0, 255) as u32) << 16)
    }
    pub fn countdown_color(fuse: i32) -> u32 {
        let green = (255f32 * fuse as f32 / 80.).floor() as i32;
        0xffff0000 | ((green.min(255) as u32 & 255) << 8)
    }
    pub fn frame(&self, input: &Value) -> Value {
        let camera = serde_json::from_value::<[f64; 3]>(input["camera"].clone()).unwrap_or([0.; 3]);
        let show = self.value("ShowTimer.Show");
        let visible = SHOW.map(|s| show.as_array().is_some_and(|v| v.iter().any(|v| v == s)));
        let labels=input["entities"].as_array().into_iter().flatten().filter_map(|e|{
            let fuse=e["fuse"].as_i64()?as i32;if fuse<=0||!self.boolean("ShowTimer.Enabled"){return None;}
            let mut pos=serde_json::from_value::<[f64;3]>(e["center"].clone()).ok()?;pos[1]+=f64::from(self.number("ShowTimer.RenderY"));
            let distance=(0..3).map(|i|(pos[i]-camera[i]).powi(2)).sum::<f64>().sqrt()as f32;
            Some(json!({"id":e["id"],"position":pos,"priority":fuse.wrapping_neg(),"color":Self::countdown_color(fuse),"scale":self.scale(distance),"text":self.time(fuse),"owner":if self.boolean("ShowTimer.OwnerName"){e["owner"].clone()}else{Value::Null},"visible":visible}))
        }).collect::<Vec<_>>();
        json!({"labels":labels})
    }
}
