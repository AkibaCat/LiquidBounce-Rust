// SPDX-License-Identifier: GPL-3.0-or-later
//! AutoBuff native inventory facts and hypothetical splash trajectory.
use super::{
    Game,
    auto_walk_player::NativePlayer,
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NavigationRegistry},
};
use crate::navigation::{
    input::Input,
    walk_control::body::{SimulatedInput, WalkSimulation},
};
use horizon_core::geometry::{Aabb, Vec3};
use serde_json::{Value, json};
impl Game<'_> {
    pub(super) fn auto_buff_query(&mut self, text: &str) -> String {
        let Ok(v) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        if v["op"] == "trajectory" {
            let pitch = v["pitch"].as_f64().unwrap_or(85.) as f32;
            return self
                .auto_buff_rotation(pitch, v["freeze"] == true)
                .map_or_else(|| "null".into(), |r| json!(r).to_string());
        }
        let Some(p) = self.player.as_ref() else {
            return "null".into();
        };
        let at = p.position;
        let mut result = json!({"health":p.health.health(),"maxHealth":p.attributes.computed_value_of("minecraft:max_health").unwrap_or(20.),"absorption":p.health.absorption(),"dead":p.health.dead_or_dying(),"creative":self.creative,"spectator":self.spectator(),"tickCount":p.health.tick_count,"selected":self.selected,"serverSelected":self.client_hotbar_slot(),"y":at.y,"inventoryOpen":matches!(self.screen,super::Screen::Inventory|super::Screen::Creative),"effects":self.potions.effects.list.iter().map(|e|e.id).collect::<Vec<_>>()});
        let mut items = vec![];
        for (slot, stack) in self
            .inventory
            .iter()
            .take(36)
            .enumerate()
            .chain(std::iter::once((40, &self.offhand)))
        {
            let Some(s) = stack.as_ref().filter(|s| s.count > 0) else {
                continue;
            };
            let view = horizon_client::item_render::stack_view(s);
            let components = self
                .pack
                .item_renderer
                .models
                .components_view(&view)
                .to_map();
            let potion = components.get("minecraft:potion_contents");
            let mut effects = vec![];
            if let Some(base) = potion
                .and_then(|p| p.get("potion"))
                .and_then(Value::as_str)
                .and_then(horizon_client::brewing::potion)
            {
                effects.extend(base.effects.iter().map(|e| e.0.to_string()));
            }
            if let Some(custom) = potion
                .and_then(|p| p.get("custom_effects"))
                .and_then(Value::as_array)
            {
                for e in custom {
                    if let Some(id) = e["id"].as_str().and_then(horizon_client::brewing::effect) {
                        effects.push(id.id.to_string());
                    }
                }
            }
            let maximum = components
                .get("minecraft:max_stack_size")
                .and_then(Value::as_i64)
                .unwrap_or(1);
            let max_damage = components
                .get("minecraft:max_damage")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let damage = components
                .get("minecraft:damage")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            items.push(json!({"slot":slot,"id":s.item.as_ref(),"components":json!([s.item.as_ref(),components]).to_string(),"count":s.count,"maximum":maximum,"stackable":maximum>1&&(max_damage<=0||damage==0||components.contains_key("minecraft:unbreakable")),"effects":effects}));
        }
        result["items"] = json!(items);
        if v["guards"] != true {
            return result.to_string();
        }
        let distance = |pos: Vec3| {
            let d = pos.subtract(at);
            d.length_squared()
        };
        result["splashNear"] = json!(
            self.potions
                .thrown_entities()
                .iter()
                .any(|(_, p, _)| distance(*p) <= 16.)
        );
        result["lingeringNear"] = json!(self.breath_clouds.iter().any(|c| distance(Vec3::new(
            c.at[0], c.at[1], c.at[2]
        )) <= 16.
            && c.potion.as_ref().is_some_and(|p| {
                p.effects
                    .iter()
                    .any(|e| matches!(e.0, "regeneration" | "instant_health" | "strength"))
            })));
        result["candidates"] = json!(
            super::criticals_target::candidates(self, 8., false)
                .into_iter()
                .filter(|e| {
                    let Some(id) = e["id"].as_i64().map(|i| i as i32) else {
                        return false;
                    };
                    if self.armor_stands.iter().any(|a| a.id as i32 == id) {
                        return false;
                    }
                    let Some((_, p)) = super::criticals_target::tracking(self, id) else {
                        return false;
                    };
                    distance(Vec3::new(p.feet[0], p.feet[1], p.feet[2])) <= 16.
                        && e["target"]["alive"] == true
                })
                .collect::<Vec<_>>()
        );
        let landing = world::with_scope(self, || {
            let epoch = world::read(world::current_epoch);
            let registry = world::read(NavigationRegistry::capture);
            let player = NativePlayer::new(NativeLevel::new(registry, epoch));
            let key = |i: usize| v["keys"][i] == true;
            let mut input = SimulatedInput::new(Input {
                forward: key(0),
                backward: key(1),
                left: key(2),
                right: key(3),
                jump: false,
                shift: key(5),
                sprint: key(6),
            });
            input.force_stay_on_ground_surface =
                horizon_client::navigation_physics::staying_on_surface(false);
            let mut simulation = NativeSimulation::new(
                &player,
                input,
                v["movementYaw"].as_f64().unwrap_or(0.) as f32,
            );
            for _ in 0..20 {
                simulation.tick();
                if simulation.player().on_ground() {
                    return simulation
                        .player()
                        .read(|p| p.ground.supporting_block.map(|b| b.y));
                }
            }
            None
        });
        result["landingY"] = json!(landing);
        result.to_string()
    }
    fn auto_buff_velocity(&self, p: Vec3, v: Vec3) -> Vec3 {
        let state = self
            .world
            .block(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
        let drag = if self.res.facts().get(state).fluid != horizon_client::blocks::Fluid::None {
            0.8
        } else {
            0.99
        };
        Vec3::new(v.x * drag, v.y * drag - 0.05, v.z * drag)
    }
    fn auto_buff_rotation(&self, pitch: f32, freeze: bool) -> Option<[f32; 2]> {
        let p = self.player.as_ref()?;
        let at = p.position;
        let eye = self.eye_position()?;
        let mut yaw = p.yaw;
        let boxes = self.auto_buff_collision_boxes();
        for _ in 0..4 {
            let (y, a, r) = (
                yaw * (std::f32::consts::PI / 180.),
                pitch * (std::f32::consts::PI / 180.),
                (pitch - 20.) * (std::f32::consts::PI / 180.),
            );
            let (sy, cy, ca, sr) = (
                (y as f64).sin() as f32,
                (y as f64).cos() as f32,
                (a as f64).cos() as f32,
                (r as f64).sin() as f32,
            );
            let direction = Vec3::new(
                -f64::from(sy) * f64::from(ca),
                -f64::from(sr),
                f64::from(cy) * f64::from(ca),
            );
            let length = direction.length();
            let mut velocity = if length < 1.0e-4 {
                Vec3::new(0., 0., 0.)
            } else {
                direction.scale(0.5 / length)
            };
            if !freeze {
                velocity = velocity.add_vector(Vec3::new(
                    p.velocity.x,
                    if p.ground.on_ground { 0. } else { p.velocity.y },
                    p.velocity.z,
                ));
            }
            let mut pos = Vec3::new(at.x, eye[1] - 0.10000000149011612, at.z);
            velocity = self.auto_buff_velocity(pos, velocity);
            let mut hit = None;
            let mut count = 0;
            for _ in 1..300 {
                if pos.y < f64::from(self.world.shape.min_y) {
                    break;
                }
                let from = pos;
                pos = pos.add_vector(velocity);
                if let Some((false, end)) = super::spear_kill_ray::clip(self, from, pos) {
                    count += 1;
                    hit = Some(end);
                    break;
                }
                let broad = Aabb::new(
                    from.subtract(Vec3::new(0.25, 0.25, 0.25)),
                    from.add_vector(Vec3::new(0.25, 0.25, 0.25)),
                )
                .expand_towards(velocity)
                .inflate(Vec3::new(1., 1., 1.));
                if boxes.iter().any(|b| {
                    b.intersects(broad) && (b.contains(from) || b.clip(from, pos).is_some())
                }) {
                    return Some([yaw, pitch]);
                }
                velocity = self.auto_buff_velocity(pos, velocity);
                count += 1;
            }
            let Some(hit) = hit else {
                return Some([yaw, pitch]);
            };
            let dx = at.x + p.velocity.x * f64::from(count) - hit.x;
            let dz = at.z + p.velocity.z * f64::from(count) - hit.z;
            if dx * dx + dz * dz < 9. {
                return Some([yaw, pitch]);
            }
            yaw = ((-dx).atan2(dz) as f32) * (180. / std::f32::consts::PI);
        }
        Some([yaw, pitch])
    }
    fn auto_buff_collision_boxes(&self) -> Vec<Aabb> {
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
