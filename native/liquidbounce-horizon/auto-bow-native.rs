// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual held-stack facts, rendering-order entities, physics forecasts and projectile collisions.
use super::{
    Game,
    auto_bow_math::{self as math, B, Trajectory, V},
};
use horizon_core::geometry::{Aabb, Vec3};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
static LAST_BOW_STOP: AtomicI64 = AtomicI64::new(0);
static UPDATING_USE: AtomicBool = AtomicBool::new(false);
pub(super) struct UpdatingUse;
impl Drop for UpdatingUse {
    fn drop(&mut self) {
        UPDATING_USE.store(false, Ordering::Relaxed);
    }
}
pub(super) fn begin_update() -> UpdatingUse {
    UPDATING_USE.store(true, Ordering::Relaxed);
    UpdatingUse
}
pub(super) fn updating() -> bool {
    UPDATING_USE.load(Ordering::Relaxed)
}
pub(super) fn stopped(stack: &horizon_client::items::ItemStack) {
    stopped_id(stack.item.as_ref());
}
pub(super) fn stopped_id(id: &str) {
    if id == "minecraft:bow" {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as i64);
        LAST_BOW_STOP.store(now, Ordering::Relaxed);
    }
}
fn v(p: Vec3) -> V {
    V(p.x, p.y, p.z)
}
fn p(v: V) -> Vec3 {
    Vec3::new(v.0, v.1, v.2)
}
fn bounds(b: Aabb) -> B {
    B {
        lo: v(b.min_position()),
        hi: v(b.max_position()),
    }
}
fn raw(b: Aabb) -> [f64; 6] {
    let a = b.min_position();
    let z = b.max_position();
    [a.x, a.y, a.z, z.x, z.y, z.z]
}
pub(super) struct Entity {
    pub id: i32,
    pub position: Vec3,
    pub previous: Vec3,
    pub velocity: Vec3,
    pub bounds: Aabb,
    dimensions: [f32; 2],
    health: f32,
    hurt: i32,
    age: i32,
    weight: i32,
    order: ([i32; 3], u64),
    target: Value,
}
pub(super) fn entities(g: &Game<'_>) -> Vec<Entity> {
    let mut result = vec![];
    for candidate in super::criticals_target::candidates(g, f32::MAX, false) {
        let Some(id) = candidate["id"].as_i64().map(|id| id as i32) else {
            continue;
        };
        let target = candidate["target"].clone();
        let (at, old, velocity, b, dimensions, health, hurt, age, weight, seq) = if let Some(m) =
            g.mobs.iter().find(|m| m.id as i32 == id)
        {
            let (w, h, _) = m.dims();
            let player = horizon_client::command::args::normalize(&m.kind.id) == "minecraft:player";
            (
                m.position,
                m.previous,
                m.velocity,
                m.bounds(),
                [w as f32, h as f32],
                m.health,
                m.hurt_time,
                m.age as i32,
                if player {
                    0
                } else if m.is_enemy() {
                    1
                } else {
                    i32::MAX
                },
                if m.extra.section_seq == 0 {
                    m.join.get()
                } else {
                    m.extra.section_seq
                },
            )
        } else if let Some(a) = g.armor_stands.iter().find(|a| a.id as i32 == id) {
            let (w, h, _) = a.probe_dimensions();
            (
                a.position,
                a.previous,
                a.velocity,
                a.bounds(),
                [w as f32, h as f32],
                a.health,
                0,
                a.liquidbounce_tick_count,
                i32::MAX,
                a.join.get(),
            )
        } else if let Some(d) = g.dragon.as_ref().filter(|d| d.id as i32 == id) {
            let at = d.position;
            (
                at,
                d.previous,
                d.velocity,
                Aabb::new(
                    at.add_vector(Vec3::new(-8., 0., -8.)),
                    at.add_vector(Vec3::new(8., 8., 8.)),
                ),
                [16., 8.],
                d.health,
                d.hurt_time,
                d.ticks,
                1,
                0,
            )
        } else {
            let Some(guest) = g
                .lan
                .iter()
                .flat_map(|l| l.hub.guests.iter())
                .find(|x| x.entity_id == id)
            else {
                continue;
            };
            let Some(body) = guest.motion.as_ref() else {
                continue;
            };
            let at = Vec3::new(guest.position[0], guest.position[1], guest.position[2]);
            let old = Vec3::new(
                guest.previous_position[0],
                guest.previous_position[1],
                guest.previous_position[2],
            );
            (
                at,
                old,
                body.velocity,
                body.dimensions.bounds(at),
                [body.dimensions.width, body.dimensions.height],
                body.health.health(),
                body.health.hurt.hurt_time,
                body.health.tick_count as i32,
                0,
                u64::from(guest.conn),
            )
        };
        let board = &g.score.board;
        let health = board
            .display_objective("below_name")
            .filter(|o| {
                ["❤", "HP", "Health", "Здоровья", "Здоровье"]
                    .iter()
                    .any(|word| o.display_text().plain().contains(word))
            })
            .and_then(|o| {
                let name = target["player_name"]
                    .as_str()
                    .map(str::to_owned)
                    .or_else(|| {
                        g.persist
                            .uuids
                            .get(&(id as u32))
                            .map(|u| g.scoreboard_name(id as u32, *u))
                    })?;
                board.score(&name, &o.name).map(|s| s.value as f32)
            })
            .unwrap_or(health);
        let order = (
            [
                (at.x.floor() as i32) >> 4,
                (at.y.floor() as i32) >> 4,
                (at.z.floor() as i32) >> 4,
            ],
            seq,
        );
        result.push(Entity {
            id,
            position: at,
            previous: old,
            velocity,
            bounds: b,
            dimensions,
            health,
            hurt,
            age,
            weight,
            order,
            target,
        });
    }
    result.sort_by_key(|e| {
        let [x, y, z] = e.order.0;
        (x, z, y, e.order.1)
    });
    result
}
impl Game<'_> {
    fn auto_bow_item(&self, slot: usize, ticks: i32) -> Option<Value> {
        let stack = self.stack_in(slot)?;
        let item = stack.item.as_ref();
        let weapon = match item {
            "minecraft:bow" => "Bow",
            "minecraft:crossbow" => "Crossbow",
            "minecraft:trident" => "Trident",
            _ => return None,
        };
        let view = horizon_client::item_render::stack_view(stack);
        let c = self.pack.item_renderer.models.components_view(&view);
        let projectiles = c
            .get("minecraft:charged_projectiles")
            .and_then(Value::as_array);
        let charged = projectiles.is_some_and(|a| !a.is_empty());
        let fireworks =
            projectiles.is_some_and(|a| a.iter().any(|s| s["id"] == "minecraft:firework_rocket"));
        Some(
            json!({"weapon":weapon,"hand":if slot==super::shield::OFFHAND_SLOT{1}else{0},"ticks":ticks,"charged":charged,"fireworks":fireworks,"pullTime":Self::charge_ticks(stack)}),
        )
    }
    fn auto_bow_trajectory(item: &Value, always: bool, using: bool) -> Option<Trajectory> {
        match item["weapon"].as_str()? {
            "Bow" => Trajectory::bow(if always && !using {
                40
            } else {
                item["ticks"].as_i64().unwrap_or(0) as i32
            }),
            "Crossbow" => Some(Trajectory::crossbow(item["fireworks"] == true)),
            "Trident" => Some(Trajectory::trident()),
            _ => None,
        }
    }
    pub(super) fn auto_bow_query(&mut self, text: &str) -> String {
        let Ok(query) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        let Some(player) = self.player.as_ref() else {
            return "null".into();
        };
        let Some(eye) = self.eye_position() else {
            return "null".into();
        };
        let eye = V(eye[0], eye[1], eye[2]);
        let using = self
            .using
            .and_then(|(slot, ticks)| {
                self.stack_in(slot)
                    .map(|s| (slot, self.ticks_held(s, ticks)))
            })
            .and_then(|(slot, ticks)| self.auto_bow_item(slot, ticks as i32));
        let precharged = [self.client_hotbar_slot(), super::shield::OFFHAND_SLOT]
            .into_iter()
            .filter_map(|slot| self.auto_bow_item(slot, 0))
            .find(|i| i["weapon"] == "Crossbow" && i["charged"] == true);
        if query["op"] == "using" {
            return json!({"using":using,"isUsing":self.using.is_some()}).to_string();
        }
        if query["op"] == "angle" {
            let read = |key: &str| {
                query[key]
                    .as_array()
                    .map(|a| {
                        [
                            a[0].as_f64().unwrap_or(0.) as f32,
                            a[1].as_f64().unwrap_or(0.) as f32,
                        ]
                    })
                    .unwrap_or([0., 0.])
            };
            let a = read("rotation");
            let b = read("target");
            let x = Vec3::direction_from_rotation(a[1], a[0]);
            let y = Vec3::direction_from_rotation(b[1], b[0]);
            return json!(
                x.cross(y).length().atan2(x.dot(y))
                    * f64::from(horizon_core::geometry::trig::RAD_TO_DEG)
            )
            .to_string();
        }
        if query["op"] == "snapshot" {
            let view = v(Vec3::direction_from_rotation(player.pitch, player.yaw));
            let candidates=entities(self).into_iter().map(|e|json!({"id":e.id,"target":e.target,"type":e.weight,"health":e.health,"hurt":e.hurt,"age":e.age,"distance":bounds(e.bounds).distance(eye),"cosine":math::max_cos(eye,view,bounds(e.bounds))})).collect::<Vec<_>>();
            return json!({"using":using,"isUsing":self.using.is_some(),"precharged":precharged,"ground":player.ground.on_ground,"moving":player.move_vector!=[0.,0.],"regeneration":self.potions.effects.get("regeneration").is_some(),"lastBowStop":LAST_BOW_STOP.load(Ordering::Relaxed),"candidates":candidates}).to_string();
        }
        let rotation = query["rotation"]
            .as_array()
            .map(|r| {
                [
                    r[0].as_f64().unwrap_or(0.) as f32,
                    r[1].as_f64().unwrap_or(0.) as f32,
                ]
            })
            .unwrap_or([self.yaw, self.pitch]);
        if query["op"] == "hypothetical" {
            let Some(item) = using.as_ref() else {
                return "null".into();
            };
            let Some(info) = Self::auto_bow_trajectory(item, false, true) else {
                return "null".into();
            };
            let allowed = query["allowed"]
                .as_array()
                .map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
                .unwrap_or_default();
            let view = v(Vec3::direction_from_rotation(player.pitch, player.yaw));
            let targets = entities(self)
                .into_iter()
                .filter(|e| {
                    allowed.contains(&i64::from(e.id)) && {
                        let d = v(e.position).sub(eye);
                        let nearest = eye.fma(d.dot(view) / view.len2(), view);
                        v(e.position).sub(nearest).len2() < 100.
                    }
                })
                .map(|e| {
                    let forecast = (e.weight == 0)
                        .then(|| super::auto_bow_forecast::positions(self, e.id, 39))
                        .flatten();
                    (e, forecast)
                })
                .collect::<Vec<_>>();
            let mut pos = eye;
            let mut velocity =
                v(Vec3::direction_from_rotation(rotation[1], rotation[0])).scale(info.velocity);
            let mut in_ground = false;
            for tick in 0..40 {
                let before = pos;
                if !in_ground {
                    let next = pos.add(velocity);
                    velocity = velocity.scale(0.99).sub(V(0., 0.05000000074505806, 0.));
                    match super::auto_bow_ray::clip(self, p(pos), p(next), false) {
                        Some((false, hit)) => {
                            pos = v(hit);
                            in_ground = true;
                        }
                        Some((true, _)) => pos = next,
                        None => return "null".into(),
                    }
                }
                for (entity, forecast) in &targets {
                    let predicted = forecast
                        .as_ref()
                        .and_then(|f| f.get(tick))
                        .map(|a| V(a[0], a[1], a[2]))
                        .unwrap_or_else(|| v(entity.position).fma(tick as f64, v(entity.velocity)));
                    let margin = ((tick as f32 - 2.) / 20.).clamp(0., 0.3);
                    let b = entity
                        .bounds
                        .inflate(Vec3::new(
                            f64::from(margin),
                            f64::from(margin),
                            f64::from(margin),
                        ))
                        .move_by(p(predicted.sub(v(entity.position))));
                    if b.clip(p(before), p(pos)).is_some() {
                        return json!(entity.id).to_string();
                    }
                }
            }
            return "null".into();
        }
        if query["op"] == "aim" {
            if self.using.is_some() && using.is_none() {
                return "null".into();
            }
            let Some(id) = query["id"].as_i64() else {
                return "null".into();
            };
            let Some(entity) = entities(self).into_iter().find(|e| i64::from(e.id) == id) else {
                return "null".into();
            };
            let Some(item) = using.as_ref().or(precharged.as_ref()) else {
                return "null".into();
            };
            let Some(info) = Self::auto_bow_trajectory(item, true, using.is_some()) else {
                return "null".into();
            };
            let forecast = (entity.weight == 0)
                .then(|| super::auto_bow_forecast::positions(self, entity.id, 30))
                .flatten();
            let angle = math::calculate(
                info,
                eye,
                entity.dimensions,
                |t| {
                    if let Some(f) = forecast.as_ref() {
                        let t = (t + 0.5).floor().clamp(0., 30.) as usize;
                        let a = f[t.min(f.len() - 1)];
                        V(a[0], a[1], a[2])
                    } else {
                        v(entity.position).fma(t, v(entity.velocity))
                    }
                },
                |from, to| {
                    super::auto_bow_ray::clip(self, p(from), p(to), true).is_some_and(|hit| hit.0)
                },
            );
            let Some(angle) = angle else {
                return "null".into();
            };
            if query["throughWalls"] != true
                && !self.auto_bow_verified(info, angle, &entity, query["freeze"] == true)
            {
                return "null".into();
            }
            return json!(angle).to_string();
        }
        "null".into()
    }
    fn auto_bow_verified(
        &self,
        info: Trajectory,
        angle: [f32; 2],
        target: &Entity,
        freeze: bool,
    ) -> bool {
        let Some(player) = self.player.as_ref() else {
            return false;
        };
        let Some(eye) = self.eye_position() else {
            return false;
        };
        let mut pos = Vec3::new(
            player.position.x,
            eye[1] - 0.10000000149011612,
            player.position.z,
        );
        let yaw = angle[0] * (std::f32::consts::PI / 180.);
        let pitch = angle[1] * (std::f32::consts::PI / 180.);
        let sy = f64::from(yaw).sin() as f32;
        let cy = f64::from(yaw).cos() as f32;
        let sp = f64::from(pitch).sin() as f32;
        let cp = f64::from(pitch).cos() as f32;
        let direction = V(
            -f64::from(sy) * f64::from(cp),
            -f64::from(sp),
            f64::from(cy) * f64::from(cp),
        );
        let length = direction.len2();
        let mut velocity = if length.abs() < 1e-5 {
            Vec3::ZERO
        } else {
            p(direction.scale(info.velocity / length.sqrt()))
        };
        if info.copies_velocity && !freeze {
            velocity = velocity.add_vector(Vec3::new(
                player.velocity.x,
                if player.ground.on_ground {
                    0.
                } else {
                    player.velocity.y
                },
                player.velocity.z,
            ));
        }
        let boxes = self.auto_bow_collision_boxes();
        for _ in 0..300 {
            if pos.y < f64::from(self.world.shape.min_y) {
                break;
            }
            let before = pos;
            pos = pos.add_vector(velocity);
            if super::spear_kill_ray::clip(self, before, pos).is_none_or(|hit| !hit.0) {
                return false;
            }
            let r = info.radius;
            let broad = Aabb::new(
                before.subtract(Vec3::new(r, r, r)),
                before.add_vector(Vec3::new(r, r, r)),
            )
            .expand_towards(velocity)
            .inflate(Vec3::new(1., 1., 1.));
            let mut closest = f64::MAX;
            let mut hit = None;
            for b in &boxes {
                if !b.intersects(broad) {
                    continue;
                }
                if let Some(at) = b.clip(before, pos) {
                    let d = before.distance_squared(at);
                    if d < closest {
                        closest = d;
                        hit = Some(*b);
                    }
                }
            }
            if let Some(b) = hit {
                let at = target.position;
                let half = f64::from(target.dimensions[0]) * 0.5;
                let expected = Aabb::new(
                    at.add_vector(Vec3::new(-half, 0., -half)),
                    at.add_vector(Vec3::new(half, f64::from(target.dimensions[1]), half)),
                );
                return b.intersects(expected);
            }
            let state = self.world.block(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            );
            let drag = if self.res.facts().get(state).fluid != horizon_client::blocks::Fluid::None {
                info.water_drag
            } else {
                info.drag
            };
            velocity = velocity
                .scale(drag)
                .subtract(Vec3::new(0., info.gravity, 0.));
        }
        false
    }
    fn auto_bow_collision_boxes(&self) -> Vec<Aabb> {
        let mut candidates = vec![];
        let root = self.riding.map(|r| self.client_reach_root_vehicle(r));
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id) as u32;
        let mut offer = |_kind: u8,
                         _index: usize,
                         id: u32,
                         _local_id: u32,
                         _position: [f64; 3],
                         bounds: Aabb,
                         _pick_radius: f64| {
            if id != local && root.is_none_or(|r| r != self.client_reach_root_vehicle(id)) {
                candidates.push(bounds);
            }
        };
        for (i, m) in self
            .mobs
            .iter()
            .enumerate()
            .filter(|(_, m)| !m.removed && !m.extra.spectator && m.alive())
        {
            offer(
                0,
                i,
                m.id,
                m.id,
                [m.position.x, m.position.y, m.position.z],
                m.bounds(),
                0.0,
            );
        }
        let box_of = |(lo, hi): ([f64; 3], [f64; 3])| {
            Aabb::new(
                Vec3::new(lo[0], lo[1], lo[2]),
                Vec3::new(hi[0], hi[1], hi[2]),
            )
        };
        for (i, p) in self.paintings.iter().enumerate() {
            let (lo, hi) = p.bounds();
            offer(
                1,
                i,
                p.id,
                p.id,
                std::array::from_fn(|a| (lo[a] + hi[a]) * 0.5),
                box_of((lo, hi)),
                0.0,
            );
        }
        for (i, p) in self.item_frames.iter().enumerate() {
            let (lo, hi) = p.bounds();
            offer(
                2,
                i,
                p.id,
                p.id,
                std::array::from_fn(|a| (lo[a] + hi[a]) * 0.5),
                box_of((lo, hi)),
                0.0,
            );
        }
        for (i, p) in self
            .armor_stands
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.marker)
        {
            offer(
                3,
                i,
                p.id,
                p.id,
                [p.position.x, p.position.y, p.position.z],
                p.bounds(),
                0.0,
            );
        }
        for (i, p) in self.cushions.iter().enumerate() {
            offer(
                4,
                i,
                p.id,
                p.id,
                [p.position.x, p.position.y, p.position.z],
                box_of(p.bounds()),
                0.0,
            );
        }
        for (i, p) in self.boats.iter().enumerate() {
            offer(
                5,
                i,
                p.id,
                p.id,
                [p.position.x, p.position.y, p.position.z],
                p.bounds(),
                0.0,
            );
        }
        for (i, p) in self.carts.iter().enumerate() {
            offer(
                6,
                i,
                p.id,
                p.id,
                [p.position.x, p.position.y, p.position.z],
                p.bounds(),
                0.0,
            );
        }
        for (i, (&id, p)) in self.interactions.iter().enumerate() {
            offer(
                7,
                i,
                id as u32,
                id as u32,
                [p.position.x, p.position.y, p.position.z],
                p.bounds(),
                0.0,
            );
        }
        for (i, p) in self
            .creatures
            .projectiles
            .iter()
            .enumerate()
            .filter(|(_, p)| super::deflect::attackable(p))
        {
            let radius = if matches!(
                p.kind,
                horizon_client::projectiles::ProjectileKind::ShulkerBullet { .. }
            ) {
                0.0
            } else {
                1.0
            };
            offer(
                8,
                i,
                p.id,
                p.id,
                [p.position.x, p.position.y, p.position.z],
                p.bounds(),
                radius,
            );
        }
        for (i, (&id, &block)) in self.remote_knots.iter().enumerate() {
            let knot = horizon_client::mobs::leash::knot_state(block);
            let p = knot.position;
            let half = knot.width * 0.5;
            offer(
                9,
                i,
                id,
                id,
                [p.x, p.y, p.z],
                Aabb::new(
                    Vec3::new(p.x - half, p.y, p.z - half),
                    Vec3::new(p.x + half, p.y + knot.height, p.z + half),
                ),
                0.0,
            );
        }
        for (i, p) in self.crystals.iter().enumerate() {
            let pos = p.position;
            offer(
                10,
                i,
                p.id,
                super::dragon::CRYSTAL_IDS + i as u32,
                [pos.x, pos.y, pos.z],
                Aabb::new(
                    Vec3::new(pos.x - 1.0, pos.y, pos.z - 1.0),
                    Vec3::new(pos.x + 1.0, pos.y + 2.0, pos.z + 1.0),
                ),
                0.0,
            );
        }
        if let Some(d) = self.dragon.as_ref().filter(|d| d.alive()) {
            for i in 0..8 {
                let p = d.parts[i].0;
                offer(
                    11,
                    i,
                    d.id + 1 + i as u32,
                    super::dragon::SPECIAL + i as u32,
                    [p.x, p.y, p.z],
                    d.part_box(i),
                    0.0,
                );
            }
        }

        for guest in self
            .lan
            .iter()
            .flat_map(|l| l.hub.guests.iter())
            .filter(|g| g.placed && g.dimension == self.dim.id() && g.entity_id as u32 != local)
        {
            if let Some(p) = guest.motion.as_ref().filter(|p| !p.health.dead_or_dying()) {
                let at = Vec3::new(guest.position[0], guest.position[1], guest.position[2]);
                offer(
                    0,
                    0,
                    guest.entity_id as u32,
                    guest.entity_id as u32,
                    guest.position,
                    p.dimensions.bounds(at),
                    0.,
                );
            }
        }
        candidates
    }
}
