// SPDX-License-Identifier: GPL-3.0-or-later
//! Game-thread ownership and event routing for the original Agent navigator.
use super::*;
use super::{
    auto_walk_host::NativeHost,
    auto_walk_travel::{NativeTravel, WalkSearch},
    auto_walk_world::{
        self as world, NativeLevel, NativeState, NavigationRegistry, SearchLevel, SearchTerrain,
        SnapshotLevel,
    },
};
use crate::navigation as nav;
use nav::{
    auto_walk::{AutoWalk, AutoWalkHost, FollowPlayer, Mode},
    geometry::{BlockPos, Vec3 as NavVec},
    input::Input,
    navigator::Navigator,
    simulation::player_motion::MotionPlayer,
    terrain::{Terrain, TerrainLevel},
    walk_control::view_rotation::ViewRotation,
    walking_runtime::{StablePlacementOrder, WalkingRuntime},
};
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

type NativeNavigator =
    Navigator<NativeState, SearchTerrain, WalkSearch, StablePlacementOrder, NativeTravel>;
struct Runtime {
    epoch: u64,
    host: NativeHost,
    navigator: NativeNavigator,
    level: SearchLevel,
    module: AutoWalk,
    arrived: Option<BlockPos>,
}
#[derive(Clone, Copy)]
enum Event {
    Changed(u64, BlockPos),
    Confirmed(u64, BlockPos, u32),
    Corrected(u64),
}
thread_local! {
 static RUNTIME:RefCell<Option<Runtime>>=const{RefCell::new(None)};
 static CONFIG:RefCell<Value>=const{RefCell::new(Value::Null)};
 static RESULT:RefCell<Option<String>>=const{RefCell::new(None)};
 static EVENTS:RefCell<Vec<Event>>=const{RefCell::new(Vec::new())};
 static KEEP_DESTROYING:Cell<bool>=const{Cell::new(false)};
 static KEEP_USING:Cell<bool>=const{Cell::new(false)};
 static RETIRED:Cell<bool>=const{Cell::new(false)};
}
pub(super) fn keeps_destroying() -> bool {
    KEEP_DESTROYING.with(Cell::get)
}
pub(super) fn keep_destroying(value: bool) {
    KEEP_DESTROYING.with(|s| s.set(value));
}
pub(super) fn keeps_using() -> bool {
    KEEP_USING.with(Cell::get)
}
pub(super) fn keep_using(value: bool) {
    KEEP_USING.with(|s| s.set(value));
}
pub(super) fn changed(epoch: u64, pos: [i32; 3]) {
    EVENTS.with(|s| {
        s.borrow_mut()
            .push(Event::Changed(epoch, BlockPos::new(pos[0], pos[1], pos[2])))
    });
}
pub(super) fn confirmed(epoch: u64, pos: [i32; 3], state: u32) {
    EVENTS.with(|s| {
        s.borrow_mut().push(Event::Confirmed(
            epoch,
            BlockPos::new(pos[0], pos[1], pos[2]),
            state,
        ))
    });
}
pub(super) fn corrected(epoch: u64) {
    EVENTS.with(|s| s.borrow_mut().push(Event::Corrected(epoch)));
}
pub(super) fn level_changed() {
    // Retire handles without touching an old hotbar or player after replacement.
    RETIRED.with(|s| s.set(true));
    EVENTS.with(|s| s.borrow_mut().clear());
    RESULT.with(|s| s.borrow_mut().take());
    keep_destroying(false);
    keep_using(false);
    horizon_client::navigation_physics::set_safe_walk(false);
}
fn key_input(v: &Value) -> Input {
    let key = |i| v.get(i).and_then(Value::as_bool).unwrap_or(false);
    Input {
        forward: key(0),
        backward: key(1),
        left: key(2),
        right: key(3),
        jump: key(4),
        shift: key(5),
        sprint: key(6),
    }
}
fn keys(v: Input) -> [bool; 7] {
    [
        v.forward, v.backward, v.left, v.right, v.jump, v.shift, v.sprint,
    ]
}
fn point(v: NavVec) -> [f64; 3] {
    [v.x, v.y, v.z]
}
fn pos(v: BlockPos) -> [i32; 3] {
    [v.x, v.y, v.z]
}
fn boolean(v: &Value, key: &str, default: bool) -> bool {
    v[key].as_bool().unwrap_or(default)
}
fn number(v: &Value, key: &str, default: f64) -> f64 {
    v[key].as_f64().filter(|v| v.is_finite()).unwrap_or(default)
}
impl Runtime {
    fn new(epoch: u64) -> Self {
        let registry = world::read(NavigationRegistry::capture);
        let host = NativeHost::new(registry.clone(), epoch);
        let live = NativeLevel::new(registry, epoch);
        let executor: Arc<dyn nav::background::Executor> =
            Arc::new(nav::background::ThreadExecutor);
        let factory = Arc::new(
            |terrain: SearchTerrain, start, descriptor, abilities, limits| {
                let goal = nav::route_storage::shared_destination::<NativeState, SnapshotLevel>(
                    descriptor,
                    terrain.level().clone(),
                );
                nav::walk::route_search::WalkRouteSearch::new(
                    terrain, start, goal, abilities, limits,
                )
            },
        );
        let walking =
            WalkingRuntime::new(executor.clone(), factory, StablePlacementOrder::default());
        let travel = NativeTravel::new(executor, live.clone(), host.route.clone());
        Self {
            epoch,
            host,
            navigator: Navigator::new(walking, travel),
            level: SearchLevel {
                live,
                width: Terrain::<NativeState, NativeLevel>::PLAYER_WIDTH,
                height: Terrain::<NativeState, NativeLevel>::PLAYER_HEIGHT,
                can_open_doors: true,
            },
            module: AutoWalk::default(),
            arrived: None,
        }
    }
    fn configure(&mut self, value: &Value) {
        let h = &mut self.host;
        h.input = None;
        h.requested = None;
        h.exact = false;
        self.arrived = None;
        h.input_yaw = number(value, "inputYaw", f64::from(h.input_yaw)) as f32;
        h.movement_yaw = number(value, "movementYaw", f64::from(h.input_yaw)) as f32;
        h.sent = ViewRotation::new(
            value["sent"][0].as_f64().unwrap_or(f64::from(h.sent.y_rot)) as f32,
            value["sent"][1].as_f64().unwrap_or(f64::from(h.sent.x_rot)) as f32,
        );
        let camera = world::read(|g| [g.yaw, g.pitch]);
        h.current = ViewRotation::new(
            value["current"][0].as_f64().map_or(camera[0], |v| v as f32),
            value["current"][1].as_f64().map_or(camera[1], |v| v as f32),
        );
        h.external_rotation = boolean(value, "externalRotation", false);
        h.mouse_turning = boolean(value, "mouseTurning", false);
        h.is_flying = boolean(value, "isFlying", false);
        h.recasts_elytra = boolean(value, "recastsElytra", false);
        let o = &value["options"];
        h.options.can_break = boolean(o, "allowBreak", false);
        let scaffold = boolean(value, "scaffold", false);
        h.options.can_place = boolean(o, "allowPlace", false) || scaffold;
        h.options.fast_bridges = scaffold;
        h.options.places_bridge_floors = scaffold;
        h.options.can_parkour = boolean(o, "allowParkour", true);
        h.options.risky_jumps = boolean(o, "riskyJumps", false);
        h.options.max_fall = number(o, "maxFall", 3.) as i32;
        h.options.path_range = number(o, "pathRange", 64.) as i32;
        h.options.can_fly = boolean(o, "allowFly", true);
        h.options.can_elytra = boolean(o, "allowElytra", true);
        h.options.elytra_rockets = boolean(o, "elytraRockets", true);
        h.options.can_boat = boolean(o, "allowBoat", false);
        h.options.can_eat = boolean(o, "allowEat", false);
        h.options.look_ahead = false;
        if let Some(v) = value["destination"].as_array().filter(|v| v.len() == 3) {
            self.module.destination = BlockPos::new(
                v[0].as_i64().unwrap_or(0) as i32,
                v[1].as_i64().unwrap_or(0) as i32,
                v[2].as_i64().unwrap_or(0) as i32,
            );
        }
        self.module.target = value["target"].as_str().unwrap_or("").to_owned();
        self.module.follow_distance = number(value, "distance", 3.) as f32;
    }
    fn tick(&mut self, value: &Value, input: Input) -> Value {
        self.configure(value);
        for event in EVENTS.with(|s| std::mem::take(&mut *s.borrow_mut())) {
            match event {
                Event::Changed(epoch, p) if epoch == self.epoch => {
                    self.navigator.block_changed::<NativeHost>(p)
                }
                Event::Confirmed(epoch, p, id) if epoch == self.epoch => {
                    let state = self.level.live.registry.state(BlockStateId(id));
                    self.navigator.block_confirmed(&mut self.host, p, &state);
                }
                Event::Corrected(epoch) if epoch == self.epoch => {
                    self.navigator.position_corrected(&self.host)
                }
                _ => {}
            }
        }
        let mode = match value["mode"].as_str() {
            Some("Direction") => Mode::Direction,
            Some("Goto") => Mode::Goto,
            Some("Follow") => Mode::Follow,
            _ => Mode::Forward,
        };
        let running = boolean(value, "running", false);
        let enabled = boolean(value, "enabled", false);
        let mut input = input;
        {
            let mut body = Body {
                host: &mut self.host,
                navigator: &mut self.navigator,
                level: &mut self.level,
                arrived: &mut self.arrived,
            };
            self.module.select_mode(mode, running, &mut body);
            self.module.set_enabled(enabled, true, &mut body);
            if enabled && running {
                self.module.movement_input(&mut input, &mut body);
            } else {
                body.navigator.halt(body.host);
                body.host.release_all();
            }
        }
        let wants_sprint = self.host.input.is_some_and(|p| p.sprint && p.is_moving());
        let mut pressed = self.host.input.unwrap_or(input);
        pressed.sprint = input.sprint;
        let request = self
            .host
            .requested
            .map(|(r, p)| json!({"rotation":[r.y_rot,r.x_rot],"purpose":format!("{p:?}")}));
        let route = self.host.route.borrow();
        let (head, ahead) = match route.progress {
            Some(nav::course::ProgressOwner::Walking) => (
                self.navigator.walking.follower.head(&self.host, 1.),
                self.navigator.walking.follower.ahead().to_vec(),
            ),
            Some(nav::course::ProgressOwner::Straight) => (
                self.host.player.position(),
                self.navigator.course.straight_goal.into_iter().collect(),
            ),
            Some(owner) => self.navigator.course.travel.progress(owner, &self.host, 1.),
            None => (self.host.player.position(), Vec::new()),
        };
        let previous_head = match route.progress {
            Some(nav::course::ProgressOwner::Walking) => {
                self.navigator.walking.follower.head(&self.host, 0.)
            }
            Some(nav::course::ProgressOwner::Straight) => self.host.player.interpolated(0.),
            Some(owner) => {
                self.navigator
                    .course
                    .travel
                    .progress(owner, &self.host, 0.)
                    .0
            }
            None => self.host.player.interpolated(0.),
        };
        json!({"input":keys(pressed),"exact":self.host.exact,"enabled":self.module.enabled(),"running":running,"request":request,"arrived":self.arrived.map(pos),"wantsSprint":wants_sprint,"usingItem":world::read(|g|g.using.is_some()),"route":{"points":route.points.iter().copied().map(point).collect::<Vec<_>>(),"head":point(head),"previousHead":point(previous_head),"ahead":ahead.into_iter().map(point).collect::<Vec<_>>(),"goal":route.destination.or(self.navigator.course.last_goal).map(point),"start":route.start.map(point),"breaks":route.breaks.iter().copied().map(pos).collect::<Vec<_>>(),"places":route.places.iter().copied().map(pos).collect::<Vec<_>>(),"preview":route.preview,"state":route.state,"since":route.since}})
    }
}
struct Body<'a> {
    host: &'a mut NativeHost,
    navigator: &'a mut NativeNavigator,
    level: &'a mut SearchLevel,
    arrived: &'a mut Option<BlockPos>,
}
impl AutoWalkHost for Body<'_> {
    fn position(&self) -> NavVec {
        self.host.player.position()
    }
    fn yaw(&self) -> f32 {
        world::read(|g| g.yaw)
    }
    fn root_vehicle_position(&self) -> NavVec {
        self.host.root_position()
    }
    fn direction_from_rotation(&self, yaw: f32) -> NavVec {
        ViewRotation::new(yaw, 0.).view_vector()
    }
    fn is_standable(&mut self, pos: BlockPos) -> bool {
        Terrain::player(self.host.level()).is_standable(pos)
    }
    fn has_chunk_at(&self, pos: BlockPos) -> bool {
        self.level
            .live
            .game(|g| g.world.chunks.contains_key(&(pos.x >> 4, pos.z >> 4)))
    }
    fn players(&self) -> Vec<FollowPlayer> {
        world::read(|g| {
            g.remote_player_names()
                .into_iter()
                .filter_map(|(id, name)| {
                    g.mobs.iter().find(|m| m.id == id).map(|m| FollowPlayer {
                        is_local_player: false,
                        profile_name: name,
                        position: world::vec(m.position),
                    })
                })
                .collect()
        })
    }
    fn navigate(&mut self, input: &mut Input, goal: NavVec, radius: f64) {
        self.navigator.navigate(self.host, self.level, goal, radius);
        if let Some(pressed) = self.host.input {
            let sprint = input.sprint;
            *input = pressed;
            input.sprint = sprint;
        }
    }
    fn halt(&mut self) {
        self.navigator.halt(self.host);
        self.host.release_all();
    }
    fn reset(&mut self) {
        self.navigator.stop(self.host);
        self.host.release_all();
    }
    fn arrived(&mut self, destination: BlockPos) {
        *self.arrived = Some(destination);
    }
}
impl Game<'_> {
    pub(super) fn auto_walk_set_config(&mut self, value: &str) {
        if let Ok(value) = serde_json::from_str(value) {
            CONFIG.with(|s| *s.borrow_mut() = value);
        }
    }
    pub(super) fn auto_walk_get_result(&self) -> Option<String> {
        RESULT.with(|s| s.borrow().clone())
    }
    pub(super) fn native_client_auto_walk_input(
        &mut self,
        raw: [bool; 7],
    ) -> Option<([bool; 7], bool)> {
        if !self.native_window()
            || self.lan_acting.is_some()
            || self.lan_acting_uuid.is_some()
            || !self
                .player
                .as_ref()
                .is_some_and(|p| p.native_client_queries_allowed())
        {
            return None;
        }
        let config = CONFIG.with(|s| s.borrow().clone());
        if config.is_null() {
            return None;
        }
        let epoch = world::current_epoch(self);
        let input = key_input(&json!(raw));
        let retired = RETIRED.with(|s| s.replace(false));
        let result = world::with_scope(self, || {
            RUNTIME.with(|slot| {
                let mut slot = slot.borrow_mut();
                if retired || slot.as_ref().is_none_or(|r| r.epoch != epoch) {
                    *slot = Some(Runtime::new(epoch));
                }
                slot.as_mut()
                    .expect("navigation runtime")
                    .tick(&config, input)
            })
        });
        let input = keys(key_input(&result["input"]));
        let exact = result["exact"].as_bool().unwrap_or(false);
        RESULT.with(|s| *s.borrow_mut() = Some(result.to_string()));
        Some((input, exact))
    }
}
