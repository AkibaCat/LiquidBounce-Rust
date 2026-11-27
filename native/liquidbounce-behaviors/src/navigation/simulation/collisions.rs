// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent Collisions.kt: direct movement, complete step-up and canFall.
use super::super::{
    aabb::Aabb,
    geometry::{BlockPos, Vec3},
};
use super::{block_colliders::BlockColliders, block_grid::ShapeGrid};
const HALF_WIDTH: f64 = 0.6_f32 as f64 / 2.0;
const HEIGHT: f64 = 1.8_f32 as f64;
const STEP_HEIGHT: f32 = 0.6;

pub fn collide(
    colliders: &mut BlockColliders,
    grid: &mut impl ShapeGrid,
    position: Vec3,
    on_ground: bool,
    delta: Vec3,
) -> Vec3 {
    let body = Aabb::new(
        Vec3::new(position.x - HALF_WIDTH, position.y, position.z - HALF_WIDTH),
        Vec3::new(
            position.x + HALF_WIDTH,
            position.y + HEIGHT,
            position.z + HALF_WIDTH,
        ),
    );
    let mut direct = delta;
    if delta.x * delta.x + delta.y * delta.y + delta.z * delta.z != 0.0 {
        colliders.collect(grid, body.expand_towards(delta));
        colliders.collide(delta, body);
        direct = Vec3::new(colliders.moved_x, colliders.moved_y, colliders.moved_z);
    }
    let lands = delta.y != direct.y && delta.y < 0.0;
    if !(lands || on_ground) || (delta.x == direct.x && delta.z == direct.z) {
        return direct;
    }
    let grounded = if lands {
        body.moved(Vec3::new(0.0, direct.y, 0.0))
    } else {
        body
    };
    let mut region = grounded.expand_towards(Vec3::new(delta.x, f64::from(STEP_HEIGHT), delta.z));
    if !lands {
        region = region.expand_towards(Vec3::new(0.0, -1.0e-5, 0.0));
    }
    colliders.collect(grid, region);
    for height in colliders.step_heights(grounded, STEP_HEIGHT, direct.y as f32) {
        colliders.collide(Vec3::new(delta.x, f64::from(height), delta.z), grounded);
        let stepped_x = colliders.moved_x;
        let stepped_z = colliders.moved_z;
        if stepped_x * stepped_x + stepped_z * stepped_z > direct.x * direct.x + direct.z * direct.z
        {
            return Vec3::new(
                stepped_x,
                colliders.moved_y - (body.min.y - grounded.min.y),
                stepped_z,
            );
        }
    }
    direct
}
pub fn can_fall(
    colliders: &mut BlockColliders,
    grid: &mut impl ShapeGrid,
    position: Vec3,
    dx: f64,
    dz: f64,
    depth: f64,
) -> bool {
    let inset = 1.0e-7;
    let min_x = position.x - HALF_WIDTH + inset + dx;
    let min_z = position.z - HALF_WIDTH + inset + dz;
    let width = 2.0 * (HALF_WIDTH - inset);
    let probe = Aabb::new(
        Vec3::new(min_x, position.y - depth - inset, min_z),
        Vec3::new(min_x + width, position.y, min_z + width),
    );
    for bx in (probe.min.x.floor() as i32)..=(probe.max.x.floor() as i32) {
        for bz in (probe.min.z.floor() as i32)..=(probe.max.z.floor() as i32) {
            for by in (probe.min.y.floor() as i32).wrapping_sub(1)..=(probe.max.y.floor() as i32) {
                if colliders.intersects(grid, BlockPos::new(bx, by, bz), probe) {
                    return false;
                }
            }
        }
    }
    true
}
