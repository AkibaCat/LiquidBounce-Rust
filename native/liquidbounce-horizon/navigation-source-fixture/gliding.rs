use super::*;
use crate::navigation::{
    background::{BackgroundLevel, Executor},
    destination::Destination,
    flight::FlyRoute,
    geometry::BlockRegion,
    replanner::{ReplannerLevel, SearchGoal},
    route_storage::shared_destination,
    search::{Failure, Limits},
    staged_search::SharedLevel,
};
use std::collections::VecDeque;
fn vecs(v: &[Vec3]) -> String {
    v.iter().copied().map(vector).collect::<Vec<_>>().join(";")
}
#[derive(Clone)]
struct Environment {
    ceiling: bool,
    height_mode: i32,
    present: bool,
}
#[derive(Clone)]
struct LiveWorld {
    world: World,
    environment: Arc<Mutex<Environment>>,
}
impl TerrainLevel<S> for LiveWorld {
    fn block_state(&self, p: BlockPos) -> S {
        self.world.block_state(p)
    }
    fn void_air(&self) -> S {
        self.world.void_air()
    }
    fn is_known(&self, p: BlockPos) -> bool {
        let data = self.world.data.lock().unwrap();
        if p.y < data.min || p.y > data.min + data.count - 1 {
            return false;
        }
        self.world
            .events
            .lock()
            .unwrap()
            .push(format!("H:{},{}", p.x, p.z));
        !data.unknown.contains(&BlockPos::new(p.x, 0, p.z))
    }
    fn is_inside_border(&self, x: f64, z: f64) -> bool {
        self.world.is_inside_border(x, z)
    }
    fn min_y(&self) -> i32 {
        self.world.min_y()
    }
    fn height(&self) -> i32 {
        self.world.height()
    }
}
impl GlideLevel<S> for LiveWorld {
    fn has_ceiling(&self) -> bool {
        self.world.events.lock().unwrap().push("E".into());
        self.environment.lock().unwrap().ceiling
    }
    fn has_chunk_at(&self, p: BlockPos) -> bool {
        self.world
            .events
            .lock()
            .unwrap()
            .push(format!("J:{}", position(p)));
        !self
            .world
            .data
            .lock()
            .unwrap()
            .unknown
            .contains(&BlockPos::new(p.x, 0, p.z))
    }
    fn has_column(&self, x: i32, z: i32) -> bool {
        self.world.events.lock().unwrap().push(format!("H:{x},{z}"));
        !self
            .world
            .data
            .lock()
            .unwrap()
            .unknown
            .contains(&BlockPos::new(x, 0, z))
    }
    fn column_height(&self, x: i32, z: i32) -> i32 {
        self.world.events.lock().unwrap().push(format!("Z:{x},{z}"));
        match self.environment.lock().unwrap().height_mode {
            0 => 0,
            1 => 120,
            _ => {
                if x < 0 {
                    20
                } else {
                    0
                }
            }
        }
    }
}
#[derive(Clone)]
struct Stack {
    id: i32,
    glide: bool,
    damageable: bool,
    maximum: i32,
    damage: i32,
    rocket: bool,
    count: i32,
    flight: Option<i32>,
    events: Events,
}
impl Stack {
    fn log(&self, s: &str) {
        self.events.lock().unwrap().push(format!("{}:{s}", self.id));
    }
}
impl GlideStack for Stack {
    fn can_glide_chest(&self) -> bool {
        self.log("glide:CHEST");
        self.glide
    }
    fn is_damageable(&self) -> bool {
        self.log("damageable");
        self.damageable
    }
    fn max_damage(&self) -> i32 {
        self.log("max");
        self.maximum
    }
    fn damage(&self) -> i32 {
        self.log("damage");
        self.damage
    }
    fn is_firework_rocket(&self) -> bool {
        self.log("item");
        self.rocket
    }
    fn count(&self) -> i32 {
        self.log("count");
        self.count
    }
    fn firework_flight_duration(&self) -> Option<i32> {
        self.log("flight");
        self.flight
    }
}
#[derive(Clone)]
struct Inventory {
    stacks: Arc<Mutex<HashMap<i32, Stack>>>,
    slots: [i32; 9],
    events: Events,
}
impl GlideInventory for Inventory {
    type Stack = Stack;
    fn item(&self, slot: i32) -> Stack {
        self.events.lock().unwrap().push(format!("I:{slot}"));
        self.stacks.lock().unwrap()[&self.slots[slot as usize]].clone()
    }
}
#[derive(Clone)]
struct Actor {
    position: Vec3,
    motion: Vec3,
    bounds: Aabb,
    ground: bool,
    flying: bool,
    horizontal: bool,
    vertical: bool,
    below: bool,
    inventory: Inventory,
    chest: i32,
    offhand: i32,
    events: Events,
}
impl Actor {
    fn log(&self, s: &str) {
        self.events.lock().unwrap().push(s.into());
    }
}
impl EquippedGlidePlayer for Actor {
    type Stack = Stack;
    type Inventory = Inventory;
    fn inventory(&self) -> &Inventory {
        self.log("i");
        &self.inventory
    }
    fn offhand_item(&self) -> Stack {
        self.log("o");
        self.inventory.stacks.lock().unwrap()[&self.offhand].clone()
    }
    fn chest_item(&self) -> Stack {
        self.log("c");
        self.inventory.stacks.lock().unwrap()[&self.chest].clone()
    }
}
impl ContactPlayer for Actor {
    fn is_fall_flying(&self) -> bool {
        self.log("f");
        self.flying
    }
    fn horizontal_collision(&self) -> bool {
        self.log("h");
        self.horizontal
    }
    fn vertical_collision(&self) -> bool {
        self.log("v");
        self.vertical
    }
    fn vertical_collision_below(&self) -> bool {
        self.log("b");
        self.below
    }
    fn bounding_box(&self) -> Aabb {
        self.log("A");
        self.bounds
    }
}
impl GlidePlayer for Actor {
    fn position(&self) -> Vec3 {
        self.log("p");
        self.position
    }
    fn on_ground(&self) -> bool {
        self.log("g");
        self.ground
    }
    fn delta_movement(&self) -> Vec3 {
        self.log("d");
        self.motion
    }
    fn block_position(&self) -> BlockPos {
        self.log("q");
        BlockPos::containing(self.position)
    }
}
struct Body {
    actor: Actor,
    world: LiveWorld,
    clearance: f64,
    safe_fall: i32,
    can_elytra: bool,
    use_rockets: bool,
    hashes: [i32; 6],
}
impl GlideBody for Body {
    type Player = Actor;
    type State = S;
    type Level = LiveWorld;
    fn player(&self) -> &Actor {
        self.actor.log("P");
        &self.actor
    }
    fn level(&self) -> LiveWorld {
        self.actor.log("L");
        self.world.clone()
    }
    fn glide_clearance(&self) -> f64 {
        self.actor.log("O");
        self.clearance
    }
    fn safe_fall(&self) -> i32 {
        self.actor.log("S");
        self.safe_fall
    }
    fn direction_hash(&self, d: Direction) -> i32 {
        self.hashes[d as usize]
    }
    fn press(&mut self, i: Input, exact: bool) {
        self.actor.log(&format!("K:{}:{exact}", i.java_hash()))
    }
    fn look(&mut self, y: f32, x: f32, purpose: GlideLook) {
        self.actor.log(&format!(
            "V:{},{}:{}",
            y.to_bits(),
            x.to_bits(),
            match purpose {
                GlideLook::Movement => "MOVEMENT",
                GlideLook::Steer => "STEER",
            }
        ))
    }
    fn select_hotbar_slot(&mut self, slot: i32) {
        self.actor.log(&format!("B:{slot}"))
    }
    fn release_hotbar_slot(&mut self) {
        self.actor.log("R")
    }
    fn use_item(&mut self, hand: GlideHand) {
        self.actor.log(match hand {
            GlideHand::Main => "U:MAIN_HAND",
            GlideHand::Off => "U:OFF_HAND",
        })
    }
}
type WorkQueue = Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>;
#[derive(Clone, Default)]
struct Queue(WorkQueue);
impl Executor for Queue {
    fn execute(&self, t: Box<dyn FnOnce() + Send>) {
        self.0.lock().unwrap().push_back(t)
    }
}
impl Queue {
    fn run(&self) {
        let task = self.0.lock().unwrap().pop_front();
        if let Some(task) = task {
            task()
        }
    }
    fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }
}
type Snapshot = Terrain<S, SharedLevel<World>>;
struct BackgroundWorld(LiveWorld);
impl BackgroundLevel<Snapshot> for BackgroundWorld {
    fn snapshot(&mut self, r: BlockRegion) -> Option<Snapshot> {
        self.0.world.events.lock().unwrap().push("L".into());
        if !self.0.environment.lock().unwrap().present {
            return None;
        }
        self.0.world.events.lock().unwrap().push(format!(
            "N:{},{},{},{},{},{}",
            r.min.x, r.min.y, r.min.z, r.max.x, r.max.y, r.max.z
        ));
        let snapshot = World {
            data: Arc::new(Mutex::new(self.0.world.data.lock().unwrap().clone())),
            registry: self.0.world.registry.clone(),
            events: self.0.world.events.clone(),
        };
        Some(Terrain::player(SharedLevel::new(snapshot)))
    }
    fn has_level(&mut self) -> bool {
        self.0.world.events.lock().unwrap().push("L".into());
        self.0.environment.lock().unwrap().present
    }
    fn invalidate(&mut self, t: &mut Snapshot, p: BlockPos) -> bool {
        let state = self.0.block_state(p);
        t.invalidate(p, &state)
    }
}
impl ReplannerLevel<Snapshot> for BackgroundWorld {
    fn is_known(&mut self, p: BlockPos) -> bool {
        self.0.is_known(p)
    }
}
#[derive(Clone)]
struct Config {
    plan: Vec<String>,
    air: Vec<String>,
    points: Vec<Vec3>,
    rockets: i32,
    approach: f64,
    reaches: bool,
}
struct PlanSearch {
    terrain: Snapshot,
    start: Vec3,
    goal: Vec3,
    config: Config,
    step: usize,
    events: Events,
}
struct AirSearch {
    terrain: Snapshot,
    start: Vec3,
    config: Config,
    step: usize,
    events: Events,
}
fn outcome<R>(token: &str, found: impl FnOnce() -> R) -> Outcome<R> {
    match token.as_bytes()[0] {
        b'S' => Outcome::Searching,
        b'F' => Outcome::Found(found()),
        b'A' => Outcome::AlreadyThere,
        b'E' => Outcome::Failed(match &token[1..] {
            "NO_PATH" => Failure::NoPath,
            "TOO_FAR" => Failure::TooFar,
            "TIMED_OUT" => Failure::TimedOut,
            _ => panic!("token"),
        }),
        _ => panic!("controlled search failure"),
    }
}
impl OwnedTerrainSearch<Snapshot> for PlanSearch {
    type Route = Route;
    fn terrain_mut(&mut self) -> &mut Snapshot {
        &mut self.terrain
    }
    fn advance(&mut self) -> Outcome<Route> {
        let t = &self.config.plan[self.step.min(self.config.plan.len() - 1)];
        self.events
            .lock()
            .unwrap()
            .push(format!("A:G:{}", self.step));
        self.step += 1;
        outcome(t, || Route {
            start: self.start,
            points: self.config.points.clone(),
            landing: self.goal,
            rockets: self.config.rockets,
            approach: self.config.approach,
        })
    }
    fn preview(&mut self) -> Option<Vec<Vec3>> {
        self.events
            .lock()
            .unwrap()
            .push(format!("Q:G:{}", self.step));
        (self.config.plan[self.step.saturating_sub(1).min(self.config.plan.len() - 1)] == "S1")
            .then(|| vec![Vec3::new(self.step as f64, 0.0, 0.0)])
    }
}
impl OwnedTerrainSearch<Snapshot> for AirSearch {
    type Route = FlyRoute;
    fn terrain_mut(&mut self) -> &mut Snapshot {
        &mut self.terrain
    }
    fn advance(&mut self) -> Outcome<FlyRoute> {
        let t = &self.config.air[self.step.min(self.config.air.len() - 1)];
        self.events
            .lock()
            .unwrap()
            .push(format!("A:A:{}", self.step));
        self.step += 1;
        outcome(t, || FlyRoute {
            start: self.start,
            points: self.config.points.clone(),
            can_reach: self.config.reaches,
        })
    }
    fn preview(&mut self) -> Option<Vec<Vec3>> {
        self.events
            .lock()
            .unwrap()
            .push(format!("Q:A:{}", self.step));
        (self.config.air[self.step.saturating_sub(1).min(self.config.air.len() - 1)] == "S1")
            .then(|| vec![Vec3::new(self.step as f64, 0.0, 0.0)])
    }
}
struct Report(Events);
impl ContactListener for Report {
    fn touched(&mut self, p: BlockPos, d: Direction) {
        self.0
            .lock()
            .unwrap()
            .push(format!("T:{}:{}", position(p), direction(Some(d))))
    }
}
impl GlideReport for Report {
    fn blocks_empty(&mut self) {
        self.0.lock().unwrap().push("R:B:0,0".into())
    }
    fn following(&mut self, points: &[Vec3], landing: Vec3) {
        self.0
            .lock()
            .unwrap()
            .push(format!("R:F:{}:true:{}", vecs(points), vector(landing)))
    }
}
type Controller = Gliding<Snapshot, PlanSearch, AirSearch, S, LiveWorld>;
struct Runtime {
    gliding: Controller,
    body: Body,
    queue: Queue,
    config: Arc<Mutex<Config>>,
    background: BackgroundWorld,
}
impl Runtime {
    fn state(&self) -> String {
        let g = &self.gliding;
        let state = g.glide.as_ref().map_or_else(
            || "-".into(),
            |glide| {
                let (name, target, rockets, passage) = match &glide.steering {
                    Control::Landing(l) => ("Landing", l.target(), &l.rockets, "-".into()),
                    Control::Cruise(c) => ("Cruise", c.target(), c.rockets(), "-".into()),
                    Control::Passage(p) => (
                        "Passage",
                        p.target(),
                        &p.rockets,
                        format!(
                            "{}~{}~{}~{}",
                            p.lands_at_target,
                            p.drift().to_bits(),
                            p.remaining().to_bits(),
                            vecs(p.route())
                        ),
                    ),
                };
                format!(
                    "{}^{}^{}^{}^{}^{name}^{}^{}^{}^{},{},{}^{passage}",
                    vector(glide.goal),
                    glide.ticks,
                    glide.has_opened,
                    glide.pressed_at,
                    glide.jumped,
                    vector(target),
                    vector(glide.landing()),
                    glide
                        .path
                        .as_ref()
                        .map_or_else(|| "-".into(), |p| vecs(&p.points)),
                    rockets.left(),
                    rockets.boost_ticks(),
                    rockets.can_fire()
                )
            },
        );
        let a = &g.airways;
        let r = &a.routes;
        format!(
            "{}|{}|{}|{}|{}|{}|{state}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            g.ticks,
            g.retry_at,
            g.should_fire_rocket,
            g.holds_rocket_slot,
            g.is_active(),
            g.sprint(),
            g.search.is_idle(),
            g.search.preview().map_or_else(|| "-".into(), vecs),
            a.has_route,
            a.reaches_goal,
            a.found_at,
            r.ticks(),
            r.failed(),
            r.ends_at_stage(),
            r.is_searching(),
            r.is_due(false),
            r.is_due(true),
            self.queue.len()
        )
    }
}
fn create_runtime(world: World, stacks: Arc<Mutex<HashMap<i32, Stack>>>) -> Runtime {
    let events = world.events.clone();
    let environment = Arc::new(Mutex::new(Environment {
        ceiling: false,
        height_mode: 0,
        present: true,
    }));
    let live = LiveWorld {
        world: world.clone(),
        environment,
    };
    let queue = Queue::default();
    let config = Arc::new(Mutex::new(Config {
        plan: vec!["F".into()],
        air: vec!["F".into()],
        points: vec![Vec3::new(0.5, 10.0, 0.5), Vec3::new(40.5, 0.0, 0.5)],
        rockets: 0,
        approach: 0.0,
        reaches: true,
    }));
    let plan_config = config.clone();
    let plan_events = events.clone();
    let plan = Arc::new(
        move |mut terrain: Snapshot, start, goal, radius: f64, abilities: GlideAbilities| {
            plan_events.lock().unwrap().push(format!(
                "F:G:{}:{}:{}:{},{},{}",
                vector(start),
                vector(goal),
                radius.to_bits(),
                abilities.rockets,
                abilities.rocket_ticks,
                abilities.glide_ticks
            ));
            terrain.state(BlockPos::default());
            PlanSearch {
                terrain,
                start,
                goal,
                config: plan_config.lock().unwrap().clone(),
                step: 0,
                events: plan_events.clone(),
            }
        },
    );
    let air_config = config.clone();
    let air_events = events.clone();
    let air = Arc::new(
        move |mut terrain: Snapshot, start, goal: SearchGoal, clearance: f64, limits: Limits| {
            let destination = shared_destination::<S, World>(goal, terrain.level().clone());
            let checked = [
                BlockPos::new(0, 0, 0),
                BlockPos::new(4, 0, 0),
                BlockPos::new(100, 0, 0),
            ]
            .into_iter()
            .map(|p| {
                format!(
                    "{},{}",
                    destination.is_goal(p),
                    destination.distance(p).to_bits()
                )
            })
            .collect::<Vec<_>>()
            .join(";");
            air_events.lock().unwrap().push(format!(
                "F:A:{}:{}:{}:{}:{}:{}:{}:{checked}",
                vector(start),
                goal.stage_range.is_some(),
                clearance.to_bits(),
                limits.max_length.to_bits(),
                limits.max_visited_nodes,
                limits.visited_nodes_per_slice,
                limits.max_compute_nanos
            ));
            terrain.state(BlockPos::default());
            AirSearch {
                terrain,
                start,
                config: air_config.lock().unwrap().clone(),
                step: 0,
                events: air_events.clone(),
            }
        },
    );
    let source = live.clone();
    let provider = Arc::new(move || {
        source.world.events.lock().unwrap().push("L".into());
        source.clone()
    });
    let gliding = Gliding::new(Arc::new(queue.clone()), plan, air, provider);
    let hashes: Vec<i32> = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("direction-hashes.txt"),
    )
    .unwrap()
    .split(',')
    .map(|v| v.parse().unwrap())
    .collect();
    Runtime {
        gliding,
        body: Body {
            actor: Actor {
                position: Vec3::ZERO,
                motion: Vec3::ZERO,
                bounds: Aabb::from_block(BlockPos::default()),
                ground: false,
                flying: false,
                horizontal: false,
                vertical: false,
                below: false,
                inventory: Inventory {
                    stacks: stacks.clone(),
                    slots: [0; 9],
                    events: events.clone(),
                },
                chest: 1,
                offhand: 0,
                events,
            },
            world: live.clone(),
            clearance: 0.0,
            safe_fall: 3,
            can_elytra: true,
            use_rockets: true,
            hashes: hashes.try_into().unwrap(),
        },
        queue,
        config,
        background: BackgroundWorld(live),
    }
}
fn extra(
    a: &[&str],
    terrain: &mut Terrain<S, World>,
    runtime: &mut Option<Runtime>,
    stacks: &Arc<Mutex<HashMap<i32, Stack>>>,
) -> String {
    let i = |s: &str| s.parse::<i32>().unwrap();
    let flag = |s: &str| s.parse::<bool>().unwrap();
    if a[0] == "runtime" {
        *runtime = Some(create_runtime(terrain.level().clone(), stacks.clone()));
        return "-".into();
    }
    if a[0] == "stack" {
        let id = i(a[1]);
        stacks.lock().unwrap().insert(
            id,
            Stack {
                id,
                glide: flag(a[2]),
                damageable: flag(a[3]),
                maximum: i(a[4]),
                damage: i(a[5]),
                rocket: flag(a[6]),
                count: i(a[7]),
                flight: (a[8] != "-").then(|| i(a[8])),
                events: terrain.level().events.clone(),
            },
        );
        return "-".into();
    }
    let r = runtime.as_mut().unwrap();
    match a[0] {
        "player" => {
            let p = &mut r.body.actor;
            p.position = v(a[1]);
            p.ground = flag(a[2]);
            p.flying = flag(a[3]);
            p.horizontal = flag(a[5]);
            p.vertical = flag(a[6]);
            p.below = flag(a[7]);
            p.bounds = b(a[8]);
            "-".into()
        }
        "equipment" => {
            r.body.actor.offhand = i(a[1]);
            r.body.actor.chest = i(a[2]);
            r.body.actor.inventory.slots = a[3]
                .split(',')
                .map(i)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            "-".into()
        }
        "options" => {
            r.body.can_elytra = flag(a[1]);
            r.body.use_rockets = flag(a[2]);
            r.body.clearance = d(a[3]);
            r.body.safe_fall = i(a[4]);
            "-".into()
        }
        "capture" => {
            r.body.actor.log("O");
            super::super::capture::capture_glide(r.body.can_elytra, r.body.use_rockets, || {
                r.body.actor.log("P");
                r.body.actor.clone()
            })
            .map_or_else(
                || "-".into(),
                |a| format!("{},{},{}", a.rockets, a.rocket_ticks, a.glide_ticks),
            )
        }
        "delta" => {
            r.body.actor.motion = v(a[1]);
            "-".into()
        }
        "environment" => {
            *r.body.world.environment.lock().unwrap() = Environment {
                ceiling: flag(a[1]),
                height_mode: i(a[2]),
                present: flag(a[3]),
            };
            "-".into()
        }
        "program" => {
            *r.config.lock().unwrap() = Config {
                plan: a[1].split(',').map(str::to_owned).collect(),
                air: a[2].split(',').map(str::to_owned).collect(),
                points: a[3].split(';').map(v).collect(),
                rockets: i(a[4]),
                approach: d(a[5]),
                reaches: flag(a[6]),
            };
            "-".into()
        }
        "nav" => {
            let abilities = (a[3] != "-").then(|| {
                let q: Vec<_> = a[3].split(',').map(i).collect();
                GlideAbilities {
                    rockets: q[0],
                    rocket_ticks: q[1],
                    glide_ticks: q[2],
                }
            });
            let result = r.gliding.navigate(
                &mut r.body,
                &mut r.background,
                &mut Report(terrain.level().events.clone()),
                v(a[1]),
                d(a[2]),
                abilities,
                flag(a[4]),
                f32::from_bits(a[5].parse().unwrap()),
            );
            format!("{result}#{}", r.state())
        }
        "glideHalt" => {
            r.gliding.halt(&mut r.body);
            r.state()
        }
        "run" => {
            r.queue.run();
            r.state()
        }
        "glideLevel" => {
            r.gliding.level_changed();
            r.state()
        }
        "glideChange" => {
            r.gliding.block_changed(p(a[1]));
            r.state()
        }
        _ => panic!("{a:?}"),
    }
}
