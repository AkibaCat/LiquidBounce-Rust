// SPDX-License-Identifier: GPL-3.0-or-later
//! Source FaceTargetPositionFactory geometry for Scaffold's eight aim modes.
pub type Point = [f64; 3];
pub fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: Point, b: f64) -> Point {
    a.map(|v| v * b)
}
fn dot(a: Point, b: Point) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn length(a: Point) -> f64 {
    dot(a, a).sqrt()
}
fn normalize(a: Point) -> Point {
    let n = length(a);
    if n < 1.0e-5 {
        [0.; 3]
    } else {
        a.map(|v| v / n)
    }
}
fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn fma(at: Point, t: f64, d: Point) -> Point {
    std::array::from_fn(|i| t.mul_add(d[i], at[i]))
}
fn distance(a: Point, b: Point) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}
fn zero(a: Point) -> bool {
    dot(a, a).abs() < 1.0e-5
}
fn wrap(angle: f32) -> f32 {
    let mut a = angle % 360.;
    if a >= 180. {
        a -= 360.;
    }
    if a < -180. {
        a += 360.;
    }
    a
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Center,
    Random,
    Stabilized,
    NearestRotation,
    ReverseYaw,
    DiagonalYaw,
    AngleYaw,
    EdgePoint,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    pub from: Point,
    pub to: Point,
}
impl Face {
    pub fn dimensions(self) -> Point {
        sub(self.to, self.from)
    }
    pub fn area(self) -> f64 {
        let d = self.dimensions();
        d[0] * d[1] + d[1] * d[2] + d[0] * d[2]
    }
    pub fn center(self) -> Point {
        let d = self.dimensions();
        std::array::from_fn(|i| self.from[i] + 0.5 * d[i])
    }
    pub fn trim(self) -> Self {
        let offset = scale(self.dimensions(), 0.15);
        let low = add(self.from, offset);
        let high = sub(self.to, offset);
        let center = self.center();
        Self {
            from: std::array::from_fn(|i| if low[i] > high[i] { center[i] } else { low[i] }),
            to: std::array::from_fn(|i| if low[i] > high[i] { center[i] } else { high[i] }),
        }
    }
    fn offset(self, p: Point) -> Self {
        Self {
            from: add(self.from, p),
            to: add(self.to, p),
        }
    }
    fn axes(self) -> (Point, Point) {
        let d = self.dimensions();
        if d[0] == 0. {
            ([0., d[1], 0.], [0., 0., d[2]])
        } else if d[1] == 0. {
            ([d[0], 0., 0.], [0., 0., d[2]])
        } else {
            assert_eq!(d[2], 0.);
            ([0., d[1], 0.], [d[0], 0., 0.])
        }
    }
    fn edges(self) -> Vec<Linear> {
        let (a, b) = self.axes();
        let mut edges = Vec::new();
        for d in [a, b] {
            if !zero(d) {
                edges.push(Linear {
                    anchor: self.from,
                    direction: d,
                    segment: true,
                });
                edges.push(Linear {
                    anchor: self.to,
                    direction: scale(d, -1.),
                    segment: true,
                });
            }
        }
        edges
    }
    fn plane(self) -> Plane {
        let (a, b) = self.axes();
        Plane::from_params(self.from, a, b)
    }
    fn nearest(self, line: Linear) -> Point {
        let intersection = self.plane().intersection(line);
        let edges = self.edges();
        let center = self.center();
        if let Some(point) = intersection {
            if edges.is_empty()
                || edges.iter().all(|edge| {
                    let at = edge.at(0.5);
                    dot(sub(at, point), sub(at, center)) > 0.
                })
            {
                return point;
            }
        }
        let mut nearest = None;
        let mut best = f64::INFINITY;
        for edge in edges {
            if let Some((a, b)) = edge.nearest_pair(line) {
                let d = distance(a, b);
                if nearest.is_none() || d < best {
                    nearest = Some(a);
                    best = d;
                }
            }
        }
        nearest.or(intersection).unwrap_or(center)
    }
    fn coerce(self, line: Linear) -> Option<Linear> {
        let mut candidates: Vec<_> = self
            .edges()
            .into_iter()
            .filter_map(|edge| line.nearest_pair(edge).map(|(a, b)| (distance(a, b), b)))
            .collect();
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
        let a = candidates.first()?.1;
        let b = candidates.get(1)?.1;
        let direction = sub(b, a);
        (!zero(direction)).then_some(Linear {
            anchor: a,
            direction,
            segment: true,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Linear {
    pub anchor: Point,
    pub direction: Point,
    pub segment: bool,
}
impl Linear {
    fn at(self, t: f64) -> Point {
        fma(self.anchor, t, self.direction)
    }
    fn project(self, t: f64) -> f64 {
        if !t.is_finite() {
            f64::NAN
        } else if self.segment {
            t.clamp(0., 1.)
        } else {
            t
        }
    }
    fn normal_parameter(self, t: f64) -> Option<f64> {
        (t.is_finite() && (!self.segment || (-1e-9..=1. + 1e-9).contains(&t)))
            .then(|| self.project(t))
    }
    pub fn nearest(self, point: Point) -> Point {
        let t = self.project(
            dot(self.direction, sub(point, self.anchor)) / dot(self.direction, self.direction),
        );
        assert!(!t.is_nan(), "Scaffold line cannot project this point");
        self.at(t)
    }
    fn nearest_pair(self, other: Self) -> Option<(Point, Point)> {
        let delta = sub(self.anchor, other.anchor);
        let a = dot(self.direction, self.direction);
        let b = dot(self.direction, other.direction);
        let c = dot(other.direction, other.direction);
        let d = dot(self.direction, delta);
        let e = dot(other.direction, delta);
        let determinant = a * c - b * b;
        let mut best = None;
        let mut best_distance = f64::INFINITY;
        let mut candidate = |u, v| {
            let (Some(u), Some(v)) = (self.normal_parameter(u), other.normal_parameter(v)) else {
                return;
            };
            let d: Point =
                std::array::from_fn(|i| delta[i] + self.direction[i] * u - other.direction[i] * v);
            let distance = dot(d, d);
            if best.is_none() || distance < best_distance - 1e-9 {
                best = Some((u, v));
                best_distance = distance;
            }
        };
        if determinant > 1e-9 * a * c {
            candidate((b * e - c * d) / determinant, (a * e - b * d) / determinant);
        }
        if self.segment {
            for u in [0., 1.] {
                candidate(u, other.project((b * u + e) / c));
            }
        }
        if other.segment {
            for v in [0., 1.] {
                candidate(self.project((b * v - d) / a), v);
            }
        }
        candidate(self.project(-d / a), 0.);
        candidate(0., other.project(e / c));
        best.map(|(a, b)| (self.at(a), other.at(b)))
    }
}
#[derive(Clone, Copy, Debug)]
struct Plane {
    position: Point,
    normal: Point,
}
impl Plane {
    fn from_params(position: Point, a: Point, b: Point) -> Self {
        let normal = normalize(cross(a, b));
        assert!(!zero(normal), "Scaffold plane must have nonzero normal");
        Self { position, normal }
    }
    fn intersection(self, line: Linear) -> Option<Point> {
        let d = dot(self.position, self.normal);
        let e = dot(line.direction, self.normal);
        if e.abs() < 1.0e-5 {
            return None;
        }
        let t = (d - dot(line.anchor, self.normal)) / e;
        Some(line.at(line.normal_parameter(t)?))
    }
    fn intersection_plane(self, other: Self) -> Option<Linear> {
        let first = other.normal;
        let second = self.normal;
        let direction = cross(first, second);
        let squared = dot(direction, direction);
        if squared.abs() < 1e-5 {
            return None;
        }
        let first_distance = dot(first, other.position);
        let second_distance = dot(second, self.position);
        let point = scale(
            add(
                scale(cross(second, direction), first_distance),
                scale(cross(direction, first), second_distance),
            ),
            1. / squared,
        );
        Some(Linear {
            anchor: point,
            direction,
            segment: false,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub position: Point,
    pub eye: Point,
    pub rotation: [f32; 2],
    pub moving: bool,
}
pub trait Math {
    fn rotation_direction(&mut self, rotation: [f32; 2]) -> Point;
    fn yaw_axis(&mut self, yaw: f32) -> Point;
    fn point_yaw(&mut self, delta: Point) -> f32;
    fn random_between(&mut self, from: f64, to: f64) -> f64;
}
fn nearest(face: Face, target: Point, location: Location, math: &mut impl Math) -> Point {
    if face.area().abs() < 1e-5 {
        return face.from;
    }
    let line = Linear {
        anchor: sub(location.eye, target),
        direction: math.rotation_direction(location.rotation),
        segment: false,
    };
    face.nearest(line)
}
fn stabilized(
    face: Face,
    target: Point,
    location: Location,
    optimal: Option<Linear>,
    math: &mut impl Math,
) -> Point {
    let trimmed = face.trim().offset(target);
    let target_face = optimal
        .and_then(|optimal| {
            let nearest = optimal.nearest(location.position);
            let direction = normalize(sub(location.position, nearest));
            let from_player = Linear {
                anchor: location.eye,
                direction: optimal.direction,
                segment: false,
            };
            let collision = trimmed.plane().intersection(from_player)?;
            let b = fma(location.position, 2., direction);
            // AABB constructor sorts the crop's bounds before clamping each point.
            let a = [collision[0], location.position[1] - 2., collision[2]];
            let b = [b[0], location.position[1] + 1., b[2]];
            let clamp =
                |p: Point| std::array::from_fn(|i| p[i].clamp(a[i].min(b[i]), a[i].max(b[i])));
            let clipped = Face {
                from: clamp(trimmed.from),
                to: clamp(trimmed.to),
            };
            (clipped.area() >= 0.0001).then_some(clipped)
        })
        .unwrap_or(trimmed);
    nearest(
        target_face.offset(scale(target, -1.)),
        target,
        location,
        math,
    )
}
fn closest_yaw(segment: Linear, target: f32, eye: Point, math: &mut impl Math) -> Point {
    let start = math.point_yaw(sub(segment.anchor, eye));
    let end = math.point_yaw(sub(segment.at(1.), eye));
    let diff = wrap(end - start);
    let target = wrap(target - start);
    let t = if diff != 0. { target / diff } else { 0. };
    segment.at(f64::from(t.clamp(0., 1.)))
}
fn aim_yaw(
    face: Face,
    target: Point,
    location: Location,
    angle: f32,
    math: &mut impl Math,
) -> Option<Point> {
    if face.area().abs() < 1e-5 {
        return Some(face.from);
    }
    let eye = sub(location.eye, target);
    let yaw = wrap(location.rotation[0]);
    let high = wrap(yaw + angle);
    let low = wrap(yaw - angle);
    let plane = face.plane();
    let mut sample = |yaw| {
        let direction = math.yaw_axis(yaw);
        let yaw_plane = Plane::from_params(eye, direction, [0., 1., 0.]);
        let intersection = plane.intersection_plane(yaw_plane)?;
        let segment = face.coerce(intersection)?;
        let point = closest_yaw(segment, yaw, eye, math);
        let tolerance = wrap(math.point_yaw(sub(point, eye)) - yaw).abs();
        Some((point, tolerance))
    };
    let high = sample(high);
    let low = sample(low);
    match (high, low) {
        (Some((a, x)), Some((b, y))) if x <= 5. && y <= 5. => Some(if x < y { a } else { b }),
        (Some((a, x)), _) if x <= 5. => Some(a),
        (_, Some((b, y))) if y <= 5. => Some(b),
        _ => None,
    }
}
pub fn sample(
    mode: Mode,
    face: Face,
    target: Point,
    location: Location,
    optimal: Option<Linear>,
    math: &mut impl Math,
) -> Point {
    match mode {
        Mode::Center => face.center(),
        Mode::Random => {
            let face = face.trim();
            std::array::from_fn(|i| {
                if face.from[i] == face.to[i] {
                    face.from[i]
                } else {
                    math.random_between(face.from[i], face.to[i])
                }
            })
        }
        Mode::NearestRotation => nearest(face.trim(), target, location, math),
        Mode::Stabilized => stabilized(face, target, location, optimal, math),
        Mode::ReverseYaw | Mode::DiagonalYaw | Mode::AngleYaw => {
            let face = face.trim();
            let angle = match mode {
                Mode::ReverseYaw => 180.,
                Mode::DiagonalYaw => 75.,
                _ => 45.,
            };
            if location.moving {
                if let Some(point) = aim_yaw(face, target, location, angle, math) {
                    return point;
                }
            }
            nearest(face, target, location, math)
        }
        Mode::EdgePoint => {
            let face = face.trim();
            if !location.moving {
                return nearest(face, target, location, math);
            }
            let player = sub(location.position, target);
            let mut point = face.from;
            let mut best = f64::NEG_INFINITY;
            for x in [face.from[0], face.to[0]] {
                for y in [face.from[1], face.to[1]] {
                    for z in [face.from[2], face.to[2]] {
                        let p = [x, y, z];
                        let d = distance(p, player);
                        if d > best {
                            point = p;
                            best = d;
                        }
                    }
                }
            }
            point
        }
    }
}
