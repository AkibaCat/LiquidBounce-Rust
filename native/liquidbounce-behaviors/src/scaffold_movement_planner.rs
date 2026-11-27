// SPDX-License-Identifier: GPL-3.0-or-later
//! ScaffoldMovementPlanner: support ranking, history and 8-way hysteresis.
use std::{cmp::Ordering, collections::VecDeque};
pub type Position = [f64; 3];
pub type BlockPos = [i32; 3];
pub type Bounds = [f64; 6];
const SURFACE_EPSILON: f64 = 1.0e-3;
const OVERLAP_HYSTERESIS: f64 = 0.02;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub position: Position,
    pub direction: Position,
}
impl Line {
    pub fn nearest(self, point: Position) -> Position {
        let delta = std::array::from_fn(|i| point[i] - self.position[i]);
        let t = dot(self.direction, delta) / dot(self.direction, self.direction);
        std::array::from_fn(|i| t.mul_add(self.direction[i], self.position[i]))
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SupportReference {
    pub block: BlockPos,
    pub offset_x: f64,
    pub offset_z: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub block: BlockPos,
    pub overlap: f64,
    pub surface_delta: f64,
    pub distance_squared: f64,
}
impl Candidate {
    /// Nonempty shape is checked by the native caller, independently of overlap.
    pub fn from_boxes(block: BlockPos, boxes: &[Bounds], player: Position, bounds: Bounds) -> Self {
        let mut surface_delta = f64::INFINITY;
        let mut overlap = 0.0;
        for shape in boxes {
            let min_x = f64::from(block[0]) + shape[0];
            let max_x = f64::from(block[0]) + shape[3];
            let max_y = f64::from(block[1]) + shape[4];
            let min_z = f64::from(block[2]) + shape[2];
            let max_z = f64::from(block[2]) + shape[5];
            let overlap_x = bounds[3].min(max_x) - bounds[0].max(min_x);
            let overlap_z = bounds[5].min(max_z) - bounds[2].max(min_z);
            if overlap_x <= 0.0 || overlap_z <= 0.0 {
                continue;
            }
            let delta = (bounds[1] - max_y).abs();
            let area = overlap_x * overlap_z;
            if delta + SURFACE_EPSILON < surface_delta {
                surface_delta = delta;
                overlap = area;
            } else if (delta - surface_delta).abs() <= SURFACE_EPSILON {
                overlap += area;
            }
        }
        let dx = player[0] - (f64::from(block[0]) + 0.5);
        let dz = player[2] - (f64::from(block[2]) + 0.5);
        Self {
            block,
            overlap,
            surface_delta,
            distance_squared: dx * dx + dz * dz,
        }
    }
    pub fn compare(self, other: Self) -> Ordering {
        if self.surface_delta + SURFACE_EPSILON < other.surface_delta {
            Ordering::Less
        } else if other.surface_delta + SURFACE_EPSILON < self.surface_delta {
            Ordering::Greater
        } else if self.overlap > other.overlap + OVERLAP_HYSTERESIS {
            Ordering::Less
        } else if self.overlap + OVERLAP_HYSTERESIS < other.overlap {
            Ordering::Greater
        } else if self.distance_squared < other.distance_squared {
            Ordering::Less
        } else if self.distance_squared > other.distance_squared {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
    pub fn stable_against(self, best: Self) -> bool {
        self.surface_delta <= best.surface_delta + SURFACE_EPSILON
            && self.overlap + OVERLAP_HYSTERESIS >= best.overlap
    }
}
fn dot(a: Position, b: Position) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn wrap(angle: f32) -> f32 {
    let mut value = angle % 360.0;
    if value >= 180.0 {
        value -= 360.0;
    }
    if value < -180.0 {
        value += 360.0;
    }
    value
}
pub fn movement_angle(mut yaw: f32, [forward, backward, left, right]: [bool; 4]) -> f32 {
    let multiplier = if backward && !forward {
        yaw += 180.0;
        -0.5
    } else if forward && !backward {
        0.5
    } else {
        1.0
    };
    if left && !right {
        yaw -= 90.0 * multiplier;
    }
    if right && !left {
        yaw += 90.0 * multiplier;
    }
    yaw
}
#[derive(Clone, Debug)]
pub struct Planner {
    pub history: VecDeque<BlockPos>,
    pub last_position: Option<BlockPos>,
    pub support: Option<SupportReference>,
    pub direction_angle: f32,
}
impl Default for Planner {
    fn default() -> Self {
        Self {
            history: VecDeque::new(),
            last_position: None,
            support: None,
            direction_angle: f32::NAN,
        }
    }
}
impl Planner {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn track(&mut self, block: BlockPos) {
        if self.history.back() == Some(&block) {
            return;
        }
        while self.history.len() >= 4 {
            self.history.pop_front();
        }
        self.history.push_back(block);
    }
    pub fn choose_angle(&mut self, current: f32) -> f32 {
        if !self.direction_angle.is_nan() && wrap(self.direction_angle - current).abs() <= 30.0 {
            return self.direction_angle;
        }
        let number = (current / 180.0 * 4.0 + 4.0).round_ties_even();
        self.direction_angle = wrap((number - 4.0) / 4.0 * 180.0);
        self.direction_angle
    }
    fn history_line(&self) -> Option<Line> {
        let last = self.history.back()?;
        let previous = self.history.get(self.history.len().checked_sub(2)?)?;
        let a = [
            f64::from(previous[0]) + 0.5,
            f64::from(previous[1]),
            f64::from(previous[2]) + 0.5,
        ];
        let b = [
            f64::from(last[0]) + 0.5,
            f64::from(last[1]),
            f64::from(last[2]) + 0.5,
        ];
        let delta: Position = std::array::from_fn(|i| b[i] - a[i]);
        let length = dot(delta, delta).sqrt();
        let direction = if length < 1.0e-5 {
            [0.0; 3]
        } else {
            delta.map(|v| v / length)
        };
        Some(Line {
            position: std::array::from_fn(|i| (a[i] + b[i]) * 0.5),
            direction,
        })
    }
    pub fn line(
        &mut self,
        player: Position,
        current_angle: f32,
        candidates: &[Candidate],
        mut direction_from_rotation: impl FnMut(f32) -> Position,
    ) -> Option<Line> {
        // Direction state changes even when no support candidate exists.
        let direction = direction_from_rotation(self.choose_angle(current_angle));
        let Some(mut best) = candidates.first().copied() else {
            self.support = None;
            self.last_position = None;
            return None;
        };
        for &candidate in &candidates[1..] {
            if candidate.compare(best) == Ordering::Less {
                best = candidate;
            }
        }
        let preferred = [self.history.back().copied(), self.last_position]
            .into_iter()
            .flatten()
            .filter_map(|position| candidates.iter().find(|c| c.block == position).copied())
            .find(|candidate| candidate.stable_against(best));
        let chosen = preferred.unwrap_or(best);
        self.last_position = Some(chosen.block);
        let support = SupportReference {
            block: chosen.block,
            offset_x: player[0] - (f64::from(chosen.block[0]) + 0.5),
            offset_z: player[2] - (f64::from(chosen.block[2]) + 0.5),
        };
        self.support = Some(support);
        let anchor = self
            .history_line()
            .filter(|line| dot(line.direction, direction) >= 0.5)
            .map_or_else(
                || {
                    [
                        f64::from(support.block[0]) + 0.5 + support.offset_x,
                        player[1],
                        f64::from(support.block[2]) + 0.5 + support.offset_z,
                    ]
                },
                |line| line.nearest(player),
            );
        Some(Line {
            position: [anchor[0], player[1], anchor[2]],
            direction,
        })
    }
}
