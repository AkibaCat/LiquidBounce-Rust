// SPDX-License-Identifier: GPL-3.0-or-later
//! Real client LivingEntity candidates, including dead entities for Targets.Dead.
use super::Game;
use horizon_client::particles::{TrackingBody, TrackingTarget};
use horizon_core::geometry::{Aabb, Axis, Vec3};
use serde_json::{Value, json};
struct Candidate {
    id: i32,
    bounds: Aabb,
    eye: Vec3,
    target: Option<Value>,
}
fn entities(g: &Game<'_>) -> Vec<Candidate> {
    let local = g.remote.as_ref().map_or(0, |r| r.entity_id);
    let mut out: Vec<_> = g
        .mobs
        .iter()
        .filter(|m| {
            !m.removed && m.id as i32 != local && g.dragon.as_ref().is_none_or(|d| d.id != m.id)
        })
        .map(|m| Candidate {
            id: m.id as i32,
            bounds: m.bounds(),
            eye: Vec3::new(m.position.x, m.position.y + m.eye_height(), m.position.z),
            target: None,
        })
        .collect();
    out.extend(g.armor_stands.iter().map(|a| Candidate {
        id: a.id as i32,
        bounds: a.bounds(),
        eye: Vec3::new(
            a.position.x,
            a.position.y + if a.marker { 0. } else { a.probe_dimensions().2 },
            a.position.z,
        ),
        target: None,
    }));
    if let Some(d) = g.dragon.as_ref() {
        let p = d.position;
        out.push(Candidate {
            id: d.id as i32,
            bounds: Aabb::new(
                Vec3::new(p.x - 8., p.y, p.z - 8.),
                Vec3::new(p.x + 8., p.y + 8., p.z + 8.),
            ),
            eye: Vec3::new(p.x, p.y + 6.8, p.z),
            target: None,
        });
    }
    for guest in g
        .lan
        .iter()
        .flat_map(|l| l.hub.guests.iter())
        .filter(|p| p.placed && p.dimension == g.dim.id() && p.entity_id != local)
    {
        if out.iter().any(|p| p.id == guest.entity_id) {
            continue;
        }
        let Some(p) = guest.motion.as_ref() else {
            continue;
        };
        let at = Vec3::new(guest.position[0], guest.position[1], guest.position[2]);
        let within = g.border.geometry().is_within_bounds(
            f64::from(at.x.floor() as i32),
            f64::from(at.z.floor() as i32),
            0.,
        );
        let target = json!({"kind":"minecraft:player","alive":!p.health.dead_or_dying(),"invisible":guest.effects.get("invisibility").is_some(),"sleeping":p.sleeping,"is_local":false,"has_local_passenger":false,"within_border":within,"player_name":guest.name,"abstract_client_player":true,"teams":g.client_teams_facts(guest.entity_id)});
        out.push(Candidate {
            id: guest.entity_id,
            bounds: p.dimensions.bounds(at),
            eye: Vec3::new(at.x, at.y + f64::from(p.dimensions.eye_height), at.z),
            target: Some(target),
        });
    }
    out
}
pub(super) fn candidates(g: &Game<'_>, range: f32, visible: bool) -> Vec<Value> {
    let Some(p) = g.player.as_ref() else {
        return vec![];
    };
    let at = p.position;
    let eye = Vec3::new(at.x, at.y + f64::from(p.dimensions.eye_height), at.z);
    let r = f64::from(range);
    let query = Aabb::new(
        Vec3::new(eye.x - r, eye.y - r, eye.z - r),
        Vec3::new(eye.x + r, eye.y + r, eye.z + r),
    );
    entities(g).into_iter().filter(|e|query.intersects(e.bounds)&&e.bounds.distance_squared_to_point(eye)<=r*r&&(!visible||super::spear_kill_ray::clip(g,eye,e.eye).is_some_and(|p|p.0))).map(|e|json!({"id":e.id,"target":e.target.unwrap_or_else(||serde_json::from_str(&g.native_client_targeting_entity(e.id)).unwrap_or(Value::Null))})).collect()
}
pub(super) fn tracking(g: &Game<'_>, id: i32) -> Option<(TrackingTarget, TrackingBody)> {
    let e = entities(g).into_iter().find(|e| e.id == id)?;
    let b = e.bounds;
    let at = b.min_position();
    Some((
        TrackingTarget::Entity(id),
        TrackingBody {
            feet: [
                (b.min(Axis::X) + b.max(Axis::X)) / 2.,
                at.y,
                (b.min(Axis::Z) + b.max(Axis::Z)) / 2.,
            ],
            width: b.max(Axis::X) - b.min(Axis::X),
            height: b.max(Axis::Y) - b.min(Axis::Y),
        },
    ))
}
