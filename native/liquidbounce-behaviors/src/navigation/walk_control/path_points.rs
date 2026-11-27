// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent PathPoints.kt geometry. Exact body simulations live in floor_prediction.
use super::{SHORTCUT_STEPS, directional_input, distance_segment, horizontal, subtract};
use crate::navigation::{
    geometry::{Vec3, java_min},
    input::Input,
    walk::{
        WalkState,
        route::{Move, Route, Step},
    },
    walk_jump::{vector_yaw, wrap_degrees},
};
pub trait StepControl {
    fn builds(&self) -> bool;
    fn is_level_walk(&self) -> bool;
    fn is_rising_bridge(&self) -> bool;
    fn is_plain(&self) -> bool;
    fn past_edge(&self, position: Vec3) -> f64;
    fn outward(&self, velocity: Vec3) -> f64;
    fn hold_back(&self, position: Vec3, velocity: Vec3, frame_y_rot: f32) -> Input;
}
impl StepControl for Step {
    fn builds(&self) -> bool {
        !self.breaks.is_empty() || !self.places.is_empty()
    }
    fn is_level_walk(&self) -> bool {
        matches!(self.movement, Move::Traverse | Move::Diagonal) && !self.shift && !self.edge
    }
    fn is_rising_bridge(&self) -> bool {
        self.movement == Move::Ascend
            && self.breaks.is_empty()
            && self.opens.is_empty()
            && self.jump.is_none()
            && self.places.len() == 2
            && self.places[0] == self.to.offset(0, -2, 0)
            && self.places[1] == self.to.offset(0, -1, 0)
    }
    fn is_plain(&self) -> bool {
        self.is_level_walk()
            && self.jump.is_none()
            && !self.builds()
            && self.opens.is_empty()
            && !self.is_under_water
    }
    fn past_edge(&self, position: Vec3) -> f64 {
        let dx = self.to.x.wrapping_sub(self.from.x);
        let dz = self.to.z.wrapping_sub(self.from.z);
        let from = Vec3::bottom_center(self.from);
        let edge = Vec3::new(
            from.x + f64::from(dx) * 0.5,
            from.y,
            from.z + f64::from(dz) * 0.5,
        );
        (position.x - edge.x) * f64::from(dx) + (position.z - edge.z) * f64::from(dz)
    }
    fn outward(&self, velocity: Vec3) -> f64 {
        velocity.x * f64::from(self.to.x.wrapping_sub(self.from.x))
            + velocity.z * f64::from(self.to.z.wrapping_sub(self.from.z))
    }
    fn hold_back(&self, position: Vec3, velocity: Vec3, frame_y_rot: f32) -> Input {
        let past = self.past_edge(position);
        if past <= 0.0 || self.outward(velocity) <= 0.0 && past < 0.3 {
            return Input::EMPTY;
        }
        directional_input(
            Vec3::new(
                f64::from(self.from.x.wrapping_sub(self.to.x)),
                f64::from(self.from.y.wrapping_sub(self.to.y)),
                f64::from(self.from.z.wrapping_sub(self.to.z)),
            ),
            frame_y_rot,
        )
    }
}
pub fn run_end(steps: &[Step], start: usize) -> usize {
    let mut end = start;
    while end < steps.len() && steps[end].is_bridge() {
        end += 1;
    }
    end
}
pub fn continues_straight(step: &Step, next: &Step) -> bool {
    next.to.x.wrapping_sub(next.from.x) == step.to.x.wrapping_sub(step.from.x)
        && next.to.z.wrapping_sub(next.from.z) == step.to.z.wrapping_sub(step.from.z)
        && (step.to.x != step.from.x || step.to.z != step.from.z)
}
pub fn has_passed(step: &Step, position: Vec3) -> bool {
    let from = Vec3::bottom_center(step.from);
    let dx = step.feet.x - from.x;
    let dz = step.feet.z - from.z;
    let length = (dx * dx + dz * dz).sqrt();
    if length == 0.0 || (position.y - step.feet.y).abs() >= 1.25 {
        return false;
    }
    let along = ((position.x - from.x) * dx + (position.z - from.z) * dz) / length;
    let beside = ((position.x - from.x) * dz - (position.z - from.z) * dx).abs() / length;
    along >= length && beside <= 0.6
}
pub trait RouteControl {
    fn is_off_path(&self, index: usize, position: Vec3) -> bool;
    fn join_index(&self, position: Vec3) -> Option<usize>;
    fn look_ahead(&self, index: usize, target: Vec3, position: Vec3, distance: f64)
    -> Option<Vec3>;
    fn point_ahead(&self, index: usize, position: Vec3, distance: f64) -> Vec3;
    fn turn_ahead(&self, index: usize, position: Vec3, distance: f64) -> f32;
}
impl<B: WalkState> RouteControl for Route<B> {
    fn is_off_path(&self, index: usize, position: Vec3) -> bool {
        let Some(step) = self.steps().get(index) else {
            return false;
        };
        let points = self.points();
        let from = points[index];
        if position.y < java_min(from.y, step.feet.y) - 0.9 {
            return true;
        }
        if step
            .jump
            .as_ref()
            .is_some_and(|jump| distance_segment(position, jump.run_up, from) <= 1.5)
        {
            return false;
        }
        for i in index..(points.len() - 1).min(index + SHORTCUT_STEPS) {
            if distance_segment(position, points[i], points[i + 1]) <= 1.5 {
                return false;
            }
            if !self.steps()[i].is_plain() {
                break;
            }
        }
        true
    }
    fn join_index(&self, position: Vec3) -> Option<usize> {
        let mut best = None;
        let mut best_distance = f64::MAX;
        for index in 0..self.points().len().min(8) {
            if index > 0 && self.steps()[index - 1].builds() {
                break;
            }
            let point = self.points()[index];
            let distance = horizontal(position, point);
            if distance <= 1.0 && (position.y - point.y).abs() <= 1.0 && distance < best_distance {
                best = Some(index);
                best_distance = distance;
            }
        }
        best
    }
    fn look_ahead(
        &self,
        index: usize,
        target: Vec3,
        position: Vec3,
        distance: f64,
    ) -> Option<Vec3> {
        let points = self.points();
        let end = points.len().min(index + 1 + SHORTCUT_STEPS);
        let from = (index + 1..end)
            .find(|&i| points[i] == target)
            .unwrap_or(index + 1);
        (from..points.len().min(from + 4)).find_map(|i| {
            let dx = points[i].x - position.x;
            let dz = points[i].z - position.z;
            (dx * dx + dz * dz >= distance * distance).then_some(points[i])
        })
    }
    fn point_ahead(&self, index: usize, position: Vec3, distance: f64) -> Vec3 {
        point_ahead_tagged(self, index, position, distance).0
    }
    fn turn_ahead(&self, index: usize, position: Vec3, distance: f64) -> f32 {
        let mut from = position;
        let mut left = distance;
        let mut heading = f32::NAN;
        let mut sharpest = 0.0_f32;
        for i in index + 1..self.points().len() {
            let to = self.points()[i];
            let length = horizontal(from, to);
            if length > 0.3 {
                let way = subtract(to, from);
                let y_rot = vector_yaw(way.x, way.z);
                if !heading.is_nan() {
                    let angle = wrap_degrees(y_rot - heading).abs();
                    sharpest = if sharpest.is_nan() {
                        sharpest
                    } else if angle.is_nan() {
                        angle
                    } else {
                        sharpest.max(angle)
                    };
                }
                heading = y_rot;
            }
            left -= length;
            if left <= 0.0 {
                break;
            }
            from = to;
        }
        sharpest
    }
}

/// The tag is Some(step index) only for an existing feet object returned by
/// the source. Interpolation constructs a new Vec3 even at share zero or one.
pub fn point_ahead_tagged<B: WalkState>(
    route: &Route<B>,
    index: usize,
    position: Vec3,
    distance: f64,
) -> (Vec3, Option<usize>) {
    let mut from = position;
    let mut left = distance;
    for i in index + 1..route.points().len() {
        let to = route.points()[i];
        let climbs = matches!(
            route.steps()[i - 1].movement,
            Move::ClimbUp | Move::ClimbDown
        );
        let length = if climbs {
            from.distance(to)
        } else {
            horizontal(from, to)
        };
        if length >= left {
            let share = left / length;
            return (
                Vec3::new(
                    from.x + share * (to.x - from.x),
                    from.y + share * (to.y - from.y),
                    from.z + share * (to.z - from.z),
                ),
                None,
            );
        }
        left -= length;
        from = to;
    }
    (
        *route.points().last().expect("route has start point"),
        route.steps().len().checked_sub(1),
    )
}
