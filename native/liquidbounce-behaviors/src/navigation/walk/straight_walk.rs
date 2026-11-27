// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent StraightWalk.kt, including sampled clearance and side drops.
use super::nodes::{DX, DZ};
use crate::navigation::{
    geometry::{BlockPos, Vec3, java_min},
    terrain::{Terrain, TerrainLevel, TerrainState},
};

pub fn is_straight_walkable<B: TerrainState, L: TerrainLevel<B>>(
    terrain: &mut Terrain<B, L>,
    from: Vec3,
    to: Vec3,
    stop_distance: f64,
    safe_fall: i32,
) -> bool {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    let length = (dx * dx + dz * dz).sqrt();
    let samples = (length / 0.5).ceil() as i32;
    let mut level = from.y;
    if is_near_drop(
        terrain,
        from.x.floor() as i32,
        level,
        from.z.floor() as i32,
        safe_fall,
    ) {
        return false;
    }
    for sample in 1..=samples {
        let t = java_min(f64::from(sample) * 0.5, length) / length;
        let x = from.x + dx * t;
        let z = from.z + dz * t;
        level = ground_level(terrain, x, z, level);
        if level.is_nan()
            || is_near_drop(
                terrain,
                x.floor() as i32,
                level,
                z.floor() as i32,
                safe_fall,
            )
        {
            return false;
        }
        if to.distance_sqr(Vec3::new(x, level, z)) <= stop_distance * stop_distance {
            return true;
        }
    }
    false
}
fn is_near_drop<B: TerrainState, L: TerrainLevel<B>>(
    terrain: &mut Terrain<B, L>,
    x: i32,
    level: f64,
    z: i32,
    safe_fall: i32,
) -> bool {
    let y = level.floor() as i32;
    for (dx, dz) in DX.into_iter().zip(DZ) {
        let side = BlockPos::new(x.wrapping_add(dx), y, z.wrapping_add(dz));
        if !terrain.is_known(side) {
            return true;
        }
        let open = terrain.is_open(side) && !terrain.is_water(side);
        if open && !is_safe_drop(terrain, side, level, safe_fall) {
            return true;
        }
    }
    false
}
fn is_safe_drop<B: TerrainState, L: TerrainLevel<B>>(
    terrain: &mut Terrain<B, L>,
    pos: BlockPos,
    level: f64,
    safe_fall: i32,
) -> bool {
    for y in (pos.y.wrapping_sub(safe_fall).wrapping_sub(1)..=pos.y).rev() {
        let q = BlockPos::new(pos.x, y, pos.z);
        let stand = terrain.stand_height(q);
        if terrain.is_water(q) {
            return true;
        }
        if !stand.is_nan() {
            return level - stand <= f64::from(safe_fall);
        }
        if !terrain.is_passable(q) {
            return false;
        }
    }
    false
}
fn ground_level<B: TerrainState, L: TerrainLevel<B>>(
    terrain: &mut Terrain<B, L>,
    x: f64,
    z: f64,
    level: f64,
) -> f64 {
    let block_x = x.floor() as i32;
    let block_z = z.floor() as i32;
    let step = Terrain::<B, L>::STEP_HEIGHT;
    for y in (((level - step).floor() as i32)..=((level + step).floor() as i32)).rev() {
        let height = terrain.stand_height(BlockPos::new(block_x, y, block_z));
        if !height.is_nan() && (height - level).abs() <= step {
            return if terrain.is_clear(terrain.body_box(Vec3::new(x, height, z))) {
                height
            } else {
                f64::NAN
            };
        }
    }
    f64::NAN
}
