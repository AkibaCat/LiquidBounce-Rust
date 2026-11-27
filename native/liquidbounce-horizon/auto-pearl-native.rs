// SPDX-License-Identifier: GPL-3.0-or-later
//! Raw pearl spawn facts and real collision-backed trajectory continuations.
use super::{
    Game,
    auto_bow_math::{self as math, B, Trajectory, V},
};
use horizon_core::geometry::{Aabb, Vec3};
use serde_json::{Value, json};
thread_local! {static SPAWNS:std::cell::RefCell<Vec<Value>>=const{std::cell::RefCell::new(Vec::new())};}
fn v(p: Vec3) -> V {
    V(p.x, p.y, p.z)
}
fn p(v: V) -> Vec3 {
    Vec3::new(v.0, v.1, v.2)
}
fn b(a: Aabb) -> B {
    B {
        lo: v(a.min_position()),
        hi: v(a.max_position()),
    }
}
fn vector(q: &Value) -> Option<V> {
    let q = q.as_array()?;
    Some(V(
        q.first()?.as_f64()?,
        q.get(1)?.as_f64()?,
        q.get(2)?.as_f64()?,
    ))
}
const GENERIC: Trajectory = Trajectory {
    velocity: 1.5,
    gravity: 0.03,
    drag: 0.99,
    water_drag: 0.8,
    radius: 0.25,
    copies_velocity: true,
};
impl Game<'_> {
    pub(super) fn auto_pearl_spawn(&self, owner: Option<i32>, position: Vec3, velocity: Vec3) {
        if !self.native_window() {
            return;
        }
        let Some(player) = self.player.as_ref() else {
            return;
        };
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
        // Resolve once at receipt, as ThrowableProjectile.recreateFromPacket does.
        let owner = owner.filter(|id| *id == local || self.native_entity(*id as u32).is_some());
        let eye = V(
            player.position.x,
            player.position.y + f64::from(player.dimensions.eye_height),
            player.position.z,
        );
        let view = v(Vec3::direction_from_rotation(player.pitch, player.yaw));
        let bounds = B {
            lo: V(position.x - 0.125, position.y, position.z - 0.125),
            hi: V(position.x + 0.125, position.y + 0.25, position.z + 0.125),
        };
        let canonical = self.native_client_slot_owner();
        let target = owner.filter(|_| canonical).map(|id| {
            serde_json::from_str::<Value>(&self.native_client_targeting_entity(id))
                .unwrap_or(Value::Null)
        });
        let epoch = self
            .remote
            .as_ref()
            .and_then(|r| r.native_epoch())
            .or_else(|| self.native_lifecycle.epoch())
            .map(|epoch| epoch.get());
        let spawn = json!({"epoch":epoch,"owner":owner,"self":owner==Some(local),"target":target,"deferred":!canonical,"cosine":canonical.then(||math::max_cos(eye,view,bounds)),"position":v(position).array(),"velocity":v(velocity).array()});
        SPAWNS.with_borrow_mut(|q| q.push(spawn));
    }
    fn auto_pearl_simulate(&self, position: V, velocity: V, owner: i32) -> Option<V> {
        let boxes = self
            .auto_pearl_collision_boxes(owner)
            .into_iter()
            .map(b)
            .collect::<Vec<_>>();
        super::auto_pearl_model::simulate(
            position,
            velocity,
            f64::from(self.world.shape.min_y),
            &boxes,
            |a, z| {
                super::auto_pearl_ray::clip(self, p(a), p(z), owner, false)
                    .map(|(miss, hit)| (miss, v(hit)))
            },
            |at| {
                let state = self.world.block(
                    at.0.floor() as i32,
                    at.1.floor() as i32,
                    at.2.floor() as i32,
                );
                self.res.facts().get(state).fluid != horizon_client::blocks::Fluid::None
            },
        )
    }
    pub(super) fn auto_pearl_rotation(&self, destination: V, feet: bool) -> Option<[f32; 2]> {
        let player = self.player.as_ref()?;
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
        math::calculate(
            GENERIC,
            if feet {
                v(player.position)
            } else {
                V(
                    player.position.x,
                    player.position.y + f64::from(player.dimensions.eye_height),
                    player.position.z,
                )
            },
            [1., 0.],
            |_| destination,
            |a, z| super::auto_pearl_ray::clip(self, p(a), p(z), local, false).is_some_and(|h| h.0),
        )
    }
    fn auto_pearl_local_hit(&self, rotation: [f32; 2], freeze: bool) -> Option<V> {
        let player = self.player.as_ref()?;
        // Projectile.setVelocity uses Kotlin Float radians followed by Math.sin/cos.
        let yaw = rotation[0] * std::f32::consts::PI / 180.;
        let pitch = rotation[1] * std::f32::consts::PI / 180.;
        let (sy, cy) = ((f64::from(yaw).sin() as f32), (f64::from(yaw).cos() as f32));
        let (sp, cp) = (
            (f64::from(pitch).sin() as f32),
            (f64::from(pitch).cos() as f32),
        );
        let mut velocity = V(
            -f64::from(sy) * f64::from(cp),
            -f64::from(sp),
            f64::from(cy) * f64::from(cp),
        )
        .normalize()
        .scale(1.5);
        if !freeze {
            velocity = velocity.add(V(
                player.velocity.x,
                if player.ground.on_ground {
                    0.
                } else {
                    player.velocity.y
                },
                player.velocity.z,
            ));
        }
        let pos = V(
            player.position.x,
            player.position.y + f64::from(player.dimensions.eye_height) - 0.10000000149011612,
            player.position.z,
        );
        self.auto_pearl_simulate(
            pos,
            velocity,
            self.remote.as_ref().map_or(0, |r| r.entity_id),
        )
    }
    pub(super) fn auto_pearl_query(&self, text: &str) -> String {
        let Ok(q) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        if q["op"] == "clear" {
            SPAWNS.with_borrow_mut(Vec::clear);
            return "null".into();
        }
        let Some(player) = self.player.as_ref() else {
            SPAWNS.with_borrow_mut(Vec::clear);
            return "null".into();
        };
        if q["op"] == "spawns" {
            return json!({"slot":self.native_client_middle_click_pearl_slot(),"player":v(player.position).array(),"spawns":SPAWNS.with_borrow_mut(|spawns|{let epoch=serde_json::to_value(self.remote.as_ref().and_then(|r|r.native_epoch()).or_else(||self.native_lifecycle.epoch()).map(|epoch|epoch.get())).unwrap_or(Value::Null);std::mem::take(spawns).into_iter().filter(|s|s["epoch"]==epoch).map(|mut s|{if s["deferred"]==true{if let Some(at)=vector(&s["position"]){let bounds=B{lo:V(at.0-0.125,at.1,at.2-0.125),hi:V(at.0+0.125,at.1+0.25,at.2+0.125)};let eye=V(player.position.x,player.position.y+f64::from(player.dimensions.eye_height),player.position.z);s["cosine"]=json!(math::max_cos(eye,v(Vec3::direction_from_rotation(player.pitch,player.yaw)),bounds));}s["target"]=s["owner"].as_i64().map(|id|serde_json::from_str::<Value>(&self.native_client_targeting_entity(id as i32)).unwrap_or(Value::Null)).unwrap_or(Value::Null);}s}).collect::<Vec<_>>()})}).to_string();
        }
        if q["op"] == "slot" {
            return json!(self.native_client_middle_click_pearl_slot()).to_string();
        }
        if q["op"] == "angle" {
            let read = |s: &str| {
                q[s].as_array()
                    .filter(|a| a.len() == 2)
                    .map(|a| {
                        [
                            a[0].as_f64().unwrap_or(0.) as f32,
                            a[1].as_f64().unwrap_or(0.) as f32,
                        ]
                    })
                    .unwrap_or([0.; 2])
            };
            let a = read("rotation");
            let z = read("target");
            let a = Vec3::direction_from_rotation(a[1], a[0]);
            let z = Vec3::direction_from_rotation(z[1], z[0]);
            return json!(
                (a.cross(z).length().atan2(a.dot(z))
                    * f64::from(horizon_core::geometry::trig::RAD_TO_DEG)) as f32
            )
            .to_string();
        }
        if q["op"] == "solve" {
            let (Some(position), Some(velocity)) = (vector(&q["position"]), vector(&q["velocity"]))
            else {
                return "null".into();
            };
            let owner = q["owner"].as_i64().map_or_else(
                || self.remote.as_ref().map_or(0, |r| r.entity_id),
                |id| id as i32,
            );
            let Some(destination) = self.auto_pearl_simulate(position, velocity, owner) else {
                return "null".into();
            };
            let Some(rotation) = self.auto_pearl_rotation(destination, false) else {
                return "null".into();
            };
            let hit = self.auto_pearl_local_hit(rotation, q["freeze"] == true);
            return json!({"destination":destination.array(),"rotation":rotation,"hit":hit.map(V::array)}).to_string();
        }
        "null".into()
    }
    // AUTO_PEARL_COLLISION_BOXES
}
