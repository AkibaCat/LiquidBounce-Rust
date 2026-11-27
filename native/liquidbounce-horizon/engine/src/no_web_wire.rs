// SPDX-License-Identifier: GPL-3.0-or-later
use super::model::{Action, ActionKind, Hit, NoWeb};
use serde_json::{Value, json};
pub fn hit(h: Hit) -> Value {
    json!({"pos":h.pos,"face":h.face,"point":h.point})
}
fn pos(v: &Value) -> Option<[i32; 3]> {
    Some([
        v[0].as_i64()? as i32,
        v[1].as_i64()? as i32,
        v[2].as_i64()? as i32,
    ])
}
pub fn read_hit(v: &Value) -> Option<Hit> {
    Some(Hit {
        pos: pos(&v["pos"])?,
        face: pos(&v["face"])?,
        point: [
            v["point"][0].as_f64()?,
            v["point"][1].as_f64()?,
            v["point"][2].as_f64()?,
        ],
    })
}
pub fn action(a: Action) -> Value {
    let kind = match a.kind {
        ActionKind::Top { web, target, min_y } => {
            json!({"type":"top","web":web,"target":target,"minY":min_y})
        }
        ActionKind::Directional { web, side } => {
            json!({"type":"directional","web":web,"side":side})
        }
        ActionKind::Pickup { pos } => json!({"type":"pickup","pos":pos}),
    };
    json!({"slot":a.slot,"hit":hit(a.hit),"kind":kind})
}
pub fn read_action(v: &Value) -> Option<Action> {
    let k = &v["kind"];
    Some(Action {
        slot: v["slot"].as_i64()? as i32,
        hit: read_hit(&v["hit"])?,
        kind: match k["type"].as_str()? {
            "top" => ActionKind::Top {
                web: pos(&k["web"])?,
                target: pos(&k["target"])?,
                min_y: k["minY"].as_f64()?,
            },
            "directional" => ActionKind::Directional {
                web: pos(&k["web"])?,
                side: pos(&k["side"])?,
            },
            "pickup" => ActionKind::Pickup {
                pos: pos(&k["pos"])?,
            },
            _ => return None,
        },
    })
}
pub fn state(s: &NoWeb) -> Value {
    json!({"webs":s.webs,"pickups":s.pickups,"last":s.last_success,"pickup":s.settings.pickup,"span":s.settings.pickup_span})
}
pub fn read_state(v: &Value, s: &mut NoWeb) {
    s.webs = v["webs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(pos)
        .collect();
    s.pickups = v["pickups"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| Some((pos(&v[0])?, v[1].as_u64()?)))
        .collect();
    s.last_success = pos(&v["last"][0]).zip(v["last"][1].as_u64());
    s.settings.pickup = v["pickup"].as_bool().unwrap_or(true);
    if let (Some(a), Some(b)) = (v["span"][0].as_f64(), v["span"][1].as_f64()) {
        s.settings.pickup_span = [a as f32, b as f32];
    }
}
