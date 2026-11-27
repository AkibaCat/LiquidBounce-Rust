// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientPlacementEspEvent, Host};
use liquidbounce_behaviors::placement_renderer::{Box6, Draw, Pos};
use serde_json::{Value, json};
pub(crate) fn placement_esp_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |v| v.as_millis() as i64)
}
fn positions(v: &Value) -> Vec<(Pos, Box6)> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| {
            Some((
                serde_json::from_value(v["position"].clone()).ok()?,
                serde_json::from_value(v["bounds"].clone()).ok()?,
            ))
        })
        .collect()
}
fn geometry(draws: impl IntoIterator<Item = Draw>) -> Value {
    json!(draws.into_iter().map(|d|json!({"position":d.position,"bounds":d.bounds,"color":d.color,"outline":d.outline,"faces":d.faces,"edges":d.edges})).collect::<Vec<_>>())
}
impl LiquidBounceHooks {
    pub(crate) fn placement_esp(&self, _host: &dyn Host, event: &mut ClientPlacementEspEvent) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let state = self.state();
        let mut m = state.behaviors.borrow_mut();
        let prophunt = m.query_running(&state.registry, "ProphuntESP");
        let void = m.query_running(&state.registry, "VoidESP");
        let scaffold=m.query_running(&state.registry,"Scaffold");
        let liquid_filler=m.query_running(&state.registry,"LiquidFiller");
        let hole_filler=m.query_running(&state.registry,"HoleFiller");
        let bed_defender=m.query_running(&state.registry,"BedDefender");
        let time = v["time"].as_i64().unwrap_or_else(placement_esp_now);
        match v["action"].as_str() {
            Some("options") => {
                if v["tick"].as_bool() == Some(true) {
                    m.prophunt_esp.render.tick();
                    m.void_esp.render.tick();
                    m.liquid_filler.placer.target_render.tick();
                    m.liquid_filler.placer.placed_render.tick();
                    m.hole_filler.placer.target_render.tick();
                    m.hole_filler.placer.placed_render.tick();
                    m.bed_defender.placer.target_render.tick();
                    m.bed_defender.placer.placed_render.tick();
                }
                *event.output=json!({"prophunt":if prophunt{m.prophunt_esp.tracking}else{0},"void":if void{json!({"threshold":m.void_esp.y_threshold,"side":m.void_esp.range_side,"facing":m.void_esp.range_facing})}else{Value::Null}}).to_string();
            }
            Some("prophunt") if prophunt => {
                m.prophunt_esp.add(
                    v["source"].as_u64().unwrap_or(0) as u32,
                    positions(&v["blocks"]),
                    time,
                );
                if v["tick"].as_bool() == Some(true) {
                    m.prophunt_esp.render.update_all();
                }
            }
            Some("void") if void => m.void_esp.update(positions(&v["blocks"]), time),
            Some("frame") => {
                let mut draws = vec![];
                if m.prophunt_esp.render.running(prophunt) {
                    draws.extend(m.prophunt_esp.render.render(time));
                }
                if m.void_esp.render.running(void) {
                    draws.extend(m.void_esp.render.render(time));
                }
                if m.scaffold.render.running(scaffold){draws.extend(m.scaffold.render.render(time));}
                if m.liquid_filler.placer.target_render.running(liquid_filler){draws.extend(m.liquid_filler.placer.target_render.render(time));}
                if m.liquid_filler.placer.placed_render.running(liquid_filler){draws.extend(m.liquid_filler.placer.placed_render.render(time));}
                if m.hole_filler.placer.target_render.running(hole_filler){draws.extend(m.hole_filler.placer.target_render.render(time));}
                if m.hole_filler.placer.placed_render.running(hole_filler){draws.extend(m.hole_filler.placer.placed_render.render(time));}
                if m.bed_defender.placer.target_render.running(bed_defender){draws.extend(m.bed_defender.placer.target_render.render(time));}
                if m.bed_defender.placer.placed_render.running(bed_defender){draws.extend(m.bed_defender.placer.placed_render.render(time));}
                *event.output = geometry(draws).to_string();
            }
            _ => {}
        }
    }
}
