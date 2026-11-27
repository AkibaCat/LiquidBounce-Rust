// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientStorageEspEvent, Host};
use liquidbounce_behaviors::storage_esp::{self, Block, Mesh, Point, StorageEsp};
use serde_json::{Value, json};
fn point(p: Point, c: [f32; 4]) -> Value {
    json!({"position":p,"color":c})
}
fn add(
    mesh: Mesh,
    color: i32,
    alpha: Option<u8>,
    outline: bool,
    fade: bool,
    s: &StorageEsp,
    camera: Point,
    q: &mut Vec<Value>,
    l: &mut Vec<Value>,
) {
    let c = storage_esp::rgba(color);
    let vertex = |p: Point, a: f32| {
        let mut c = c;
        c[3] = a * if fade { s.fade_at(p, camera) } else { 1. };
        point(p, c)
    };
    for quad in mesh.quads {
        q.push(json!(
            quad.map(|p| vertex(p, alpha.map_or(c[3], |a| a as f32 / 255.)))
        ));
    }
    if outline {
        for line in mesh.lines {
            l.push(json!(line.map(|p| vertex(p, 100. / 255.))));
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn storage_esp(&self, _host: &dyn Host, event: &mut ClientStorageEspEvent) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        if !modules.query_running(&state.registry, "StorageESP") {
            return;
        }
        let s = &modules.storage_esp;
        if !s.active(modules.query_running(&state.registry, "ChestStealer")) {
            return;
        }
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let Ok(camera) = serde_json::from_value::<Point>(v["camera"].clone()) else {
            return;
        };
        let eye = serde_json::from_value::<Point>(v["eye"].clone()).unwrap_or([0., 0., -1.]);
        let eye = std::array::from_fn(|i| camera[i] + eye[i]);
        let mut blocks = vec![];
        let mut models = vec![];
        let mut targets = vec![];
        let (mut quads, mut lines, mut plain_q, mut plain_l) = (vec![], vec![], vec![], vec![]);
        let interacted = modules.chest_stealer.storage_interacted();
        let visible_tracers = s
            .categories
            .iter()
            .any(|c| c.tracers && (c.color as u32 >> 24) != 0);
        for b in v["blocks"].as_array().into_iter().flatten() {
            let Some(i) = b["name"].as_str().and_then(storage_esp::block_category) else {
                continue;
            };
            let Some(c) = s.category(i) else {
                continue;
            };
            let Ok(pos) = serde_json::from_value::<[i32; 3]>(b["position"].clone()) else {
                continue;
            };
            if interacted.contains(&pos) {
                continue;
            }
            let at = pos.map(f64::from);
            let within = s.within(at, camera);
            if c.tracers && (c.color as u32 >> 24) != 0 && within {
                plain_l.push(json!([
                    point(eye, storage_esp::rgba(c.color)),
                    point(at.map(|x| x + 0.5), storage_esp::rgba(c.color))
                ]));
            }
            if (c.color as u32 >> 24) == 0 {
                continue;
            }
            if s.glow && b["model"].as_bool() == Some(true) {
                if within {
                    models.push(json!({"position":pos,"color":c.color as u32}));
                }
                continue;
            }
            let boxes = serde_json::from_value(b["boxes"].clone()).unwrap_or_default();
            blocks.push(Block {
                position: pos,
                category: i,
                boxes,
            });
        }
        for (i, mesh) in storage_esp::shapes(&blocks, s.merge_adjacent) {
            let c = &s.categories[i];
            add(
                mesh,
                c.color,
                (!s.glow).then_some(50),
                !s.glow && s.outline,
                true,
                s,
                camera,
                &mut quads,
                &mut lines,
            );
        }
        for e in v["entities"].as_array().into_iter().flatten() {
            let Some(i) = e["name"].as_str().and_then(|n| {
                storage_esp::entity_category(n, e["chested"].as_bool().unwrap_or(false))
            }) else {
                continue;
            };
            let Some(c) = s.category(i) else {
                continue;
            };
            let Ok(pos) = serde_json::from_value::<Point>(e["position"].clone()) else {
                continue;
            };
            if !s.within(pos, camera) {
                continue;
            }
            let at = serde_json::from_value::<Point>(e["interpolated"].clone()).unwrap_or(pos);
            let width = e["width"].as_f64().unwrap_or(1.);
            let height = e["height"].as_f64().unwrap_or(1.);
            if visible_tracers && c.tracers {
                let mut top = at;
                top[1] += height;
                let color = storage_esp::rgba(c.color);
                plain_l.extend([
                    json!([point(eye, color), point(at, color)]),
                    json!([point(at, color), point(top, color)]),
                ]);
            }
            if (c.color as u32 >> 24) == 0 {
                continue;
            }
            if s.glow {
                if let Some(id) = e["id"].as_i64() {
                    targets.push(json!([id, c.color as u32]));
                }
            } else {
                let bounds = [
                    at[0] - width / 2. - 0.05,
                    at[1] - 0.05,
                    at[2] - width / 2. - 0.05,
                    at[0] + width / 2. + 0.05,
                    at[1] + height + 0.05,
                    at[2] + width / 2. + 0.05,
                ];
                add(
                    storage_esp::box_mesh(bounds),
                    c.color,
                    Some(50),
                    s.outline,
                    false,
                    s,
                    camera,
                    &mut plain_q,
                    &mut plain_l,
                );
            }
        }
        let effect=s.effect().map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity}));
        *event.geometry=json!({"body":{"quads":quads,"lines":lines,"effect":effect,"targets":targets},"plain":{"quads":plain_q,"lines":plain_l,"effect":null},"models":models}).to_string();
    }
}
