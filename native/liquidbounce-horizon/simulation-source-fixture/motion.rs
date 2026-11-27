use crate::navigation::{
    aabb::Direction,
    geometry::{BlockPos, Vec3},
    simulation::{
        Mechanics,
        block_grid::{BlockGrid, ShapeGrid, SimulationState},
        motion_traits::MotionTraits,
        player_motion::{MotionPlayer, PlayerMotion},
    },
    terrain::{BlockKind, TerrainLevel, TerrainState},
    voxel_shape::VoxelShape,
    walk::WalkState,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
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
    let a: Vec<i32> = s.split(',').map(|v| v.parse().unwrap()).collect();
    BlockPos::new(a[0], a[1], a[2])
}
fn raw(v: Vec3) -> String {
    [v.x, v.y, v.z].map(|v| v.to_bits().to_string()).join(",")
}
#[derive(Clone)]
struct State(Arc<Facts>);
struct Facts {
    id: i32,
    block: String,
    flags: i32,
    fluid: bool,
    air: bool,
    shape: i32,
    friction: f32,
    speed: f32,
    jump: f32,
    bounce: f32,
    reduction: f32,
}
impl TerrainState for State {
    fn same_instance(&self, o: &Self) -> bool {
        Arc::ptr_eq(&self.0, &o.0)
    }
    fn same_block(&self, o: &Self) -> bool {
        self.0.block == o.0.block
    }
    fn same_fluid(&self, o: &Self) -> bool {
        self.0.fluid == o.0.fluid
    }
    fn is_hazard(&self) -> bool {
        false
    }
    fn is_water(&self) -> bool {
        self.0.fluid
    }
    fn has_liquid(&self) -> bool {
        self.0.fluid
    }
    fn is_climbable(&self) -> bool {
        false
    }
    fn kind(&self, _: BlockKind) -> bool {
        false
    }
    fn open(&self) -> Option<bool> {
        None
    }
    fn with_open(&self, _: bool) -> Self {
        self.clone()
    }
    fn is_upper_half(&self) -> bool {
        false
    }
    fn horizontal_facing(&self) -> Option<Direction> {
        None
    }
    fn bounce_restitution(&self) -> f32 {
        self.0.bounce
    }
    fn collision_shape<L: TerrainLevel<Self>>(&self, _: &L, _: BlockPos) -> VoxelShape {
        panic!("native shape-grid fixture supplies collision shapes directly")
    }
}
impl WalkState for State {
    fn is_air(&self) -> bool {
        self.0.air
    }
    fn can_be_replaced(&self) -> bool {
        self.0.air
    }
    fn is_falling(&self) -> bool {
        false
    }
    fn is_slime(&self) -> bool {
        self.0.flags & 1 != 0
    }
    fn friction(&self) -> f32 {
        self.0.friction
    }
    fn speed_factor(&self) -> f32 {
        self.0.speed
    }
    fn jump_factor(&self) -> f32 {
        self.0.jump
    }
}
impl SimulationState for State {
    fn is_fence(&self) -> bool {
        self.0.flags & 2 != 0
    }
    fn is_wall(&self) -> bool {
        self.0.flags & 4 != 0
    }
    fn is_fence_gate(&self) -> bool {
        self.0.flags & 8 != 0
    }
    fn is_cobweb(&self) -> bool {
        self.0.block == "cobweb"
    }
    fn is_step_air(&self) -> bool {
        self.0.block == "air" || self.0.block == "void_air"
    }
    fn fall_distance_reduction(&self) -> f32 {
        self.0.reduction
    }
}
#[derive(Default)]
struct World {
    shapes: HashMap<i32, VoxelShape>,
    states: HashMap<i32, State>,
    blocks: HashMap<BlockPos, i32>,
    floor: i32,
    floor_state: i32,
    trace: Rc<RefCell<Vec<String>>>,
}
impl World {
    fn id(&self, p: BlockPos) -> i32 {
        self.blocks
            .get(&p)
            .copied()
            .unwrap_or(if p.y == self.floor {
                self.floor_state
            } else {
                0
            })
    }
}
impl ShapeGrid for World {
    fn shape(&mut self, p: BlockPos) -> VoxelShape {
        self.trace
            .borrow_mut()
            .push(format!("S:{},{},{}", p.x, p.y, p.z));
        self.shapes[&self.states[&self.id(p)].0.shape].clone()
    }
}
impl BlockGrid for World {
    type State = State;
    fn state(&mut self, p: BlockPos) -> State {
        self.trace
            .borrow_mut()
            .push(format!("B:{},{},{}", p.x, p.y, p.z));
        self.states[&self.id(p)].clone()
    }
}
struct PlayerCapture {
    pos: Vec3,
    delta: Vec3,
    ground: bool,
    sprint: bool,
    delay: i32,
    crouch: bool,
    support: Option<BlockPos>,
    trace: Rc<RefCell<Vec<String>>>,
}
impl MotionPlayer for PlayerCapture {
    fn position(&self) -> Vec3 {
        self.trace.borrow_mut().push("P:position".into());
        self.pos
    }
    fn delta_movement(&self) -> Vec3 {
        self.trace.borrow_mut().push("P:getDeltaMovement".into());
        self.delta
    }
    fn on_ground(&self) -> bool {
        self.trace.borrow_mut().push("P:onGround".into());
        self.ground
    }
    fn is_sprinting(&self) -> bool {
        self.trace.borrow_mut().push("P:isSprinting".into());
        self.sprint
    }
    fn jump_delay(&self) -> i32 {
        self.trace.borrow_mut().push("P:getNoJumpDelay".into());
        self.delay
    }
    fn is_crouching(&self) -> bool {
        self.trace.borrow_mut().push("P:isCrouching".into());
        self.crouch
    }
    fn supporting_block(&self) -> Option<BlockPos> {
        self.trace
            .borrow_mut()
            .push("P:getMainSupportingBlockPos".into());
        self.support
    }
}
fn snapshot(m: &PlayerMotion) -> String {
    let block = m.block_position();
    [
        raw(m.position()),
        raw(m.delta_movement()),
        m.on_ground.to_string(),
        m.horizontal_collision.to_string(),
        m.is_sprinting.to_string(),
        m.landing_fall.to_bits().to_string(),
        m.peak_y.to_bits().to_string(),
        m.jump_delay.to_string(),
        format!(
            "{},{},{},{}",
            m.support.exists, m.support.x, m.support.y, m.support.z
        ),
        m.ground_without_blocks.to_string(),
        m.crouching.to_string(),
        m.sneaking.to_string(),
        m.air_sprint.to_string(),
        m.walked.to_bits().to_string(),
        m.next_step.to_string(),
        format!("{},{},{}", block.x, block.y, block.z),
        m.horizontal_speed().to_bits().to_string(),
        format!(
            "{},{},{},{}",
            m.traits.walking_speed.to_bits(),
            m.traits.jump_strength.to_bits(),
            m.traits.jump_boost_power.to_bits(),
            if m.traits.mechanics == Mechanics::Modern {
                "MODERN"
            } else {
                "V1_8"
            }
        ),
    ]
    .join(";")
}
#[test]
fn unchanged_player_motion_modern_and_1_8_match_all_sequential_states_and_queries() {
    let commands = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/commands.txt"));
    let expected: Vec<_> = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/results.txt"))
        .lines()
        .collect();
    let mut world = World::default();
    let mut motion = PlayerMotion::default();
    let mut other = None;
    for (index, command) in commands.lines().enumerate() {
        let a: Vec<_> = command.split(' ').collect();
        let mut result = "-".to_owned();
        match a[0] {
            "shape" => {
                let shape = match a[2] {
                    "E" => VoxelShape::empty(),
                    "F" => VoxelShape::block(),
                    s => {
                        let values: Vec<_> = s.split(',').map(d).collect();
                        VoxelShape::new(values.as_chunks::<6>().0.to_vec())
                    }
                };
                world.shapes.insert(a[1].parse().unwrap(), shape);
            }
            "define" => {
                let id = a[1].parse().unwrap();
                world.states.insert(
                    id,
                    State(Arc::new(Facts {
                        id,
                        block: a[2].into(),
                        flags: a[3].parse().unwrap(),
                        fluid: a[4].parse().unwrap(),
                        air: a[5].parse().unwrap(),
                        shape: a[6].parse().unwrap(),
                        friction: f(a[7]),
                        speed: f(a[8]),
                        jump: f(a[9]),
                        bounce: f(a[10]),
                        reduction: f(a[11]),
                    })),
                );
            }
            "new" => {
                world.blocks.clear();
                world.floor = a[1].parse().unwrap();
                world.floor_state = a[2].parse().unwrap();
                motion = PlayerMotion::new(MotionTraits::new(
                    f(a[4]),
                    f(a[5]),
                    f(a[6]),
                    if a[3] == "modern" {
                        Mechanics::Modern
                    } else {
                        Mechanics::V1_8
                    },
                ));
                other = None;
            }
            "set" => {
                world.blocks.insert(p(a[1]), a[2].parse().unwrap());
            }
            "reset" => motion.reset(
                &mut world,
                v(a[1]),
                v(a[2]),
                a[3].parse().unwrap(),
                a[4].parse().unwrap(),
                a[5].parse().unwrap(),
            ),
            "resetPlayer" => {
                let player = PlayerCapture {
                    pos: v(a[1]),
                    delta: v(a[2]),
                    ground: a[3].parse().unwrap(),
                    sprint: a[4].parse().unwrap(),
                    delay: a[5].parse().unwrap(),
                    crouch: a[6].parse().unwrap(),
                    support: if a[7] == "-" { None } else { Some(p(a[7])) },
                    trace: Rc::clone(&world.trace),
                };
                motion.reset_player(&mut world, &player);
            }
            "tick" => motion.tick(
                &mut world,
                f(a[1]),
                a[2].parse().unwrap(),
                a[3].parse().unwrap(),
                a[4].parse().unwrap(),
                a[5].parse().unwrap(),
                a[6].parse().unwrap(),
            ),
            "copy" => {
                let copy = motion.copy();
                other = Some(motion);
                motion = copy;
            }
            "swap" => std::mem::swap(&mut motion, other.as_mut().unwrap()),
            "copyFrom" => motion.copy_from(other.as_ref().unwrap()),
            "fields" => {
                motion.crouching = a[1].parse().unwrap();
                motion.sneaking = a[2].parse().unwrap();
                motion.landing_fall = d(a[3]);
                motion.peak_y = d(a[4]);
                motion.jump_delay = a[5].parse().unwrap();
            }
            "footing" => result = motion.footing(&mut world, p(a[1])).to_bits().to_string(),
            "stands" => {
                let p = p(a[1]);
                result = motion.stands_on(p.x, p.y, p.z).to_string();
            }
            op => panic!("unknown command {op}"),
        }
        let state = if a[0] == "shape" || a[0] == "define" {
            "-".into()
        } else {
            snapshot(&motion)
        };
        let actual = format!("{result}|{state}|{}", world.trace.borrow().join("/"));
        world.trace.borrow_mut().clear();
        assert_eq!(actual, expected[index], "command {index}: {command}");
    }
    assert_eq!(commands.lines().count(), expected.len());
}
