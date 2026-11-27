// SPDX-License-Identifier: GPL-3.0-or-later
//! Native Agent body. Every operation reaches the real scoped client Game.
use super::*;
use super::{
    auto_walk_player::{NativeInventory, NativePlayer, NativeStack, box_},
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NativeState, NavigationRegistry},
};
use crate::navigation as nav;
use nav::{
    aabb::{Aabb as NavBox, Direction as NavDirection},
    geometry::{BlockPos as NavPos, Vec3 as NavVec},
    input::Input,
    navigation_context::NavigationContext,
    options::Options,
    terrain::{TerrainLevel, TerrainState},
    walk_control::{
        block_actions::{ActionHost, DoubleHalf, PlacedBlock},
        block_sight::SightHost,
        block_targets::{BlockHit, Hand as NavHand, TargetHost},
        body::{SimulatedInput, WalkBody},
        turning::LookPurpose,
        view_rotation::ViewRotation,
    },
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[derive(Default)]
pub(super) struct NativeRoute {
    pub points: Vec<NavVec>,
    pub breaks: Vec<NavPos>,
    pub places: Vec<NavPos>,
    pub progress: Option<nav::course::ProgressOwner>,
    pub preview: bool,
    pub contacts: Vec<(NavPos, NavDirection)>,
    pub state: &'static str,
    pub since: u64,
    pub start: Option<NavVec>,
    pub destination: Option<NavVec>,
}
impl NativeRoute {
    fn change(&mut self, state: &'static str) {
        if self.state != state {
            self.state = state;
            self.since = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |t| t.as_millis() as u64);
        }
    }
}
pub(super) struct NativeHost {
    pub player: NativePlayer,
    pub options: Options<NativeState, NativeStack>,
    pub context: NavigationContext,
    pub input_yaw: f32,
    pub movement_yaw: f32,
    pub sent: ViewRotation,
    pub current: ViewRotation,
    pub requested: Option<(ViewRotation, LookPurpose)>,
    pub input: Option<Input>,
    pub exact: bool,
    pub safe_walk: bool,
    pub external_rotation: bool,
    pub mouse_turning: bool,
    pub is_flying: bool,
    pub recasts_elytra: bool,
    pub route: Rc<RefCell<NativeRoute>>,
    own_slot: Option<usize>,
    ledge_ticks: i32,
}
impl NativeHost {
    pub fn new(registry: Arc<NavigationRegistry>, epoch: u64) -> Self {
        let (yaw, pitch) = world::read(|g| (g.yaw, g.pitch));
        let mut options = Options::new(nav::options::PlacementPolicy {
            placeable: Arc::new(NativeStack::scaffold_placeable),
            unfavourable: Arc::new(|_| false),
        });
        options.look_ahead = false;
        Self {
            player: NativePlayer::new(NativeLevel::new(registry, epoch)),
            options,
            context: NavigationContext::default(),
            input_yaw: yaw,
            movement_yaw: yaw,
            sent: ViewRotation::new(yaw, pitch),
            current: ViewRotation::new(yaw, pitch),
            requested: None,
            input: None,
            exact: false,
            safe_walk: false,
            external_rotation: false,
            mouse_turning: false,
            is_flying: false,
            recasts_elytra: false,
            route: Rc::default(),
            own_slot: None,
            ledge_ticks: 0,
        }
    }
    pub fn level(&self) -> NativeLevel {
        self.player.inventory.level.clone()
    }
    pub fn selected(&mut self, slot: i32) {
        if !(0..9).contains(&slot) {
            return;
        }
        let old = world::write(|g| {
            let old = g.selected;
            g.selected = slot as usize;
            g.remote_send_carried();
            old
        });
        if self.own_slot.is_none() {
            self.own_slot = Some(old);
        }
    }
    pub fn release(&mut self) {
        if let Some(slot) = self.own_slot.take() {
            world::write(|g| {
                g.selected = slot;
                g.remote_send_carried();
            });
        }
    }
    pub fn release_all(&mut self) {
        if super::auto_walk_runtime::keeps_destroying() {
            world::write(|g| {
                g.stop_remote_mining();
            });
        }
        super::auto_walk_runtime::keep_destroying(false);
        super::auto_walk_runtime::keep_using(false);
        self.release();
        self.input = None;
        self.exact = false;
        self.safe_walk = false;
        self.requested = None;
        self.ledge_ticks = 0;
        horizon_client::navigation_physics::set_safe_walk(false);
    }
    pub fn turn(&mut self, rotation: ViewRotation, purpose: LookPurpose) {
        if purpose == LookPurpose::View {
            world::write(|g| {
                g.yaw = rotation.y_rot;
                g.pitch = rotation.x_rot.clamp(-90., 90.);
                if let Some(p) = g.player.as_mut() {
                    p.yaw = g.yaw;
                    p.pitch = g.pitch;
                }
            });
        } else {
            let rotation = if purpose == LookPurpose::Steer && self.player.read(|p| p.fall_flying) {
                rotation
            } else {
                let from = self.current;
                let step = self.step();
                let quantize = |v: f32| {
                    if step > 0. {
                        let counts = v / step;
                        let floor = counts.floor();
                        (if counts - floor >= 0.5 {
                            floor + 1.
                        } else {
                            floor
                        }) as i32 as f32
                            * step
                    } else {
                        v
                    }
                };
                ViewRotation::new(
                    from.y_rot
                        + quantize(nav::walk_jump::wrap_degrees(rotation.y_rot - from.y_rot)),
                    (from.x_rot + quantize(rotation.x_rot - from.x_rot)).clamp(-90., 90.),
                )
            };
            self.requested = Some((rotation, purpose));
        }
    }
    pub fn use_hand(&mut self, hand: NavHand) -> bool {
        let hand = hand_(hand);
        world::write(|g| {
            g.remote_send_carried();
            let used = g.remote_use_item(hand);
            super::auto_walk_runtime::keep_using(g.using.is_some());
            used
        })
    }
    pub fn swap(&mut self, slot: i32, hotbar: i32) {
        world::write(|g| {
            g.native_client_auto_tool_swap(slot, hotbar);
        });
    }
    pub fn can_click(&self) -> bool {
        self.context
            .can_click_inventory(self.player.read(|p| p.sprinting))
    }
    pub fn step(&self) -> f32 {
        world::read(|g| {
            let s = f64::from(g.settings.sensitivity) * f64::from(0.6_f32) + f64::from(0.2_f32);
            (s * s * s * 8.0 * 0.15) as f32
        })
    }
    pub fn root_position(&self) -> NavVec {
        world::read(|g| {
            g.riding
                .map(|id| g.client_reach_root_vehicle(id))
                .and_then(|id| g.native_entity(id))
                .and_then(|e| g.native_entity_position(e))
                .map_or_else(
                    || world::vec(g.player.as_ref().expect("navigation player").position),
                    |p| NavVec::new(p[0], p[1], p[2]),
                )
        })
    }
    pub fn root_motion(&self) -> NavVec {
        world::read(|g| {
            if let Some(id) = g.riding {
                if let Some(b) = g.boats.iter().find(|b| b.id == id) {
                    return world::vec(b.velocity);
                }
                if let Some(m) = g.mobs.iter().find(|m| m.id == id) {
                    return world::vec(m.velocity);
                }
            }
            world::vec(g.player.as_ref().expect("navigation player").velocity)
        })
    }
    pub fn trace(&self, from: NavVec, to: NavVec, fluids: bool) -> BlockHit {
        world::read(|g| {
            let d = NavVec::new(to.x - from.x, to.y - from.y, to.z - from.z);
            let length = (d.x * d.x + d.y * d.y + d.z * d.z).sqrt();
            if length == 0. {
                return miss(to);
            }
            g.clip_ray_fluids(
                [from.x, from.y, from.z],
                [d.x / length, d.y / length, d.z / length],
                length,
                fluids,
            )
            .map_or_else(|| miss(to), hit_)
        })
    }
    pub fn forced_hit<R>(hit: BlockHit, run: impl for<'a> FnOnce(&mut Game<'a>) -> R) -> R {
        world::write(|g| {
            let old = g.forced_hit;
            g.forced_hit = Some(core_hit(hit));
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(g)));
            g.forced_hit = old;
            match result {
                Ok(result) => result,
                Err(error) => std::panic::resume_unwind(error),
            }
        })
    }
    pub fn mining(&mut self, pos: NavPos, face: NavDirection, start: bool) -> bool {
        let hit = BlockHit {
            is_block: true,
            location: NavVec::new(
                f64::from(pos.x) + 0.5,
                f64::from(pos.y) + 0.5,
                f64::from(pos.z) + 0.5,
            ),
            direction: face,
            pos,
        };
        Self::forced_hit(hit, |g| {
            if g.remote.is_none()
                || g.remote_break_restricted([pos.x, pos.y, pos.z])
                || !g.res.items().can_destroy_block(g.held(), g.creative)
                || g.res.facts().is_air(g.world.block(pos.x, pos.y, pos.z))
            {
                return false;
            }
            g.remote_send_carried();
            g.remote_mining(start);
            super::auto_walk_runtime::keep_destroying(if start {
                g.remote.as_ref().is_some_and(|r| r.is_destroying)
            } else {
                true
            });
            g.native_client_anti_afk_swing();
            true
        })
    }
}
fn hand_(hand: NavHand) -> horizon_client::first_person::Hand {
    match hand {
        NavHand::Main => horizon_client::first_person::Hand::Main,
        NavHand::Off => horizon_client::first_person::Hand::Off,
    }
}
pub(super) fn face_(face: NavDirection) -> [i32; 3] {
    match face {
        NavDirection::Down => [0, -1, 0],
        NavDirection::Up => [0, 1, 0],
        NavDirection::North => [0, 0, -1],
        NavDirection::South => [0, 0, 1],
        NavDirection::West => [-1, 0, 0],
        NavDirection::East => [1, 0, 0],
    }
}
fn hit_(hit: Hit) -> BlockHit {
    BlockHit {
        is_block: true,
        location: NavVec::new(hit.point[0], hit.point[1], hit.point[2]),
        direction: match hit.face {
            [0, -1, 0] => NavDirection::Down,
            [0, 1, 0] => NavDirection::Up,
            [0, 0, -1] => NavDirection::North,
            [0, 0, 1] => NavDirection::South,
            [-1, 0, 0] => NavDirection::West,
            _ => NavDirection::East,
        },
        pos: NavPos::new(hit.pos[0], hit.pos[1], hit.pos[2]),
    }
}
fn core_hit(hit: BlockHit) -> Hit {
    Hit {
        pos: [hit.pos.x, hit.pos.y, hit.pos.z],
        face: face_(hit.direction),
        point: [hit.location.x, hit.location.y, hit.location.z],
    }
}
fn miss(point: NavVec) -> BlockHit {
    BlockHit {
        is_block: false,
        location: point,
        direction: NavDirection::Down,
        pos: NavPos::containing(point),
    }
}
impl WalkBody for NativeHost {
    type Player = NativePlayer;
    type LiveLevel = NativeLevel;
    type Simulation = NativeSimulation;
    fn player(&self) -> &NativePlayer {
        &self.player
    }
    fn live_level(&self) -> NativeLevel {
        self.level()
    }
    fn exact_simulation(&self) -> bool {
        true
    }
    fn input_y_rot(&self) -> f32 {
        self.input_yaw
    }
    fn movement_y_rot(&self) -> f32 {
        self.movement_yaw
    }
    fn simulate(&self, input: SimulatedInput, yaw: f32) -> NativeSimulation {
        NativeSimulation::new(&self.player, input, yaw)
    }
}
impl SightHost for NativeHost {
    type State = NativeState;
    fn state(&self, pos: NavPos) -> NativeState {
        self.level().block_state(pos)
    }
    fn outline_shape(&self, _state: &NativeState, pos: NavPos) -> nav::voxel_shape::VoxelShape {
        world::read(|g| {
            nav::voxel_shape::VoxelShape::new(
                g.outline_boxes([pos.x, pos.y, pos.z])
                    .into_iter()
                    .map(|b| {
                        [
                            b[0] - f64::from(pos.x),
                            b[1] - f64::from(pos.y),
                            b[2] - f64::from(pos.z),
                            b[3] - f64::from(pos.x),
                            b[4] - f64::from(pos.y),
                            b[5] - f64::from(pos.z),
                        ]
                    })
                    .collect(),
            )
        })
    }
    fn clip(&self, from: NavVec, to: NavVec) -> BlockHit {
        self.trace(from, to, false)
    }
}
impl TargetHost for NativeHost {
    type Inventory = NativeInventory;
    type Policy = nav::options::PlacementPolicy<NativeStack>;
    fn eye_position(&self) -> NavVec {
        self.player.eye()
    }
    fn reach(&self) -> f64 {
        world::read(|g| g.reach())
    }
    fn block_position(&self) -> NavPos {
        NavPos::containing(self.player.position())
    }
    fn uuid_most_significant_bits(&self) -> i64 {
        world::write(|g| {
            let words = g.acting_player_uuid();
            (i64::from(words[0]) << 32) | i64::from(words[1] as u32)
        })
    }
    fn sent_rotation(&self) -> ViewRotation {
        self.sent
    }
    fn view_step(&self) -> f32 {
        self.step()
    }
    fn pick(&self, rotation: ViewRotation, reach: f64, eye: Option<NavVec>) -> BlockHit {
        let from = eye.unwrap_or_else(|| self.player.eye());
        let d = rotation.view_vector();
        self.trace(
            from,
            NavVec::new(
                from.x + d.x * reach,
                from.y + d.y * reach,
                from.z + d.z * reach,
            ),
            false,
        )
    }
    fn inventory(&self) -> &NativeInventory {
        &self.player.inventory
    }
    fn placeable_blocks(&self) -> &Self::Policy {
        &self.options.placeable_blocks
    }
    fn is_tool_for(&self, stack: &NativeStack, state: &NativeState) -> bool {
        stack.correct(state)
    }
    fn destroy_speed(&self, stack: &NativeStack, state: &NativeState) -> f32 {
        stack.speed(state)
    }
    fn efficiency_level(&self, stack: &NativeStack) -> i32 {
        stack.efficiency()
    }
    fn is_wear_free(&self, stack: &NativeStack) -> bool {
        stack.max_damage() == 0 || stack.components.get("minecraft:unbreakable").is_some()
    }
}
impl ActionHost for NativeHost {
    fn collision_empty(&self, state: &NativeState, pos: NavPos) -> bool {
        state.collision_shape(&self.level(), pos).is_empty()
    }
    fn double_half(&self, state: &NativeState) -> Option<DoubleHalf> {
        match state.property("half")? {
            "lower" => Some(DoubleHalf::Lower),
            "upper" => Some(DoubleHalf::Upper),
            _ => None,
        }
    }
    fn player_box(&self) -> NavBox {
        self.player.read(|p| box_(p.bounds()))
    }
    fn look(&mut self, rotation: ViewRotation, purpose: LookPurpose) {
        self.turn(rotation, purpose)
    }
    fn select_hotbar_slot(&mut self, slot: i32) {
        self.selected(slot)
    }
    fn release_hotbar_slot(&mut self) {
        self.release()
    }
    fn start_destroy(&mut self, pos: NavPos, face: NavDirection) -> bool {
        self.mining(pos, face, true)
    }
    fn continue_destroy(&mut self, pos: NavPos, face: NavDirection) -> bool {
        self.mining(pos, face, false)
    }
    fn stop_destroy(&mut self) {
        super::auto_walk_runtime::keep_destroying(false);
        world::write(|g| {
            g.stop_remote_mining();
        });
    }
    fn use_item_on(&mut self, hand: NavHand, hit: BlockHit) -> bool {
        if !hit.is_block {
            return false;
        }
        let sent = self.sent;
        Self::forced_hit(hit, |g| {
            let old = (g.yaw, g.pitch, g.player.as_ref().map(|p| (p.yaw, p.pitch)));
            g.yaw = sent.y_rot;
            g.pitch = sent.x_rot;
            if let Some(p) = g.player.as_mut() {
                p.yaw = sent.y_rot;
                p.pitch = sent.x_rot;
            }
            let used = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                g.remote_send_carried();
                g.remote_use_item_on(hand_(hand))
            }));
            g.yaw = old.0;
            g.pitch = old.1;
            if let (Some(p), Some((yaw, pitch))) = (g.player.as_mut(), old.2) {
                p.yaw = yaw;
                p.pitch = pitch;
            }
            match used {
                Ok(used) => used,
                Err(error) => std::panic::resume_unwind(error),
            }
        })
    }
    fn latency_millis(&self) -> i32 {
        world::read(|g| g.own_latency().unwrap_or(0))
    }
    fn block_placed(&mut self, _: &Arc<PlacedBlock>) {}
    fn block_kept(&mut self, _: &Arc<PlacedBlock>) {}
    fn block_rejected(&mut self, _: &Arc<PlacedBlock>) {}
    fn server_put_back_broken(&mut self, _: NavPos) {}
    fn server_took_back_placed(&mut self, _: NavPos) {}
}
impl nav::walk_control::ability_capture::CaptureHost for NativeHost {
    type Stack = NativeStack;
    type Inventory = NativeInventory;
    fn capture_options(
        &self,
    ) -> nav::walk_control::ability_capture::CaptureOptions<NativeState, NativeStack> {
        self.options.capture()
    }
    fn capture_inventory(&self) -> &NativeInventory {
        &self.player.inventory
    }
    fn safe_fall_distance(&self) -> f64 {
        self.player.attribute("minecraft:safe_fall_distance")
    }
    fn health(&self) -> f32 {
        self.player.read(|p| p.health.health())
    }
    fn jump_climbs_ladders(&self) -> bool {
        true
    }
}
impl nav::walk_control::follower::FollowerHost for NativeHost {
    fn places_bridge_floors(&self) -> bool {
        self.options.places_bridge_floors
    }
    fn bridging(&self) -> nav::walk_control::bridging::Bridging {
        self.options.bridging
    }
    fn risky_bridges(&self) -> bool {
        self.options.risky_bridges
    }
    fn sneaking_speed(&self) -> f64 {
        self.player.attribute("minecraft:sneaking_speed")
    }
    fn step_reached(&mut self, _: usize, _: NavPos) {}
    fn fell_off_bridge(&mut self, _: NavPos) {}
    fn interpolated_position(&self, partial: f32) -> NavVec {
        self.player.interpolated(partial)
    }
    fn crossing_started(&self, _: usize, _: nav::walk_control::bridging::Bridging) {}
    fn crossing_fallback(
        &self,
        _: nav::walk_control::bridging::Bridging,
        _: Option<nav::walk_control::bridging::Bridging>,
    ) {
    }
    fn crossing_hopped(&self, _: Option<nav::walk_control::bridging::Bridging>, _: i32) {}
}
impl nav::walk_control::planning::PlanningHost for NativeHost {
    fn max_length(&self) -> f64 {
        self.options.max_length()
    }
    fn following_path_debug(&mut self, _: usize, _: NavVec) {}
    fn route_found(&mut self, _: &Arc<nav::walk::route::Route<NativeState>>) {}
    fn route_searching(&mut self, _: NavVec, _: NavVec, _: f64) {}
}
impl nav::look_ahead::ViewHost for NativeHost {
    fn look_settings(&self) -> nav::look_ahead::LookSettings {
        nav::look_ahead::LookSettings {
            enabled: self.options.look_ahead,
            turn_speed: self.options.look_ahead_turn_speed,
            distance: self.options.look_ahead_distance,
            x_rot: self.options.look_ahead_x_rot,
        }
    }
    fn view_taken_over(&self) -> bool {
        self.mouse_turning || !self.external_rotation
    }
    fn view_position(&self) -> NavVec {
        self.player.position()
    }
    fn view_y_rot(&self) -> f32 {
        world::read(|g| g.yaw)
    }
    fn view_x_rot(&self) -> f32 {
        world::read(|g| g.pitch)
    }
    fn root_vehicle_motion(&self) -> NavVec {
        self.root_motion()
    }
    fn mouse_view_step(&self) -> f32 {
        self.step()
    }
    fn turn_look(&mut self, rotation: ViewRotation, purpose: LookPurpose) {
        self.turn(rotation, purpose)
    }
}
impl nav::route_report::RouteListener for NativeHost {
    type Progress = nav::course::ProgressOwner;
    fn route_searching(
        &mut self,
        start: NavVec,
        goal: NavVec,
        _: f64,
        preview: Option<nav::route_report::IdentityRef<'_, [NavVec]>>,
    ) {
        let mut route = self.route.borrow_mut();
        route.points = preview.map_or_else(Vec::new, |p| p.value.to_vec());
        route.preview = true;
        route.progress = None;
        route.start = Some(start);
        route.destination = Some(goal);
        route.change("PENDING");
    }
    fn route_following(
        &mut self,
        points: nav::route_report::IdentityRef<'_, [NavVec]>,
        progress: nav::route_report::IdentityRef<'_, Self::Progress>,
    ) {
        let mut route = self.route.borrow_mut();
        route.points = points.value.to_vec();
        route.preview = false;
        route.progress = Some(*progress.value);
        route.start = route.points.first().copied();
        route.destination = route.points.last().copied();
        route.change("ACTIVE");
    }
    fn blocks_ahead(
        &mut self,
        breaks: nav::route_report::IdentityRef<'_, [NavPos]>,
        places: nav::route_report::IdentityRef<'_, [NavPos]>,
    ) {
        let mut route = self.route.borrow_mut();
        route.breaks = breaks.value.to_vec();
        route.places = places.value.to_vec();
    }
    fn route_cleared(&mut self) {
        *self.route.borrow_mut() = NativeRoute::default();
    }
    fn route_invalid(&mut self) {
        self.route.borrow_mut().change("INVALID");
    }
    fn route_done(&mut self) {
        self.route.borrow_mut().change("DONE");
    }
}
impl nav::course::CourseHost for NativeHost {
    fn root_vehicle_position(&self) -> NavVec {
        self.root_position()
    }
    fn player_width(&self) -> f64 {
        self.player.read(|p| f64::from(p.dimensions.width))
    }
    fn navigation_arrived(&mut self, _: NavVec) {}
    fn navigation_failed(&mut self, _: NavVec) {}
    fn navigation_stuck(&mut self, _: NavPos) {}
}
impl nav::snacking::SnackHost for NativeHost {
    fn is_using_item(&self) -> bool {
        world::read(|g| g.using.is_some())
    }
    fn food_level(&self) -> i32 {
        self.player.read(|p| p.health.food.food)
    }
    fn can_eat(&self) -> bool {
        self.options.can_eat
    }
    fn is_in_water(&self) -> bool {
        self.player.read(|p| p.fluids.was_touching_water)
    }
    fn on_ground(&self) -> bool {
        self.player.on_ground()
    }
    fn is_passenger(&self) -> bool {
        self.player.read(|p| p.is_passenger)
    }
    fn best_food_slot(&self) -> i32 {
        nav::snacking::best_food_slot(&self.player.inventory)
    }
    fn can_click_inventory(&self) -> bool {
        self.can_click()
    }
    fn should_stand(&mut self) {
        self.context.set_should_stand(true)
    }
    fn suitable_hotbar_slot(&self) -> i32 {
        self.player.inventory.suitable_slot()
    }
    fn swap_with_hotbar(&mut self, inventory: i32, hotbar: i32) {
        self.swap(inventory, hotbar)
    }
    fn select_hotbar_slot(&mut self, slot: i32) {
        self.selected(slot)
    }
    fn release_hotbar_slot(&mut self) {
        self.release()
    }
    fn use_item(&mut self, hand: NavHand) -> bool {
        self.use_hand(hand)
    }
}
impl nav::navigator::NavigatorHost for NativeHost {
    fn navigation_context(&self) -> &NavigationContext {
        &self.context
    }
    fn min_press_ticks(&self) -> i32 {
        self.options.min_press_ticks
    }
    fn min_release_ticks(&self) -> i32 {
        self.options.min_release_ticks
    }
    fn set_safe_walk(&mut self, safe: bool) {
        self.safe_walk = safe;
        horizon_client::navigation_physics::set_safe_walk(safe);
    }
    fn commit_navigation_input(&mut self, mut input: Input, exact: bool) {
        if self.safe_walk {
            use nav::walk_control::body::WalkSimulation;
            let at_ledge = if self.player.on_ground() && !input.jump && !input.shift {
                let mut simulated =
                    NativeSimulation::new(&self.player, SimulatedInput::new(input), self.input_yaw);
                simulated.tick();
                simulated.has_backed_off_edge()
            } else {
                false
            };
            self.ledge_ticks = if at_ledge {
                3
            } else {
                (self.ledge_ticks - 1).max(0)
            };
            if self.ledge_ticks > 0 {
                input.shift = true;
                input.sprint = false;
            }
        } else {
            self.ledge_ticks = 0;
        }
        self.input = Some(input);
        self.exact = exact;
    }
}
impl nav::flight_body::FlightBody for NativeHost {
    type Player = NativePlayer;
    fn player(&self) -> &NativePlayer {
        &self.player
    }
    fn is_flying(&self) -> bool {
        self.is_flying
    }
    fn press(&mut self, input: Input, exact: bool) {
        self.context.press(input, exact)
    }
    fn no_block_collision(&mut self, bounds: NavBox) -> bool {
        world::write(|g| {
            let mut motion = horizon_client::motion::MotionWorld {
                entities: &[],
                player: g.player.as_mut().expect("navigation player"),
                scene: &mut g.world,
                assets: &g.pack.assets,
                collision_binding: g.collision,
                resources: g.motion,
                health: &g.health_resources,
            };
            motion
                .no_player_collision(super::auto_walk_player::core_box(bounds))
                .expect("navigation block collision")
        })
    }
}
impl nav::glide::body::GlideBody for NativeHost {
    type Player = NativePlayer;
    type State = NativeState;
    type Level = NativeLevel;
    fn player(&self) -> &NativePlayer {
        &self.player
    }
    fn level(&self) -> NativeLevel {
        self.level()
    }
    fn glide_clearance(&self) -> f64 {
        self.options.glide_clearance
    }
    fn safe_fall(&self) -> i32 {
        nav::walk_control::ability_capture::safe_fall(self)
    }
    fn press(&mut self, input: Input, exact: bool) {
        self.context.press(input, exact)
    }
    fn look(&mut self, yaw: f32, pitch: f32, purpose: nav::glide::body::GlideLook) {
        self.turn(
            ViewRotation::new(yaw, pitch),
            match purpose {
                nav::glide::body::GlideLook::Movement => LookPurpose::Movement,
                nav::glide::body::GlideLook::Steer => LookPurpose::Steer,
            },
        )
    }
    fn select_hotbar_slot(&mut self, slot: i32) {
        self.selected(slot)
    }
    fn release_hotbar_slot(&mut self) {
        self.release()
    }
    fn use_item(&mut self, hand: nav::glide::body::GlideHand) {
        self.use_hand(match hand {
            nav::glide::body::GlideHand::Main => NavHand::Main,
            nav::glide::body::GlideHand::Off => NavHand::Off,
        });
    }
}
impl nav::glide::roof_bounce::RoofBody for NativeHost {
    type Player = NativePlayer;
    type State = NativeState;
    type Level = NativeLevel;
    fn player(&self) -> &NativePlayer {
        &self.player
    }
    fn recasts_elytra(&self) -> bool {
        self.recasts_elytra
    }
    fn live_level(&self) -> NativeLevel {
        self.level()
    }
    fn press(&mut self, input: Input, exact: bool) {
        self.context.press(input, exact)
    }
    fn look_steer(&mut self, yaw: f32, pitch: f32) {
        self.turn(ViewRotation::new(yaw, pitch), LookPurpose::Steer)
    }
}
