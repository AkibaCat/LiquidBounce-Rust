// SPDX-License-Identifier: GPL-3.0-or-later
//! ProjectilePuncher targeting and RotationFinding geometry, ae0eb396.
//! Kept dependency free so the native world adapter uses the same decisions.
pub type Vec3 = [f64; 3];
pub type Bounds = [f64; 6];
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn scale(a: Vec3, b: f64) -> Vec3 {
    a.map(|v| v * b)
}
pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn length(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}
fn fma(a: Vec3, b: f64, c: Vec3) -> Vec3 {
    std::array::from_fn(|i| b.mul_add(c[i], a[i]))
}
pub fn inflate(b: Bounds, r: f64) -> Bounds {
    std::array::from_fn(|i| b[i] + if i < 3 { -r } else { r })
}
pub fn center(b: Bounds) -> Vec3 {
    std::array::from_fn(|i| (b[i] + b[i + 3]) * 0.5)
}
pub fn contains(b: Bounds, p: Vec3) -> bool {
    (0..3).all(|i| p[i] >= b[i] && p[i] < b[i + 3])
}
pub fn nearest(b: Bounds, p: Vec3) -> Vec3 {
    std::array::from_fn(|i| p[i].clamp(b[i], b[i + 3]))
}
pub fn distance_squared(b: Bounds, p: Vec3) -> f64 {
    let d = sub(p, nearest(b, p));
    dot(d, d)
}
/// MC AABB.clip: entering faces only, strict t<1 and 1e-7 edge tolerances.
pub fn clip(b: Bounds, from: Vec3, to: Vec3) -> Option<Vec3> {
    let d = sub(to, from);
    let mut best = 1.;
    let mut hit = false;
    for axis in 0..3 {
        let side = if d[axis] > 1e-7 {
            b[axis]
        } else if d[axis] < -1e-7 {
            b[axis + 3]
        } else {
            continue;
        };
        let t = (side - from[axis]) / d[axis];
        if !(t > 0. && t < best) {
            continue;
        }
        let a = (axis + 1) % 3;
        let z = (axis + 2) % 3;
        let x = from[a] + t * d[a];
        let y = from[z] + t * d[z];
        if x > b[a] - 1e-7 && x < b[a + 3] + 1e-7 && y > b[z] - 1e-7 && y < b[z + 3] + 1e-7 {
            best = t;
            hit = true;
        }
    }
    hit.then(|| add(from, scale(d, best)))
}
pub fn first_hit(b: Bounds, from: Vec3, to: Vec3) -> Option<Vec3> {
    if contains(b, from) {
        clip(b, to, from)
    } else {
        clip(b, from, to)
    }
}
pub fn ray_hits(b: Bounds, eye: Vec3, view: Vec3) -> bool {
    let mut lo = 0f64;
    let mut hi = f64::INFINITY;
    for i in 0..3 {
        if view[i] != 0. {
            let a = (b[i] - eye[i]) / view[i];
            let z = (b[i + 3] - eye[i]) / view[i];
            lo = lo.max(a.min(z));
            hi = hi.min(a.max(z));
            if lo > hi {
                return false;
            }
        } else if eye[i] < b[i] || eye[i] > b[i + 3] {
            return false;
        }
    }
    true
}
fn cos_at(a: f64, c: f64) -> f64 {
    if c <= 0. { 1. } else { a / c.sqrt() }
}
fn edge_cos(eye: Vec3, view: Vec3, point: Vec3, width: Vec3) -> f64 {
    let p = sub(point, eye);
    let a = dot(p, view);
    let b = dot(width, view);
    let c = dot(p, p);
    let d = dot(p, width);
    let e = dot(width, width);
    let mut best = cos_at(a, c).max(cos_at(a + b, c + 2. * d + e));
    let t = (a * d - b * c) / (b * d - a * e);
    if t > 0. && t < 1. {
        best = best.max(cos_at(a + b * t, c + 2. * d * t + e * t * t));
    }
    best
}
/// Maximum cosine to the physical box, including its silhouette edges.
pub fn max_cos(b: Bounds, eye: Vec3, view: Vec3) -> f64 {
    if ray_hits(b, eye, view) {
        return 1.;
    }
    let fronts: [[bool; 2]; 3] = std::array::from_fn(|i| [eye[i] < b[i], eye[i] > b[i + 3]]);
    let mut best = -1f64;
    for axis in 0..3 {
        let a = (axis + 1) % 3;
        let z = (axis + 2) % 3;
        for ai in 0..2 {
            for zi in 0..2 {
                if fronts[a][ai] == fronts[z][zi] {
                    continue;
                }
                let mut p = [0.; 3];
                p[axis] = b[axis];
                p[a] = b[a + 3 * ai];
                p[z] = b[z + 3 * zi];
                let mut w = [0.; 3];
                w[axis] = b[axis + 3] - b[axis];
                best = best.max(edge_cos(eye, view, p, w));
            }
        }
    }
    best
}
fn wrap(v: f32) -> f32 {
    let mut v = v % 360.;
    if v >= 180. {
        v -= 360.;
    }
    if v < -180. {
        v += 360.;
    }
    v
}
pub fn looking(eye: Vec3, p: Vec3) -> [f32; 2] {
    let d = sub(p, eye);
    [
        wrap(d[2].atan2(d[0]).to_degrees() as f32 - 90.),
        wrap(-d[1].atan2((d[0] * d[0] + d[2] * d[2]).sqrt()).to_degrees() as f32),
    ]
}
fn difference(a: [f32; 2], b: [f32; 2]) -> f32 {
    let x = wrap(b[0] - a[0]);
    let y = b[1] - a[1];
    x * x + y * y
}
fn ray_first(b: Bounds, eye: Vec3, direction: Vec3) -> Option<Vec3> {
    let mut low = f64::NEG_INFINITY;
    let mut high = f64::INFINITY;
    for i in 0..3 {
        if direction[i] == 0. {
            if eye[i] < b[i] || eye[i] > b[i + 3] {
                return None;
            }
        } else {
            let a = (b[i] - eye[i]) / direction[i];
            let z = (b[i + 3] - eye[i]) / direction[i];
            low = low.max(a.min(z));
            high = high.min(a.max(z));
        }
    }
    if high < low || high < 0. {
        None
    } else {
        Some(add(
            eye,
            scale(direction, if low >= 0. { low } else { high }),
        ))
    }
}
fn projected_points(eye: Vec3, b: Bounds, visit: &mut impl FnMut(Vec3)) {
    let dir = sub(center(b), eye);
    let unit = if (dot(dir, dir) - 1.).abs() < 1e-4 {
        dir
    } else {
        scale(dir, 1. / length(dir))
    };
    let vertices: Vec<Vec3> = (0..8)
        .map(|i| {
            [
                b[3 * ((i >> 2) & 1)],
                b[1 + 3 * ((i >> 1) & 1)],
                b[2 + 3 * (i & 1)],
            ]
        })
        .collect();
    let mut origin = None;
    let mut distance = f64::INFINITY;
    for v in &vertices {
        let p = add(eye, scale(dir, dot(sub(*v, eye), dir) / dot(dir, dir)));
        let d = dot(sub(p, eye), sub(p, eye));
        if d < distance {
            distance = d;
            origin = Some(p);
        }
    }
    let p = origin.unwrap();
    let origin = std::array::from_fn(|i| p[i] + 0.1 * (eye[i] - p[i]));
    let yaw = unit[2].atan2(unit[0]) as f32;
    let pitch = unit[1].atan2((unit[0] * unit[0] + unit[2] * unit[2]).sqrt()) as f32;
    let (sy, cy) = (f64::from(yaw).sin() as f32, f64::from(yaw).cos() as f32);
    let (sp, cp) = (f64::from(pitch).sin() as f32, f64::from(pitch).cos() as f32);
    // JOML Matrix3f products and Vector3f conversion stay float throughout.
    let to = [
        [cy * cp, -cy * sp, -sy],
        [sp, cp, 0.],
        [sy * cp, -sy * sp, cy],
    ];
    let back = [
        [cp * cy, sp, cp * sy],
        [-sp * cy, cp, -sp * sy],
        [-sy, 0., cy],
    ];
    let mul = |m: [[f32; 3]; 3], v: [f32; 3]| -> Vec3 {
        std::array::from_fn(|i| f64::from(m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2]))
    };
    let mut low = [0f32; 2];
    let mut high = [0f32; 2];
    for v in vertices {
        let line = sub(v, eye);
        let denominator = dot(line, unit);
        if denominator.abs() < 1e-5 {
            continue;
        }
        let hit = add(
            eye,
            scale(line, (dot(origin, unit) - dot(eye, unit)) / denominator),
        );
        let v = mul(back, sub(hit, origin).map(|v| v as f32));
        for (i, a) in [1, 2].into_iter().enumerate() {
            low[i] = low[i].min(v[a] as f32);
            high[i] = high[i].max(v[a] as f32);
        }
    }
    let frame = mul(to, [0., low[0], low[1]]);
    let from = std::array::from_fn(|i| f64::from(frame[i] as f32 + origin[i] as f32));
    let dy = mul(to, [0., high[0] - low[0], 0.]);
    let dz = mul(to, [0., 0., high[1] - low[1]]);
    let a = dot(dy, dy).abs() < 1e-5;
    let z = dot(dz, dz).abs() < 1e-5;
    let (na, nz) = match (a, z) {
        (true, true) => (1, 1),
        (true, false) => (1, 256),
        (false, true) => (256, 1),
        _ => {
            let na = ((256. / (length(dz) / length(dy))).sqrt().floor() as i32).clamp(1, 256);
            (na, (256 / na).max(1))
        }
    };
    for a in 0..na {
        for z in 0..nz {
            let p = fma(
                fma(from, (f64::from(a) + 0.5) / f64::from(na), dy),
                (f64::from(z) + 0.5) / f64::from(nz),
                dz,
            );
            let end = std::array::from_fn(|i| p[i] + -100. * (eye[i] - p[i]));
            if let Some(p) = clip(b, eye, end) {
                visit(p);
            }
        }
    }
}
fn scan(eye: Vec3, b: Bounds, visit: &mut impl FnMut(Vec3)) {
    if !contains(b, eye) {
        projected_points(eye, b, visit);
        return;
    }
    // Direction.entries: DOWN, UP, NORTH, SOUTH, WEST, EAST.
    for (axis, upper) in [
        (1, false),
        (1, true),
        (2, false),
        (2, true),
        (0, false),
        (0, true),
    ] {
        let axes: Vec<_> = (0..3).filter(|a| *a != axis).collect();
        let (a, z) = (axes[0], axes[1]);
        let da = b[a + 3] - b[a];
        let dz = b[z + 3] - b[z];
        let spacing = 0.1f64.max((da * dz / 100.).sqrt());
        let na = (da / spacing).ceil().max(1.) as i32;
        let nz = (dz / spacing).ceil().max(1.) as i32;
        for ia in 0..na {
            for iz in 0..nz {
                let mut p = [b[0], b[1], b[2]];
                p[axis] = b[axis + if upper { 3 } else { 0 }];
                p[a] = da.mul_add((f64::from(ia) + 0.5) / f64::from(na), b[a]);
                p[z] = dz.mul_add((f64::from(iz) + 0.5) / f64::from(nz), b[z]);
                visit(p);
            }
        }
    }
}
/// Outline-only raytraceBox with zero wall range and LeastDifferencePreference.
pub fn raytrace_box(
    eye: Vec3,
    b: Bounds,
    range: f64,
    base: [f32; 2],
    view: Vec3,
    visible: &mut impl FnMut(Vec3, Vec3) -> bool,
) -> Option<[f32; 2]> {
    let mut preferred = fma(eye, range, view);
    if !contains(b, preferred)
        && let Some(hit) = ray_first(b, eye, sub(preferred, eye))
        && dot(sub(hit, eye), sub(hit, eye)) < range * range
    {
        preferred = hit;
    }
    if let Some(hit) = first_hit(b, eye, preferred)
        && visible(eye, hit)
        && dot(sub(hit, eye), sub(hit, eye)) < range * range
    {
        return Some(looking(eye, hit));
    }
    let mut best = None;
    let mut score = f32::INFINITY;
    let mut consider = |p| {
        let Some(hit) = first_hit(b, eye, fma(eye, 2., sub(p, eye))) else {
            return;
        };
        if !visible(eye, hit) || dot(sub(hit, eye), sub(hit, eye)) >= range * range {
            return;
        }
        let rotation = looking(eye, p);
        let difference = difference(base, rotation);
        if best.is_none() || score > difference {
            score = difference;
            best = Some(rotation);
        }
    };
    consider(preferred);
    consider(nearest(b, eye));
    scan(eye, b, &mut consider);
    best
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub id: u32,
    pub kind: u8,
    pub within_border: bool,
    pub position: Vec3,
    pub previous: Vec3,
    pub physical: Bounds,
    pub pick_radius: f64,
    pub dimensions: [f64; 2],
}
impl Candidate {
    pub fn pick_box(self) -> Bounds {
        inflate(self.physical, self.pick_radius)
    }
}
pub fn should_attack(
    c: Candidate,
    selected: u8,
    player_physical: Bounds,
    player_pick: Bounds,
) -> bool {
    if !c.within_border || selected & (1 << c.kind) == 0 {
        return false;
    }
    let velocity = sub(c.position, c.previous);
    if dot(velocity, velocity).abs() < 1e-5 {
        return false;
    }
    let toward = sub(center(player_physical), c.position);
    let toward = dot(toward, velocity) > 0.5 * length(velocity) * length(toward);
    let extended = inflate(player_pick, (c.pick_box()[3] - c.pick_box()[0]) / 2.);
    // The pinned source uses velocity*20 as the absolute second endpoint.
    toward
        || (!contains(extended, c.position)
            && clip(extended, c.position, scale(velocity, 20.)).is_some())
}
pub fn predicted_box(c: Candidate) -> Bounds {
    let p = sub(add(c.position, c.position), c.previous);
    let half = f64::from(c.dimensions[0] as f32 / 2.);
    [
        p[0] - half,
        p[1],
        p[2] - half,
        p[0] + half,
        p[1] + c.dimensions[1],
        p[2] + half,
    ]
}
