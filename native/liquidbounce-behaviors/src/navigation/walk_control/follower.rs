// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent walking Follower orchestration over the original helpers and actions.
use super::{
    REACH_HORIZONTAL,
    block_actions::{ActionHost, BlockActions, PlacedBlock, PlacementOrder},
    block_targets::BlockTargets,
    body::{ControlState, WalkBody, WalkPlayer, is_stable},
    bridge_host::{BridgeHost, PlacementIdentity},
    bridging::Bridging,
    crossing::Crossing,
    hops::Hops,
    overshoot::Overshoot,
    path_points::{self, RouteControl, StepControl},
    step_check,
    step_progress::{BridgeLine, Movement, StepProgress},
    stride::Stride,
    swimming::Swimming,
    turning::LookPurpose,
    view_rotation::ViewRotation,
};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    input::{Input, MovementKeys},
    progress::FollowState,
    simulation::player_motion::MotionPlayer,
    terrain::Terrain,
    walk::route::{Move, Route, Step},
};
use std::{cell::RefCell, sync::Arc};

pub trait FollowerHost: WalkBody + ActionHost<State = <Self::Player as WalkPlayer>::State> {
    fn places_bridge_floors(&self) -> bool;
    fn bridging(&self) -> Bridging;
    fn risky_bridges(&self) -> bool;
    fn sneaking_speed(&self) -> f64;
    fn step_reached(&mut self, index: usize, to: BlockPos);
    fn fell_off_bridge(&mut self, to: BlockPos);
    fn interpolated_position(&self, partial_ticks: f32) -> Vec3;
    fn crossing_started(&self, length: usize, method: Bridging);
    fn crossing_fallback(&self, method: Bridging, fallback: Option<Bridging>);
    fn crossing_hopped(&self, method: Option<Bridging>, placed: i32);
}

#[derive(Default)]
struct PlacementTracker {
    last: Option<Arc<PlacedBlock>>,
    serial: u64,
}
impl PlacementTracker {
    fn identity(&mut self, placement: &Option<Arc<PlacedBlock>>) -> Option<PlacementIdentity> {
        let Some(placement) = placement else {
            return None;
        };
        if !self
            .last
            .as_ref()
            .is_some_and(|last| Arc::ptr_eq(last, placement))
        {
            self.serial = self.serial.wrapping_add(1);
            self.last = Some(placement.clone());
        }
        Some(PlacementIdentity(self.serial))
    }
}
struct BridgeBinding<'a, H, O: PlacementOrder> {
    host: &'a mut H,
    actions: &'a mut BlockActions<O>,
    stride: &'a Stride,
    identity: &'a RefCell<PlacementTracker>,
}
impl<H: FollowerHost, O: PlacementOrder> WalkBody for BridgeBinding<'_, H, O> {
    type Player = H::Player;
    type LiveLevel = H::LiveLevel;
    type Simulation = H::Simulation;
    fn player(&self) -> &Self::Player {
        self.host.player()
    }
    fn live_level(&self) -> Self::LiveLevel {
        self.host.live_level()
    }
    fn exact_simulation(&self) -> bool {
        self.host.exact_simulation()
    }
    fn input_y_rot(&self) -> f32 {
        self.host.input_y_rot()
    }
    fn movement_y_rot(&self) -> f32 {
        self.host.movement_y_rot()
    }
    fn simulate(&self, input: super::body::SimulatedInput, yaw: f32) -> Self::Simulation {
        self.host.simulate(input, yaw)
    }
}
impl<H: FollowerHost, O: PlacementOrder> BridgeHost for BridgeBinding<'_, H, O> {
    fn sent_rotation(&self) -> ViewRotation {
        self.host.sent_rotation()
    }
    fn look(&mut self, rotation: ViewRotation, purpose: LookPurpose) {
        self.host.look(rotation, purpose)
    }
    fn is_placed(&self, pos: BlockPos) -> bool {
        step_check::is_placed(&self.host.live_level(), pos)
    }
    fn place(&mut self, pos: BlockPos, ready: bool, held: ViewRotation) {
        self.actions.place(self.host, pos, ready, Some(held))
    }
    fn placement(&self) -> Option<PlacementIdentity> {
        self.identity.borrow_mut().identity(&self.actions.placement)
    }
    fn rejections(&self) -> i32 {
        self.actions.rejections
    }
    fn action_rotation(&self) -> Option<ViewRotation> {
        self.actions.rotation
    }
    fn is_unconfirmed(&self, pos: BlockPos) -> bool {
        self.actions.is_unconfirmed(pos)
    }
    fn bridging(&self) -> Bridging {
        self.host.bridging()
    }
    fn risky_bridges(&self) -> bool {
        self.host.risky_bridges()
    }
    fn sneaking_speed(&self) -> f64 {
        self.host.sneaking_speed()
    }
    fn stride_moved(&self) -> Vec3 {
        self.stride.moved()
    }
    fn clicks_from(&self, eye: Vec3, rotation: ViewRotation, floor: BlockPos) -> bool {
        BlockTargets::clicks_from(self.host, eye, rotation, floor)
    }
    fn placeable_slot(&self) -> Option<i32> {
        BlockTargets::placeable_slot(self.host)
    }
    fn placement_rotation(&self, pos: BlockPos, slot: i32, nearest: bool) -> Option<ViewRotation> {
        BlockTargets::plan_placement(self.host, pos, slot, nearest).map(|plan| plan.rotation)
    }
    fn crossing_started(&self, length: usize, method: Bridging) {
        self.host.crossing_started(length, method)
    }
    fn crossing_fallback(&self, method: Bridging, fallback: Option<Bridging>) {
        self.host.crossing_fallback(method, fallback)
    }
    fn crossing_hopped(&self, method: Option<Bridging>, placed: i32) {
        self.host.crossing_hopped(method, placed)
    }
}

