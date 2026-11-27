// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent BlockSight's complete block visibility sampling. Clip is the host's
//! actual outline ray query, preserving entity context in the bound host.
use super::{block_targets::BlockHit, subtract, view_rotation::ViewRotation};
use crate::navigation::{
    aabb::{Aabb, Direction},
    geometry::{BlockPos, Vec3},
    voxel_shape::VoxelShape,
    walk::WalkState,
    walk_jump::wrap_degrees,
};

pub trait SightHost {
    type State: WalkState;
    fn state(&self, pos: BlockPos) -> Self::State;
    fn outline_shape(&self, state: &Self::State, pos: BlockPos) -> VoxelShape;
    fn clip(&self, from: Vec3, to: Vec3) -> BlockHit;
}
pub fn is_seen(
    host: &impl SightHost,
    eye: Vec3,
    point: Vec3,
    pos: BlockPos,
    face: Option<Direction>,
) -> bool {
    let direction = subtract(point, eye);
    let past = Vec3::new(
        eye.x + direction.x * (1.0 + 1.0e-3),
        eye.y + direction.y * (1.0 + 1.0e-3),
        eye.z + direction.z * (1.0 + 1.0e-3),
    );
    let hit = host.clip(eye, past);
    hit.is_block && hit.pos == pos && face.is_none_or(|face| face == hit.direction)
}
pub fn seen_point(
    host: &impl SightHost,
    eye: Vec3,
    pos: BlockPos,
    reach: f64,
    face: Option<Direction>,
    from: ViewRotation,
    step: f32,
) -> Option<Vec3> {
    let shape = host.outline_shape(&host.state(pos), pos);
    let fallback = [Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0))];
    let boxes = if shape.is_empty() {
        &fallback[..]
    } else {
        shape.aabbs()
    };
    let mut seen = Vec::new();
    for bounds in boxes {
        for i in 0..5 {
            for j in 0..5 {
                for k in 0..5 {
                    let lerp = |index, min, max| {
                        let share = (f64::from(index) + 0.5) / 5.0;
                        min + share * (max - min)
                    };
                    let point = Vec3::new(
                        f64::from(pos.x) + lerp(i, bounds.min.x, bounds.max.x),
                        f64::from(pos.y) + lerp(j, bounds.min.y, bounds.max.y),
                        f64::from(pos.z) + lerp(k, bounds.min.z, bounds.max.z),
                    );
                    if eye.distance_sqr(point) > reach * reach
                        || !is_seen(host, eye, point, pos, face)
                    {
                        continue;
                    }
                    let rotation = ViewRotation::from_vector(subtract(point, eye));
                    seen.push((
                        point,
                        wrap_degrees(rotation.y_rot - from.y_rot).abs()
                            + (rotation.x_rot - from.x_rot).abs(),
                    ));
                }
            }
        }
    }
    seen.sort_by(|(_, a), (_, b)| java_float_compare(*a, *b));
    seen.into_iter()
        .find(|(point, _)| {
            step <= 0.0 || is_seen_around(host, eye, *point, pos, face, step / 2.0_f32)
        })
        .map(|(point, _)| point)
}
fn is_seen_around(
    host: &impl SightHost,
    eye: Vec3,
    point: Vec3,
    pos: BlockPos,
    face: Option<Direction>,
    off: f32,
) -> bool {
    let rotation = ViewRotation::from_vector(subtract(point, eye));
    let length = eye.distance(point) + 1.0;
    [(off, 0.0), (-off, 0.0), (0.0, off), (0.0, -off)]
        .into_iter()
        .all(|(yaw, pitch)| {
            let look =
                ViewRotation::new(rotation.y_rot + yaw, rotation.x_rot + pitch).view_vector();
            is_seen(
                host,
                eye,
                Vec3::new(
                    eye.x + look.x * length,
                    eye.y + look.y * length,
                    eye.z + look.z * length,
                ),
                pos,
                face,
            )
        })
}
fn java_float_compare(a: f32, b: f32) -> std::cmp::Ordering {
    if a < b {
        return std::cmp::Ordering::Less;
    }
    if a > b {
        return std::cmp::Ordering::Greater;
    }
    let bits = |value: f32| {
        if value.is_nan() {
            0x7fc0_0000_i32
        } else {
            value.to_bits() as i32
        }
    };
    bits(a).cmp(&bits(b))
}
