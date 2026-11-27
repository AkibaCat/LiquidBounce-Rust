// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent JumpTargets.kt. Callbacks run immediately on the same Terrain.
use crate::navigation::{
    geometry::{BlockPos, java_max, java_min},
    simulation::motion_traits::MotionTraits,
    terrain::{Terrain, TerrainLevel, TerrainState},
};
use std::sync::OnceLock;
const DX: [i32; 8] = [1, -1, 0, 0, 1, 1, -1, -1];
const DZ: [i32; 8] = [0, 0, 1, -1, 1, -1, 1, -1];
#[derive(Clone, Debug)]
pub struct JumpTargets {
    traits: MotionTraits,
    extra_height: f64,
    max_rise: f64,
    reach_scale: f64,
}
#[derive(Clone, Copy)]
struct Offset {
    x: i32,
    z: i32,
    distance: f64,
}
impl JumpTargets {
    pub fn new(traits: MotionTraits) -> Self {
        let default = MotionTraits::default();
        let extra_height = traits.jump_height - default.jump_height;
        Self {
            traits,
            extra_height,
            max_rise: 1.25 + extra_height,
            reach_scale: java_max(
                1.0,
                f64::from(traits.walking_speed) / f64::from(default.walking_speed),
            ),
        }
    }
    pub fn lift(&self, upward: f64) -> f64 {
        MotionTraits::rise(java_max(upward, self.traits.jump_power)) - self.traits.jump_height
    }
    pub fn for_each<
        B: TerrainState,
        L: TerrainLevel<B>,
        F: FnMut(&mut Terrain<B, L>, i32, i32, i32, f64) -> bool,
    >(
        &mut self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        height: f64,
        max_drop: i32,
        sink: &mut F,
    ) {
        self.for_each_options(t, from, height, max_drop, max_drop, 0.0, false, sink);
    }
    #[allow(clippy::too_many_arguments)]
    pub fn for_each_options<
        B: TerrainState,
        L: TerrainLevel<B>,
        F: FnMut(&mut Terrain<B, L>, i32, i32, i32, f64) -> bool,
    >(
        &mut self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        height: f64,
        max_drop: i32,
        bounce_drop: i32,
        upward: f64,
        around: bool,
        sink: &mut F,
    ) {
        let BlockPos { x, y, z } = from;
        let lift = self.lift(upward);
        let bouncing = lift > 0.0;
        if !bouncing && !self.is_edge(t, x, y, z, height) {
            return;
        }
        let rises = self.max_rise + lift;
        let max_up = rises.ceil() as i32;
        let farthest = if bouncing { 14.0 } else { 8.0 };
        for offset in offsets() {
            if offset.distance > farthest {
                return;
            }
            let tx = x.wrapping_add(offset.x);
            let tz = z.wrapping_add(offset.z);
            for ty in (y.wrapping_sub(max_drop.max(bounce_drop))..=y.wrapping_add(max_up)).rev() {
                let pos = BlockPos::new(tx, ty, tz);
                if t.is_water(pos) {
                    continue;
                }
                let stand = t.stand_height(pos);
                let top = if stand.is_nan() {
                    self.catch_height(t, pos)
                } else {
                    stand
                };
                let rise = top - height;
                let in_range = !top.is_nan()
                    && rise <= rises
                    && (ty >= y.wrapping_sub(max_drop) || t.restitution(tx, top, tz) > 0.0);
                if !in_range
                    || !self.is_jumped_to(t, from, height, *offset, ty, rise, bouncing, around)
                {
                    continue;
                }
                if !sink(t, tx, ty, tz, top) {
                    return;
                }
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn is_jumped_to<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        height: f64,
        offset: Offset,
        ty: i32,
        rise: f64,
        bouncing: bool,
        around: bool,
    ) -> bool {
        if self.walk_reaches(t, from, offset, ty.wrapping_sub(from.y)) {
            false
        } else if bouncing {
            ty > from.y || self.crosses_gap(t, from, height, offset)
        } else {
            offset.distance <= self.max_reach(rise)
                && self.leaves_over_gap(t, from, height, offset, ty, around)
        }
    }
    fn catch_height<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
    ) -> f64 {
        if t.is_climbable(p) && t.has_room(p, t.height()) {
            f64::from(p.y)
        } else {
            f64::NAN
        }
    }
    fn is_edge<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        x: i32,
        y: i32,
        z: i32,
        height: f64,
    ) -> bool {
        for i in 0..DX.len() {
            if self.is_dip(
                t,
                BlockPos::new(x.wrapping_add(DX[i]), y, z.wrapping_add(DZ[i])),
                height,
            ) {
                return true;
            }
        }
        false
    }
    fn is_dip<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
        height: f64,
    ) -> bool {
        let stand = t.stand_height(p);
        t.is_open(p) && (stand.is_nan() || stand < height - 0.25)
    }
    fn is_gap<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
        height: f64,
    ) -> bool {
        self.is_dip(t, p, height) && t.stand_height(p.offset(0, -1, 0)).is_nan()
    }
    fn leaves_over_gap<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        height: f64,
        offset: Offset,
        ty: i32,
        around: bool,
    ) -> bool {
        let BlockPos { x, y, z } = from;
        let steps = offset.x.abs().max(offset.z.abs()) * 10;
        for step in 1..=steps {
            let along = (f64::from(step) - 0.5) / f64::from(steps);
            let cx = (f64::from(x) + 0.5 + f64::from(offset.x) * along).floor() as i32;
            let cz = (f64::from(z) + 0.5 + f64::from(offset.z) * along).floor() as i32;
            if cx != x || cz != z {
                let p = BlockPos::new(cx, y, cz);
                return cx == x.wrapping_add(offset.x) && cz == z.wrapping_add(offset.z)
                    || self.is_gap(t, p, height)
                    || ty >= y && self.is_dip(t, p, height)
                    || around
                        && self.is_jumped_around(t, p, x, z)
                        && !self.is_step(t, p, height)
                        && (self.crosses_gap(t, from, height, offset)
                            || self.has_gap_beside(t, p, height));
            }
        }
        false
    }
    fn is_jumped_around<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
        from_x: i32,
        from_z: i32,
    ) -> bool {
        if t.is_open(p) {
            return false;
        }
        let mut wall: Option<usize> = None;
        for i in 0..4 {
            let sx = p.x.wrapping_add(DX[i]);
            let sz = p.z.wrapping_add(DZ[i]);
            if (sx != from_x || sz != from_z) && !t.is_open(BlockPos::new(sx, p.y, sz)) {
                if wall.is_some() {
                    return false;
                }
                wall = Some(i);
            }
        }
        wall.is_none_or(|w| {
            p.x.wrapping_sub(from_x)
                .wrapping_abs()
                .wrapping_add(p.z.wrapping_sub(from_z).wrapping_abs())
                == 1
                && p.x
                    .wrapping_sub(from_x)
                    .wrapping_mul(DX[w])
                    .wrapping_add(p.z.wrapping_sub(from_z).wrapping_mul(DZ[w]))
                    == 0
        })
    }
    fn is_step<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
        height: f64,
    ) -> bool {
        for feet in p.y..=p.y.wrapping_add(1) {
            let top = t.stand_height(BlockPos::new(p.x, feet, p.z));
            if !top.is_nan() && top - height <= self.max_rise {
                return true;
            }
        }
        false
    }
    fn has_gap_beside<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
        height: f64,
    ) -> bool {
        for i in 0..4 {
            if self.is_gap(t, p.offset(DX[i], 0, DZ[i]), height) {
                return true;
            }
        }
        false
    }
    fn crosses_gap<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        height: f64,
        offset: Offset,
    ) -> bool {
        let BlockPos { x, y, z } = from;
        let steps = offset.x.abs().max(offset.z.abs()) * 10;
        for step in 1..=steps {
            let along = (f64::from(step) - 0.5) / f64::from(steps);
            let cx = (f64::from(x) + 0.5 + f64::from(offset.x) * along).floor() as i32;
            let cz = (f64::from(z) + 0.5 + f64::from(offset.z) * along).floor() as i32;
            let between = (cx != x || cz != z)
                && (cx != x.wrapping_add(offset.x) || cz != z.wrapping_add(offset.z));
            if between && self.is_gap(t, BlockPos::new(cx, y, cz), height) {
                return true;
            }
        }
        false
    }
    fn walk_reaches<B: TerrainState, L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        offset: Offset,
        dy: i32,
    ) -> bool {
        if offset.x.abs() > 1 || offset.z.abs() > 1 {
            false
        } else if offset.x == 0 || offset.z == 0 {
            dy <= 1
        } else {
            dy == 0
                && t.is_open(from.offset(offset.x, 0, 0))
                && t.is_open(from.offset(0, 0, offset.z))
        }
    }
    fn max_reach(&self, rise: f64) -> f64 {
        let climb = rise - self.extra_height;
        let reach = if climb > 0.0 {
            5.1
        } else if climb > -1.0 {
            5.8
        } else {
            5.8 - climb * 0.5
        };
        java_min(reach * self.reach_scale, 8.0)
    }
}
fn offsets() -> &'static [Offset] {
    static OFFSETS: OnceLock<Vec<Offset>> = OnceLock::new();
    OFFSETS.get_or_init(|| {
        let mut values = Vec::new();
        for x in -14..=14 {
            for z in -14..=14 {
                let distance = f64::from(x * x + z * z).sqrt();
                if (x != 0 || z != 0) && distance <= 14.0 {
                    values.push(Offset { x, z, distance });
                }
            }
        }
        values.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        values
    })
}
