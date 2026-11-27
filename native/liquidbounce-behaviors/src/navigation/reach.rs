// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent Terrain Reach.kt. Rays use the real world's outline/collider clip.
use super::{
    aabb::Aabb,
    geometry::{BlockPos, Vec3},
    terrain::{Terrain, TerrainLevel, TerrainState},
};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipBlock {
    Outline,
    Collider,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RayHit {
    Miss,
    Block(BlockPos),
}
pub trait ReachLevel<B: TerrainState>: TerrainLevel<B> {
    fn clip(&self, from: Vec3, to: Vec3, shapes: ClipBlock) -> RayHit;
}
#[derive(Clone, Copy, Debug)]
pub struct ReachOptions {
    pub reach: f64,
    pub from: Vec3,
    pub block: Option<BlockPos>,
    pub eye_height: f64,
    pub requires_support: bool,
}
impl ReachOptions {
    pub fn new(reach: f64, from: Vec3) -> Self {
        Self {
            reach,
            from,
            block: None,
            eye_height: 1.62_f32 as f64,
            requires_support: true,
        }
    }
}
pub fn reach_positions<B: TerrainState, L: ReachLevel<B>>(
    terrain: &mut Terrain<B, L>,
    body: Aabb,
    options: ReachOptions,
) -> Vec<BlockPos> {
    let center = BlockPos::containing(body.center());
    let radius = options.reach.ceil() as i32;
    let mut positions = Vec::new();
    for dx in radius.wrapping_neg()..=radius {
        for dz in radius.wrapping_neg()..=radius {
            for dy in radius.wrapping_neg().wrapping_sub(2)..=radius {
                let pos = center.offset(dx, dy, dz);
                let Some(feet) = feet_at(terrain, pos, options.requires_support) else {
                    continue;
                };
                let eyes = Vec3::new(feet.x, feet.y + options.eye_height, feet.z);
                if !terrain.body_box(feet).intersects(body)
                    && body.distance_sqr(eyes) <= options.reach * options.reach
                    && sees(terrain, eyes, body, options.reach, options.block)
                {
                    positions.push(pos);
                }
            }
        }
    }
    positions.sort_by(|left, right| {
        let left = feet_at(terrain, *left, options.requires_support)
            .map_or(f64::MAX, |feet| feet.distance_sqr(options.from));
        let right = feet_at(terrain, *right, options.requires_support)
            .map_or(f64::MAX, |feet| feet.distance_sqr(options.from));
        double_compare(left, right)
    });
    positions.truncate(8);
    positions
}
fn feet_at<B: TerrainState, L: TerrainLevel<B>>(
    terrain: &mut Terrain<B, L>,
    pos: BlockPos,
    requires_support: bool,
) -> Option<Vec3> {
    if let Some(feet) = terrain.feet(pos) {
        return Some(feet);
    }
    if requires_support {
        return None;
    }
    let feet = Vec3::bottom_center(pos);
    terrain.is_clear(terrain.body_box(feet)).then_some(feet)
}
pub fn sees<B: TerrainState, L: ReachLevel<B>>(
    terrain: &Terrain<B, L>,
    eyes: Vec3,
    body: Aabb,
    reach: f64,
    block: Option<BlockPos>,
) -> bool {
    if body.contains(eyes) {
        return true;
    }
    let shapes = if block.is_some() {
        ClipBlock::Outline
    } else {
        ClipBlock::Collider
    };
    for fx in [0.05, 0.275, 0.5, 0.725, 0.95] {
        for fy in [0.05, 0.275, 0.5, 0.725, 0.95] {
            for fz in [0.05, 0.275, 0.5, 0.725, 0.95] {
                let point = Vec3::new(
                    body.min.x + fx * (body.max.x - body.min.x),
                    body.min.y + fy * (body.max.y - body.min.y),
                    body.min.z + fz * (body.max.z - body.min.z),
                );
                if eyes.distance_sqr(point) > reach * reach {
                    continue;
                }
                let hit = terrain.level().clip(eyes, point, shapes);
                if match block {
                    Some(block) => hit == RayHit::Block(block),
                    None => hit == RayHit::Miss,
                } {
                    return true;
                }
            }
        }
    }
    false
}
fn double_compare(left: f64, right: f64) -> Ordering {
    if left < right {
        return Ordering::Less;
    }
    if left > right {
        return Ordering::Greater;
    }
    let bits = |value: f64| {
        if value.is_nan() {
            0x7ff8_0000_0000_0000i64
        } else {
            value.to_bits() as i64
        }
    };
    bits(left).cmp(&bits(right))
}
