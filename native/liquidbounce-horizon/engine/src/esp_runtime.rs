// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientEspEvent, Host};
use liquidbounce_behaviors::{
    combine_mobs::{CombineMobs, Key},
    esp::{Entity, Mesh},
    targeting::{Target, VisualCamera},
};
use serde_json::{Value, json};
fn entity(v: &Value) -> Option<Entity> {
    Some(Entity {
        identity: v["identity"].as_u64()?,
        id: v["id"].as_i64()? as i32,
        position: serde_json::from_value(v["position"].clone()).ok()?,
        interpolated: serde_json::from_value(v["interpolated"].clone()).ok()?,
        width: v["width"].as_f64()? as f32,
        height: v["height"].as_f64()? as f32,
        bounding_height: v["boundingHeight"]
            .as_f64()
            .or_else(|| v["height"].as_f64())? as f32,
        living: v["living"].as_bool().unwrap_or(true),
        player: v["kind"] == "minecraft:player",
        hurt: v["hurt"].as_i64().unwrap_or(0) > 0,
        invisible: v["target"]["invisible"].as_bool() == Some(true),
        friend: false,
        tag: None,
        health: v["health"].as_f64().unwrap_or(0.) as f32,
        max_health: v["maxHealth"].as_f64().unwrap_or(20.) as f32,
        object: v["object"].as_u64().unwrap_or(0) as u8,
        own_vehicle: v["ownVehicle"].as_bool() == Some(true),
    })
}
fn mesh(v: Mesh) -> Value {
    let point = |p: liquidbounce_behaviors::breadcrumbs::ColoredPoint| json!({"position":p.position,"color":p.color});
    json!({"quads":v.quads.into_iter().map(|q|q.map(point)).collect::<Vec<_>>(),"lines":v.lines.into_iter().map(|q|q.map(point)).collect::<Vec<_>>(),"effect":null})
}
impl LiquidBounceHooks {
    pub(crate) fn esp(&self, _host: &dyn Host, event: &mut ClientEspEvent) {
        let Ok(input) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let mut state = self.state();
        if input["phase"] == "clear" {
            state.behaviors.borrow_mut().esp.clear();
            return;
        }
        if !state
            .behaviors
            .borrow()
            .query_running(&state.registry, "ESP")
        {
            state.behaviors.borrow_mut().esp.clear();
            return;
        }
        if input["phase"] == "overlay" {
            let b = state.behaviors.borrow();
            let s = &b.esp;
            if s.mode != 1 {
                return;
            }
            let scale = input["guiScale"].as_f64().unwrap_or(0.) as f32;
            let mut rects = vec![];
            for v in input["entities"].as_array().into_iter().flatten() {
                let Ok(rect) = serde_json::from_value(v["rect"].clone()) else {
                    continue;
                };
                let color = v["color"].as_u64().unwrap_or(0) as u32;
                let health = (v["living"].as_bool() == Some(true)).then(|| {
                    (
                        v["health"].as_f64().unwrap_or(0.) as f32,
                        v["maxHealth"].as_f64().unwrap_or(20.) as f32,
                    )
                });
                rects.extend(
                    s.overlay(rect, color, health, scale)
                        .into_iter()
                        .map(|r| json!({"rect":r.bounds,"color":r.color})),
                );
            }
            *event.output = json!(rects).to_string();
            return;
        }
        let entries = input["entities"].as_array().cloned().unwrap_or_default();
        let snapshots = entries
            .iter()
            .map(|e| crate::targeting::snapshot(&e["target"].to_string()))
            .collect::<Vec<_>>();
        let camera = serde_json::from_value(input["camera"].clone()).unwrap_or([0.; 3]);
        let perspective = input["perspective"].as_i64().unwrap_or(0) as i32;
        let tick = input["tick"].as_bool() == Some(true);
        let free_look = {
            let b = state.behaviors.borrow();
            state
                .registry
                .module(b.query_ids["FreeLook"])
                .is_some_and(|m| m.enabled())
        };
        let mut current = vec![];
        let mut show = vec![];
        let mut valid_entries = vec![];
        for (v, snapshot) in entries.iter().zip(&snapshots) {
            let Some(mut e) = entity(v) else {
                continue;
            };
            let mut shown = false;
            if let Some(s) = snapshot {
                let info = state.target_info(e.id, s);
                shown = s.entity.kind.living
                    && state.global_targets.visual.should_show(
                        &s.entity,
                        info,
                        VisualCamera {
                            raw_first_person: perspective == 0,
                            free_cam: false,
                            free_look,
                        },
                    );
                e.tag = info.resolved_color();
                e.friend = e.player
                    && s.player_name
                        .as_deref()
                        .is_some_and(|n| state.friends.contains(n));
            }
            current.push(e);
            show.push(shown);
            valid_entries.push(v);
        }
        let refresh = {
            let b = state.behaviors.borrow();
            tick || (b.esp.perspective.is_some_and(|p| p != perspective)
                && state.global_targets.visual.contains(Target::SelfEntity))
        };
        if refresh {
            let (combine_running, mut combine) = {
                let b = state.behaviors.borrow();
                let mut c = CombineMobs::default();
                c.combine_armor_stands = b.combine_mobs.combine_armor_stands;
                c.combine_minecarts = b.combine_mobs.combine_minecarts;
                (b.query_running(&state.registry, "CombineMobs"), c)
            };
            let mut selected = std::collections::BTreeSet::new();
            for ((e, shown), v) in current.iter().zip(&show).zip(&valid_entries) {
                let object = state.behaviors.borrow().esp.is_object(e);
                if object {
                    selected.insert(e.identity);
                }
                if !shown {
                    continue;
                }
                let key = Key::at(
                    v["kind"].as_str().unwrap_or(""),
                    v["baby"].as_bool().unwrap_or(false),
                    e.position,
                );
                if !combine.track(
                    combine_running,
                    key,
                    v["group"].as_u64().unwrap_or(3) as u8,
                    true,
                ) {
                    selected.insert(e.identity);
                }
            }
            let mut b = state.behaviors.borrow_mut();
            b.esp.selected = selected;
            b.esp.perspective = Some(perspective);
            b.combine_mobs.clear_nametags();
            if tick {
                b.esp.prepare(&current, camera);
            }
        }
        if tick {
            return;
        }
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let nanos = START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_nanos() as f64;
        let mut b = state.behaviors.borrow_mut();
        let s = &mut b.esp;
        s.true_sight.clear();
        if s.mode == 3 {
            s.true_sight.extend(
                current
                    .iter()
                    .zip(&show)
                    .filter_map(|(e, shown)| (*shown && e.living).then_some(e.id)),
            );
        }
        let mut targets = vec![];
        let mut colors = vec![];
        if s.mode == 3 {
            for (e, shown) in current.iter().zip(&show) {
                if !s.within(e, camera) {
                    continue;
                }
                let object = s.is_object(e);
                if e.living || object {
                    colors.push((e.id, s.color(e, camera, nanos)));
                }
                if *shown || object {
                    targets.push((e.id, s.color(e, camera, nanos)));
                }
            }
        }
        let boxes = if s.mode <= 1 {
            s.boxes(&current, camera, nanos)
        } else {
            vec![]
        };
        let plain = match s.mode {
            0 => mesh(s.box_mesh(&boxes)),
            2 => mesh(s.legacy_mesh(
                &current,
                camera,
                input["yaw"].as_f64().unwrap_or(0.) as f32,
                nanos,
            )),
            _ => Value::Null,
        };
        let overlays = if s.mode == 1 {
            s.prepared.iter().filter_map(|p|{let e=current.iter().find(|e|e.identity==p.identity)?;let bounds=std::array::from_fn::<_,6,_>(|i|p.bounds[i]+e.interpolated[i%3]);Some(json!({"bounds":bounds,"color":s.color(e,camera,nanos),"living":e.living,"health":e.health,"maxHealth":e.max_health}))}).collect::<Vec<_>>()
        } else {
            vec![]
        };
        let effect=s.effect().map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity}));
        *event.output=json!({"glow":{"quads":[],"lines":[],"effect":effect,"targets":targets,"colors":colors},"plain":plain,"overlay":overlays}).to_string();
    }
}