#[derive(Clone)]
struct FeetReference<B: ControlState> {
    route: Arc<Route<B>>,
    index: usize,
}
pub struct Follower<B: ControlState> {
    path: Option<Arc<Route<B>>>,
    index: usize,
    pub stride: Stride,
    water: Swimming,
    hops: Hops,
    hop_reference: Option<FeetReference<B>>,
    overshoot: Overshoot,
    crossing: Crossing,
    placement_identity: RefCell<PlacementTracker>,
    pub stuck_at: Option<BlockPos>,
    movement: Movement,
    bridge_line: Option<BridgeLine>,
    pub remaining_breaks: Vec<BlockPos>,
    pub remaining_places: Vec<BlockPos>,
    progress: StepProgress,
}
impl<B: ControlState> Default for Follower<B> {
    fn default() -> Self {
        Self {
            path: None,
            index: 0,
            stride: Stride::default(),
            water: Swimming::default(),
            hops: Hops::default(),
            hop_reference: None,
            overshoot: Overshoot::default(),
            crossing: Crossing::default(),
            placement_identity: RefCell::default(),
            stuck_at: None,
            movement: Movement::default(),
            bridge_line: None,
            remaining_breaks: Vec::new(),
            remaining_places: Vec::new(),
            progress: StepProgress::default(),
        }
    }
}
impl<B: ControlState> Follower<B> {
    pub fn path(&self) -> Option<&Arc<Route<B>>> {
        self.path.as_ref()
    }
    pub fn index(&self) -> usize {
        self.index
    }
    fn step(&self) -> Option<&Step> {
        self.path.as_ref()?.steps().get(self.index)
    }
    pub fn is_finished(&self) -> bool {
        self.path.is_some() && self.step().is_none()
    }
    pub fn is_moving(&self) -> bool {
        self.step().is_some()
    }
    pub fn movement(&self) -> Movement {
        self.movement
    }
    pub fn building(&self) -> bool {
        self.step().is_some_and(StepControl::builds)
            || self
                .path
                .as_ref()
                .and_then(|p| p.steps().get(self.index + 1))
                .is_some_and(StepControl::builds)
    }
    pub fn next_block<H: FollowerHost>(&self, host: &H) -> Option<BlockPos>
    where
        H::Player: WalkPlayer<State = B>,
    {
        let path = self.path.as_ref()?;
        for step in &path.steps()
            [self.index.min(path.steps().len())..(self.index + 2).min(path.steps().len())]
        {
            let pending = step
                .breaks
                .iter()
                .find(|&&p| !step_check::is_cleared(&host.live_level(), p))
                .or_else(|| {
                    step.places
                        .iter()
                        .find(|&&p| !step_check::is_placed(&host.live_level(), p))
                });
            if let Some(&pending) = pending {
                return Some(pending);
            }
        }
        None
    }
    pub fn view(&self) -> Option<ViewRotation> {
        self.crossing.view
    }
    pub fn bridge_ahead(&self) -> usize {
        let Some(path) = &self.path else {
            return 0;
        };
        let start = if self.step().is_some_and(Step::is_bridge) {
            self.index
        } else {
            self.index + 1
        };
        path_points::run_end(path.steps(), start) - start
    }
    pub fn swim_ahead(&self) -> &[Step] {
        let Some(path) = &self.path else {
            return &[];
        };
        let start = if self.step().is_some_and(|s| s.movement == Move::Swim) {
            self.index
        } else {
            self.index + 1
        };
        let start = start.min(path.steps().len());
        let count = path.steps()[start..]
            .iter()
            .take_while(|s| s.movement == Move::Swim)
            .count();
        &path.steps()[start..start + count]
    }
    pub fn bridge_y_rot(&self) -> Option<f32> {
        self.bridge_line.as_ref().map(BridgeLine::heading)
    }
    pub fn bridge_target(&self) -> Option<Vec3> {
        self.bridge_line.map(|line| line.to)
    }
    pub fn overshoots(&self) -> i32 {
        self.overshoot.count()
    }
    pub fn follow(&mut self, path: Arc<Route<B>>, start_index: usize) {
        self.path = Some(path);
        self.index = start_index;
        self.step_changed();
    }
    pub fn clear(&mut self) {
        self.path = None;
        self.index = 0;
        self.movement = Movement::default();
        self.clear_hop();
        self.bridge_line = None;
        self.crossing.clear();
        self.step_changed();
    }
    pub fn position_corrected(&mut self) {
        self.clear_hop();
        self.progress = StepProgress::default();
    }
    fn clear_hop(&mut self) {
        self.hops.clear();
        self.hop_reference = None;
    }
    pub fn is_off_path<H: FollowerHost>(&self, host: &H) -> bool
    where
        H::Player: WalkPlayer<State = B>,
    {
        self.path.as_ref().is_some_and(|p| {
            self.step().is_some()
                && is_stable(host.player())
                && p.is_off_path(self.index, host.player().position())
        })
    }

