// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientBlinkEspEvent, Host};
use liquidbounce_behaviors::{blink_esp::Player, breadcrumbs::Geometry};
use serde_json::{Value, json};
fn geometry(mesh: Geometry) -> Value {
    let point = |p: liquidbounce_behaviors::breadcrumbs::ColoredPoint| json!({"position":p.position,"color":p.color});
    json!({"quads":mesh.quads.into_iter().map(|q|q.map(point)).collect::<Vec<_>>(),"lines":mesh.lines.into_iter().map(|q|q.map(point)).collect::<Vec<_>>(),"effect":null,"rings":[]})
}
impl LiquidBounceHooks {
    pub(crate) fn blink_esp(&self, host: &dyn Host, event: &mut ClientBlinkEspEvent<'_>) {
        let partial = event.input.parse::<f32>().unwrap_or(0.);
        let Ok(p) = serde_json::from_str::<Value>(&host.client_blink_visual(partial)) else {
            return;
        };
        if p.is_null() {
            return;
        }
        let state = self.state();
        let modules = state.behaviors.borrow();
        let positions = modules.criticals.positions();
        let r = modules
            .rotations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .actual_server;
        let rotation = [r.yaw, r.pitch];
        let s = &modules.blink_esp;
        let visual = s.visual(
            &positions,
            rotation,
            Player {
                first_person: p["firstPerson"].as_bool().unwrap_or(true),
                width: p["width"].as_f64().unwrap_or(0.) as f32,
                height: p["height"].as_f64().unwrap_or(0.) as f32,
                pose: p["pose"].as_i64().unwrap_or(0) as i32,
                swim: p["swimAmount"].as_f64().unwrap_or(0.) as f32,
            },
        );
        let mut body = geometry(visual.body);
        if let Some((position, radius, color, outline)) = visual.ring {
            body["rings"] = json!([{"position":position,"radius":radius,"color":color,"outline":false},{"position":position,"radius":radius,"color":outline,"outline":true}]);
        }
        let mut line = geometry(visual.line);
        line["effect"]=s.render.effect().map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity})).unwrap_or(Value::Null);
        let model=visual.model.map(|position|json!({"position":position,"rotation":rotation,"lightPercent":s.light_percent,"outlineColor":s.model_outline}));
        crate::flag_check_runtime::append_geometry(&modules.flag_check,p["firstPerson"].as_bool().unwrap_or(true),&mut body);
        *event.geometry = json!({"body":body,"line":line,"model":model}).to_string();
    }
}
