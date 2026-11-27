use crate::navigation::{
    aabb::{Aabb, Axis},
    geometry::{BlockPos, Vec3},
    simulation::{
        Mechanics, block_colliders::BlockColliders, block_grid::ShapeGrid, collisions,
        edge_backoff::EdgeBackOff, motion_traits::MotionTraits, support::Support,
    },
    voxel_shape::VoxelShape,
};
use std::collections::HashMap;
fn d(s: &str) -> f64 {
    f64::from_bits(s.parse().unwrap())
}
fn f(s: &str) -> f32 {
    f32::from_bits(s.parse().unwrap())
}
fn v(s: &str) -> Vec3 {
    let a: Vec<_> = s.split(',').map(d).collect();
    Vec3::new(a[0], a[1], a[2])
}
fn p(s: &str) -> BlockPos {
    let a: Vec<i32> = s.split(',').map(|s| s.parse().unwrap()).collect();
    BlockPos::new(a[0], a[1], a[2])
}
fn b(s: &str) -> Aabb {
    let a: Vec<_> = s.split(',').map(d).collect();
    Aabb::new(Vec3::new(a[0], a[1], a[2]), Vec3::new(a[3], a[4], a[5]))
}
fn raw(v: Vec3) -> String {
    [v.x, v.y, v.z].map(|v| v.to_bits().to_string()).join(",")
}
#[derive(Default)]
struct World {
    shapes: HashMap<i32, VoxelShape>,
    blocks: HashMap<BlockPos, i32>,
    floor: i32,
    floor_shape: i32,
    trace: Vec<String>,
}
impl ShapeGrid for World {
    fn shape(&mut self, pos: BlockPos) -> VoxelShape {
        self.trace.push(format!("S:{},{},{}", pos.x, pos.y, pos.z));
        let id = self
            .blocks
            .get(&pos)
            .copied()
            .unwrap_or(if pos.y == self.floor {
                self.floor_shape
            } else {
                0
            });
        self.shapes[&id].clone()
    }
}
#[test]
fn unchanged_agent_collision_sources_match_all_steps_and_shape_cache_queries() {
    let commands = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/commands.txt"));
    let expected: Vec<_> = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/results.txt"))
        .lines()
        .collect();
    let mut world = World::default();
    let mut colliders = BlockColliders::default();
    let mut support = Support::default();
    let mut edge = EdgeBackOff::default();
    for (index, command) in commands.lines().enumerate() {
        let a: Vec<_> = command.split(' ').collect();
        let mut result = "-".to_owned();
        match a[0] {
            "shape" => {
                let shape = match a[2] {
                    "E" => VoxelShape::empty(),
                    "F" => VoxelShape::block(),
                    s => {
                        let numbers: Vec<_> = s.split(',').map(d).collect();
                        VoxelShape::new(numbers.as_chunks::<6>().0.to_vec())
                    }
                };
                world.shapes.insert(a[1].parse().unwrap(), shape);
            }
            "new" => {
                world.blocks.clear();
                world.floor = a[1].parse().unwrap();
                world.floor_shape = a[2].parse().unwrap();
                colliders = BlockColliders::default();
                support = Support::default();
                edge = EdgeBackOff::default();
            }
            "set" => {
                world.blocks.insert(p(a[1]), a[2].parse().unwrap());
            }
            "intersects" => {
                result = colliders
                    .intersects(&mut world, p(a[1]), b(a[2]))
                    .to_string()
            }
            "footing" => {
                result = colliders
                    .footing(&mut world, p(a[1]), d(a[2]), d(a[3]), d(a[4]))
                    .to_bits()
                    .to_string()
            }
            "collect" => colliders.collect(&mut world, b(a[1])),
            "move" => {
                colliders.collide(v(a[1]), b(a[2]));
                result = raw(Vec3::new(
                    colliders.moved_x,
                    colliders.moved_y,
                    colliders.moved_z,
                ));
            }
            "offset" => {
                result = colliders
                    .offset(Axis::ALL[a[1].parse::<usize>().unwrap()], b(a[2]), d(a[3]))
                    .to_bits()
                    .to_string()
            }
            "steps" => {
                result = colliders
                    .step_heights(b(a[1]), f(a[2]), f(a[3]))
                    .iter()
                    .map(|v| v.to_bits().to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }
            "collide" => {
                result = raw(collisions::collide(
                    &mut colliders,
                    &mut world,
                    v(a[1]),
                    a[2].parse().unwrap(),
                    v(a[3]),
                ))
            }
            "fall" => {
                result = collisions::can_fall(
                    &mut colliders,
                    &mut world,
                    v(a[1]),
                    d(a[2]),
                    d(a[3]),
                    d(a[4]),
                )
                .to_string()
            }
            "support" => {
                support.find(&mut colliders, &mut world, v(a[1]), d(a[2]), d(a[3]));
                result = format!(
                    "{},{},{},{}",
                    support.exists, support.x, support.y, support.z
                );
            }
            "clear" => {
                support.clear();
                result = format!(
                    "{},{},{},{}",
                    support.exists, support.x, support.y, support.z
                );
            }
            "edge" => {
                edge.back_off(d(a[2]), d(a[3]), |dx, dz, depth| {
                    collisions::can_fall(&mut colliders, &mut world, v(a[1]), dx, dz, depth)
                });
                result = format!("{},{}", edge.x.to_bits(), edge.z.to_bits());
            }
            "traits" => {
                let t = MotionTraits::new(
                    f(a[1]),
                    f(a[2]),
                    f(a[3]),
                    if a[4] == "modern" {
                        Mechanics::Modern
                    } else {
                        Mechanics::V1_8
                    },
                );
                result = format!(
                    "{},{},{},{},{},{},{}",
                    t.walking_speed.to_bits(),
                    t.jump_strength.to_bits(),
                    t.jump_boost_power.to_bits(),
                    t.jump_power.to_bits(),
                    t.jump_height.to_bits(),
                    t.mechanics.max_packets_per_tick(),
                    t.mechanics.is_move_judged_per_tick()
                );
            }
            "rise" => result = MotionTraits::rise(d(a[1])).to_bits().to_string(),
            op => panic!("unknown command {op}"),
        }
        let actual = format!("{result}|{}", world.trace.join("/"));
        world.trace.clear();
        assert_eq!(actual, expected[index], "command {index}: {command}");
    }
    assert_eq!(commands.lines().count(), expected.len());
}
