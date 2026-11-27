// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac Terrain. Registry facts and shape queries are supplied
//! by the real game adapter; caching, opened shapes and movement tests live here.
use super::{
    aabb::{Aabb, Axis, Direction},
    geometry::{BlockPos, Vec3},
    voxel_shape::VoxelShape,
};
use std::collections::HashMap;
#[path = "terrain_cache.rs"]
mod cache;
#[path = "terrain_motion.rs"]
mod motion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Door,
    OpensByHand,
    FenceGate,
    Trapdoor,
    IronTrapdoor,
    Ladder,
}
pub trait TerrainState: Clone {
    fn same_instance(&self, other: &Self) -> bool;
    fn same_block(&self, other: &Self) -> bool;
    fn same_fluid(&self, other: &Self) -> bool;
    fn is_hazard(&self) -> bool;
    fn is_water(&self) -> bool;
    fn has_liquid(&self) -> bool;
    fn is_climbable(&self) -> bool;
    fn kind(&self, kind: BlockKind) -> bool;
    fn open(&self) -> Option<bool>;
    fn with_open(&self, open: bool) -> Self;
    fn is_upper_half(&self) -> bool;
    fn horizontal_facing(&self) -> Option<Direction>;
    fn bounce_restitution(&self) -> f32;
    fn collision_shape<L: TerrainLevel<Self>>(&self, level: &L, pos: BlockPos) -> VoxelShape;
}
pub trait TerrainLevel<B: TerrainState> {
    fn block_state(&self, pos: BlockPos) -> B;
    fn void_air(&self) -> B;
    fn is_known(&self, pos: BlockPos) -> bool;
    fn is_inside_border(&self, x: f64, z: f64) -> bool;
    fn min_y(&self) -> i32;
    fn height(&self) -> i32;
    /// Native levels may evaluate collision with their actual entity/world
    /// context. Existing common hosts retain their state-owned shape path.
    fn collision_shape_for(&self, _state: &B, _pos: BlockPos) -> Option<VoxelShape> { None }
    fn max_y(&self) -> i32 {
        self.min_y().wrapping_add(self.height()).wrapping_sub(1)
    }
    fn axis_step_order(&self, movement: Vec3) -> [Axis; 3] {
        if movement.x.abs() < movement.z.abs() {
            [Axis::Y, Axis::Z, Axis::X]
        } else {
            [Axis::Y, Axis::X, Axis::Z]
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestinationCheck {
    Valid,
    OutsideWorld,
    OutsideBorder,
    NotLoaded,
    NotStandable,
}
pub struct Terrain<B: TerrainState, L: TerrainLevel<B>> {
    level: L,
    width: f64,
    height: f64,
    can_open_doors: bool,
    half_width: f64,
    states: HashMap<i64, B>,
    shapes: HashMap<i64, VoxelShape>,
    stands: HashMap<i64, f64>,
    crouch_stands: HashMap<i64, f64>,
    closed: HashMap<i64, VoxelShape>,
}
impl<B: TerrainState, L: TerrainLevel<B>> Terrain<B, L> {
    pub const PLAYER_WIDTH: f64 = 0.6_f32 as f64;
    pub const PLAYER_HEIGHT: f64 = 1.8_f32 as f64;
    pub const CROUCH_HEIGHT: f64 = 1.5;
    pub const STEP_HEIGHT: f64 = 0.6;
    pub fn new(level: L, width: f64, height: f64, can_open_doors: bool) -> Self {
        Self {
            level,
            width,
            height,
            can_open_doors,
            half_width: width / 2.0,
            states: HashMap::new(),
            shapes: HashMap::new(),
            stands: HashMap::new(),
            crouch_stands: HashMap::new(),
            closed: HashMap::new(),
        }
    }
    pub fn player(level: L) -> Self {
        Self::new(level, Self::PLAYER_WIDTH, Self::PLAYER_HEIGHT, false)
    }
    pub fn level(&self) -> &L {
        &self.level
    }
    pub fn width(&self) -> f64 {
        self.width
    }
    pub fn height(&self) -> f64 {
        self.height
    }
    pub fn can_open_doors(&self) -> bool {
        self.can_open_doors
    }
    pub fn min_y(&self) -> i32 {
        self.level.min_y()
    }
    pub fn max_y(&self) -> i32 {
        self.level.max_y()
    }
    pub fn body_box(&self, feet: Vec3) -> Aabb {
        self.body_box_height(feet, self.height)
    }
    pub fn body_box_height(&self, feet: Vec3, height: f64) -> Aabb {
        Aabb::new(
            Vec3::new(feet.x - self.half_width, feet.y, feet.z - self.half_width),
            Vec3::new(
                feet.x + self.half_width,
                feet.y + height,
                feet.z + self.half_width,
            ),
        )
    }
    pub fn is_known(&self, pos: BlockPos) -> bool {
        self.level.is_known(pos)
            && self
                .level
                .is_inside_border(f64::from(pos.x) + 0.5, f64::from(pos.z) + 0.5)
    }
    pub fn state(&mut self, pos: BlockPos) -> B {
        if let Some(state) = self.states.get(&pos.packed()) {
            return state.clone();
        }
        let state = if self.is_known(pos) {
            self.level.block_state(pos)
        } else {
            self.level.void_air()
        };
        self.states.insert(pos.packed(), state.clone());
        state
    }
    pub fn shape(&mut self, pos: BlockPos) -> VoxelShape {
        let key = pos.packed();
        if let Some(shape) = self.shapes.get(&key) {
            return shape.clone();
        }
        let state = self.state(pos);
        let shape = if !self.is_known(pos) || state.is_hazard() {
            VoxelShape::block()
        } else if self.counts_open(&state, pos) {
            self.closed
                .insert(key, state.collision_shape(&self.level, pos));
            state.with_open(true).collision_shape(&self.level, pos)
        } else {
            state.collision_shape(&self.level, pos)
        };
        self.shapes.insert(key, shape.clone());
        shape
    }
    pub fn closed_in(&mut self, body: Aabb, from: Aabb, opens: &mut Vec<BlockPos>) {
        if !self.can_open_doors {
            return;
        }
        visit_blocks(body, |pos| {
            self.shape(pos);
            if let Some(closed) = self.closed.get(&pos.packed())
                && intersects(closed, pos, body)
                && !intersects(closed, pos, from)
            {
                let state = self.state(pos);
                let target = if state.is_upper_half() {
                    pos.offset(0, -1, 0)
                } else {
                    pos
                };
                if !opens.contains(&target) {
                    opens.push(target);
                }
            }
            true
        });
    }
    fn counts_open(&mut self, state: &B, pos: BlockPos) -> bool {
        if !self.can_open_doors || state.open() != Some(false) {
            return false;
        }
        if state.kind(BlockKind::Door) {
            state.kind(BlockKind::OpensByHand)
        } else if state.kind(BlockKind::FenceGate) {
            true
        } else if state.kind(BlockKind::Trapdoor) {
            !state.kind(BlockKind::IronTrapdoor) && !self.is_floor(pos)
        } else {
            false
        }
    }
    fn is_floor(&mut self, pos: BlockPos) -> bool {
        !self.state(pos.offset(0, -1, 0)).is_climbable()
            && self.is_empty(pos.offset(0, 1, 0))
            && self.is_empty(pos.offset(0, 2, 0))
    }
    fn is_empty(&mut self, pos: BlockPos) -> bool {
        self.state(pos).collision_shape(&self.level, pos).is_empty()
    }
    pub fn is_passable(&mut self, pos: BlockPos) -> bool {
        self.shape(pos).is_empty()
    }
    pub fn is_hazard(&mut self, pos: BlockPos) -> bool {
        self.is_known(pos) && self.state(pos).is_hazard()
    }
    pub fn is_water(&mut self, pos: BlockPos) -> bool {
        self.state(pos).is_water()
    }
    pub fn is_climbable(&mut self, pos: BlockPos) -> bool {
        self.state(pos).is_climbable()
    }
    pub fn climb_side(&mut self, pos: BlockPos) -> Option<Direction> {
        let state = self.state(pos);
        if state.kind(BlockKind::Ladder) {
            return Some(opposite(
                state
                    .horizontal_facing()
                    .expect("ladder has horizontal facing"),
            ));
        }
        for side in [
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            let offset = step(side);
            let at = pos.offset(offset.0, 0, offset.2);
            if self.is_grip(at) || self.is_grip(at.above()) {
                return Some(side);
            }
        }
        None
    }
    fn is_grip(&mut self, pos: BlockPos) -> bool {
        self.is_known(pos) && !self.is_hazard(pos) && !self.is_passable(pos)
    }
    pub fn restitution(&mut self, x: i32, top: f64, z: i32) -> f64 {
        f64::from(
            self.state(BlockPos::new(
                x,
                (top - f64::from(0.2_f32)).floor() as i32,
                z,
            ))
            .bounce_restitution(),
        )
    }
}
fn step(direction: Direction) -> (i32, i32, i32) {
    match direction {
        Direction::Down => (0, -1, 0),
        Direction::Up => (0, 1, 0),
        Direction::North => (0, 0, -1),
        Direction::South => (0, 0, 1),
        Direction::West => (-1, 0, 0),
        Direction::East => (1, 0, 0),
    }
}
fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::Down => Direction::Up,
        Direction::Up => Direction::Down,
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::West => Direction::East,
        Direction::East => Direction::West,
    }
}
fn visit_blocks(body: Aabb, mut action: impl FnMut(BlockPos) -> bool) -> bool {
    let min = BlockPos::new(
        (body.min.x + 1.0e-7).floor() as i32,
        ((body.min.y + 1.0e-7).floor() as i32).wrapping_sub(1),
        (body.min.z + 1.0e-7).floor() as i32,
    );
    let max = BlockPos::new(
        (body.max.x - 1.0e-7).floor() as i32,
        (body.max.y - 1.0e-7).floor() as i32,
        (body.max.z - 1.0e-7).floor() as i32,
    );
    for x in min.x..=max.x {
        for z in min.z..=max.z {
            for y in min.y..=max.y {
                if !action(BlockPos::new(x, y, z)) {
                    return false;
                }
            }
        }
    }
    true
}
fn intersects(shape: &VoxelShape, pos: BlockPos, body: Aabb) -> bool {
    if shape.same_instance(&VoxelShape::block()) {
        return body.intersects(Aabb::from_block(pos));
    }
    shape
        .aabbs()
        .iter()
        .any(|b| body.intersects(b.moved_block(pos)))
}
