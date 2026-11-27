// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{
    first_person::{Hand, SwingAnimation},
    projectile_puncher_math as m,
    projectiles::ProjectileKind,
};
use horizon_core::geometry::{Aabb, Vec3};
use serde_json::{Value, json};
fn raw(b: Aabb) -> m::Bounds {
    let a = b.min_position();
    let z = b.max_position();
    [a.x, a.y, a.z, z.x, z.y, z.z]
}
fn vec(p: Vec3) -> m::Vec3 {
    [p.x, p.y, p.z]
}
fn value(v: m::Vec3) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}
fn candidate(g: &Game<'_>, p: &horizon_client::projectiles::Projectile) -> Option<m::Candidate> {
    let kind = match p.kind {
        ProjectileKind::LargeFireball { .. } => 0,
        ProjectileKind::ShulkerBullet { .. } => 1,
        ProjectileKind::WindCharge | ProjectileKind::BreezeWindCharge => 2,
        _ => return None,
    };
    let [min_x, min_z, max_x, max_z] = g.border.geometry().bounds(0.);
    let x = f64::from(p.position.x.floor() as i32);
    let z = f64::from(p.position.z.floor() as i32);
    Some(m::Candidate {
        id: p.id,
        kind,
        within_border: x + 1. > min_x && x < max_x && z + 1. > min_z && z < max_z,
        position: vec(p.position),
        previous: vec(p.previous),
        physical: raw(p.bounds()),
        pick_radius: g.client_pick_margin(p.id as i32, if kind == 1 { 0. } else { 1. }),
        dimensions: [p.kind.size(); 2],
    })
}
fn snapshot(c: m::Candidate) -> Value {
    json!({"id":c.id,"position":c.position,"physical":c.physical,"radius":c.pick_radius})
}
fn outline(g: &Game<'_>, from: m::Vec3, to: m::Vec3) -> bool {
    use horizon_world::world_ray::{RayBlockMode, RayFluidMode};
    super::client_ghost_hand::clip(
        g,
        &mut super::no_web_ray::NoWebRay(g),
        value(from),
        value(to),
        RayBlockMode::Outline,
        RayFluidMode::None,
        4096,
    )
    .is_ok_and(|h| h.miss)
}
impl Game<'_> {
    pub(super) fn client_projectile_puncher_plan(
        &self,
        range: f32,
        selected: u8,
        only_air: bool,
        fov: f32,
        yaw: f32,
        pitch: f32,
    ) -> Option<Value> {
        let player = self.player.as_ref()?;
        if self.spectator() {
            return Some(json!({"spectator":true}));
        }
        let empty = || json!({"spectator":false,"target":null});
        if only_air && player.ground.on_ground {
            return Some(empty());
        }
        let eye = [
            player.position.x,
            player.position.y + f64::from(player.dimensions.eye_height),
            player.position.z,
        ];
        let physical = raw(player.bounds());
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
        let player_pick = m::inflate(physical, self.client_pick_margin(local, 0.));
        let range_sq = f64::from(range * range);
        let physical_view = super::client_rotation_direction(self.yaw, self.pitch);
        let base_view = super::client_rotation_direction(yaw, pitch);
        let mut candidates: Vec<_> = self
            .creatures
            .projectiles
            .iter()
            .filter_map(|p| candidate(self, p).map(|c| (c, p.join.get())))
            .collect();
        candidates.sort_by(|(a, ai), (b, bi)| {
            m::distance_squared(a.pick_box(), eye)
                .total_cmp(&m::distance_squared(b.pick_box(), eye))
                .then(ai.cmp(bi))
        });
        for (c, _) in candidates {
            if !m::should_attack(c, selected, physical, player_pick)
                || m::distance_squared(m::predicted_box(c), eye) > range_sq
            {
                continue;
            }
            if fov < 180.
                && horizon_core::geometry::trig::cos(f64::from(fov).to_radians()) as f64
                    > m::max_cos(c.physical, eye, physical_view)
            {
                continue;
            }
            if let Some(rotation) = m::raytrace_box(
                eye,
                c.pick_box(),
                f64::from(range),
                [yaw, pitch],
                base_view,
                &mut |a, b| outline(self, a, b),
            ) {
                return Some(json!({"spectator":false,"target":snapshot(c),"rotation":rotation}));
            }
        }
        Some(empty())
    }
    pub(super) fn client_projectile_puncher_ready(
        &self,
        target: &str,
        range: f32,
        yaw: f32,
        pitch: f32,
    ) -> bool {
        let Ok(old) = serde_json::from_str::<Value>(target) else {
            return false;
        };
        let Some(id) = old["id"].as_u64().map(|v| v as u32) else {
            return false;
        };
        let current = self
            .creatures
            .projectiles
            .iter()
            .find(|p| p.id == id)
            .and_then(|p| candidate(self, p))
            .map(snapshot)
            .unwrap_or(old);
        let (Ok(physical), Ok(position)) = (
            serde_json::from_value::<[f64; 6]>(current["physical"].clone()),
            serde_json::from_value::<[f64; 3]>(current["position"].clone()),
        ) else {
            return false;
        };
        let Some(player) = self.player.as_ref() else {
            return false;
        };
        let player_eye = [
            player.position.x,
            player.position.y + f64::from(player.dimensions.eye_height),
            player.position.z,
        ];
        let radius = current["radius"].as_f64().unwrap_or(0.);
        if m::distance_squared(m::inflate(physical, radius), player_eye) > f64::from(range * range)
        {
            return false;
        }
        let local = self.remote.as_ref().map_or(0, |r| r.entity_id) as u32;
        let camera_id = self.remote_camera_id().unwrap_or(local);
        if id == camera_id {
            return false;
        }
        let camera = self.camera_entity();
        let eye = camera.map_or(player_eye, |c| {
            [
                c.position[0],
                c.position[1] + f64::from(c.eye_height),
                c.position[2],
            ]
        });
        let bounds = if camera.is_some() {
            let Some(entity) = self.native_entity(camera_id) else {
                return false;
            };
            let Some([lo, hi]) = self.native_entity_bounding_box(entity) else {
                return false;
            };
            Aabb::new(value(lo), value(hi))
        } else {
            player.bounds()
        };
        let root = if camera_id == local {
            self.riding
                .map(|id| self.client_reach_root_vehicle(id))
                .unwrap_or(local)
        } else {
            self.client_reach_root_vehicle(camera_id)
        };
        let candidates = [super::ClientReachCandidate {
            kind: 8,
            index: 0,
            id: id as i32,
            local_id: id,
            position,
            bounds: Aabb::new(
                value([physical[0], physical[1], physical[2]]),
                value([physical[3], physical[4], physical[5]]),
            ),
            pick_radius: radius,
            same_root_vehicle: root == self.client_reach_root_vehicle(id),
            can_pick_inside: true,
        }];
        let direction = super::client_rotation_direction(yaw, pitch);
        let Some(hit) = super::client_reach_pick(
            candidates,
            value(eye),
            value(direction),
            bounds,
            f64::from(range),
        ) else {
            return false;
        };
        hit.distance_squared <= 0.
            || hit.distance_squared <= f64::from(range).powi(2)
                && super::client_middle_click::visible(
                    self,
                    value(eye),
                    value(hit.point),
                    camera_id,
                )
    }
    pub(super) fn client_projectile_puncher_animation(&self) -> SwingAnimation {
        let mut animation = SwingAnimation::DEFAULT;
        if let Some(stack) = self
            .inventory
            .get(self.client_hotbar_slot())
            .and_then(Option::as_ref)
        {
            let view = horizon_client::item_render::stack_view(stack);
            let components = self.pack.item_renderer.models.components_view(&view);
            if let Some(v) = components.get("minecraft:attack_animation") {
                animation.kind = match v["type"].as_str().unwrap_or("whack") {
                    "none" => 0,
                    "stab" => 2,
                    _ => 1,
                };
                animation.duration = v["duration"].as_i64().unwrap_or(6) as i32;
            }
        }
        animation
    }
    pub(super) fn client_projectile_puncher_swing(&mut self, mode: u8) {
        if mode == 1 {
            return;
        }
        if mode != 2 {
            let animation = self.client_projectile_puncher_animation();
            self.swing_hand(Hand::Main, animation);
        }
        if mode == 0 || mode == 2 {
            self.send(horizon_protocol::packets::play::Serverbound::Punch(
                horizon_protocol::packets::Empty,
            ));
        }
    }
    pub(super) fn client_projectile_puncher_attack(&mut self, id: u32, swing: u8) {
        if self.client_projectile_puncher_piercing() {
            return;
        }
        // AttackEntityEvent, AutoWeapon, VehicleOneHit and other listeners execute
        // before carried sync and the genuine ordinary attack packet.
        if !self.client_weapon_before_attack(id, false) {
            return;
        }
        if horizon_core::version::PROTOCOL_VERSION <= 47 {
            self.client_projectile_puncher_swing(swing);
        }
        self.native_client_ensure_carried();
        self.send(horizon_protocol::packets::play::Serverbound::Attack(
            horizon_protocol::packets::play::Attack { id: id as i32 },
        ));
        if !self.spectator() {
            if self.remote.is_none() {
                if let Some(i) = self.creatures.projectiles.iter().position(|p| p.id == id) {
                    let old = self.client_projectile_puncher_hide_implicit;
                    self.client_projectile_puncher_hide_implicit = true;
                    if matches!(
                        self.creatures.projectiles[i].kind,
                        ProjectileKind::ShulkerBullet { .. }
                    ) {
                        self.hit_shulker_bullet(None, i);
                    } else {
                        self.deflect(i);
                    }
                    self.client_projectile_puncher_hide_implicit = old;
                }
            } else {
                let at = self
                    .player
                    .as_ref()
                    .map(|p| [p.position.x, p.position.y, p.position.z]);
                self.audio
                    .play("minecraft:entity.player.attack.nodamage", at, 1., 1.);
            }
        }
        self.reset_attack_strength();
        if horizon_core::version::PROTOCOL_VERSION > 47 {
            self.client_projectile_puncher_swing(swing);
        }
    }
}
