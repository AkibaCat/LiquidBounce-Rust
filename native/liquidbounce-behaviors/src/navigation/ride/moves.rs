// SPDX-License-Identifier: GPL-3.0-or-later
use super::super::{
    aabb::Aabb,
    astar::WeightedEdge,
    geometry::{BlockPos, Vec3, java_max},
    terrain::{Terrain, TerrainLevel, TerrainState},
    walk::costs::Costs,
};
use super::vehicle::{JUMP_CHARGE_TICKS, Vehicle, VehicleKind};
use std::collections::HashMap;

pub trait RideState: TerrainState {
    fn friction(&self) -> f32;
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub position: Vec3,
    pub speed: f64,
}
pub struct Moves<B: RideState, L: TerrainLevel<B>> {
    pub terrain: Terrain<B, L>,
    pub vehicle: Vehicle,
    half_width: f64,
    max_rise: f64,
    spots: HashMap<i64, Option<Spot>>,
}
impl<B: RideState, L: TerrainLevel<B>> Moves<B, L> {
    pub fn new(terrain: Terrain<B, L>, vehicle: Vehicle) -> Self {
        Self {
            terrain,
            vehicle,
            half_width: vehicle.width / 2.0,
            max_rise: java_max(vehicle.step_height, vehicle.jump_height),
            spots: HashMap::new(),
        }
    }
    pub fn spot(&mut self, node: BlockPos) -> Option<Spot> {
        let key = node.packed();
        if let Some(cached) = self.spots.get(&key) {
            return *cached;
        }
        let spot = self.compute_spot(node);
        self.spots.insert(key, spot);
        spot
    }
    pub fn is_jump(&self, from: Spot, to: Spot) -> bool {
        to.position.y - from.position.y > self.vehicle.step_height + 1.0e-4
    }
    pub fn neighbors(&mut self, node: BlockPos) -> Vec<WeightedEdge<BlockPos>> {
        let Some(from) = self.spot(node) else {
            return Vec::new();
        };
        let mut edges = Vec::new();
        for (dx, dz) in [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
            let x = node.x.wrapping_add(dx);
            let z = node.z.wrapping_add(dz);
            self.add_rises(&mut edges, from, x, node.y, z);
            self.add_level_or_drop(&mut edges, from, x, node.y, z);
        }
        edges
    }
    fn add_rises(
        &mut self,
        edges: &mut Vec<WeightedEdge<BlockPos>>,
        from: Spot,
        x: i32,
        y: i32,
        z: i32,
    ) {
        for dy in (1..=self.max_rise.ceil() as i32).rev() {
            let node = BlockPos::new(x, y.wrapping_add(dy), z);
            let Some(to) = self.spot(node) else { continue };
            let rise = to.position.y - from.position.y;
            if rise > 0.0
                && rise <= self.max_rise + 1.0e-4
                && self.passes(from.position, to.position)
            {
                edges.push(WeightedEdge {
                    node,
                    cost: self.cost(from, to),
                });
            }
        }
    }
    fn add_level_or_drop(
        &mut self,
        edges: &mut Vec<WeightedEdge<BlockPos>>,
        from: Spot,
        x: i32,
        y: i32,
        z: i32,
    ) {
        let end = self.vehicle.safe_fall.wrapping_add(1).wrapping_neg();
        for dy in (end..=0).rev() {
            let node = BlockPos::new(x, y.wrapping_add(dy), z);
            let Some(to) = self.spot(node) else {
                if self.terrain.is_passable(node) {
                    continue;
                } else {
                    return;
                }
            };
            let rise = to.position.y - from.position.y;
            if rise <= self.vehicle.step_height + 1.0e-4
                && -rise <= f64::from(self.vehicle.safe_fall) + 1.0e-4
                && self.passes(from.position, to.position)
            {
                edges.push(WeightedEdge {
                    node,
                    cost: self.cost(from, to),
                });
            }
            return;
        }
    }
    pub fn is_straight(&mut self, from: Vec3, to: Vec3) -> bool {
        let level = from.y;
        if (to.y - level).abs() > 0.6 {
            return false;
        }
        let dx = to.x - from.x;
        let dz = to.z - from.z;
        let samples = 1.max(((dx * dx + dz * dz).sqrt() / 0.5).ceil() as i32);
        let node_y = (level + 1.0e-4).floor() as i32;
        for sample in 1..=samples {
            let t = f64::from(sample) / f64::from(samples);
            let x = from.x + dx * t;
            let z = from.z + dz * t;
            let Some(spot) = self.spot(BlockPos::new(x.floor() as i32, node_y, z.floor() as i32))
            else {
                return false;
            };
            if (spot.position.y - level).abs() > 0.6
                || !self.terrain.is_clear(self.body_box(x, level, z))
            {
                return false;
            }
        }
        true
    }
    fn compute_spot(&mut self, node: BlockPos) -> Option<Spot> {
        if !self.terrain.is_known(node) {
            return None;
        }
        if self.vehicle.kind == VehicleKind::Mount {
            let rest = self.terrain.stand_height(node);
            self.resting_at(node.x, rest, node.z, self.vehicle.speed)
        } else if self.is_water_surface(node) {
            self.resting_at(node.x, f64::from(node.y) + 0.5, node.z, self.vehicle.speed)
        } else if self.is_ice(node) {
            self.resting_at(node.x, f64::from(node.y), node.z, self.vehicle.ice_speed)
        } else {
            None
        }
    }
    fn resting_at(&mut self, x: i32, rest: f64, z: i32, speed: f64) -> Option<Spot> {
        if rest.is_nan() {
            return None;
        }
        for (ax, az) in [
            (0.5, 0.5),
            (0.0, 0.5),
            (1.0, 0.5),
            (0.5, 0.0),
            (0.5, 1.0),
            (0.0, 0.0),
            (0.0, 1.0),
            (1.0, 0.0),
            (1.0, 1.0),
        ] {
            let position = Vec3::new(f64::from(x) + ax, rest, f64::from(z) + az);
            if self
                .terrain
                .is_clear(self.body_box(position.x, rest, position.z))
            {
                return Some(Spot { position, speed });
            }
        }
        None
    }
    fn is_water_surface(&mut self, node: BlockPos) -> bool {
        self.terrain.is_water(node) && !self.terrain.is_water(node.above())
    }
    fn is_ice(&mut self, node: BlockPos) -> bool {
        !self.terrain.is_water(node)
            && self.terrain.is_passable(node)
            && self.terrain.state(node.offset(0, -1, 0)).friction() >= 0.9_f32
            && !self.terrain.is_passable(node.offset(0, -1, 0))
    }
    pub fn passes(&mut self, from: Vec3, to: Vec3) -> bool {
        let level = java_max(from.y, to.y);
        if self.is_clear_line(from.x, from.z, to.x, to.z, level) {
            return true;
        }
        if from.x == to.x || from.z == to.z {
            return false;
        }
        self.is_clear_line(from.x, from.z, from.x, to.z, level)
            && self.is_clear_line(from.x, to.z, to.x, to.z, level)
            || self.is_clear_line(from.x, from.z, to.x, from.z, level)
                && self.is_clear_line(to.x, from.z, to.x, to.z, level)
    }
    fn is_clear_line(
        &mut self,
        from_x: f64,
        from_z: f64,
        to_x: f64,
        to_z: f64,
        level: f64,
    ) -> bool {
        let dx = to_x - from_x;
        let dz = to_z - from_z;
        let samples = 1.max(((dx * dx + dz * dz).sqrt() / 0.5).ceil() as i32);
        for sample in 0..=samples {
            let t = f64::from(sample) / f64::from(samples);
            if !self
                .terrain
                .is_clear(self.body_box(from_x + dx * t, level, from_z + dz * t))
            {
                return false;
            }
        }
        true
    }
    pub fn cost(&self, from: Spot, to: Spot) -> f64 {
        let dx = from.position.x - to.position.x;
        let dz = from.position.z - to.position.z;
        let half = (dx * dx + dz * dz).sqrt() / 2.0;
        let mut ticks = half / from.speed + half / to.speed;
        if self.is_jump(from, to) {
            ticks +=
                f64::from(JUMP_CHARGE_TICKS) + Costs::fall(self.vehicle.jump_height.ceil() as i32)
        }
        let drop = from.position.y - to.position.y;
        if drop > self.vehicle.step_height {
            ticks += Costs::fall(drop.ceil() as i32)
        }
        ticks
    }
    fn body_box(&self, x: f64, y: f64, z: f64) -> Aabb {
        Aabb::new(
            Vec3::new(x - self.half_width, y + 0.01, z - self.half_width),
            Vec3::new(
                x + self.half_width,
                y + self.vehicle.height,
                z + self.half_width,
            ),
        )
    }
}
