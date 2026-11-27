// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete common Crossing.kt controller. Host operations below are the actual
//! original Actions/Targets/StepCheck calls, not geometric approximations.
use super::{
    body::{SimulatedInput, WalkPlayer, WalkSimulation},
    bridge::Bridge,
    bridge_host::{BridgeHost, PlacementIdentity},
    bridging::Bridging,
    directional_input,
    floor_prediction::can_click_floor,
    path_points::StepControl,
    prediction::wants_jump,
    step_progress::Movement,
    subtract,
    telly::Telly,
    view_rotation::ViewRotation,
};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    input::{Input, MovementKeys},
    simulation::player_motion::MotionPlayer,
    walk::route::Step,
    walk_jump::wrap_degrees,
};

pub struct Crossing {
    bridge: Option<Bridge>,
    pub method: Option<Bridging>,
    rejections: i32,
    placed: i32,
    last_placement: Option<PlacementIdentity>,
    hop_ticks: i32,
    double_tap_ticks: i32,
    is_hopping: bool,
    nudge: Option<ViewRotation>,
    nudged_floor: Option<BlockPos>,
    refused: bool,
    pub view: Option<ViewRotation>,
    pub acting: Option<BlockPos>,
    telly: Telly,
}
impl Default for Crossing {
    fn default() -> Self {
        Self {
            bridge: None,
            method: None,
            rejections: 0,
            placed: 0,
            last_placement: None,
            hop_ticks: -1,
            double_tap_ticks: 0,
            is_hopping: false,
            nudge: None,
            nudged_floor: None,
            refused: false,
            view: None,
            acting: None,
            telly: Telly::default(),
        }
    }
}
impl Crossing {
    /// Diagnostic state retained between ticks, in original declaration order.
    pub fn timing(&self) -> (i32, i32, i32, i32, bool, bool) {
        (
            self.rejections,
            self.placed,
            self.hop_ticks,
            self.double_tap_ticks,
            self.is_hopping,
            self.refused,
        )
    }
    pub fn takes(&mut self, host: &impl BridgeHost, steps: &[Step], index: usize) -> bool {
        let Some(found) = Bridge::at(steps, index) else {
            self.clear();
            return false;
        };
        if Some(found.to) != self.bridge.as_ref().map(|bridge| bridge.to) {
            let sneaking_speed = host.sneaking_speed();
            let can_sprint = host.player().food_level() > 6;
            // options() is intentionally read at both original call sites.
            self.method = Some(host.bridging().pick(
                &found,
                host.risky_bridges(),
                sneaking_speed,
                can_sprint,
            ));
            self.placed = 0;
            self.last_placement = host.placement();
            self.rejections = host.rejections();
            self.refused = false;
            self.hop_ticks = -1;
            host.crossing_started(found.length(), self.method.expect("picked bridge method"));
        }
        self.bridge = Some(found.clone());
        if host.rejections() > self.rejections && self.method.is_some() {
            self.rejections = host.rejections();
            self.refused = true;
            let fallback = if self.method == Some(Bridging::Ninja) || found.rises {
                None
            } else {
                Some(Bridging::Ninja)
            };
            host.crossing_fallback(self.method.expect("active bridge method"), fallback);
            self.method = fallback;
        }
        if self.method.is_none() {
            self.view = None;
        }
        self.method.is_some()
    }
    pub fn clear(&mut self) {
        self.double_tap_ticks = 0;
        self.bridge = None;
        self.method = None;
        self.view = None;
        self.acting = None;
    }
    pub fn tick(
        &mut self,
        host: &mut impl BridgeHost,
        keys: &mut MovementKeys,
        frame_y_rot: f32,
        steps: &[Step],
        index: usize,
    ) -> Movement {
        let bridge = self
            .bridge
            .clone()
            .expect("Crossing.takes must select a bridge");
        let method = self.method.expect("Crossing.takes must select a method");
        self.count_placement(host, method);
        if self.double_tap_ticks > 0 {
            self.double_tap_ticks -= 1;
        }
        self.acting = None;
        if method == Bridging::Telly {
            let movement = self
                .telly
                .tick(host, keys, frame_y_rot, steps, &bridge, method.x_rot());
            self.view = self.telly.view;
            self.acting = self.telly.acting;
            return movement;
        }
        let step = &steps[index];
        if step.is_rising_bridge() {
            self.rise(host, keys, frame_y_rot, step, method, &bridge)
        } else {
            self.level(host, keys, frame_y_rot, steps, index, method, &bridge)
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn level(
        &mut self,
        host: &mut impl BridgeHost,
        keys: &mut MovementKeys,
        frame_y_rot: f32,
        steps: &[Step],
        index: usize,
        method: Bridging,
        bridge: &Bridge,
    ) -> Movement {
        let view = Self::back_view(method, bridge, method.x_rot());
        self.view = Some(view);
        let step = &steps[index];
        let mut target = step;
        let mut floor = step
            .places
            .iter()
            .find(|&&pos| !host.is_placed(pos))
            .copied();
        let unconfirmed = step
            .places
            .iter()
            .find(|&&pos| host.is_unconfirmed(pos))
            .copied();
        if floor.is_none()
            && let Some(unconfirmed) = unconfirmed
        {
            if self.waits(host, method) {
                return self.hold(host, keys, frame_y_rot, step, unconfirmed);
            }
            if let Some(next) = steps.get(index + 1)
                && index + 1 < bridge.end
                && next.is_bridge()
            {
                target = next;
                floor = next
                    .places
                    .iter()
                    .find(|&&pos| !host.is_placed(pos))
                    .copied();
            }
        }
        let position = host.player().position();
        keys.directional_input =
            directional_input(self.tapped(target, method, bridge, position), frame_y_rot);
        keys.jump = false;
        keys.sneak =
            method.sneaks_throughout() || self.sneaks_at_edge(host, keys, target, floor, method);
        if let Some(pending) = floor {
            let ready = !self.waits(host, method)
                || can_click_floor(target, host, keys, frame_y_rot, method.sneaks_throughout());
            let held = self.aim(host, pending, view);
            host.place(pending, ready, held);
            self.acting = Some(pending);
        }
        self.hop(host, keys);
        Movement {
            target: Some(target.feet),
            ..Movement::default()
        }
    }
    fn rise(
        &mut self,
        host: &mut impl BridgeHost,
        keys: &mut MovementKeys,
        frame_y_rot: f32,
        step: &Step,
        method: Bridging,
        bridge: &Bridge,
    ) -> Movement {
        let (low, high) = (step.places[0], step.places[1]);
        let position = host.player().position();
        keys.directional_input = directional_input(subtract(step.feet, position), frame_y_rot);
        keys.jump = false;
        self.view = Some(Self::back_view(
            method,
            bridge,
            if host.is_placed(low) {
                84.0
            } else {
                method.x_rot()
            },
        ));
        let view = self.view.unwrap();
        if !host.is_placed(low) {
            keys.sneak = self.sneaks_at_edge(host, keys, step, Some(low), Bridging::Ninja);
            let ready =
                !self.waits(host, method) || can_click_floor(step, host, keys, frame_y_rot, false);
            let held = self.aim(host, low, view);
            host.place(low, ready, held);
            self.acting = Some(low);
        } else if self.waits(host, method) && host.is_unconfirmed(low) {
            return self.hold(host, keys, frame_y_rot, step, low);
        } else if !host.is_placed(high) {
            host.place(high, true, view);
            self.acting = Some(high);
            if !host.player().on_ground() {
                keys.sneak = host.player().delta_movement().y < 0.0;
            } else if step.past_edge(position) < -0.2 {
                keys.sneak = true;
            } else if horizontal_length(host.player().delta_movement()) > 0.1 {
                keys.directional_input = Input::EMPTY;
                keys.sneak = true;
            } else {
                keys.jump = true;
            }
        } else if self.waits(host, method) && host.is_unconfirmed(high) {
            return self.hold(host, keys, frame_y_rot, step, high);
        } else {
            keys.jump = wants_jump(step, host.player());
            keys.sneak = !host.player().on_ground() && host.player().delta_movement().y < 0.0;
        }
        Movement {
            target: Some(step.feet),
            ..Movement::default()
        }
    }
    fn hold(
        &mut self,
        host: &impl BridgeHost,
        keys: &mut MovementKeys,
        frame_y_rot: f32,
        step: &Step,
        pos: BlockPos,
    ) -> Movement {
        let below = pos.y < step.from.y;
        keys.directional_input = if below {
            step.hold_back(
                host.player().position(),
                host.player().delta_movement(),
                frame_y_rot,
            )
        } else {
            Input::EMPTY
        };
        keys.jump = false;
        keys.sneak = keys.directional_input == Input::EMPTY
            || self.method.is_some_and(Bridging::sneaks_throughout);
        self.acting = Some(pos);
        Movement {
            target: Some(step.feet),
            ..Movement::default()
        }
    }
    fn aim(&mut self, host: &impl BridgeHost, floor: BlockPos, view: ViewRotation) -> ViewRotation {
        let standing = host.player().on_ground() && length_sqr(host.stride_moved()) < 0.1 * 0.1;
        if !standing || self.nudged_floor != Some(floor) {
            self.nudge = None;
        }
        if let Some(held) = self.nudge {
            if !Self::is_steady(host, held, 1.0)
                || host.clicks_from(host.player().eye_position(), host.sent_rotation(), floor)
            {
                return held;
            }
        } else if !standing
            || !Self::is_steady(host, view, 1.0)
            || host.clicks_from(host.player().eye_position(), view, floor)
        {
            return view;
        }
        self.nudged_floor = Some(floor);
        self.nudge = host
            .placeable_slot()
            .and_then(|slot| host.placement_rotation(floor, slot, true));
        self.nudge.unwrap_or(view)
    }
    fn sneaks_at_edge(
        &mut self,
        host: &impl BridgeHost,
        keys: &mut MovementKeys,
        step: &Step,
        floor: Option<BlockPos>,
        method: Bridging,
    ) -> bool {
        if !host.player().on_ground() {
            return false;
        }
        let mut next = host.simulate(
            SimulatedInput::new(keys.to_input(self.double_tap_ticks > 0)),
            host.input_y_rot(),
        );
        next.tick();
        if !next.has_backed_off_edge() {
            let stops = floor.is_none() || !method.clicks_past_edge() || method.resets(self.placed);
            let walking = horizontal_length(host.player().delta_movement()) > 0.1;
            if stops
                && walking
                && next.player().on_ground()
                && step.past_edge(next.player().position()) > 0.25
            {
                keys.directional_input = Input::EMPTY;
                self.double_tap_ticks = 7;
            }
            return false;
        }
        let aimed = host.action_rotation();
        if floor.is_none()
            || aimed.is_none()
            || !method.clicks_past_edge()
            || method.resets(self.placed)
        {
            return true;
        }
        !(Self::is_steady(host, aimed.unwrap(), 1.0)
            && next.player().on_ground()
            && host.clicks_from(next.player().eye_position(), aimed.unwrap(), floor.unwrap()))
    }
    fn hop(&mut self, host: &impl BridgeHost, keys: &mut MovementKeys) {
        if self.is_hopping {
            self.is_hopping = !host.player().on_ground();
            if self.is_hopping {
                keys.directional_input = Input::EMPTY;
                keys.sneak = false;
                return;
            }
        }
        if self.hop_ticks < 0 || !host.player().on_ground() {
            return;
        }
        if horizontal_length(host.player().delta_movement()) > 0.1 {
            self.hop_ticks = self.hop_ticks.wrapping_add(1);
            if self.hop_ticks <= 10 {
                keys.directional_input = Input::EMPTY;
                keys.sneak = true;
                return;
            }
        }
        let input = Input::EMPTY.with_keys(true, false, false);
        let mut simulated = host.simulate(SimulatedInput::new(input), host.input_y_rot());
        simulated.tick();
        let mut lands = false;
        for _ in 0..20 {
            if simulated.player().on_ground() {
                lands = simulated.player().y() >= host.player().y() - 0.01;
                break;
            }
            simulated.tick();
        }
        if lands {
            keys.directional_input = Input::EMPTY;
            keys.jump = true;
            keys.sneak = false;
            self.is_hopping = true;
            self.hop_ticks = -1;
            host.crossing_hopped(self.method, self.placed);
        } else {
            self.hop_ticks = self.hop_ticks.wrapping_add(1);
            if self.hop_ticks > 10 {
                self.hop_ticks = -1;
            }
        }
    }
    fn count_placement(&mut self, host: &impl BridgeHost, method: Bridging) {
        let placement = host.placement();
        if placement.is_none() || placement == self.last_placement {
            return;
        }
        self.last_placement = placement;
        if method.hops_on_reset() && method.resets(self.placed) {
            self.hop_ticks = 0;
        }
        self.placed = self.placed.wrapping_add(1);
    }
    fn tapped(&self, target: &Step, method: Bridging, bridge: &Bridge, position: Vec3) -> Vec3 {
        let toward = subtract(target.feet, position);
        let dx = target.to.x.wrapping_sub(target.from.x);
        let dz = target.to.z.wrapping_sub(target.from.z);
        let along = position.x * f64::from(dx) + position.z * f64::from(dz);
        let side = method.tap(self.placed, along - along.floor(), bridge.turn_after);
        if side == 0 || toward.x * toward.x + toward.z * toward.z < 0.01 {
            return toward;
        }
        let length = length_sqr(toward).sqrt();
        let ahead = if length < f64::from(1.0e-5_f32) {
            Vec3::ZERO
        } else {
            Vec3::new(toward.x / length, toward.y / length, toward.z / length)
        };
        Vec3::new(
            ahead.x - f64::from(dz.wrapping_mul(side)),
            0.0,
            ahead.z + f64::from(dx.wrapping_mul(side)),
        )
    }
    fn back_view(method: Bridging, bridge: &Bridge, x_rot: f32) -> ViewRotation {
        let offset = if bridge.is_straight {
            method.y_rot_offset()
        } else {
            0.0
        };
        ViewRotation::new(wrap_degrees(bridge.y_rot + 180.0 + offset), x_rot)
    }
    fn is_steady(host: &impl BridgeHost, view: ViewRotation, tolerance: f32) -> bool {
        let sent = host.sent_rotation();
        wrap_degrees(sent.y_rot - view.y_rot).abs() < tolerance
            && (sent.x_rot - view.x_rot).abs() < tolerance
    }
    fn waits(&self, host: &impl BridgeHost, method: Bridging) -> bool {
        self.refused || !host.risky_bridges() && !method.clicks_past_edge()
    }
}
fn horizontal_length(value: Vec3) -> f64 {
    (value.x * value.x + value.z * value.z).sqrt()
}
fn length_sqr(value: Vec3) -> f64 {
    value.x * value.x + value.y * value.y + value.z * value.z
}
