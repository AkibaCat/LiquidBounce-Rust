// SPDX-License-Identifier: GPL-3.0-or-later
//! MovementUtils.findEdgeCollision with actual standing dimensions and shapes.
use super::Game;
use horizon_core::{
    geometry::{Aabb, Vec3},
    position::BlockPos,
};
use horizon_world::collision_context::CollisionContext;

fn fma(at: Vec3, scale: f64, direction: Vec3) -> Vec3 {
    Vec3::new(
        scale.mul_add(direction.x, at.x),
        scale.mul_add(direction.y, at.y),
        scale.mul_add(direction.z, at.z),
    )
}
pub(super) fn find(game: &Game<'_>, from: Vec3, to: Vec3, allowed_drop: f32) -> Option<Vec3> {
    let direction = to.subtract(from);
    if direction.length_squared() <= 1.0e-12 {
        return None;
    }
    let player = game.player.as_ref()?;
    let dimensions = horizon_client::pose::Pose::Standing.dimensions(player.scale);
    let union = dimensions.bounds(from).union_bounds(dimensions.bounds(to));
    let min = union.min_position();
    let max = union.max_position();
    let minimum = [
        (min.x - 0.3 - 1.0e-7).floor() as i32,
        (min.y - f64::from(allowed_drop) - 1.0e-7).floor() as i32,
        (min.z - 0.3 - 1.0e-7).floor() as i32,
    ];
    let maximum = [
        (max.x + 0.3 + 1.0e-7).floor() as i32,
        (min.y + 1.0e-7).floor() as i32,
        (max.z + 0.3 + 1.0e-7).floor() as i32,
    ];
    let extended_from = fma(from, -1000.0, direction);
    let extended_to = fma(to, 1000.0, direction);
    let mut scene = horizon_client::collisions::SceneCollisions {
        world: &game.world,
        binding: game.collision,
        subject: None,
    };
    let mut boxes = Vec::new();
    // BlockPos.betweenClosed enumerates x, then y, then z.
    for z in minimum[2]..=maximum[2] {
        for y in minimum[1]..=maximum[1] {
            for x in minimum[0]..=maximum[0] {
                let state = game.world.block(x, y, z);
                let shape = game
                    .collision
                    .collision_shape_with_context(
                        &mut scene,
                        state,
                        BlockPos::new(x, y, z),
                        &CollisionContext::empty(false),
                    )
                    .expect("Scaffold edge support shape");
                for bounds in shape.boxes() {
                    let min = bounds.min_position();
                    let max = bounds.max_position();
                    let adjusted = Aabb::new(
                        Vec3::new(min.x - 0.3, min.y - 1.0, min.z - 0.3),
                        Vec3::new(
                            max.x + 0.3,
                            max.y + f64::from(allowed_drop) + 0.05,
                            max.z + 0.3,
                        ),
                    )
                    .move_by(Vec3::new(
                        f64::from(x),
                        f64::from(y),
                        f64::from(z),
                    ));
                    if adjusted.clip(extended_from, extended_to).is_some() {
                        boxes.push(adjusted);
                    }
                }
            }
        }
    }
    let mut at = from;
    loop {
        let containing: Vec<_> = boxes.iter().copied().filter(|b| b.contains(at)).collect();
        if containing.is_empty() {
            return Some(at);
        }
        if containing.iter().any(|b| b.contains(to)) {
            return None;
        }
        let mut next = None;
        let mut distance = f64::INFINITY;
        for bounds in &containing {
            let point = bounds
                .clip(extended_to, extended_from)
                .expect("Scaffold reverse edge ray");
            let candidate = point.distance_squared(to);
            if next.is_none() || candidate < distance {
                next = Some(point);
                distance = candidate;
            }
        }
        at = next.expect("Scaffold containing support");
        boxes.retain(|b| !containing.contains(b));
    }
}
