// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Nodes.kt. Terrain is passed explicitly to avoid self-borrows.
use super::{WalkState, abilities::WalkAbilities, water::Water};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    terrain::{Terrain, TerrainLevel},
};

pub const DX: [i32; 4] = [1, -1, 0, 0];
pub const DZ: [i32; 4] = [0, 0, 1, -1];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Landing {
    pub y: i32,
    pub drop: f64,
    pub damage: i32,
}
pub struct Nodes<B: WalkState> {
    pub water: Water,
    abilities: WalkAbilities<B>,
    can_place: bool,
    max_damage: i32,
    max_drop: i32,
    half_width: f64,
}
impl<B: WalkState> Nodes<B> {
    pub fn new(width: f64, abilities: WalkAbilities<B>) -> Self {
        let max_damage = 0.max((f64::from(abilities.health - 10.0).ceil() as i32).wrapping_sub(1));
        Self {
            water: Water::new(abilities.can_sprint),
            can_place: abilities.can_place && abilities.placeable_blocks > 0,
            max_damage,
            max_drop: abilities
                .safe_fall
                .wrapping_add(max_damage)
                .max(abilities.water_fall)
                .wrapping_add(1),
            half_width: width / 2.0,
            abilities,
        }
    }
    pub fn is_node<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        self.stands(t, p) || self.swims(t, p) || self.climbs(t, p) || self.crouches(t, p)
    }
    pub fn is_underwater<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        self.swims(t, p) && self.water.is_submerged(t, p)
    }
    pub fn stands<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        !t.stand_height(p).is_nan()
    }
    pub fn swims<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        self.water.fits(t, p)
    }
    pub fn climbs<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        t.is_climbable(p) && t.has_room(p, t.height())
    }
    pub fn crouches<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        !self.stands(t, p) && !t.crouch_height(p).is_nan() && !t.is_water(p) && !t.is_climbable(p)
    }
    pub fn height<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> f64 {
        let stand = t.stand_height(p);
        if !stand.is_nan() {
            stand
        } else if self.swims(t, p) || self.climbs(t, p) {
            f64::from(p.y)
        } else {
            let crouch = t.crouch_height(p);
            if !crouch.is_nan() {
                crouch
            } else {
                let floor = self.floor_top(t, p);
                if floor.is_nan() {
                    f64::from(p.y)
                } else {
                    floor
                }
            }
        }
    }
    pub fn feet<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> Vec3 {
        let shape = t.shape(p);
        if shape.is_empty() || !self.climbs(t, p) {
            Vec3::new(
                f64::from(p.x) + 0.5,
                self.height(t, p),
                f64::from(p.z) + 0.5,
            )
        } else {
            let b = shape.bounds().expect("nonempty ladder shape");
            Vec3::new(
                f64::from(p.x) + room_middle(b.min.x, b.max.x),
                self.height(t, p),
                f64::from(p.z) + room_middle(b.min.z, b.max.z),
            )
        }
    }
    // The negated comparison intentionally retains Kotlin's NaN top sentinel.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn floor_top<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> f64 {
        let below = p.offset(0, -1, 0);
        if !self.is_harmless(t, below) {
            return f64::NAN;
        }
        let mut top = f64::NAN;
        for b in t.shape(below).aabbs() {
            let box_top = f64::from(p.y.wrapping_sub(1)) + b.max.y;
            let inside = b.min.x < 0.5 + self.half_width
                && b.max.x > 0.5 - self.half_width
                && b.min.z < 0.5 + self.half_width
                && b.max.z > 0.5 - self.half_width;
            if inside
                && box_top >= f64::from(p.y)
                && box_top < f64::from(p.y.wrapping_add(1))
                && !(box_top <= top)
            {
                top = box_top;
            }
        }
        top
    }
    pub fn floor_block<L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        x: i32,
        z: i32,
        height: f64,
    ) -> B {
        t.state(BlockPos::new(x, (height - 0.500001).floor() as i32, z))
    }
    pub fn landing<L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        x: i32,
        top: i32,
        z: i32,
        height: f64,
    ) -> Option<Landing> {
        for y in (top.wrapping_sub(self.max_drop).max(t.min_y())..=top).rev() {
            let p = BlockPos::new(x, y, z);
            let stand = t.stand_height(p);
            if self.swims(t, p) {
                let drop = height - f64::from(y);
                return (drop <= f64::from(self.abilities.water_fall)).then_some(Landing {
                    y,
                    drop,
                    damage: 0,
                });
            }
            if !stand.is_nan() {
                let drop = height - stand;
                let damage = 0.max((drop - f64::from(self.abilities.safe_fall)).ceil() as i32);
                return (damage <= self.max_damage).then_some(Landing { y, drop, damage });
            }
            if self.climbs(t, p) {
                return Some(Landing {
                    y,
                    drop: height - f64::from(y),
                    damage: 0,
                });
            }
            if !t.is_passable(p) {
                return None;
            }
        }
        None
    }
    pub fn is_near_drop<L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
        height: f64,
    ) -> bool {
        for (dx, dz) in DX.into_iter().zip(DZ) {
            let side = p.offset(dx, 0, dz);
            if !t.is_known(side) {
                return true;
            }
            let open = t.is_open(side) && self.floor_top(t, side).is_nan() && !t.is_water(side);
            if open
                && self
                    .landing(t, side.x, p.y.wrapping_sub(1), side.z, height)
                    .map_or(1, |l| l.damage)
                    > 0
            {
                return true;
            }
        }
        false
    }
    pub fn is_harmless<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        t.is_known(p) && !t.is_hazard(p)
    }
    pub fn is_solid<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        self.is_harmless(t, p) && !t.is_passable(p)
    }
    pub fn is_placeable<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        self.can_place
            && self.is_harmless(t, p)
            && t.state(p).can_be_replaced()
            && !self.abilities.reserved.contains(&p.packed())
    }
    pub fn is_dry<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        if t.state(p).has_liquid() || t.state(p.above()).has_liquid() {
            return false;
        }
        for (dx, dz) in DX.into_iter().zip(DZ) {
            if t.state(p.offset(dx, 0, dz)).has_liquid() {
                return false;
            }
        }
        true
    }
}
fn room_middle(min: f64, max: f64) -> f64 {
    if min > 0.0 && max >= 1.0 {
        min / 2.0
    } else if max < 1.0 && min <= 0.0 {
        (max + 1.0) / 2.0
    } else {
        0.5
    }
}
