// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use serde_json::{Value, json};
#[derive(Default)]
pub(super) struct Bodies {
    guests: std::collections::BTreeMap<u128, [f32; 2]>,
}
fn wrap(v: f32) -> f32 {
    let v = v % 360.;
    if v >= 180. {
        v - 360.
    } else if v < -180. {
        v + 360.
    } else {
        v
    }
}
fn rot(old: f32, new: f32, alpha: f64) -> f32 {
    old + alpha as f32 * wrap(new - old)
}
fn position(a: [f64; 3], b: [f64; 3], alpha: f64) -> [f64; 3] {
    std::array::from_fn(|i| alpha.mul_add(b[i] - a[i], a[i]))
}
// Same player tickHeadTurn path as native perspective::update_player_model,
// fed the real guest's movement, swing and vehicle state. Guests do not carry yBodyRot.
pub(super) fn guest_body(
    old: f32,
    yaw: f32,
    dx: f64,
    dz: f64,
    swing: bool,
    vehicle: Option<f32>,
) -> f32 {
    if let Some(body) = vehicle {
        return old + wrap(body - old);
    }
    let mut target = old;
    if dx * dx + dz * dz > 0.002_500_000_2 {
        let walk = (dz.atan2(dx) as f32).to_degrees() - 90.;
        let facing = (wrap(yaw) - walk).abs();
        target = if 95. < facing && facing < 265. {
            walk - 180.
        } else {
            walk
        };
    }
    if swing {
        target = yaw;
    }
    let mut body = old + wrap(target - old) * 0.3;
    let diff = wrap(yaw - body);
    if diff.abs() > 50. {
        body += diff - diff.signum() * 50.;
    }
    body
}
impl Game<'_> {
    pub(super) fn client_wings_tick(&mut self) {
        let mut bodies = self.client_wings_bodies.borrow_mut();
        let mut live = std::collections::BTreeSet::new();
        for host in self.lan.iter() {
            for g in host
                .hub
                .guests
                .iter()
                .filter(|g| g.placed && g.dimension == self.dim.id())
            {
                live.insert(g.uuid);
                let old = bodies.guests.get(&g.uuid).map_or(g.yaw, |v| v[1]);
                let swinging = host
                    .play
                    .get(&g.conn)
                    .is_some_and(|p| p.client_wings_swinging());
                let vehicle = g.vehicle.and_then(|id| {
                    self.boats
                        .iter()
                        .find(|b| b.id == id)
                        .map(|b| b.yaw)
                        .or_else(|| {
                            self.mobs.iter().find(|m| m.id == id).map(|m| {
                                horizon_client::mobs::riders::solve_body_rot(g.yaw, m.body_yaw)
                            })
                        })
                });
                let now = guest_body(
                    old,
                    g.yaw,
                    g.position[0] - g.previous_position[0],
                    g.position[2] - g.previous_position[2],
                    swinging,
                    vehicle,
                );
                bodies.guests.insert(g.uuid, [old, now]);
            }
        }
        bodies.guests.retain(|id, _| live.contains(id));
    }
    fn client_wings_entities(&self, alpha: f64) -> Vec<Value> {
        let mut out = vec![];
        if let Some(p) = self.player.as_ref() {
            let at = [p.position.x, p.position.y, p.position.z];
            let before = [
                self.previous_position.x,
                self.previous_position.y,
                self.previous_position.z,
            ];
            let mut body = self
                .player_model
                .as_ref()
                .map_or(self.yaw, |m| rot(m.previous_body_yaw, m.body_yaw, alpha));
            let (mut head, mut pitch) = (self.yaw, self.pitch);
            self.native_client_rotation_model(alpha as f32, &mut body, &mut head, &mut pitch);
            out.push(json!({"local":true,"position":at,"interpolated":position(before,at,alpha),"bodyYaw":body,"scale":p.attributes.computed_value_of("minecraft:scale").expect("registered player scale")as f32,"height":p.dimensions.height,"chest":self.armor[1].as_ref().is_some_and(|s|s.count>0),"hurt":p.health.hurt.hurt_time!=0}));
        }
        for m in self
            .mobs
            .iter()
            .filter(|m| !m.removed && m.kind.id == "minecraft:player")
        {
            let at = [m.position.x, m.position.y, m.position.z];
            let before = [m.previous.x, m.previous.y, m.previous.z];
            out.push(json!({"local":false,"name":self.remote_player_name(m.id),"position":at,"interpolated":position(before,at,alpha),"bodyYaw":rot(m.previous_body_yaw,m.body_yaw,alpha),"scale":m.model_scale(),"height":m.dims().1 as f32,"chest":m.equipment_stacks()[4].as_ref().is_some_and(|s|s.count>0),"hurt":m.hurt_time!=0}));
        }
        let bodies = self.client_wings_bodies.borrow();
        for host in self.lan.iter() {
            for g in host
                .hub
                .guests
                .iter()
                .filter(|g| g.placed && g.dimension == self.dim.id())
            {
                let Some(p) = g.motion.as_ref() else { continue };
                let body = bodies
                    .guests
                    .get(&g.uuid)
                    .map_or(g.yaw, |v| rot(v[0], v[1], alpha));
                out.push(json!({"local":false,"name":g.name,"position":g.position,"interpolated":position(g.previous_position,g.position,alpha),"bodyYaw":body,"scale":p.attributes.computed_value_of("minecraft:scale").expect("registered player scale")as f32,"height":p.dimensions.height,"chest":g.armor[1].as_ref().is_some_and(|s|s.count>0),"hurt":p.health.hurt.hurt_time!=0}));
            }
        }
        out
    }
    pub(super) fn client_wings_frame(&self, alpha: f64) {
        let input=json!({"firstPerson":self.camera_type.first_person(),"shift":self.player.as_ref().is_some_and(|p|p.shift_key_down),"entities":self.client_wings_entities(alpha)}).to_string();
        let mut geometry = "null".to_owned();
        self.native_client_wings(&input, &mut geometry);
        horizon_client::gfx::wings::begin_frame(&geometry);
    }
    pub(super) fn client_wings_reset(&self) {
        self.client_wings_bodies.borrow_mut().guests.clear();
        horizon_client::gfx::wings::begin_frame("null");
    }
}