    pub fn tick<H: FollowerHost, O: PlacementOrder>(
        &mut self,
        host: &mut H,
        actions: &mut BlockActions<O>,
        keys: &mut MovementKeys,
        yaw: f32,
    ) -> FollowState
    where
        H::Player: WalkPlayer<State = B>,
    {
        self.stride.tick(host.player());
        self.movement = Movement::default();
        self.advance(host, actions);
        let Some(step) = self.step().cloned() else {
            actions.idle(host);
            Self::stand_still(host, keys);
            return FollowState::Finished;
        };
        if step.is_bridge() && host.places_bridge_floors() {
            self.crossing.clear();
            return self.cross_bridge(host, actions, keys, yaw);
        }
        self.bridge_line = None;
        if !step.places.is_empty() && actions.rejected_in_row >= 3 {
            actions.forget_rejections();
            actions.idle(host);
            Self::stand_still(host, keys);
            return self.give_up(&step);
        }
        if (step.is_bridge() || step.is_rising_bridge())
            && host.player().y() < f64::from(step.from.y) - 1.0
        {
            self.crossing.clear();
            actions.idle(host);
            Self::stand_still(host, keys);
            if !host.player().on_ground() || self.stuck_at == Some(step.to) {
                return FollowState::Walking;
            }
            host.fell_off_bridge(step.to);
            return self.give_up(&step);
        }
        let path = self.path.as_ref().expect("active step has route").clone();
        let mut bridge = BridgeBinding {
            host,
            actions,
            stride: &self.stride,
            identity: &self.placement_identity,
        };
        if self.crossing.takes(&bridge, path.steps(), self.index) {
            self.movement = self
                .crossing
                .tick(&mut bridge, keys, yaw, path.steps(), self.index);
            return if let Some(acting) = self.crossing.acting {
                self.act(acting, &step)
            } else {
                self.walked(host, &step)
            };
        }
        let pending_break = step
            .breaks
            .iter()
            .find(|&&p| !step_check::is_cleared(&host.live_level(), p))
            .copied();
        let pending_place = step
            .places
            .iter()
            .find(|&&p| !step_check::is_placed(&host.live_level(), p))
            .copied();
        let unconfirmed_place = step
            .places
            .iter()
            .find(|&&p| actions.is_unconfirmed(p))
            .copied();
        let pending_open = step
            .opens
            .iter()
            .find(|&&p| {
                !step_check::is_open(&host.live_level(), p) || actions.is_open_unconfirmed(p)
            })
            .copied();
        if let Some(pos) = pending_break {
            Self::stand_still(host, keys);
            actions.mine(host, pos);
            self.act(pos, &step)
        } else if let Some(pos) = pending_place {
            self.place(host, actions, keys, yaw, &step, pos);
            self.act(pos, &step)
        } else if let Some(pos) = unconfirmed_place
            .filter(|p| step_check::is_cleared(&host.live_level(), p.offset(0, -1, 0)))
        {
            actions.idle(host);
            Self::stand_still(host, keys);
            if pos.y < step.from.y {
                keys.directional_input = step.hold_back(
                    host.player().position(),
                    host.player().delta_movement(),
                    yaw,
                );
            }
            self.hold_at_edge(keys, &step, pos, None);
            self.act(pos, &step)
        } else if let Some(pos) = pending_open {
            Self::stand_still(host, keys);
            keys.jump = host.player().is_in_water() || host.player().on_climbable();
            actions.open(host, pos);
            self.act(pos, &step)
        } else {
            self.walk(host, actions, keys, yaw, &step)
        }
    }
    fn walk<H: FollowerHost, O: PlacementOrder>(
        &mut self,
        host: &mut H,
        actions: &mut BlockActions<O>,
        keys: &mut MovementKeys,
        yaw: f32,
        step: &Step,
    ) -> FollowState
    where
        H::Player: WalkPlayer<State = B>,
    {
        actions.idle(host);
        self.progress.acting = None;
        self.movement = self.move_player(host, keys, yaw, step);
        let stuck = if step.jump.is_some() {
            self.progress.jump_ticks = self.progress.jump_ticks.wrapping_add(1);
            self.progress.jump_ticks >= 120 || self.progress.leap.hopeless
        } else {
            self.progress
                .watch
                .stalls(host.player().position().distance(step.feet))
                || self.progress.is_due_to_predict()
                    && !super::prediction::gets_ahead(host, keys, yaw, step, self.movement)
        };
        if stuck {
            self.give_up(step)
        } else {
            FollowState::Walking
        }
    }
    fn walked<H: FollowerHost>(&mut self, host: &H, step: &Step) -> FollowState
    where
        H::Player: WalkPlayer<State = B>,
    {
        self.progress.acting = None;
        if self
            .progress
            .watch
            .stalls(host.player().position().distance(step.feet))
        {
            self.give_up(step)
        } else {
            FollowState::Walking
        }
    }
    fn place<H: FollowerHost, O: PlacementOrder>(
        &mut self,
        host: &mut H,
        actions: &mut BlockActions<O>,
        keys: &mut MovementKeys,
        yaw: f32,
        step: &Step,
        pos: BlockPos,
    ) where
        H::Player: WalkPlayer<State = B>,
    {
        let position = host.player().position();
        let pillars = step.movement == Move::Pillar;
        Self::stand_still(host, keys);
        if pillars {
            keys.directional_input = super::center_input(position, step.feet, yaw);
            keys.jump = host.player().on_ground();
        } else if pos.y < step.from.y {
            keys.directional_input =
                super::directional_input(super::subtract(step.feet, position), yaw);
            self.hold_at_edge(keys, step, pos, Some(step.feet));
        }
        let ready = pos.y >= step.from.y
            || super::floor_prediction::can_click_floor(step, host, keys, yaw, true);
        actions.place(host, pos, ready, None);
    }
    fn hold_at_edge(
        &mut self,
        keys: &mut MovementKeys,
        step: &Step,
        pos: BlockPos,
        target: Option<Vec3>,
    ) {
        if pos.y < step.from.y {
            keys.sneak = true;
            self.movement = Movement {
                target,
                ..Movement::default()
            };
        } else {
            self.movement = Movement {
                target,
                safe_walk: true,
                ..Movement::default()
            };
        }
    }
    fn act(&mut self, pos: BlockPos, step: &Step) -> FollowState {
        if self.progress.acts_too_long(pos) {
            self.give_up(step)
        } else {
            FollowState::Acting
        }
    }
    fn cross_bridge<H: FollowerHost, O: PlacementOrder>(
        &mut self,
        host: &mut H,
        actions: &mut BlockActions<O>,
        keys: &mut MovementKeys,
        yaw: f32,
    ) -> FollowState
    where
        H::Player: WalkPlayer<State = B>,
    {
        actions.idle(host);
        self.progress.acting = None;
        let Some(path) = self.path.clone() else {
            return FollowState::Finished;
        };
        let steps = path.steps();
        let end = path_points::run_end(steps, self.index) - 1;
        let target = steps[end].feet;
        let position = host.player().position();
        let left = super::horizontal(position, target);
        let start = self.index;
        while self.index <= end
            && left <= super::horizontal(steps[self.index].feet, target) + REACH_HORIZONTAL
        {
            self.index += 1;
        }
        if self.index != start {
            self.reached(host, steps, start);
            self.step_changed();
            if self.index > end {
                self.bridge_line = None;
                return self.tick(host, actions, keys, yaw);
            }
        }
        let line = self
            .bridge_line
            .filter(|line| super::horizontal(line.to, target) <= 1.5)
            .unwrap_or_else(|| {
                let line = BridgeLine::new(position, target);
                self.bridge_line = Some(line);
                line
            });
        keys.directional_input =
            super::directional_input(super::subtract(line.aim_from(position), position), yaw);
        keys.jump = false;
        keys.sneak = false;
        self.movement = Movement {
            target: Some(line.to),
            sprint: true,
            safe_walk: true,
            ..Movement::default()
        };
        if self.progress.watch.stalls(left) {
            self.give_up(&steps[self.index])
        } else {
            FollowState::Walking
        }
    }
    fn give_up(&mut self, step: &Step) -> FollowState {
        self.stuck_at = Some(step.to);
        self.progress = StepProgress::default();
        FollowState::Stuck
    }
    fn move_player<H: FollowerHost>(
        &mut self,
        host: &H,
        keys: &mut MovementKeys,
        yaw: f32,
        step: &Step,
    ) -> Movement
    where
        H::Player: WalkPlayer<State = B>,
    {
        if self.water.breathes(host.player(), &host.live_level())
            || step.is_under_water && self.water.is_short_of_air(host.player())
        {
            Self::stand_still(host, keys);
            self.progress.watch.excuse();
            return Movement::default();
        }
        if step.is_under_water {
            return self.water.swim(host.player(), keys, yaw, step);
        }
        if step.movement == Move::ClimbUp {
            return super::climbing::climb_up(
                host.player(),
                &mut Terrain::player(host.live_level()),
                keys,
                yaw,
                step,
            );
        }
        if let Some(jump) = &step.jump
            && let Some(path) = &self.path
        {
            return super::leaping::leap(
                host,
                keys,
                yaw,
                step,
                jump,
                path.abilities(),
                &mut self.progress.leap,
                &self.stride,
            );
        }
        let position = host.player().position();
        if host.player().on_ground() {
            self.clear_hop();
        }
        let path = self.path.as_ref().expect("moving step has route").clone();
        let shortcut = if step.is_plain() && host.player().on_ground() {
            super::shortcut::shortcut_index(
                &path,
                self.index,
                position,
                &mut Terrain::player(host.live_level()),
            )
        } else {
            None
        };
        let aim = if let Some(index) = shortcut {
            (
                path.steps()[index].feet,
                Some(FeetReference {
                    route: path.clone(),
                    index,
                }),
            )
        } else if let Some(aim) = self.hops.aim() {
            (aim, self.hop_reference.clone())
        } else {
            (
                step.feet,
                Some(FeetReference {
                    route: path.clone(),
                    index: self.index,
                }),
            )
        };
        let index = self.index;
        let (aim, reference) =
            self.overshoot
                .forward_aim_tagged(host.player(), position, aim, None, step, |p, d| {
                    let (point, index) = path_points::point_ahead_tagged(&path, index, p, d);
                    Some((
                        point,
                        index.map(|index| FeetReference {
                            route: path.clone(),
                            index,
                        }),
                    ))
                });
        keys.directional_input = super::center_input(position, aim, yaw);
        keys.sneak = step.shift;
        let same_feet = reference
            .as_ref()
            .is_some_and(|r| Arc::ptr_eq(&r.route, &path) && r.index == self.index);
        keys.jump = super::prediction::wants_jump(step, host.player())
            || !same_feet && !super::prediction::leads_onto_stairs(step, &host.live_level()) && {
                let starts = self.hops.starts(
                    host.player(),
                    &mut Terrain::player(host.live_level()),
                    || host.movement_y_rot(),
                    &self.stride,
                    position,
                    aim,
                );
                if starts {
                    self.hop_reference = reference;
                }
                starts
            };
        Movement {
            target: Some(aim),
            sprint: !step.shift,
            is_plain_walk: step.is_level_walk(),
            safe_walk: step.edge,
            ..Movement::default()
        }
    }
    fn stand_still<H: FollowerHost>(host: &H, keys: &mut MovementKeys) {
        keys.directional_input = Input::EMPTY;
        keys.jump = host.player().is_in_water();
        keys.sneak = false;
    }
    fn advance<H: FollowerHost, O: PlacementOrder>(
        &mut self,
        host: &mut H,
        actions: &BlockActions<O>,
    ) where
        H::Player: WalkPlayer<State = B>,
    {
        let Some(path) = self.path.clone() else {
            return;
        };
        let start = self.index;
        while self.index < path.steps().len()
            && step_check::is_reached(
                host.player(),
                &host.live_level(),
                actions,
                &path.steps()[self.index],
                path.steps().get(self.index + 1),
            )
        {
            self.index += 1;
        }
        self.index = step_check::skip_ahead(host.player(), &path, self.index);
        if self.index != start {
            self.reached(host, path.steps(), start);
            self.step_changed();
        }
    }
    fn reached<H: FollowerHost>(&self, host: &mut H, steps: &[Step], start: usize) {
        for (i, step) in steps
            .iter()
            .enumerate()
            .take(self.index.min(steps.len()))
            .skip(start)
        {
            host.step_reached(i, step.to);
        }
    }
    pub fn look_ahead(&self, target: Vec3, position: Vec3, distance: f64) -> Option<Vec3> {
        self.path
            .as_ref()?
            .look_ahead(self.index, target, position, distance)
    }
    pub fn point_ahead(&self, position: Vec3, distance: f64) -> Option<Vec3> {
        self.path
            .as_ref()
            .map(|p| p.point_ahead(self.index, position, distance))
    }
    pub fn turn_ahead<H: FollowerHost>(&self, host: &H, distance: f64) -> f32 {
        self.path.as_ref().map_or(0.0, |p| {
            p.turn_ahead(self.index, host.player().position(), distance)
        })
    }
    fn step_changed(&mut self) {
        self.progress = StepProgress::default();
        let remaining = self
            .path
            .as_ref()
            .map_or(&[][..], |p| &p.steps()[self.index.min(p.steps().len())..]);
        self.remaining_breaks = remaining
            .iter()
            .flat_map(|s| s.breaks.iter().copied())
            .collect();
        self.remaining_places = remaining
            .iter()
            .flat_map(|s| s.places.iter().copied())
            .collect();
    }
    pub fn head<H: FollowerHost>(&self, host: &H, partial_ticks: f32) -> Vec3 {
        host.interpolated_position(partial_ticks)
    }
    pub fn ahead(&self) -> &[Vec3] {
        self.path.as_ref().map_or(&[][..], |p| {
            &p.points()[(self.index + 1).min(p.points().len())..]
        })
    }
}
