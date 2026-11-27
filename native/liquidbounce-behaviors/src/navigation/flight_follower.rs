// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent navigation/flight/Follower.kt.
use super::{
    flight::{Acceleration, FlyAbilities, FlyRoute},
    flight_body::{FlightActor, FlightBody, PlayerFlightAbilities},
    flight_handling::{FlightPlayer, Handling},
    geometry::{Vec3, java_max},
    input::Input,
    progress::{FollowState, ProgressWatch},
};

#[derive(Default)]
pub struct Follower {
    route: Option<Leg>,
    take_off_ticks: i32,
    is_landing: bool,
    has_landed: bool,
    pressed: Option<Input>,
    target: Option<Vec3>,
    sprint: bool,
}
impl Follower {
    pub fn target(&self) -> Option<Vec3> {
        self.target
    }
    pub fn sprint(&self) -> bool {
        self.sprint
    }
    pub fn path(&self) -> Option<&FlyRoute> {
        self.route.as_ref().map(|r| &r.path)
    }
    pub fn is_finished(&self) -> bool {
        (self.route.as_ref().is_some_and(Leg::is_finished) || self.has_landed) && !self.is_landing
    }
    pub fn is_moving(&self) -> bool {
        self.route.as_ref().is_some_and(|r| !r.is_finished())
    }
    pub fn ahead(&self) -> &[Vec3] {
        self.route
            .as_ref()
            .map_or(&[], |r| &r.path.points[r.corner.min(r.path.points.len())..])
    }
    pub fn head(&self, body: &impl FlightBody, partial_ticks: f32) -> Vec3 {
        body.player().interpolated_position(partial_ticks)
    }
    pub fn follow(&mut self, path: FlyRoute) {
        self.route = Some(Leg::new(path));
    }
    pub fn clear(&mut self) {
        self.route = None;
        self.take_off_ticks = 0;
        self.target = None;
        self.is_landing = false;
        self.has_landed = false;
        self.sprint = false;
        self.pressed = None;
    }
    pub fn is_off_path(&self, body: &impl FlightBody, handling: &Handling) -> bool {
        let Some(route) = self.route.as_ref().filter(|r| !r.is_finished()) else {
            return false;
        };
        let off_path = java_max(2.5, 2.0 * handling.speed());
        distance_segment(
            body.player().position(),
            route.path.points[route.corner - 1],
            route.path.points[route.corner],
        ) > off_path
    }
    pub fn tick(
        &mut self,
        body: &mut impl FlightBody,
        handling: &mut Handling,
        frame_y_rot: f32,
        abilities: FlyAbilities,
    ) -> FollowState {
        handling.record(body.player(), self.pressed);
        self.pressed = None;
        self.target = None;
        self.sprint = false;
        let mut route = self.route.take();
        let result = self.tick_route(body, handling, frame_y_rot, abilities, &mut route);
        self.route = route;
        result
    }
    fn tick_route(
        &mut self,
        body: &mut impl FlightBody,
        handling: &Handling,
        frame_y_rot: f32,
        abilities: FlyAbilities,
        route: &mut Option<Leg>,
    ) -> FollowState {
        if let Some(route) = route {
            Self::advance(body, handling, route);
        }
        if let Some(route) = route.as_ref()
            && !self.has_landed
            && (self.is_landing || Self::lands(body, handling, route.path.end()))
        {
            self.is_landing = self.land(body, abilities);
            if self.is_landing {
                return FollowState::Walking;
            }
            self.has_landed = true;
        }
        let Some(route) = route
            .as_mut()
            .filter(|r| !r.is_finished() && !self.has_landed)
        else {
            body.press(Input::EMPTY, true);
            return FollowState::Finished;
        };
        if !abilities.is_flying() {
            let jump = self.take_off_ticks % 2 == 0;
            self.take_off_ticks = self.take_off_ticks.wrapping_add(1);
            body.press(
                Input {
                    jump,
                    ..Input::EMPTY
                },
                true,
            );
            return FollowState::Walking;
        }
        self.take_off_ticks = 0;
        let aim = Self::aim(body, handling, route);
        self.target = Some(aim);
        self.fly(
            body,
            handling,
            frame_y_rot,
            aim,
            abilities,
            aim == route.path.end(),
        );
        Self::progress(body, handling, route)
    }
    fn lands(body: &mut impl FlightBody, handling: &Handling, end: Vec3) -> bool {
        let player = body.player();
        if body.is_flying() || !player.abilities().flying() {
            return false;
        }
        let tolerance = handling.step() / 2.0;
        let height = player.y() - end.y;
        let within = horizontal_sqr(sub(end, handling.rest(player))) <= tolerance * tolerance;
        if !within || !(0.0..=1.0).contains(&height) {
            return false;
        }
        let box_below = player
            .bounding_box()
            .expand_towards(Vec3::new(0.0, -1.0, 0.0));
        !body.no_block_collision(box_below)
    }
    fn land(&mut self, body: &mut impl FlightBody, abilities: FlyAbilities) -> bool {
        if !abilities.is_flying() && body.player().on_ground() {
            self.take_off_ticks = 0;
            return false;
        }
        let jump = if abilities.is_flying() {
            let jump = self.take_off_ticks % 2 == 0;
            self.take_off_ticks = self.take_off_ticks.wrapping_add(1);
            jump
        } else {
            false
        };
        body.press(
            Input {
                jump,
                ..Input::EMPTY
            },
            true,
        );
        true
    }
    fn progress(body: &impl FlightBody, handling: &Handling, route: &mut Leg) -> FollowState {
        let distance = body
            .player()
            .position()
            .distance(route.path.points[route.corner]);
        let missed = route.misses(distance, 2.0 * Self::final_reach(handling));
        if route.watch.stalls(distance) || missed {
            route.passed();
            FollowState::Stuck
        } else {
            FollowState::Walking
        }
    }
    fn fly(
        &mut self,
        body: &mut impl FlightBody,
        handling: &Handling,
        frame_y_rot: f32,
        aim: Vec3,
        abilities: FlyAbilities,
        is_final: bool,
    ) {
        let player = body.player();
        let error = sub(aim, handling.rest(player));
        let horizontal_tolerance = handling.step() / 2.0;
        let rise_tolerance = handling.vertical_speed() / 2.0;
        let sink_tolerance = if abilities.control() == Acceleration::Momentum && !is_final {
            0.8
        } else {
            rise_tolerance
        };
        let moves = horizontal_sqr(error) > horizontal_tolerance * horizontal_tolerance;
        let direction = if moves {
            directional_input(error, frame_y_rot)
        } else {
            Input::EMPTY
        };
        let stalled = moves
            && player.horizontal_collision()
            && handling
                .last_move()
                .map_or(0.0, |v| horizontal_sqr(v).sqrt())
                < 0.01;
        let jump = error.y > rise_tolerance || stalled;
        let leaves = player.on_ground() && error.y < -1.0;
        let shift = error.y < -sink_tolerance && !jump && !leaves;
        let input = Input {
            jump,
            shift,
            sprint: false,
            ..direction
        };
        body.press(input, true);
        self.pressed = Some(input);
        self.sprint = moves && horizontal_sqr(error).sqrt() > 3.0;
    }
    fn aim(body: &impl FlightBody, handling: &Handling, route: &Leg) -> Vec3 {
        let points = &route.path.points;
        if !handling.coasts() {
            return points[route.corner];
        }
        let mut remaining = handling.coast_distance() + 3.0;
        let mut from = body.player().position();
        for corner in route.corner..points.len() {
            let point = points[corner];
            let length = from.distance(point);
            if length >= remaining {
                let share = remaining / length;
                return Vec3::new(
                    from.x + share * (point.x - from.x),
                    from.y + share * (point.y - from.y),
                    from.z + share * (point.z - from.z),
                );
            }
            if is_sharp(points, corner) {
                return point;
            }
            remaining -= length;
            from = point;
        }
        *points.last().expect("flight route has a point")
    }
    fn advance(body: &impl FlightBody, handling: &Handling, route: &mut Leg) {
        let points = &route.path.points;
        let position = body.player().position();
        let start = route.corner;
        let final_reach = Self::final_reach(handling);
        while !route.is_finished() {
            let reach = if route.corner == points.len() - 1 {
                final_reach
            } else if !handling.coasts() {
                java_max(final_reach, handling.speed())
            } else if is_sharp(points, route.corner) {
                java_max(1.0, final_reach)
            } else {
                2.0
            };
            if points[route.corner].distance_sqr(position) >= reach * reach {
                break;
            }
            route.corner += 1;
        }
        if route.corner != start {
            route.passed();
        }
    }
    fn final_reach(handling: &Handling) -> f64 {
        let horizontal = handling.step() / 2.0;
        let vertical = handling.vertical_speed() / 2.0;
        java_max(
            0.5,
            (horizontal * horizontal + vertical * vertical).sqrt() + 0.1,
        )
    }
}
struct Leg {
    path: FlyRoute,
    corner: usize,
    watch: ProgressWatch,
    nearest: f64,
    missed: i32,
}
impl Leg {
    fn new(path: FlyRoute) -> Self {
        Self {
            path,
            corner: 1,
            watch: ProgressWatch::new(0.1, 60),
            nearest: f64::MAX,
            missed: 0,
        }
    }
    fn is_finished(&self) -> bool {
        self.corner >= self.path.points.len()
    }
    fn misses(&mut self, distance: f64, range: f64) -> bool {
        if distance < self.nearest {
            self.nearest = distance;
            return false;
        }
        if self.nearest < range && distance > self.nearest + range {
            self.missed = self.missed.wrapping_add(1);
            self.nearest = distance;
        }
        self.missed >= 3
    }
    fn passed(&mut self) {
        self.watch.reset();
        self.nearest = f64::MAX;
        self.missed = 0;
    }
}
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}
fn horizontal_sqr(v: Vec3) -> f64 {
    v.x * v.x + v.z * v.z
}
fn length(v: Vec3) -> f64 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}
fn is_sharp(points: &[Vec3], corner: usize) -> bool {
    if corner == 0 || corner >= points.len().saturating_sub(1) {
        return corner >= points.len().saturating_sub(1);
    }
    let inbound = sub(points[corner], points[corner - 1]);
    let outbound = sub(points[corner + 1], points[corner]);
    let lengths = length(inbound) * length(outbound);
    lengths > 1.0e-9
        && (inbound.x * outbound.x + inbound.y * outbound.y + inbound.z * outbound.z) / lengths
            < 0.7
}
fn directional_input(direction: Vec3, frame_y_rot: f32) -> Input {
    if horizontal_sqr(direction) < 1.0e-6 {
        return Input::EMPTY;
    }
    let angle = super::walk_jump::wrap_degrees(
        super::walk_jump::vector_yaw(direction.x, direction.z)
            - super::walk_jump::wrap_degrees(frame_y_rot),
    );
    Input {
        forward: angle.abs() < 70.0,
        backward: angle.abs() > 110.0,
        left: angle < -20.0 && angle > -160.0,
        right: angle > 20.0 && angle < 160.0,
        ..Input::EMPTY
    }
}
fn distance_segment(point: Vec3, from: Vec3, to: Vec3) -> f64 {
    let along = sub(to, from);
    let length_sqr = along.x * along.x + along.y * along.y + along.z * along.z;
    if length_sqr.abs() < f64::from(1.0e-5_f32) {
        return point.distance(from);
    }
    let projected = (along.x * (point.x - from.x)
        + along.y * (point.y - from.y)
        + along.z * (point.z - from.z))
        / length_sqr;
    assert!(projected.is_finite(), "cannot project onto flight segment");
    let share = projected.clamp(0.0, 1.0);
    point.distance(Vec3::new(
        share.mul_add(along.x, from.x),
        share.mul_add(along.y, from.y),
        share.mul_add(along.z, from.z),
    ))
}
