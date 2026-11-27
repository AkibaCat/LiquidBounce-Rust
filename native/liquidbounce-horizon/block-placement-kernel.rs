// SPDX-License-Identifier: GPL-3.0-or-later
//! TargetFinding.kt: deterministic target/face ranking over actual host queries.
use std::cmp::Ordering;
pub type Position = [f64; 3];
pub type BlockPos = [i32; 3];
pub type Direction = [i32; 3];
pub const DIRECTIONS: [Direction; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub position: Position,
    pub eye_height: f32,
    pub rotation: [f32; 2],
    pub moving: bool,
}
impl Location {
    pub fn eye(self) -> Position {
        [
            self.position[0],
            self.position[1] + f64::from(self.eye_height),
            self.position[2],
        ]
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    pub from: Position,
    pub to: Position,
}
impl Face {
    pub fn center(self) -> Position {
        std::array::from_fn(|i| self.from[i] + (self.to[i] - self.from[i]) * 0.5)
    }
    pub fn of(bounds: [f64; 6], direction: Direction) -> Self {
        let mut from = [bounds[0], bounds[1], bounds[2]];
        let mut to = [bounds[3], bounds[4], bounds[5]];
        let axis = direction
            .iter()
            .position(|&x| x != 0)
            .expect("axis direction");
        if direction[axis] > 0 {
            from[axis] = to[axis];
        } else {
            to[axis] = from[axis];
        }
        Self { from, to }
    }
    pub fn trim(self, scale: f64) -> Self {
        let center = self.center();
        let low: Position =
            std::array::from_fn(|i| self.from[i] + (self.to[i] - self.from[i]) * scale);
        let high: Position =
            std::array::from_fn(|i| self.to[i] - (self.to[i] - self.from[i]) * scale);
        Self {
            from: std::array::from_fn(|i| if low[i] > high[i] { center[i] } else { low[i] }),
            to: std::array::from_fn(|i| if low[i] > high[i] { center[i] } else { high[i] }),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub interacted: BlockPos,
    pub placed: BlockPos,
    pub direction: Direction,
    pub point: Position,
    pub min_y: f64,
    pub rotation: [f32; 2],
}
impl Target {
    pub fn matches(
        &self,
        is_block: bool,
        pos: BlockPos,
        direction: Direction,
        point: Position,
    ) -> bool {
        is_block
            && pos == self.interacted
            && direction == self.direction
            && point[1] >= self.min_y - 1.0e-7
    }
}
pub struct Options<'a> {
    pub location: Location,
    pub offsets: &'a [BlockPos],
    pub range: f64,
    pub consider_facing_away: bool,
}
pub trait Host {
    fn holds_solid(&mut self, pos: BlockPos) -> bool;
    /// The actual player's contextual outline, in block-local coordinates.
    fn outline(&mut self, pos: BlockPos) -> Vec<[f64; 6]>;
    fn priority(&mut self, pos: BlockPos) -> f64;
    fn sample(&mut self, face: Face, pos: BlockPos, location: Location) -> Option<Position>;
    fn looking_at(&mut self, point: Position, eye: Position) -> [f32; 2] {
        looking_at(point, eye)
    }
    /// BlockPlaceContext, updatePlacementContext, clickedPos and canPlace only.
    fn resolves_at(
        &mut self,
        clicked: BlockPos,
        direction: Direction,
        point: Position,
        inside: bool,
        target: BlockPos,
    ) -> bool;
}
fn add(pos: BlockPos, offset: BlockPos) -> BlockPos {
    std::array::from_fn(|i| pos[i].wrapping_add(offset[i]))
}
fn compare(a: f64, b: f64) -> Ordering {
    if a < b {
        Ordering::Less
    } else if a > b {
        Ordering::Greater
    } else {
        let bits = |x: f64| {
            if x.is_nan() {
                f64::NAN.to_bits() as i64
            } else {
                x.to_bits() as i64
            }
        };
        bits(a).cmp(&bits(b))
    }
}
fn wrap(mut x: f32) -> f32 {
    x %= 360.;
    if x >= 180. {
        x -= 360.;
    }
    if x < -180. {
        x += 360.;
    }
    x
}
pub fn looking_at(point: Position, from: Position) -> [f32; 2] {
    let [x, y, z] = std::array::from_fn(|i| point[i] - from[i]);
    [
        wrap(z.atan2(x).to_degrees() as f32 - 90.),
        wrap(-(y.atan2((x * x + z * z).sqrt()).to_degrees() as f32)),
    ]
}
fn contains(boxes: &[[f64; 6]], point: Position) -> bool {
    boxes
        .iter()
        .any(|b| (0..3).all(|i| point[i] >= b[i] && point[i] < b[i + 3]))
}
fn face_point(
    host: &mut impl Host,
    pos: BlockPos,
    direction: Direction,
    boxes: &[[f64; 6]],
    options: &Options<'_>,
) -> Option<(Face, Position)> {
    let axis = direction.iter().position(|&x| x != 0)?;
    let normal = f64::from(direction[axis]);
    let eye = options.location.eye();
    let eye_normal = (eye[axis] - f64::from(pos[axis])) * normal;
    let mut best: Option<(Face, Position)> = None;
    for &bounds in boxes {
        let face = Face::of(bounds, direction);
        if !options.consider_facing_away && eye_normal - face.from[axis] * normal < 0. {
            continue;
        }
        let Some(point) = host.sample(face, pos, options.location) else {
            continue;
        };
        let priority = (point[axis] - 0.5).powi(2);
        if best.is_none_or(|(_, old)| {
            compare(priority, (old[axis] - 0.5).powi(2)).then_with(|| compare(point[1], old[1]))
                == Ordering::Greater
        }) {
            best = Some((face, point));
        }
    }
    best
}
fn best_plan(host: &mut impl Host, pos: BlockPos, options: &Options<'_>) -> Option<Target> {
    let eye = options.location.eye();
    let target_boxes = host.outline(pos);
    let mut best = None;
    let mut best_facing = f64::NEG_INFINITY;
    for direction in DIRECTIONS {
        let neighbour = std::array::from_fn(|i| pos[i].wrapping_sub(direction[i]));
        // Vanilla Direction order, neighbour first and target second, decides ties.
        let neighbours = host.outline(neighbour);
        for (clicked, boxes) in [
            (neighbour, neighbours.as_slice()),
            (pos, target_boxes.as_slice()),
        ] {
            let Some((face, local)) = face_point(host, clicked, direction, boxes, options) else {
                continue;
            };
            let point = std::array::from_fn(|i| local[i] + f64::from(clicked[i]));
            let inside = contains(
                boxes,
                std::array::from_fn(|i| eye[i] - f64::from(clicked[i])),
            );
            if !host.resolves_at(clicked, direction, point, inside, pos) {
                continue;
            }
            let to_eye: Position = std::array::from_fn(|i| eye[i] - point[i]);
            let distance = to_eye.iter().map(|x| x * x).sum::<f64>().sqrt();
            let facing = (0..3)
                .map(|i| to_eye[i] * f64::from(direction[i]))
                .sum::<f64>()
                / distance;
            if distance > options.range || (!options.consider_facing_away && facing < 0.) {
                continue;
            }
            if facing > best_facing {
                best_facing = facing;
                best = Some(Target {
                    interacted: clicked,
                    placed: pos,
                    direction,
                    point,
                    min_y: face.from[1] + f64::from(clicked[1]),
                    rotation: host.looking_at(point, eye),
                });
            }
        }
    }
    best
}
pub fn find(host: &mut impl Host, pos: BlockPos, options: &Options<'_>) -> Option<Target> {
    if host.holds_solid(pos) {
        return None;
    }
    let mut offsets = options.offsets.to_vec();
    offsets.sort_by(|a, b| compare(host.priority(add(pos, *b)), host.priority(add(pos, *a))));
    offsets
        .into_iter()
        .find_map(|offset| best_plan(host, add(pos, offset), options))
}
/// BlockPosOffsets.generateScaffoldOffsets; sort order is independent of set insertion.
pub fn offsets(radius: i32) -> Vec<BlockPos> {
    let mut result = Vec::new();
    for x in -radius..=radius {
        for z in -radius..=radius {
            for y in [-1, 0] {
                result.push([x, y, z]);
            }
        }
    }
    result.sort_by_key(|&[x, y, z]| {
        (
            i64::from(x) * i64::from(x) + i64::from(y) * i64::from(y) + i64::from(z) * i64::from(z),
            y,
            x,
            z,
        )
    });
    result
}

/// BlockPlacementTargetFindingOptions.leastBlockDistanceToPos.
pub fn distance_squared(boxes: &[[f64; 6]], pos: BlockPos, point: Position) -> f64 {
    if boxes.is_empty() {
        return (0..3)
            .map(|i| (f64::from(pos[i]) + 0.5 - point[i]).powi(2))
            .sum();
    }
    let p: Position = std::array::from_fn(|i| point[i] - f64::from(pos[i]));
    boxes
        .iter()
        .map(|b| box_distance_squared(*b, p))
        .fold(f64::INFINITY, f64::min)
}
fn box_distance_squared(bounds: [f64; 6], point: Position) -> f64 {
    (0..3)
        .map(|i| {
            let d = (bounds[i] - point[i]).max(0.).max(point[i] - bounds[i + 3]);
            d * d
        })
        .sum()
}
/// Unbounded Line.getNearestPointTo, retaining its marker ordering, epsilon
/// comparisons and fused final coordinate calculation.
pub fn line_distance_squared(
    boxes: &[[f64; 6]],
    pos: BlockPos,
    point: Position,
    delta: Position,
) -> f64 {
    const EPS: f64 = 1.0e-9;
    const MTH_EPS: f64 = 1.0e-5_f32 as f64;
    let norm: f64 = delta.iter().map(|x| x * x).sum();
    assert!(norm.abs() >= MTH_EPS, "Line direction should not be zero");
    let origin: Position = std::array::from_fn(|i| point[i] - f64::from(pos[i]));
    if boxes.is_empty() {
        let t = (0..3).map(|i| delta[i] * (0.5 - origin[i])).sum::<f64>() / norm;
        return (0..3)
            .map(|i| (delta[i].mul_add(t, origin[i]) - 0.5).powi(2))
            .sum();
    }
    let mut shape_best = None;
    for &bounds in boxes {
        let mut candidates = Vec::with_capacity(13);
        for i in 0..3 {
            if delta[i].abs() < MTH_EPS {
                continue;
            }
            for boundary in [bounds[i], bounds[i + 3]] {
                let marker = (boundary - origin[i]) / delta[i];
                if marker.is_finite() {
                    candidates.push(marker);
                }
            }
        }
        // Kotlin's insertion sort compares numerically, preserving +/-0 ties.
        for i in 1..candidates.len() {
            let value = candidates[i];
            let mut j = i;
            while j > 0 && candidates[j - 1] > value {
                candidates[j] = candidates[j - 1];
                j -= 1;
            }
            candidates[j] = value;
        }
        candidates.dedup_by(|a, b| *a == *b);
        let markers = candidates.clone();
        let mut start = f64::NEG_INFINITY;
        for end in markers.iter().copied().chain([f64::INFINITY]) {
            let sample = if start.is_finite() && end.is_finite() {
                if start < end - EPS {
                    (start + end) * 0.5
                } else {
                    f64::NAN
                }
            } else if start.is_finite() {
                start + 1.
            } else if end.is_finite() {
                end - 1.
            } else {
                0.
            };
            if !sample.is_nan() {
                let (mut a, mut b) = (0., 0.);
                for i in 0..3 {
                    let coordinate = origin[i] + delta[i] * sample;
                    let boundary = if coordinate < bounds[i] {
                        Some(bounds[i])
                    } else if coordinate > bounds[i + 3] {
                        Some(bounds[i + 3])
                    } else {
                        None
                    };
                    if let Some(boundary) = boundary {
                        a += delta[i] * delta[i];
                        b += delta[i] * (origin[i] - boundary);
                    }
                }
                let t = if a.abs() <= EPS { sample } else { -b / a };
                if a.abs() <= EPS
                    || (t.is_finite()
                        && (!start.is_finite() || t > start + EPS)
                        && (!end.is_finite() || t < end - EPS))
                {
                    candidates.push(t);
                }
            }
            start = end;
        }
        let mut best = None;
        for t in candidates {
            if !t.is_finite() {
                continue;
            }
            let distance = box_distance_squared(
                bounds,
                std::array::from_fn(|i| delta[i].mul_add(t, origin[i])),
            );
            if best.is_none_or(|old| distance < old - EPS) {
                best = Some(distance);
            }
        }
        let distance = best.expect("Unable to find nearest point on line");
        if shape_best.is_none_or(|old| distance < old - EPS) {
            shape_best = Some(distance);
        }
    }
    shape_best.expect("nonempty outline")
}
