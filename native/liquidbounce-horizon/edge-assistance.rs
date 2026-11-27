// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleParkour and ModuleSafeWalk over the real scoped 26.3 body.
use super::*;
use super::{
    auto_walk_player::NativePlayer,
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NativeState, NavigationRegistry},
};
use crate::navigation as nav;
use nav::{
    aabb::Aabb as Box3,
    geometry::{BlockPos as Pos, Vec3 as V},
    input::Input,
    simulation::player_motion::PlayerMotion as Motion,
    terrain::{Terrain, TerrainLevel, TerrainState},
    walk_control::body::{SimulatedInput, WalkPlayer, WalkSimulation, capture_motion},
    walk_jump::{
        assisted_jump::AssistedJump, jump_targets::JumpTargets, jumps::Jumps, leap::LeapInput,
        vector_yaw as view_yaw, wrap_degrees,
    },
};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::HashSet, sync::Arc};

thread_local! {
 static CONFIG:RefCell<Value>=const{RefCell::new(Value::Null)};
 static RESULT:RefCell<Option<String>>=const{RefCell::new(None)};
 static STATE:RefCell<Option<State>>=const{RefCell::new(None)};
}
struct State {
    epoch: u64,
    player: NativePlayer,
    assisted: AssistedJump,
    aim: f32,
    parkour_running: bool,
    safe_mode: String,
    center: Option<V>,
    overwrite: i32,
    sneak: i32,
    distance: f32,
}
fn float(v: &Value, default: f32) -> f32 {
    v.as_f64()
        .filter(|n| n.is_finite())
        .map_or(default, |n| n as f32)
}
fn random_range(v: &Value, default: [i32; 2]) -> i32 {
    let a = v[0].as_i64().unwrap_or(i64::from(default[0])) as i32;
    let b = v[1].as_i64().unwrap_or(i64::from(default[1])) as i32;
    super::edge_random::random_i32_inclusive(a, b)
}
fn random_distance(v: &Value) -> f32 {
    let a = float(&v[0], 0.1);
    let b = float(&v[1], 0.15);
    a + (b - a) * super::edge_random::random_unit_f32()
}
fn input(k: [bool; 7]) -> Input {
    Input {
        forward: k[0],
        backward: k[1],
        left: k[2],
        right: k[3],
        jump: k[4],
        shift: k[5],
        sprint: k[6],
    }
}
fn keys(k: Input) -> [bool; 7] {
    [
        k.forward, k.backward, k.left, k.right, k.jump, k.shift, k.sprint,
    ]
}
fn point(p: V) -> [f64; 3] {
    [p.x, p.y, p.z]
}
fn heading(yaw: f32, k: Input) -> f32 {
    let mut yaw = yaw;
    let m = if k.backward && !k.forward {
        yaw += 180.;
        -0.5
    } else if k.forward && !k.backward {
        0.5
    } else {
        1.
    };
    if k.left && !k.right {
        yaw -= 90. * m;
    }
    if k.right && !k.left {
        yaw += 90. * m;
    }
    yaw
}
fn directional(degrees: f32) -> Input {
    Input {
        forward: degrees > -70. && degrees < 70.,
        backward: degrees < -110. || degrees > 110.,
        right: degrees > 20. && degrees < 160.,
        left: degrees > -160. && degrees < -20.,
        ..Input::EMPTY
    }
}
impl State {
    fn new(epoch: u64, config: &Value) -> Self {
        let level = NativeLevel::new(world::read(NavigationRegistry::capture), epoch);
        Self {
            epoch,
            player: NativePlayer::new(level),
            assisted: AssistedJump::default(),
            aim: f32::NAN,
            parkour_running: false,
            safe_mode: String::new(),
            center: None,
            overwrite: 0,
            sneak: 0,
            distance: random_distance(&config["safeWalk"]["distance"]),
        }
    }
    fn release(&mut self) {
        self.assisted.release();
        self.aim = f32::NAN;
    }
    fn jump_at_edge(&self, keys: &mut Input, physical_jump: bool, yaw: f32) {
        if !self.player.on_ground() || physical_jump || self.player.motion().y > 0. {
            return;
        }
        let mut input = SimulatedInput::new(*keys);
        input.keys.jump = false;
        input.keys.sprint = self.player.read(|p| p.sprinting);
        let mut simulated = NativeSimulation::new(&self.player, input, yaw);
        simulated.tick();
        if !simulated.player().on_ground() {
            keys.jump = true;
        }
    }
    fn parkour(&mut self, config: &Value, mut keys: Input) -> Input {
        let settings = &config["parkour"];
        let enabled = settings["running"] == true;
        let yaw = world::read(|g| g.yaw);
        let movement_yaw = float(&config["movementYaw"], yaw);
        let physical_jump = config["physicalJump"] == true;
        if !enabled {
            if self.parkour_running {
                self.assisted.reset();
                self.aim = f32::NAN;
            }
            self.parkour_running = false;
            RESULT.with(|r| *r.borrow_mut() = None);
            return keys;
        }
        self.parkour_running = true;
        let running = keys.is_moving() && !keys.shift;
        let free = self
            .player
            .read(|p| !p.fluids.was_touching_water && !p.on_climbable && !p.flying);
        let mut sprint = false;
        if settings["assist"] != true || !running || !free {
            self.release();
            if running {
                self.jump_at_edge(&mut keys, physical_jump, movement_yaw);
            }
        } else {
            let mut terrain = Terrain::player(self.player.inventory.level.clone());
            let motion = capture_motion(&self.player);
            let mut abilities = nav::walk::abilities::WalkAbilities::<NativeState>::new(
                false,
                false,
                0,
                true,
                self.player.food_level() > 6,
                3,
                23,
                Arc::new(|_| f64::INFINITY),
                Arc::new(HashSet::new()),
            );
            abilities.motion = motion;
            abilities.risky_jumps = true;
            let mut state = Motion::new(motion);
            state.reset_player(&mut terrain, &self.player);
            let mut jumps = Jumps::new_with_holds_y_rot(abilities, true);
            let mut targets = JumpTargets::new(motion);
            let h = heading(yaw, keys);
            let leap = LeapInput::new(
                i32::from(keys.forward) - i32::from(keys.backward),
                i32::from(keys.left) - i32::from(keys.right),
                keys.jump,
                self.player.read(|p| p.sprinting),
                false,
            );
            let max_angle = float(&settings["maxAngle"], 35.);
            let assisted = self.assisted.tick(
                &mut terrain,
                &state,
                movement_yaw,
                h,
                leap,
                max_angle,
                &mut jumps,
                &mut targets,
            );
            if settings["render"] == true {
                self.assisted.trace(
                    &mut terrain,
                    &state,
                    movement_yaw,
                    h,
                    max_angle,
                    &mut jumps,
                    &mut targets,
                );
            }
            if self.assisted.target.is_none() {
                self.aim = f32::NAN;
            }
            if let Some(leap) = assisted {
                let (mut z, mut x) = (leap.forward, leap.strafe);
                if config["silent"] == true && config["currentYaw"].is_number() {
                    (z,x)=super::edge_geometry::inverse_input(leap.forward,leap.strafe,yaw,float(&config["currentYaw"],yaw));
                }
                keys.forward = z > 0;
                keys.backward = z < 0;
                keys.left = x > 0;
                keys.right = x < 0;
                keys.jump = leap.jump;
                keys.shift = leap.shift;
                sprint = leap.sprint;
            } else {
                self.jump_at_edge(&mut keys, physical_jump, movement_yaw);
            }
        }
        if self.assisted.target.is_some() {
            let aim = self.assisted.aim_y_rot(
                self.player.position(),
                self.player.motion(),
                self.player.on_ground(),
            );
            if !aim.is_nan() {
                self.aim = aim;
            }
        }
        let arcs=self.assisted.arcs.iter().map(|a|json!({"path":a.path.iter().copied().map(point).collect::<Vec<_>>(),"takeoff":a.takeoff,"landing":[a.target.pos.x as f64,a.target.feet,a.target.pos.z as f64]})).collect::<Vec<_>>();
        let aim = self.aim.is_finite().then_some(self.aim);
        let target = self.assisted.target.as_ref().map(|t| point(t.aim()));
        let run_up = self
            .assisted
            .target
            .as_ref()
            .is_some_and(|t| t.step.is_some());
        let position = point(self.player.position());
        RESULT.with(|r| {
            *r.borrow_mut() =
                Some(json!({"sprint":sprint,"aimYaw":aim,"target":target,"arcs":arcs,"runUp":run_up,"position":position}).to_string())
        });
        keys
    }
    fn would_fall(&self, position: V) -> bool {
        let step = self.player.attribute("minecraft:step_height");
        let (width, height, dy) = self.player.read(|p| {
            (
                f64::from(p.dimensions.width),
                f64::from(p.dimensions.height),
                p.fall_distance - step,
            )
        });
        let radius = width / 2. - 0.05;
        let bounds = Box3::new(
            V::new(position.x - radius, position.y + dy, position.z - radius),
            V::new(
                position.x + radius,
                position.y + height + dy,
                position.z + radius,
            ),
        );
        // Level.noCollision includes blocks and collidable entities.
        let entities = world::read(|g| g.player_entity_collisions());
        if entities
            .iter()
            .any(|b| super::auto_walk_player::box_(*b).intersects(bounds))
        {
            return false;
        }
        world::write(|g| {
            let mut motion = horizon_client::motion::MotionWorld {
                entities: &[],
                player: g.player.as_mut().expect("edge player"),
                scene: &mut g.world,
                assets: &g.pack.assets,
                collision_binding: g.collision,
                resources: g.motion,
                health: &g.health_resources,
            };
            motion
                .no_player_collision(super::auto_walk_player::core_box(bounds))
                .expect("edge collision")
        })
    }
    fn edge_collision(&self, from: V, to: V) -> bool {
        let d = V::new(to.x - from.x, to.y - from.y, to.z - from.z);
        if d.x * d.x + d.y * d.y + d.z * d.z <= 1e-12 {
            return false;
        }
        let extended_from = V::new(
            from.x - 1000. * d.x,
            from.y - 1000. * d.y,
            from.z - 1000. * d.z,
        );
        let extended_to = V::new(to.x + 1000. * d.x, to.y + 1000. * d.y, to.z + 1000. * d.z);
        let mut boxes = Vec::new();
        let level = &self.player.inventory.level;
        for x in ((from.x.min(to.x) - 0.6 - 1e-7).floor() as i32)
            ..=((from.x.max(to.x) + 0.6 + 1e-7).floor() as i32)
        {
            for y in ((from.y.min(to.y) - 0.5 - 1e-7).floor() as i32)
                ..=((from.y.min(to.y) + 1e-7).floor() as i32)
            {
                for z in ((from.z.min(to.z) - 0.6 - 1e-7).floor() as i32)
                    ..=((from.z.max(to.z) + 0.6 + 1e-7).floor() as i32)
                {
                    let p = Pos::new(x, y, z);
                    let shape = level.block_state(p).collision_shape(level, p);
                    for b in shape.aabbs() {
                        let b = Box3::new(
                            V::new(
                                b.min.x - 0.3 + x as f64,
                                b.min.y - 1. + y as f64,
                                b.min.z - 0.3 + z as f64,
                            ),
                            V::new(
                                b.max.x + 0.3 + x as f64,
                                b.max.y + 0.55 + y as f64,
                                b.max.z + 0.3 + z as f64,
                            ),
                        );
                        if b.clip(extended_from, extended_to).is_some() {
                            boxes.push(b);
                        }
                    }
                }
            }
        }
        super::edge_geometry::crosses_edge(from,to,boxes)
    }

