use crate::navigation::{
    aabb::{Aabb, Axis, Direction},
    geometry::{BlockPos, Vec3},
    terrain::{BlockKind, DestinationCheck, Terrain, TerrainLevel, TerrainState},
    voxel_shape::VoxelShape,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};
fn d(raw: &str) -> f64 {
    f64::from_bits(raw.parse().unwrap())
}
fn values(raw: &str) -> Vec<f64> {
    raw.split(',').map(d).collect()
}
fn v(raw: &str) -> Vec3 {
    let a = values(raw);
    Vec3::new(a[0], a[1], a[2])
}
fn p(raw: &str) -> BlockPos {
    let a: Vec<i32> = raw.split(',').map(|v| v.parse().unwrap()).collect();
    BlockPos::new(a[0], a[1], a[2])
}
fn b(raw: &str) -> Aabb {
    let a = values(raw);
    Aabb::new(Vec3::new(a[0], a[1], a[2]), Vec3::new(a[3], a[4], a[5]))
}
fn position(p: BlockPos) -> String {
    format!("{},{},{}", p.x, p.y, p.z)
}
fn vector(v: Vec3) -> String {
    format!("{},{},{}", v.x.to_bits(), v.y.to_bits(), v.z.to_bits())
}
fn bbox(b: Aabb) -> String {
    format!("{},{}", vector(b.min), vector(b.max))
}
fn shape(s: &VoxelShape) -> String {
    if s.same_instance(&VoxelShape::block()) {
        "F".into()
    } else if s.same_instance(&VoxelShape::empty()) {
        "E".into()
    } else {
        format!(
            "S:{}",
            s.to_array()
                .iter()
                .map(|v| v.to_bits().to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
fn direction(d: Option<Direction>) -> &'static str {
    match d {
        None => "-",
        Some(Direction::Down) => "DOWN",
        Some(Direction::Up) => "UP",
        Some(Direction::North) => "NORTH",
        Some(Direction::South) => "SOUTH",
        Some(Direction::West) => "WEST",
        Some(Direction::East) => "EAST",
    }
}
fn directions(i: i32) -> Option<Direction> {
    if i < 0 {
        None
    } else {
        Some(
            [
                Direction::Down,
                Direction::Up,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ][i as usize],
        )
    }
}
fn check(c: DestinationCheck) -> &'static str {
    match c {
        DestinationCheck::Valid => "VALID",
        DestinationCheck::OutsideWorld => "OUTSIDE_WORLD",
        DestinationCheck::OutsideBorder => "OUTSIDE_BORDER",
        DestinationCheck::NotLoaded => "NOT_LOADED",
        DestinationCheck::NotStandable => "NOT_STANDABLE",
    }
}
type Events = Rc<RefCell<Vec<String>>>;
#[derive(Clone)]
struct Def {
    block: String,
    fluid: i32,
    open: i32,
    upper: i32,
    facing: i32,
    normal: i32,
    opened: i32,
    paired: i32,
    neighbor: i32,
}
#[derive(Default)]
struct Registry {
    states: HashMap<i32, Def>,
    blocks: HashMap<String, (i32, f32)>,
    shapes: HashMap<i32, VoxelShape>,
}
#[derive(Clone)]
struct S {
    id: i32,
    registry: Rc<RefCell<Registry>>,
    events: Events,
}
impl S {
    fn def(&self) -> Def {
        self.registry.borrow().states[&self.id].clone()
    }
    fn facts(&self) -> (i32, f32) {
        self.registry.borrow().blocks[&self.def().block]
    }
    fn air(&self) -> bool {
        matches!(self.def().block.as_str(), "air" | "void_air")
    }
}
impl TerrainState for S {
    fn same_instance(&self, o: &Self) -> bool {
        self.id == o.id
    }
    fn same_block(&self, o: &Self) -> bool {
        self.def().block == o.def().block
    }
    fn same_fluid(&self, o: &Self) -> bool {
        self.def().fluid == o.def().fluid
    }
    fn is_hazard(&self) -> bool {
        self.def().fluid == 2
            || self.facts().0 & 32 != 0
            || matches!(
                self.def().block.as_str(),
                "cobweb"
                    | "sweet_berry_bush"
                    | "powder_snow"
                    | "wither_rose"
                    | "magma_block"
                    | "campfire"
                    | "soul_campfire"
                    | "cactus"
            )
    }
    fn is_water(&self) -> bool {
        self.def().fluid == 1
    }
    fn has_liquid(&self) -> bool {
        self.def().fluid != 0
    }
    fn is_climbable(&self) -> bool {
        matches!(
            self.def().block.as_str(),
            "ladder"
                | "vine"
                | "weeping_vines"
                | "weeping_vines_plant"
                | "twisting_vines"
                | "twisting_vines_plant"
                | "cave_vines"
                | "cave_vines_plant"
        )
    }
    fn kind(&self, k: BlockKind) -> bool {
        let bit = match k {
            BlockKind::Door => 1,
            BlockKind::OpensByHand => 2,
            BlockKind::FenceGate => 4,
            BlockKind::Trapdoor => 8,
            BlockKind::Ladder => 16,
            BlockKind::IronTrapdoor => return self.def().block == "iron_trapdoor",
        };
        self.facts().0 & bit != 0
    }
    fn open(&self) -> Option<bool> {
        let n = self.def().open;
        (n >= 0).then_some(n == 1)
    }
    fn with_open(&self, open: bool) -> Self {
        let def = self.def();
        if def.open >= 0 && (def.open == 1) != open && def.paired >= 0 {
            Self {
                id: def.paired,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }
    fn is_upper_half(&self) -> bool {
        self.def().upper == 1
    }
    fn horizontal_facing(&self) -> Option<Direction> {
        directions(self.def().facing)
    }
    fn bounce_restitution(&self) -> f32 {
        self.facts().1
    }
    fn collision_shape<L: TerrainLevel<Self>>(&self, level: &L, pos: BlockPos) -> VoxelShape {
        self.events
            .borrow_mut()
            .push(format!("C:{}:{}", self.id, position(pos)));
        let def = self.def();
        let id = if def.neighbor >= 0 && !level.block_state(pos.offset(1, 0, 0)).air() {
            def.neighbor
        } else if def.open == 1 {
            def.opened
        } else {
            def.normal
        };
        self.registry.borrow().shapes[&id].clone()
    }
}
struct Data {
    min: i32,
    count: i32,
    border_min: f64,
    border_max: f64,
    legacy: bool,
    floor_y: i32,
    floor_state: i32,
    blocks: HashMap<BlockPos, i32>,
    unknown: HashSet<BlockPos>,
}
#[derive(Clone)]
struct World {
    data: Rc<RefCell<Data>>,
    registry: Rc<RefCell<Registry>>,
    events: Events,
}
impl World {
    fn state_id(&self, id: i32) -> S {
        S {
            id,
            registry: self.registry.clone(),
            events: self.events.clone(),
        }
    }
}
impl TerrainLevel<S> for World {
    fn block_state(&self, pos: BlockPos) -> S {
        self.events
            .borrow_mut()
            .push(format!("B:{}", position(pos)));
        let data = self.data.borrow();
        self.state_id(*data.blocks.get(&pos).unwrap_or(if pos.y == data.floor_y {
            &data.floor_state
        } else {
            &0
        }))
    }
    fn void_air(&self) -> S {
        self.state_id(2)
    }
    fn is_known(&self, pos: BlockPos) -> bool {
        self.events
            .borrow_mut()
            .push(format!("K:{}", position(pos)));
        let data = self.data.borrow();
        pos.y >= data.min && pos.y <= self.max_y() && !data.unknown.contains(&pos)
    }
    fn is_inside_border(&self, x: f64, z: f64) -> bool {
        self.events
            .borrow_mut()
            .push(format!("D:{},{}", x.to_bits(), z.to_bits()));
        let data = self.data.borrow();
        x >= data.border_min && x <= data.border_max && z >= data.border_min && z <= data.border_max
    }
    fn min_y(&self) -> i32 {
        self.data.borrow().min
    }
    fn height(&self) -> i32 {
        self.data.borrow().count
    }
    fn axis_step_order(&self, m: Vec3) -> [Axis; 3] {
        if !self.data.borrow().legacy && m.x.abs() < m.z.abs() {
            [Axis::Y, Axis::Z, Axis::X]
        } else {
            [Axis::Y, Axis::X, Axis::Z]
        }
    }
}
#[test]
fn unchanged_agent_terrain_shapes_stands_opening_and_invalidation() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let commands = std::fs::read_to_string(root.join("commands.txt")).unwrap();
    let expected = std::fs::read_to_string(root.join("results.txt")).unwrap();
    assert_eq!(commands.lines().count(), expected.lines().count());
    let registry = Rc::new(RefCell::new(Registry::default()));
    let events: Events = Rc::new(RefCell::new(Vec::new()));
    let mut terrain: Option<Terrain<S, World>> = None;
    let mut results = Vec::new();
    let mut mismatches = Vec::new();
    for (index, (command, expected)) in commands.lines().zip(expected.lines()).enumerate() {
        let a: Vec<_> = command.split(' ').collect();
        let mut result = "-".to_string();
        match a[0] {
            "shapeDef" => {
                let id = a[1].parse().unwrap();
                let s = match a[2] {
                    "F" => VoxelShape::block(),
                    "E" => VoxelShape::empty(),
                    "-" => VoxelShape::new(Vec::new()),
                    raw => VoxelShape::new(values(raw).as_chunks::<6>().0.to_vec()),
                };
                registry.borrow_mut().shapes.insert(id, s);
            }
            "stateDef" => {
                let id = a[1].parse().unwrap();
                let block = a[2].to_string();
                let def = Def {
                    block: block.clone(),
                    fluid: a[4].parse().unwrap(),
                    open: a[5].parse().unwrap(),
                    upper: a[6].parse().unwrap(),
                    facing: a[7].parse().unwrap(),
                    normal: a[9].parse().unwrap(),
                    opened: a[10].parse().unwrap(),
                    paired: a[11].parse().unwrap(),
                    neighbor: a[12].parse().unwrap(),
                };
                let mut r = registry.borrow_mut();
                r.states.insert(id, def);
                r.blocks.insert(
                    block,
                    (a[3].parse().unwrap(), f32::from_bits(a[8].parse().unwrap())),
                );
            }
            "new" => {
                let world = World {
                    registry: registry.clone(),
                    events: events.clone(),
                    data: Rc::new(RefCell::new(Data {
                        min: a[4].parse().unwrap(),
                        count: a[5].parse().unwrap(),
                        border_min: d(a[6]),
                        border_max: d(a[7]),
                        legacy: a[8].parse().unwrap(),
                        floor_y: a[9].parse().unwrap(),
                        floor_state: a[10].parse().unwrap(),
                        blocks: HashMap::new(),
                        unknown: HashSet::new(),
                    })),
                };
                terrain = Some(Terrain::new(world, d(a[2]), d(a[3]), a[1].parse().unwrap()));
            }
            _ => {
                let t = terrain.as_mut().unwrap();
                match a[0] {
                    "set" => {
                        t.level()
                            .data
                            .borrow_mut()
                            .blocks
                            .insert(p(a[1]), a[2].parse().unwrap());
                    }
                    "known" => {
                        if a[2] == "true" {
                            t.level().data.borrow_mut().unknown.remove(&p(a[1]));
                        } else {
                            t.level().data.borrow_mut().unknown.insert(p(a[1]));
                        }
                    }
                    "border" => {
                        let mut data = t.level().data.borrow_mut();
                        data.border_min = d(a[1]);
                        data.border_max = d(a[2]);
                    }
                    "state" => result = t.state(p(a[1])).id.to_string(),
                    "shape" => result = shape(&t.shape(p(a[1]))),
                    "flags" => {
                        let q = p(a[1]);
                        result = format!(
                            "{},{},{},{},{},{}",
                            t.is_known(q),
                            t.is_passable(q),
                            t.is_hazard(q),
                            t.is_water(q),
                            t.is_climbable(q),
                            direction(t.climb_side(q))
                        );
                    }
                    "stand" => {
                        let q = p(a[1]);
                        result = [
                            t.stand_height(q).to_bits().to_string(),
                            t.crouch_height(q).to_bits().to_string(),
                            t.is_standable(q).to_string(),
                            t.feet(q).map_or_else(|| "-".into(), vector),
                            check(t.check(q)).into(),
                            t.is_bowl(q).to_string(),
                            t.is_open(q).to_string(),
                            t.has_room(q, t.height()).to_string(),
                            t.has_room(q, Terrain::<S, World>::CROUCH_HEIGHT)
                                .to_string(),
                        ]
                        .join("|");
                    }
                    "box" => result = bbox(t.body_box_height(v(a[1]), d(a[2]))),
                    "clear" => {
                        result =
                            format!("{}|{}", t.is_clear(b(a[1])), t.contains_any_liquid(b(a[1])))
                    }
                    "sweep" => result = t.sweep(v(a[1]), v(a[2])).to_string(),
                    "support" => result = t.is_supported(v(a[1])).to_string(),
                    "bounce" => {
                        let q = p(a[1]);
                        result = t.restitution(q.x, d(a[2]), q.z).to_bits().to_string();
                    }
                    "closed" => {
                        let mut opens = if a[3] == "-" {
                            Vec::new()
                        } else {
                            a[3].split(';').map(p).collect()
                        };
                        t.closed_in(b(a[1]), b(a[2]), &mut opens);
                        result = opens
                            .into_iter()
                            .map(position)
                            .collect::<Vec<_>>()
                            .join(";");
                    }
                    "invalidate" | "change" => {
                        let q = p(a[1]);
                        let id = a[2].parse().unwrap();
                        let state = t.level().state_id(id);
                        if a[0] == "change" {
                            t.level().data.borrow_mut().blocks.insert(q, id);
                        }
                        result = t.invalidate(q, &state).to_string();
                    }
                    "limits" => {
                        result = format!(
                            "{},{},{},{},{}",
                            t.width().to_bits(),
                            t.height().to_bits(),
                            t.can_open_doors(),
                            t.min_y(),
                            t.max_y()
                        )
                    }
                    _ => panic!("{command}"),
                }
            }
        }
        result.push('|');
        result.push_str(&events.borrow().join("/"));
        events.borrow_mut().clear();
        if result != expected && mismatches.len() < 8 {
            mismatches.push(format!(
                "case {index}: {command}\nactual: {result}\nexpected: {expected}"
            ));
        }
        results.push(result);
    }
    std::fs::write(root.join("rust-results.txt"), results.join("\n") + "\n").unwrap();
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
