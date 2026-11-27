#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
extern crate self as horizon_world;
use block_collision_scan::{BlockCollisionScan, BlockCollisionWorld, CollisionBlock};
use collision_context::CollisionContext;
use entity_obstruction::EntityIdentity;
use horizon_core::{
    geometry::{Aabb, ShapeOp, Vec3, VoxelShape},
    position::{BlockPos, ChunkPos},
};
use registry::{BlockRegistry, BlockStateId};
use serde_json::{Value, json};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    io,
};
pub use world_kernel::{collision_context, entity_obstruction, registry};
mod behavior;
// The complete inherited scanner retains this upstream generic return type.
#[allow(clippy::type_complexity)]
pub mod block_collision_scan;
pub mod block_walk;
mod ground_support;
pub mod world_movement;
use world_movement::EntityMovementWorld;

fn aabb(value: &Value) -> Aabb {
    let v = value.as_array().unwrap();
    let n = |i: usize| v[i].as_f64().unwrap();
    Aabb::new(Vec3::new(n(0), n(1), n(2)), Vec3::new(n(3), n(4), n(5)))
}
fn shape(value: &Value) -> VoxelShape {
    value
        .as_array()
        .unwrap()
        .iter()
        .fold(VoxelShape::empty(), |shape, b| {
            shape
                .join(&VoxelShape::from_box(aabb(b)).unwrap(), ShapeOp::OR)
                .unwrap()
        })
}
struct Binding {
    registry: BlockRegistry,
    shapes: Vec<VoxelShape>,
    large: Vec<bool>,
    piston: Vec<bool>,
    suffocating: Vec<bool>,
    calls: Cell<usize>,
}
impl Binding {
    fn new(oracle: &Value) -> Self {
        let states = oracle["states"].as_array().unwrap();
        let mut defaults = serde_json::Map::new();
        let rows: Vec<_> = states
            .iter()
            .enumerate()
            .map(|(i, s)| {
                defaults
                    .entry(s["id"].as_str().unwrap().to_owned())
                    .or_insert(json!(i));
                json!([s["id"],false,false,"test",{"id":s["id"],"properties":s["properties"]}])
            })
            .collect();
        let facts = json!({"states":rows,"defaults":defaults});
        Self {
            registry: BlockRegistry::from_facts_json(facts.to_string().as_bytes()).unwrap(),
            shapes: states.iter().map(|s| shape(&s["boxes"])).collect(),
            large: states
                .iter()
                .map(|s| s["large"].as_bool().unwrap())
                .collect(),
            piston: states
                .iter()
                .map(|s| s["piston"].as_bool().unwrap())
                .collect(),
            suffocating: states
                .iter()
                .map(|s| s["suffocating"].as_bool().unwrap())
                .collect(),
            calls: Cell::new(0),
        }
    }
    fn registry(&self) -> &BlockRegistry {
        &self.registry
    }
    fn collision_block(&self, state: BlockStateId) -> io::Result<CollisionBlock> {
        let i = state.0 as usize;
        Ok(CollisionBlock {
            state,
            large_collision_shape: self.large[i],
            moving_piston: self.piston[i],
            empty_shape: self.shapes[i].is_empty(),
        })
    }
    fn collision_shape_with_context(
        &self,
        _: &mut SceneCollisions<'_, '_>,
        state: BlockStateId,
        _: BlockPos,
        _: &CollisionContext<'_>,
    ) -> io::Result<VoxelShape> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.shapes[state.0 as usize].clone())
    }
    fn may_collide(&self, state: BlockStateId) -> bool {
        !self.shapes[state.0 as usize].is_empty()
    }
    fn reaches_out(&self, state: BlockStateId) -> bool {
        self.large[state.0 as usize] || self.piston[state.0 as usize]
    }
}
mod chunk {
    use super::*;
    pub struct Chunk {
        pub cells: BTreeMap<(i32, i32, i32), BlockStateId>,
    }
    impl Chunk {
        pub fn block(&self, x: i32, y: i32, z: i32) -> BlockStateId {
            self.cells
                .get(&(x & 15, y, z & 15))
                .copied()
                .unwrap_or(BlockStateId(0))
        }
    }
}
struct World {
    chunks: BTreeMap<(i32, i32), Box<chunk::Chunk>>,
}
impl World {
    fn new(row: &Value) -> Self {
        let mut chunks = BTreeMap::new();
        for cx in -3..=3 {
            for cz in -3..=3 {
                chunks.insert(
                    (cx, cz),
                    Box::new(chunk::Chunk {
                        cells: BTreeMap::new(),
                    }),
                );
            }
        }
        let state = BlockStateId(row["state"].as_u64().unwrap() as u32);
        for p in row["positions"].as_array().unwrap() {
            let x = p[0].as_i64().unwrap() as i32;
            let y = p[1].as_i64().unwrap() as i32;
            let z = p[2].as_i64().unwrap() as i32;
            chunks
                .get_mut(&(x >> 4, z >> 4))
                .unwrap()
                .cells
                .insert((x & 15, y, z & 15), state);
        }
        Self { chunks }
    }
    fn may_hold(
        &self,
        lo: [i32; 3],
        hi: [i32; 3],
        mut test: impl FnMut(BlockStateId) -> bool,
    ) -> bool {
        self.chunks.iter().any(|(&(cx, cz), chunk)| {
            chunk.cells.iter().any(|(&(x, y, z), &state)| {
                let (x, z) = (x + (cx << 4), z + (cz << 4));
                x >= lo[0]
                    && x <= hi[0]
                    && y >= lo[1]
                    && y <= hi[1]
                    && z >= lo[2]
                    && z <= hi[2]
                    && test(state)
            })
        })
    }
}
struct SceneCollisions<'w, 'r> {
    world: &'w World,
    binding: &'r Binding,
    subject: Option<()>,
}
impl SceneCollisions<'_, '_> {
    fn section_reaches_out(&self, chunk: &chunk::Chunk, sy: i32) -> bool {
        chunk
            .cells
            .iter()
            .any(|(&(_, y, _), &state)| y >> 4 == sy && self.binding.reaches_out(state))
    }
    fn section_blocks_all_empty(&self, chunk: &chunk::Chunk, sy: i32) -> bool {
        chunk.cells.iter().all(|(&(_, y, _), state)| {
            y >> 4 != sy || self.binding.shapes[state.0 as usize].is_empty()
        })
    }
    /* FAST_EMPTY */
}
/* SCENE_IMPL */
/* MOVEMENT_IMPL */
struct EntityCollisions<'a, W> {
    blocks: W,
    marker: std::marker::PhantomData<&'a ()>,
}
/* WRAPPER_IMPL */
struct Suffocating<'a, 'w, 'r>(&'a mut SceneCollisions<'w, 'r>);
impl<'w> BlockCollisionWorld for Suffocating<'_, 'w, '_> {
    type Error = io::Error;
    type Chunk = &'w chunk::Chunk;
    fn changes_collision_shapes(&self) -> bool {
        self.0.changes_collision_shapes()
    }
    fn modify_collision_shape(
        &mut self,
        b: CollisionBlock,
        p: BlockPos,
        s: VoxelShape,
    ) -> io::Result<VoxelShape> {
        self.0.modify_collision_shape(b, p, s)
    }
    fn collision_chunk(&mut self, p: ChunkPos) -> io::Result<Option<Self::Chunk>> {
        self.0.collision_chunk(p)
    }
    fn collision_block(&mut self, c: Self::Chunk, p: BlockPos) -> io::Result<CollisionBlock> {
        self.0.collision_block(c, p)
    }
    fn is_suffocating(
        &mut self,
        _: Self::Chunk,
        b: CollisionBlock,
        _: BlockPos,
    ) -> io::Result<bool> {
        Ok(self.0.binding.suffocating[b.state.0 as usize])
    }
    fn context_shape(
        &mut self,
        b: CollisionBlock,
        p: BlockPos,
        c: &CollisionContext<'_>,
    ) -> io::Result<VoxelShape> {
        self.0.context_shape(b, p, c)
    }
    fn may_reach_out(&mut self, c: Self::Chunk, sy: i32) -> io::Result<bool> {
        self.0.may_reach_out(c, sy)
    }
    fn section_is_empty(&mut self, c: Self::Chunk, sy: i32) -> io::Result<bool> {
        self.0.section_is_empty(c, sy)
    }
}
struct MobWorld<'a> {
    world: &'a World,
    collision: &'a Binding,
}
impl MobWorld<'_> {
    fn block_collision_boxes(&self, x: i32, y: i32, z: i32) -> Vec<Aabb> {
        let Some(chunk) = self.world.chunks.get(&(x >> 4, z >> 4)) else {
            return vec![];
        };
        self.collision.shapes[chunk.block(x, y, z).0 as usize]
            .moved(Vec3::new(x.into(), y.into(), z.into()))
            .boxes()
            .to_vec()
    }
    /* SUPPORT */
}
mod mod_api {
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(|m| &**m)
    }
    #[allow(unused_imports)]
    mod render {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/api-source/crates/horizon-client-render/src/gfx/native_render/generated.rs"
        ));
    }
    pub use render::*;
    #[allow(unused_imports)]
    mod host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs"));
    }
    pub use host::*;
    mod hooks {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs"));
    }
    pub use hooks::*;
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: std::sync::OnceLock<Vec<Box<dyn Hooks>>> = std::sync::OnceLock::new();
        MODS.get_or_init(|| vec![Box::new(TestHooks)])
    }
    struct TestHooks;
    impl Hooks for TestHooks {
        fn client_block_walk(&self, _: &dyn Host, event: &mut ClientBlockWalkEvent) {
            let flags = crate::FLAGS.lock().unwrap();
            event.blocks.clone_from(&flags.0);
            event.apply = flags.1;
        }
    }
}
static FLAGS: std::sync::Mutex<(String, bool)> = std::sync::Mutex::new((String::new(), false));
mod game {
    struct Game<'a> {
        window: bool,
        lan_acting_uuid: Option<()>,
        marker: std::marker::PhantomData<&'a ()>,
    }
    impl Default for Game<'_> {
        fn default() -> Self {
            Self {
                window: true,
                lan_acting_uuid: None,
                marker: std::marker::PhantomData,
            }
        }
    }
    impl Game<'_> {
        fn native_window(&self) -> bool {
            self.window
        }
    }
    mod native_hooks {
        use super::Game;
        use crate::mod_api; /* NATIVE */
    }
    mod mod_api_host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs"));
    }
    #[test]
    fn generated_policy_event_updates_only_from_actual_window_and_clears_disabled() {
        *crate::FLAGS.lock().unwrap() = ("[\"minecraft:cobweb\"]".into(), true);
        Game::default().native_client_tick_pre();
        assert!(crate::block_walk::forces_full_block("minecraft:cobweb"));
        *crate::FLAGS.lock().unwrap() = ("[]".into(), false);
        for window in [false, true] {
            let mut game = Game {
                window,
                lan_acting_uuid: if window { Some(()) } else { None },
                ..Default::default()
            };
            game.native_client_tick_pre();
            assert!(crate::block_walk::active());
        }
        Game::default().native_client_tick_pre();
        assert!(!crate::block_walk::active());
    }
}
fn oracle() -> Value {
    serde_json::from_str(include_str!("mc-oracle.json")).unwrap()
}
fn set(row: &Value) {
    let selected = if row["active"].as_bool().unwrap() {
        serde_json::from_value(row["selected"].clone()).unwrap()
    } else {
        BTreeSet::new()
    };
    block_walk::sync(selected);
}
fn scan<W: BlockCollisionWorld>(
    world: &mut W,
    bounds: Aabb,
    suffocating: bool,
) -> Vec<block_collision_scan::BlockCollisionHit>
where
    W::Error: std::fmt::Debug,
{
    BlockCollisionScan::new(
        world,
        CollisionContext::empty(false),
        bounds,
        suffocating,
        1_000_000,
    )
    .unwrap()
    .map(Result::unwrap)
    .collect()
}
fn check(row: &Value, actual: &[block_collision_scan::BlockCollisionHit], index: usize) {
    let expected = row["hits"].as_array().unwrap();
    assert_eq!(actual.len(), expected.len(), "MC scan count {index}");
    for (hit, want) in actual.iter().zip(expected) {
        assert_eq!(
            json!([hit.position.x, hit.position.y, hit.position.z]),
            want["position"],
            "MC cursor {index}"
        );
        assert!(
            !hit.shape
                .join_is_not_empty(&shape(&want["boxes"]), ShapeOp::XOR)
                .unwrap(),
            "MC geometry {index}"
        );
    }
}
#[test]
fn actual_mc_block_collision_cursor_and_shapes_all_policies() {
    let data = oracle();
    let binding = Binding::new(&data);
    for (index, row) in data["scans"].as_array().unwrap().iter().enumerate() {
        let world = World::new(row);
        let mut scene = SceneCollisions {
            world: &world,
            binding: &binding,
            subject: None,
        };
        set(row);
        let mut execute = || {
            if row["onlySuffocating"].as_bool().unwrap() {
                check(
                    row,
                    &scan(&mut Suffocating(&mut scene), aabb(&row["bounds"]), true),
                    index,
                );
            } else {
                check(row, &scan(&mut scene, aabb(&row["bounds"]), false), index);
            }
        };
        if row["suppressed"].as_bool().unwrap() {
            block_walk::without_shape_changes(execute);
        } else {
            execute();
        }
    }
    block_walk::sync(BTreeSet::new());
}
#[cfg(feature = "combined-airjump")]
#[test]
fn actual_mc_combined_airjump_and_blockwalk_obey_shared_shape_flag() {
    let data = oracle();
    let binding = Binding::new(&data);
    for (index, row) in data["combinedScans"].as_array().unwrap().iter().enumerate() {
        let world = World::new(row);
        let mut scene = SceneCollisions {
            world: &world,
            binding: &binding,
            subject: None,
        };
        set(row);
        let mut execute = || {
            block_collision_scan::with_client_ghost_block(
                Some(row["ghostFloor"].as_i64().unwrap() as i32),
                || {
                    if row["onlySuffocating"].as_bool().unwrap() {
                        check(
                            row,
                            &scan(&mut Suffocating(&mut scene), aabb(&row["bounds"]), true),
                            index,
                        );
                    } else {
                        check(row, &scan(&mut scene, aabb(&row["bounds"]), false), index);
                    }
                },
            )
        };
        if row["suppressed"].as_bool().unwrap() {
            block_walk::without_shape_changes(execute);
        } else {
            execute();
        }
    }
    block_walk::sync(BTreeSet::new());
}
#[test]
fn actual_player_and_mob_support_agree_with_mc_collision_candidates() {
    let data = oracle();
    let binding = Binding::new(&data);
    let from = Vec3::new(0.5, 1., 0.5);
    for (index, row) in data["scans"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, r)| !r["onlySuffocating"].as_bool().unwrap())
    {
        let world = World::new(row);
        set(row);
        let expected = row["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| {
                let p = &hit["position"];
                [
                    p[0].as_i64().unwrap() as i32,
                    p[1].as_i64().unwrap() as i32,
                    p[2].as_i64().unwrap() as i32,
                ]
            })
            .min_by(|a, b| {
                let distance = |p: &[i32; 3]| {
                    (f64::from(p[0]) + 0.5 - from.x).powi(2)
                        + (f64::from(p[1]) + 0.5 - from.y).powi(2)
                        + (f64::from(p[2]) + 0.5 - from.z).powi(2)
                };
                distance(a)
                    .total_cmp(&distance(b))
                    .then_with(|| (b[1], b[2], b[0]).cmp(&(a[1], a[2], a[0])))
            });
        let execute = || {
            let mut scene = SceneCollisions {
                world: &world,
                binding: &binding,
                subject: None,
            };
            let found = ground_support::find_supporting_block(
                &mut scene,
                CollisionContext::empty(false),
                from,
                aabb(&row["bounds"]),
                1_000_000,
            )
            .unwrap()
            .map(|p| [p.x, p.y, p.z]);
            assert_eq!(found, expected, "player support {index}");
            assert_eq!(
                MobWorld {
                    world: &world,
                    collision: &binding
                }
                .supporting_block(aabb(&row["bounds"]), from),
                expected,
                "mob support {index}"
            );
        };
        if row["suppressed"].as_bool().unwrap() {
            block_walk::without_shape_changes(execute);
        } else {
            execute();
        }
    }
    block_walk::sync(BTreeSet::new());
}
#[test]
fn physical_movement_lands_on_cobweb_while_direct_shape_remains_empty_and_wrappers_forward() {
    let data = oracle();
    let binding = Binding::new(&data);
    let row = json!({"state":1,"positions":[[0,0,0]]});
    let world = World::new(&row);
    for active in [false, true] {
        block_walk::sync(if active {
            BTreeSet::from(["minecraft:cobweb".into()])
        } else {
            BTreeSet::new()
        });
        let scene = SceneCollisions {
            world: &world,
            binding: &binding,
            subject: None,
        };
        let mut wrapped = EntityCollisions {
            blocks: scene,
            marker: std::marker::PhantomData,
        };
        assert_eq!(wrapped.changes_collision_shapes(), active);
        let at = BlockPos::new(0, 0, 0);
        let block = binding.collision_block(BlockStateId(1)).unwrap();
        let forwarded = wrapped
            .modify_collision_shape(block, at, VoxelShape::empty())
            .unwrap();
        assert_eq!(forwarded.is_empty(), !active);
        let empty_bounds = Aabb::new(Vec3::new(0.2, 0.2, 0.2), Vec3::new(0.8, 0.8, 0.8));
        assert_eq!(
            wrapped.blocks.cells_without_collision(empty_bounds),
            !active
        );
        let bounds = Aabb::new(Vec3::new(0.2, 2., 0.2), Vec3::new(0.8, 3.8, 0.8));
        let movement = world_movement::resolve_world_movement(
            &mut wrapped.blocks,
            world_movement::MovementRequest {
                entity: EntityIdentity(0),
                context: CollisionContext::empty(false),
                bounds,
                movement: Vec3::new(0., -2., 0.),
                max_step: 0.,
                on_ground: false,
                candidate_budget: 10000,
            },
        )
        .unwrap();
        assert_eq!(movement.y, if active { -1. } else { -2. });
        assert!(
            MobWorld {
                world: &world,
                collision: &binding
            }
            .block_collision_boxes(0, 0, 0)
            .is_empty()
        );
    }
    assert!(binding.calls.get() > 0);
    block_walk::sync(BTreeSet::new());
}
#[test]
fn suppression_nests_and_restores_after_unwind() {
    block_walk::sync(BTreeSet::from(["minecraft:air".into()]));
    assert!(block_walk::active());
    let result = std::panic::catch_unwind(|| {
        block_walk::without_shape_changes(|| {
            assert!(!block_walk::active());
            block_walk::without_shape_changes(|| assert!(!block_walk::active()));
            assert!(!block_walk::active());
            panic!("test unwind");
        })
    });
    assert!(result.is_err());
    assert!(block_walk::active());
    block_walk::sync(BTreeSet::new());
}
#[test]
fn exact_mc_registry_and_source_derived_gson_import_oracle() {
    let data = oracle();
    let registry: Vec<String> = serde_json::from_value(data["registry"].clone()).unwrap();
    for (index, row) in data["imports"].as_array().unwrap().iter().enumerate() {
        let mut state = behavior::BlockWalk::default();
        let result = state.deserialize_blocks(&row["input"], &registry);
        assert_eq!(
            result.is_ok(),
            row["success"].as_bool().unwrap(),
            "registry import result {index}"
        );
        assert_eq!(
            json!(state.sorted_block_ids()),
            row["blocks"],
            "registry import value {index}"
        );
    }
}
