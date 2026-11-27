// SPDX-License-Identifier: GPL-3.0-or-later
//! SpearKillTargetFinder. Ray 1 uses the camera; ray 2 the managed rotation.
use super::Game;
use horizon_core::geometry::{Aabb, Vec3};
pub(super) fn look(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::direction_from_rotation(pitch, yaw)
}
fn raw(b: Aabb) -> [f64; 6] {
    let a = b.min_position();
    let z = b.max_position();
    [a.x, a.y, a.z, z.x, z.y, z.z]
}
fn section(p: Vec3) -> [i32; 3] {
    [
        (p.x.floor() as i32) >> 4,
        (p.y.floor() as i32) >> 4,
        (p.z.floor() as i32) >> 4,
    ]
}
pub(super) struct Living {
    pub id: i32,
    pub position: Vec3,
    pub previous: Vec3,
    pub bounds: Aabb,
    pub order: ([i32; 3], u64),
}
fn at(id: i32, position: Vec3, previous: Vec3, bounds: Aabb, seq: u64) -> Living {
    Living {
        id,
        position,
        previous,
        bounds,
        order: (section(position), seq),
    }
}
pub(super) fn living(g: &Game<'_>) -> Vec<Living> {
    let local = g.remote.as_ref().map_or(0, |r| r.entity_id);
    let mut entities: Vec<_> = g
        .mobs
        .iter()
        .filter(|m| {
            m.id as i32 != local
                && m.alive()
                && !m.removed
                && g.dragon.as_ref().is_none_or(|d| d.id != m.id)
        })
        .map(|m| Living {
            id: m.id as i32,
            position: m.position,
            previous: m.previous,
            bounds: m.bounds(),
            order: (
                section(m.position),
                if m.extra.section_seq == 0 {
                    m.join.get()
                } else {
                    m.extra.section_seq
                },
            ),
        })
        .collect();
    entities.extend(g.armor_stands.iter().filter(|s| s.health > 0.0).map(|s| {
        at(
            s.id as i32,
            s.position,
            s.previous,
            s.bounds(),
            s.join.get(),
        )
    }));
    if let Some(d) = g.dragon.as_ref().filter(|d| d.alive()) {
        let p = d.position;
        entities.push(at(
            d.id as i32,
            p,
            d.previous,
            Aabb::new(
                Vec3::new(p.x - 8.0, p.y, p.z - 8.0),
                Vec3::new(p.x + 8.0, p.y + 8.0, p.z + 8.0),
            ),
            0,
        ));
    }
    for guest in g
        .lan
        .iter()
        .flat_map(|l| l.hub.guests.iter())
        .filter(|p| p.placed && p.dimension == g.dim.id() && p.entity_id != local)
    {
        if entities.iter().any(|p| p.id == guest.entity_id) {
            continue;
        }
        if let Some(body) = guest.motion.as_ref().filter(|b| !b.health.dead_or_dying()) {
            let p = Vec3::new(guest.position[0], guest.position[1], guest.position[2]);
            let old = Vec3::new(
                guest.previous_position[0],
                guest.previous_position[1],
                guest.previous_position[2],
            );
            entities.push(at(
                guest.entity_id,
                p,
                old,
                body.dimensions.bounds(p),
                u64::from(guest.conn),
            ));
        }
    }
    entities.sort_by_key(|e| {
        let [x, y, z] = e.order.0;
        (x, z, y, e.order.1)
    });
    entities
}
pub(super) fn boxes(g: &Game<'_>, id: i32) -> Vec<[f64; 6]> {
    if let Some(d) = g.dragon.as_ref().filter(|d| d.id as i32 == id) {
        return (0..8).map(|i| raw(d.part_box(i))).collect();
    }
    if let Some(m) = g.mobs.iter().find(|m| m.id as i32 == id) {
        return vec![raw(m.bounds())];
    }
    if let Some(s) = g.armor_stands.iter().find(|s| s.id as i32 == id) {
        return vec![raw(s.bounds())];
    }
    living(g)
        .into_iter()
        .find(|e| e.id == id)
        .map_or_else(Vec::new, |e| vec![raw(e.bounds)])
}
pub(super) fn find(g: &mut Game<'_>, maximum: f32) -> Option<(i32, f64)> {
    let player = g.player.as_ref()?;
    let p = player.position;
    let eye = Vec3::new(p.x, p.y + f64::from(player.dimensions.eye_height), p.z);
    let look = look(g.yaw, g.pitch);
    let delta = look.scale(f64::from(maximum));
    // Kotlin Vec3.fma uses Math.fma for each coordinate.
    let end = Vec3::new(
        f64::from(maximum).mul_add(look.x, eye.x),
        f64::from(maximum).mul_add(look.y, eye.y),
        f64::from(maximum).mul_add(look.z, eye.z),
    );
    let half = f64::from(player.dimensions.width) / 2.0;
    let query = player
        .bounds()
        .expand_towards(delta)
        .inflate(Vec3::new(half, half, half));
    let speed =
        player.velocity.x * look.x + player.velocity.y * look.y + player.velocity.z * look.z;
    let range = g
        .using
        .and_then(|(slot, _)| g.stack_in(slot))
        .and_then(|s| g.stack_attack_range(s));
    let (mut min, mut max, mut min_creative, mut max_creative) = if let Some(r) = range {
        let read = |k: &str, d: f32| r[k].as_f64().map_or(d, |v| v as f32);
        (
            read("min_reach", 0.0),
            read("max_reach", 3.0),
            read("min_creative_reach", 0.0),
            read("max_creative_reach", 5.0),
        )
    } else {
        let max = g.interaction_range("minecraft:entity_interaction_range", 3.0, 5.0) as f32;
        (0.0, max, 0.0, max)
    };
    g.native_client_attack_range(&mut min, &mut max, &mut min_creative, &mut max_creative);
    let (minimum, maximum) = if g.creative {
        (min_creative, max_creative)
    } else {
        (min, max)
    };
    let lunge = (f64::from(minimum) + (f64::from(maximum) + speed.max(0.0))) / 2.0;
    let [border_min_x, border_min_z, border_max_x, border_max_z] = g.border.geometry().bounds(0.0);
    let (mut yaw, mut pitch, mut managed) = (g.yaw, g.pitch, false);
    g.native_client_rotation_aim(&mut yaw, &mut pitch, &mut managed);
    let managed_look = self::look(yaw, pitch);
    let mut best = None;
    let mut best_sq = f64::MAX;
    for entity in living(g) {
        let x = entity.position.x.floor();
        let z = entity.position.z.floor();
        if !(x + 1.0 > border_min_x
            && x < border_max_x
            && z + 1.0 > border_min_z
            && z < border_max_z)
            || !query.intersects(entity.bounds)
        {
            continue;
        }
        let Some(hit) = entity.bounds.clip(eye, end) else {
            continue;
        };
        if super::spear_kill_ray::clip(g, eye, hit).is_none_or(|hit| !hit.0) {
            continue;
        }
        let distance = (eye.distance_squared(hit).sqrt() - lunge).max(0.0);
        if distance <= 0.0 {
            continue;
        }
        let sq = distance * distance;
        if sq >= best_sq {
            continue;
        }
        let Some((miss, at)) =
            super::spear_kill_ray::clip(g, eye, eye.add_vector(managed_look.scale(distance)))
        else {
            continue;
        };
        if !miss && at.distance_squared(eye) < sq {
            continue;
        }
        best = Some((entity.id, distance));
        best_sq = sq;
    }
    best
}
