// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::ClientItemEspEvent;
use liquidbounce_behaviors::{
    item_esp::{self, Kind, Mode},
    storage_esp::Point,
};
use serde_json::{Value, json};
fn point(v: &Value) -> Option<Point> {
    serde_json::from_value(v.clone()).ok()
}
fn selected(s: &item_esp::ItemEsp, e: &Value, camera: Point) -> bool {
    let Some(eye) = point(&e["eye"]) else {
        return false;
    };
    let kind = match e["kind"].as_str() {
        Some("item") => Kind::Item(e["item"].as_str().unwrap_or("minecraft:air")),
        Some("arrow") => Kind::Arrow {
            allowed: e["allowed"] == true,
            effect_color: e["effectColor"].as_i64().unwrap_or(-1) as i32,
        },
        Some("spectral") => Kind::Spectral {
            allowed: e["allowed"] == true,
        },
        Some("trident") => Kind::Trident,
        _ => return false,
    };
    s.should_render(kind, eye, camera)
}
impl LiquidBounceHooks {
    pub(crate) fn item_esp(&self, event: &mut ClientItemEspEvent<'_>) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if !modules.query_running(&state.registry, "ItemESP") {
            modules.item_esp.clear();
            *event.output = json!({"selected":[],"glow":null,"plain":null}).to_string();
            return;
        }
        let s = &mut modules.item_esp;
        let camera = point(&input["camera"]).unwrap_or([0.; 3]);
        let live = input["entities"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if input["tick"] == true {
            s.clear();
            if s.mode != Mode::Glow {
                for e in live {
                    if selected(s, e, camera) {
                        if let Some(id) = e["identity"].as_u64() {
                            s.selected.insert(id);
                        }
                    }
                }
            }
            *event.output = json!({"selected":s.selected}).to_string();
            return;
        }
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let nanos = START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_nanos() as f64;
        let color = s.color_at(nanos);
        let mut quads = vec![];
        let mut lines = vec![];
        let vertex = |p: Point, c: [f32; 4]| json!({"position":p,"color":c});
        if s.tracers {
            let eye = point(&input["eyeVector"]).unwrap_or([0., 0., 1.]);
            let start = std::array::from_fn(|i| camera[i] + eye[i]);
            for e in live {
                if selected(s, e, camera) {
                    if let Some(p) = point(&e["interpolated"]) {
                        let relative: [f32; 3] = std::array::from_fn(|i| (p[i] - camera[i]) as f32);
                        let p = std::array::from_fn(|i| camera[i] + f64::from(relative[i]));
                        lines.push([vertex(start, color), vertex(p, color)]);
                    }
                }
            }
        }
        if s.mode == Mode::Box {
            let boxes = input["captured"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|e| {
                    e["identity"]
                        .as_u64()
                        .is_some_and(|id| s.selected.contains(&id))
                })
                .filter_map(|e| point(&e["interpolated"]).map(item_esp::item_box))
                .collect::<Vec<_>>();
            let boxes = if s.merge_intersecting {
                item_esp::merge_boxes(&boxes)
            } else {
                boxes
            };
            let face = [color[0], color[1], color[2], 50. / 255.];
            let line = [color[0], color[1], color[2], 100. / 255.];
            for b in boxes {
                let mesh = liquidbounce_behaviors::storage_esp::box_mesh(b);
                for q in mesh.quads {
                    quads.push(q.map(|p| vertex(p, face)));
                }
                for l in mesh.lines {
                    lines.push(l.map(|p| vertex(p, line)));
                }
            }
        } else if s.mode == Mode::Legacy2D {
            let yaw = input["yaw"].as_f64().unwrap_or(0.) as f32;
            let background = [0., 0., 0., s.background_alpha as f32 / 255.];
            for e in input["captured"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|e| {
                    e["identity"]
                        .as_u64()
                        .is_some_and(|id| s.selected.contains(&id))
                })
            {
                let Some(mut pos) = point(&e["interpolated"]) else {
                    continue;
                };
                pos[1] += f64::from(s.y_offset);
                for (q, bg) in item_esp::legacy_marker(
                    pos,
                    camera,
                    yaw,
                    e["height"].as_f64().unwrap_or(0.),
                    s.scale,
                ) {
                    quads.push(q.map(|p| vertex(p, if bg { background } else { color })));
                }
            }
        }
        let glow = if s.mode == Mode::Glow {
            let effect = s.effect();
            let argb = ((color[3] * 255.) as u32) << 24
                | ((color[0] * 255.) as u32) << 16
                | ((color[1] * 255.) as u32) << 8
                | (color[2] * 255.) as u32;
            let targets = live
                .iter()
                .filter(|e| selected(s, e, camera))
                .filter_map(|e| e["id"].as_i64().map(|id| json!([id, argb])))
                .collect::<Vec<_>>();
            json!({"quads":[],"lines":[],"targets":targets,"effect":{"style":effect.style,"params":effect.params,"accent":effect.accent,"range":effect.range,"opacity":effect.opacity}})
        } else {
            Value::Null
        };
        *event.output=json!({"selected":s.selected,"glow":glow,"plain":{"quads":quads,"lines":lines,"effect":null}}).to_string();
    }
}
