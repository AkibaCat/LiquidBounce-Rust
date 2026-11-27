// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete common BlockTargets selection and placement geometry. The host binds
//! real sight, outline, inventory and item facts; no approximate raycast is used.
use super::{
    block_sight::{self, SightHost},
    subtract,
    view_rotation::ViewRotation,
};
use crate::navigation::{
    aabb::{Axis, Direction},
    geometry::{BlockPos, Vec3},
    walk::WalkState,
    walk_jump::wrap_degrees,
};

pub const DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];
pub fn unit(face: Direction) -> Vec3 {
    match face {
        Direction::Down => Vec3::new(0.0, -1.0, 0.0),
        Direction::Up => Vec3::new(0.0, 1.0, 0.0),
        Direction::North => Vec3::new(0.0, 0.0, -1.0),
        Direction::South => Vec3::new(0.0, 0.0, 1.0),
        Direction::West => Vec3::new(-1.0, 0.0, 0.0),
        Direction::East => Vec3::new(1.0, 0.0, 0.0),
    }
}
pub fn axis(face: Direction) -> Axis {
    match face {
        Direction::Down | Direction::Up => Axis::Y,
        Direction::North | Direction::South => Axis::Z,
        Direction::West | Direction::East => Axis::X,
    }
}
pub fn opposite(face: Direction) -> Direction {
    DIRECTIONS[ordinal(face) ^ 1]
}
pub fn ordinal(face: Direction) -> usize {
    match face {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}
pub fn relative(pos: BlockPos, face: Direction) -> BlockPos {
    let u = unit(face);
    pos.offset(u.x as i32, u.y as i32, u.z as i32)
}
fn center(pos: BlockPos) -> Vec3 {
    Vec3::new(
        f64::from(pos.x) + 0.5,
        f64::from(pos.y) + 0.5,
        f64::from(pos.z) + 0.5,
    )
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}
fn scale(a: Vec3, b: f64) -> Vec3 {
    Vec3::new(a.x * b, a.y * b, a.z * b)
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    Main,
    Off,
}
#[derive(Clone, Copy, Debug)]
pub struct BlockHit {
    pub is_block: bool,
    pub location: Vec3,
    pub direction: Direction,
    pub pos: BlockPos,
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub pos: BlockPos,
    pub against: BlockPos,
    pub face: Direction,
    pub point: Vec3,
    pub rotation: ViewRotation,
    pub slot: i32,
}
impl Plan {
    pub fn hand(&self) -> Hand {
        if self.slot < 0 { Hand::Off } else { Hand::Main }
    }
}

pub trait TargetInventory {
    type Stack: Clone;
    fn item(&self, slot: i32) -> Self::Stack;
    fn selected_slot(&self) -> i32;
}
pub trait PlaceableBlocks<T> {
    fn is_placeable(&self, stack: &T) -> bool;
    fn is_unfavourable(&self, stack: &T) -> bool;
}
pub trait TargetHost: SightHost + Sized {
    type Inventory: TargetInventory;
    type Policy: PlaceableBlocks<<Self::Inventory as TargetInventory>::Stack>;
    fn outline_empty(&self, state: &Self::State, pos: BlockPos) -> bool {
        self.outline_shape(state, pos).is_empty()
    }
    fn eye_position(&self) -> Vec3;
    fn reach(&self) -> f64;
    fn block_position(&self) -> BlockPos;
    fn uuid_most_significant_bits(&self) -> i64;
    fn sent_rotation(&self) -> ViewRotation;
    fn view_step(&self) -> f32;
    fn seen_point(
        &self,
        eye: Vec3,
        pos: BlockPos,
        reach: f64,
        from: ViewRotation,
        step: f32,
    ) -> Option<Vec3> {
        block_sight::seen_point(self, eye, pos, reach, None, from, step)
    }
    fn pick(&self, rotation: ViewRotation, reach: f64, eye: Option<Vec3>) -> BlockHit;
    fn is_seen(&self, eye: Vec3, point: Vec3, pos: BlockPos, face: Direction) -> bool {
        block_sight::is_seen(self, eye, point, pos, Some(face))
    }
    fn inventory(&self) -> &Self::Inventory;
    fn placeable_blocks(&self) -> &Self::Policy;
    fn is_tool_for(
        &self,
        stack: &<Self::Inventory as TargetInventory>::Stack,
        state: &Self::State,
    ) -> bool;
    fn destroy_speed(
        &self,
        stack: &<Self::Inventory as TargetInventory>::Stack,
        state: &Self::State,
    ) -> f32;
    fn efficiency_level(&self, stack: &<Self::Inventory as TargetInventory>::Stack) -> i32;
    fn is_wear_free(&self, stack: &<Self::Inventory as TargetInventory>::Stack) -> bool;
    fn hotbar(&self) -> Vec<<Self::Inventory as TargetInventory>::Stack> {
        let inventory = self.inventory();
        (0..9).map(|slot| inventory.item(slot)).collect()
    }
}
pub struct BlockTargets;
impl BlockTargets {
    pub fn aim_at(host: &impl TargetHost, pos: BlockPos) -> Option<ViewRotation> {
        let eye = host.eye_position();
        let reach = host.reach();
        let point = host.seen_point(eye, pos, reach, host.sent_rotation(), host.view_step());
        point.map(|point| ViewRotation::from_vector(subtract(point, eye)))
    }
    pub fn raycast(host: &impl TargetHost, rotation: ViewRotation, reach: f64) -> BlockHit {
        host.pick(rotation, reach, None)
    }
    pub fn plan_placement(
        host: &impl TargetHost,
        pos: BlockPos,
        slot: i32,
        nearest: bool,
    ) -> Option<Plan> {
        if !host.state(pos).can_be_replaced() {
            return None;
        }
        let eye = host.eye_position();
        let reach = host.reach();
        let feet = host.block_position();
        let under_feet = nearest || pos.x == feet.x && pos.z == feet.z && pos.y <= feet.y;
        let mut best = None;
        let mut best_turn = f32::MAX;
        for direction in DIRECTIONS {
            let against = relative(pos, direction);
            let state = host.state(against);
            if state.can_be_replaced() || host.outline_empty(&state, against) {
                continue;
            }
            let face = opposite(direction);
            let center = add(center(against), scale(unit(face), 0.5));
            let outward = dot(subtract(eye, center), unit(face));
            if outward <= 0.0 {
                continue;
            }
            let seed = host.uuid_most_significant_bits()
                ^ against.packed().wrapping_add(ordinal(face) as i64);
            let click = click_point(against, face, seed);
            let point = if under_feet {
                Self::nearest_on_face(host, against, face, eye)
            } else if eye.distance_sqr(click) <= reach * reach
                && host.is_seen(eye, click, against, face)
            {
                click
            } else {
                center
            };
            if eye.distance_sqr(point) > reach * reach || !host.is_seen(eye, point, against, face) {
                continue;
            }
            let rotation = ViewRotation::from_vector(subtract(point, eye));
            let turn = Self::turn(host, rotation);
            if turn < best_turn {
                best = Some(Plan {
                    pos,
                    against,
                    face,
                    point,
                    rotation,
                    slot,
                });
                best_turn = turn;
            }
        }
        best
    }
    pub fn held_click(
        host: &impl TargetHost,
        pos: BlockPos,
        rotation: ViewRotation,
    ) -> Option<Plan> {
        let slot = Self::placeable_slot(host)?;
        let hit = Self::raycast(host, rotation, host.reach());
        if !hit.is_block
            || relative(hit.pos, hit.direction) != pos
            || !host.state(pos).can_be_replaced()
        {
            return None;
        }
        Some(Plan {
            pos,
            against: hit.pos,
            face: hit.direction,
            point: hit.location,
            rotation,
            slot,
        })
    }
    pub fn clicks_from(
        host: &impl TargetHost,
        eye: Vec3,
        rotation: ViewRotation,
        pos: BlockPos,
    ) -> bool {
        let hit = host.pick(rotation, host.reach(), Some(eye));
        hit.is_block && relative(hit.pos, hit.direction) == pos
    }
    pub fn digging_slot<H: TargetHost>(host: &H, state: &H::State) -> Option<i32> {
        let hotbar = host.hotbar();
        let mut best = None;
        let mut best_speed = 0.0_f32;
        for (slot, stack) in hotbar.iter().enumerate() {
            if !host.is_tool_for(stack, state) {
                continue;
            }
            let mut speed = host.destroy_speed(stack, state);
            let level = host.efficiency_level(stack);
            if speed > 1.0 && level > 0 {
                speed += level.wrapping_mul(level) as f32 + 1.0_f32;
            }
            if speed > best_speed {
                best = Some(slot as i32);
                best_speed = speed;
            }
        }
        if best.is_some() {
            return best;
        }
        let selected = host.inventory().selected_slot();
        let mut distance = i32::MAX;
        for (slot, stack) in hotbar.iter().enumerate() {
            if host.is_wear_free(stack) {
                let diff = (slot as i32).wrapping_sub(selected).wrapping_abs();
                if best.is_none() || diff < distance {
                    best = Some(slot as i32);
                    distance = diff;
                }
            }
        }
        best
    }
    pub fn placeable_slot(host: &impl TargetHost) -> Option<i32> {
        let blocks = host.placeable_blocks();
        Self::closest_slot(host, |stack| {
            blocks.is_placeable(stack) && !blocks.is_unfavourable(stack)
        })
        .or_else(|| Self::closest_slot(host, |stack| blocks.is_placeable(stack)))
    }
    fn closest_slot<H: TargetHost>(
        host: &H,
        accepts: impl Fn(&<H::Inventory as TargetInventory>::Stack) -> bool,
    ) -> Option<i32> {
        let inventory = host.inventory();
        if accepts(&inventory.item(40)) {
            return Some(-1);
        }
        let selected = inventory.selected_slot();
        let hotbar = host.hotbar();
        let mut best = None;
        let mut distance = i32::MAX;
        for slot in 0..hotbar.len() {
            if accepts(&inventory.item(slot as i32)) {
                let diff = (slot as i32).wrapping_sub(selected).wrapping_abs();
                if best.is_none() || diff < distance {
                    best = Some(slot as i32);
                    distance = diff;
                }
            }
        }
        best
    }
    fn nearest_on_face(
        host: &impl TargetHost,
        against: BlockPos,
        face: Direction,
        eye: Vec3,
    ) -> Vec3 {
        let normal = unit(face);
        let center = add(center(against), scale(normal, 0.5));
        let look = host.sent_rotation().view_vector();
        let along = dot(look, normal);
        let on_plane = if along.abs() < 1.0e-3 {
            center
        } else {
            add(eye, scale(look, dot(subtract(center, eye), normal) / along))
        };
        let inside = |a, min: i32| {
            let choose = |v: Vec3| match a {
                Axis::X => v.x,
                Axis::Y => v.y,
                Axis::Z => v.z,
            };
            if axis(face) == a {
                choose(center)
            } else {
                let value = choose(on_plane);
                let low = f64::from(min) + 0.05;
                let high = f64::from(min) + 1.0 - 0.05;
                if value < low {
                    low
                } else if value > high {
                    high
                } else {
                    value
                }
            }
        };
        Vec3::new(
            inside(Axis::X, against.x),
            inside(Axis::Y, against.y),
            inside(Axis::Z, against.z),
        )
    }
    fn turn(host: &impl TargetHost, rotation: ViewRotation) -> f32 {
        let sent = host.sent_rotation();
        wrap_degrees(rotation.y_rot - sent.y_rot).abs() + (rotation.x_rot - sent.x_rot).abs()
    }
}

/// java.util.Random(seed), including nextDouble's two draws and nextBoolean.
struct JavaRandom(u64);
impl JavaRandom {
    fn new(seed: i64) -> Self {
        Self((seed as u64 ^ 0x5deece66d) & ((1 << 48) - 1))
    }
    fn bits(&mut self, bits: u32) -> u64 {
        self.0 = self.0.wrapping_mul(0x5deece66d).wrapping_add(11) & ((1 << 48) - 1);
        self.0 >> (48 - bits)
    }
    fn double(&mut self) -> f64 {
        ((self.bits(26) << 27) + self.bits(27)) as f64 / ((1u64 << 53) as f64)
    }
    fn boolean(&mut self) -> bool {
        self.bits(1) != 0
    }
}
pub fn click_point(pos: BlockPos, face: Direction, seed: i64) -> Vec3 {
    let mut random = JavaRandom::new(seed);
    let middle = add(center(pos), scale(unit(face), 0.5));
    let mut along = |a: Axis, value: f64| {
        if axis(face) == a {
            return value;
        }
        let offset = 0.08 + random.double() * (0.35 - 0.08);
        if random.boolean() {
            value + offset
        } else {
            value - offset
        }
    };
    Vec3::new(
        along(Axis::X, middle.x),
        along(Axis::Y, middle.y),
        along(Axis::Z, middle.z),
    )
}
