// SPDX-License-Identifier: GPL-3.0-or-later
//! Original glide Contacts, retaining face/query and JVM HashSet callback order.
use super::super::{
    aabb::{Aabb, Direction},
    geometry::{BlockPos, Vec3},
    terrain::{TerrainLevel, TerrainState},
};
pub trait ContactPlayer {
    fn is_fall_flying(&self) -> bool;
    fn horizontal_collision(&self) -> bool;
    fn vertical_collision(&self) -> bool;
    fn vertical_collision_below(&self) -> bool;
    fn bounding_box(&self) -> Aabb;
}
pub trait ContactBody {
    type Player: ContactPlayer;
    type State: TerrainState;
    type Level: TerrainLevel<Self::State>;
    fn player(&self) -> &Self::Player;
    fn live_level(&self) -> Self::Level;
    /// java.lang.Enum.hashCode of the host's Agent Direction entries, captured once.
    fn direction_hash(&self, direction: Direction) -> i32;
}
pub trait ContactListener {
    fn touched(&mut self, pos: BlockPos, direction: Direction);
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Touch {
    pos: BlockPos,
    direction: Direction,
}
#[derive(Default)]
pub struct Contacts {
    touching: Vec<Touch>,
}
impl Contacts {
    pub fn tick(&mut self, body: &impl ContactBody, listener: &mut impl ContactListener) {
        let player = body.player();
        if !player.is_fall_flying()
            || !player.horizontal_collision() && !player.vertical_collision()
        {
            self.touching.clear();
            return;
        }
        let mut now = Vec::with_capacity(6);
        for direction in [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            let hit = match direction {
                Direction::Down => player.vertical_collision_below(),
                Direction::Up => player.vertical_collision() && !player.vertical_collision_below(),
                _ => player.horizontal_collision(),
            };
            if hit {
                let bbox = player.bounding_box();
                if let Some(pos) = touched_block(body.live_level(), bbox, direction) {
                    now.push(Touch { pos, direction });
                }
            }
        }
        // At most six entries: default JDK HashSet's table stays at capacity 16.
        // Stable ordering retains insertion order within each collision chain.
        now.sort_by_key(|touch| {
            let p = touch.pos;
            let hash =
                p.y.wrapping_add(p.z.wrapping_mul(31))
                    .wrapping_mul(31)
                    .wrapping_add(p.x)
                    .wrapping_mul(31)
                    .wrapping_add(body.direction_hash(touch.direction));
            let hash = hash as u32;
            (hash ^ (hash >> 16)) & 15
        });
        for touch in &now {
            if !self.touching.contains(touch) {
                listener.touched(touch.pos, touch.direction);
            }
        }
        self.touching = now;
    }
}
pub fn touched_block<B: TerrainState, L: TerrainLevel<B>>(
    level: L,
    bbox: Aabb,
    direction: Direction,
) -> Option<BlockPos> {
    let (x, y, z) = match direction {
        Direction::Down => (0, -1, 0),
        Direction::Up => (0, 1, 0),
        Direction::North => (0, 0, -1),
        Direction::South => (0, 0, 1),
        Direction::West => (-1, 0, 0),
        Direction::East => (1, 0, 0),
    };
    let probe = Vec3::new(
        f64::from(x) * 1.0e-3,
        f64::from(y) * 1.0e-3,
        f64::from(z) * 1.0e-3,
    );
    let face = bbox.expand_towards(probe).intersect(bbox.moved(Vec3::new(
        probe.x * 2.0,
        probe.y * 2.0,
        probe.z * 2.0,
    )));
    for x in face.min.x.floor() as i32..=face.max.x.floor() as i32 {
        for y in face.min.y.floor() as i32..=face.max.y.floor() as i32 {
            for z in face.min.z.floor() as i32..=face.max.z.floor() as i32 {
                let pos = BlockPos::new(x, y, z);
                let shape = level.block_state(pos).collision_shape(&level, pos);
                if shape
                    .aabbs()
                    .iter()
                    .any(|bounds| bounds.moved_block(pos).intersects(face))
                {
                    return Some(pos);
                }
            }
        }
    }
    None
}
