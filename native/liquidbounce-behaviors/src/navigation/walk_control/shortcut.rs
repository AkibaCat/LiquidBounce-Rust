// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent StraightWalk.kt and walking Shortcut.kt, using the real Terrain cache.
use super::{SHORTCUT_STEPS, path_points::StepControl};
use crate::navigation::{
    geometry::{BlockPos, Vec3, java_min},
    terrain::{Terrain, TerrainLevel, TerrainState},
    walk::{WalkState, route::Route},
};

fn floor(value: f64) -> i32 {
    let integer = value as i32;
    if value < f64::from(integer) {
        integer.wrapping_sub(1)
    } else {
        integer
    }
}

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
    if is_near_drop(terrain, floor(from.x), level, floor(from.z), safe_fall) {
        return false;
    }
    for sample in 1..=samples {
        let t = java_min(f64::from(sample) * 0.5, length) / length;
        let x = from.x + dx * t;
        let z = from.z + dz * t;
        level = ground_level(terrain, x, z, level);
        if level.is_nan() || is_near_drop(terrain, floor(x), level, floor(z), safe_fall) {
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
    let y = floor(level);
    for (side_x, side_z) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let pos = BlockPos::new(x.wrapping_add(side_x), y, z.wrapping_add(side_z));
        if !terrain.is_known(pos) {
            return true;
        }
        let open = terrain.is_open(pos) && !terrain.is_water(pos);
        if open && !is_safe_drop(terrain, pos, level, safe_fall) {
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
    let bottom = pos.y.wrapping_sub(safe_fall).wrapping_sub(1);
    for y in (bottom..=pos.y).rev() {
        let drop_pos = BlockPos::new(pos.x, y, pos.z);
        let stand = terrain.stand_height(drop_pos);
        if terrain.is_water(drop_pos) {
            return true;
        }
        if !stand.is_nan() {
            return level - stand <= f64::from(safe_fall);
        }
        if !terrain.is_passable(drop_pos) {
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
    let block_x = floor(x);
    let block_z = floor(z);
    let step = Terrain::<B, L>::STEP_HEIGHT;
    for y in (floor(level - step)..=floor(level + step)).rev() {
        let height = terrain.stand_height(BlockPos::new(block_x, y, block_z));
        if !height.is_nan() && (height - level).abs() <= step {
            let body = terrain.body_box(Vec3::new(x, height, z));
            return if terrain.is_clear(body) {
                height
            } else {
                f64::NAN
            };
        }
    }
    f64::NAN
}

pub fn shortcut<B: WalkState, L: TerrainLevel<B>>(
    route: &Route<B>,
    index: usize,
    position: Vec3,
    terrain: &mut Terrain<B, L>,
) -> Option<Vec3> {
    shortcut_index(route, index, position, terrain).map(|i| route.steps()[i].feet)
}
pub fn shortcut_index<B: WalkState, L: TerrainLevel<B>>(
    route: &Route<B>,
    index: usize,
    position: Vec3,
    terrain: &mut Terrain<B, L>,
) -> Option<usize> {
    let mut end = index;
    while end + 1 < route.steps().len().min(index + SHORTCUT_STEPS)
        && route.steps()[end + 1].is_plain()
    {
        end += 1;
    }
    for i in (index + 1..=end).rev() {
        let point = route.steps()[i].feet;
        if is_straight_walkable(terrain, position, point, 0.25, route.abilities().safe_fall) {
            return Some(i);
        }
    }
    None
}