    fn close_to_edge(&self, keys: Input, distance: f64, yaw: f32) -> bool {
        let mut sim_input = SimulatedInput::new(keys);
        sim_input.keys.jump = false;
        sim_input.keys.shift = false;
        sim_input.keys.sprint = self.player.read(|p| p.sprinting);
        let mut simulation = NativeSimulation::new(&self.player, sim_input, yaw);
        simulation.tick();
        let next = simulation.player().motion();
        let speed = (next.x * next.x + next.z * next.z).sqrt();
        let direction = if speed > 0.003 {
            V::new(next.x / speed, 0., next.z / speed)
        } else {
            nav::walk_control::view_rotation::ViewRotation::new(
                heading(world::read(|g| g.yaw), keys),
                0.,
            )
            .view_vector()
        };
        let p = self.player.position();
        let from = V::new(p.x, p.y - 0.1, p.z);
        let to = V::new(
            from.x + distance * direction.x,
            from.y,
            from.z + distance * direction.z,
        );
        self.edge_collision(from, to) || self.would_fall(p) || {
            let p = simulation.predicted_position();
            self.would_fall(V::new(p.x + next.x, p.y, p.z + next.z))
        }
    }
    fn safe_walk(&mut self, config: &Value, mut keys: Input) -> Input {
        let settings = &config["safeWalk"];
        let mode = if settings["running"] == true {
            settings["mode"].as_str().unwrap_or("Safe")
        } else {
            "None"
        };
        if mode != self.safe_mode {
            self.center = None;
            self.overwrite = 0;
            self.distance = random_distance(&settings["distance"]);
            self.safe_mode = mode.into();
        }
        horizon_client::navigation_physics::set_module_safe_walk(mode == "Safe");
        if mode != "OnEdge" {
            return keys;
        }
        let position = self.player.position();
        let velocity = self.player.motion();
        let speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
        if self.player.on_ground()
            && !keys.shift
            && self.close_to_edge(
                keys,
                speed.min(f64::from(self.distance)),
                float(&config["movementYaw"], world::read(|g| g.yaw)),
            )
        {
            if let Some(center) = self.center {
                let mut sim_input = SimulatedInput::new(keys);
                sim_input.keys.sprint = self.player.read(|p| p.sprinting);
                let mut simulation = NativeSimulation::new(
                    &self.player,
                    sim_input,
                    float(&config["movementYaw"], world::read(|g| g.yaw)),
                );
                simulation.tick();
                let next = simulation.predicted_position();
                let d = |p: V| (center.x - p.x).powi(2) + (center.z - p.z).powi(2);
                if d(next) <= d(position) {
                    return keys;
                }
            }
            if self.overwrite == 0 {
                self.distance = random_distance(&settings["distance"]);
                self.overwrite = random_range(&settings["keep"], [1, 2]);
            }
            if self.sneak == 0 {
                self.sneak = random_range(&settings["sneak"], [0, 0]);
            }
        }
        if self.overwrite > 0 {
            self.overwrite -= 1;
            let mode = settings["onEdgeMode"].as_str().unwrap_or("Stop");
            if mode == "Invert" {
                (keys.forward, keys.backward) = (keys.backward, keys.forward);
                (keys.left, keys.right) = (keys.right, keys.left);
                keys.jump = false;
            } else if mode == "Center" || speed > 0.05 {
                let center = self.center.unwrap_or_else(|| {
                    let p = Pos::containing(position);
                    V::new(p.x as f64 + 0.5, p.y as f64, p.z as f64 + 0.5)
                });
                let degrees = wrap_degrees(
                    view_yaw(center.x - position.x, center.z - position.z)
                        - wrap_degrees(world::read(|g| g.yaw)),
                );
                let next = directional(degrees);
                keys.forward = next.forward;
                keys.backward = next.backward;
                keys.left = next.left;
                keys.right = next.right;
            } else {
                keys.forward = false;
                keys.backward = false;
                keys.left = false;
                keys.right = false;
                keys.jump = false;
            }
            if settings["jump"] == true {
                keys.jump = true;
            }
        }
        if self.sneak > 0 {
            self.sneak -= 1;
            keys.shift = true;
        }
        let p = Pos::containing(position);
        let center = V::new(p.x as f64 + 0.5, p.y as f64, p.z as f64 + 0.5);
        if !self.would_fall(center) {
            self.center = Some(center);
        }
        keys
    }
}
pub(super) fn clear() {
    STATE.with(|s| *s.borrow_mut() = None);
    RESULT.with(|s| *s.borrow_mut() = None);
    horizon_client::navigation_physics::set_module_safe_walk(false);
}
impl Game<'_> {
    pub(super) fn edge_assistance_config(&mut self, value: &str) {
        if let Ok(v) = serde_json::from_str(value) {
            CONFIG.with(|c| *c.borrow_mut() = v);
        }
    }
    pub(super) fn edge_assistance_result(&self) -> Option<String> {
        RESULT.with(|r| r.borrow().clone())
    }
    pub(super) fn edge_assistance_input(&mut self, raw: [bool; 7], after: bool) -> [bool; 7] {
        if !self.native_window()
            || self.lan_acting.is_some()
            || self.lan_acting_uuid.is_some()
            || !self
                .player
                .as_ref()
                .is_some_and(|p| p.native_client_queries_allowed())
        {
            return raw;
        }
        let config = CONFIG.with(|c| c.borrow().clone());
        if config.is_null() {
            return raw;
        }
        let epoch = world::current_epoch(self);
        world::with_scope(self, || {
            STATE.with(|slot| {
                let mut slot = slot.borrow_mut();
                if slot.as_ref().is_none_or(|s| s.epoch != epoch) {
                    *slot = Some(State::new(epoch, &config));
                }
                let state = slot.as_mut().expect("edge runtime");
                keys(if after {
                    state.safe_walk(&config, input(raw))
                } else {
                    state.parkour(&config, input(raw))
                })
            })
        })
    }
}
