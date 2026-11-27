// SPDX-License-Identifier: GPL-3.0-or-later
//! Source ports of maxCosToBox, perspective point finding and both projectile solvers.
//! Kept independent of the engine so the same arithmetic runs in native world queries.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V(pub f64, pub f64, pub f64);
impl V {
    pub fn add(self, b: Self) -> Self {
        Self(self.0 + b.0, self.1 + b.1, self.2 + b.2)
    }
    pub fn sub(self, b: Self) -> Self {
        Self(self.0 - b.0, self.1 - b.1, self.2 - b.2)
    }
    pub fn scale(self, t: f64) -> Self {
        Self(self.0 * t, self.1 * t, self.2 * t)
    }
    pub fn fma(self, t: f64, b: Self) -> Self {
        Self(
            t.mul_add(b.0, self.0),
            t.mul_add(b.1, self.1),
            t.mul_add(b.2, self.2),
        )
    }
    pub fn dot(self, b: Self) -> f64 {
        self.0 * b.0 + self.1 * b.1 + self.2 * b.2
    }
    pub fn len2(self) -> f64 {
        self.dot(self)
    }
    pub fn len(self) -> f64 {
        self.len2().sqrt()
    }
    pub fn normalize(self) -> Self {
        let l = self.len();
        if l < 9.999999747378752e-6 {
            Self::default()
        } else {
            self.scale(1. / l)
        }
    }
    pub fn lerp(self, b: Self, t: f64) -> Self {
        Self(
            self.0 + t * (b.0 - self.0),
            self.1 + t * (b.1 - self.1),
            self.2 + t * (b.2 - self.2),
        )
    }
    pub fn array(self) -> [f64; 3] {
        [self.0, self.1, self.2]
    }
}
#[derive(Clone, Copy, Debug)]
pub struct B {
    pub lo: V,
    pub hi: V,
}
impl B {
    pub fn from_array(a: [f64; 6]) -> Self {
        Self {
            lo: V(a[0], a[1], a[2]),
            hi: V(a[3], a[4], a[5]),
        }
    }
    pub fn inflate(self, r: f64) -> Self {
        let v = V(r, r, r);
        Self {
            lo: self.lo.sub(v),
            hi: self.hi.add(v),
        }
    }
    pub fn moved(self, v: V) -> Self {
        Self {
            lo: self.lo.add(v),
            hi: self.hi.add(v),
        }
    }
    pub fn center(self) -> V {
        self.lo.lerp(self.hi, 0.5)
    }
    pub fn contains(self, p: V) -> bool {
        p.0 >= self.lo.0
            && p.0 < self.hi.0
            && p.1 >= self.lo.1
            && p.1 < self.hi.1
            && p.2 >= self.lo.2
            && p.2 < self.hi.2
    }
    pub fn distance(self, p: V) -> f64 {
        V(
            (self.lo.0 - p.0).max(p.0 - self.hi.0).max(0.),
            (self.lo.1 - p.1).max(p.1 - self.hi.1).max(0.),
            (self.lo.2 - p.2).max(p.2 - self.hi.2).max(0.),
        )
        .len2()
    }
    pub fn intersects(self, b: Self) -> bool {
        self.lo.0 < b.hi.0
            && self.hi.0 > b.lo.0
            && self.lo.1 < b.hi.1
            && self.hi.1 > b.lo.1
            && self.lo.2 < b.hi.2
            && self.hi.2 > b.lo.2
    }
    pub fn vertices(self) -> [V; 8] {
        std::array::from_fn(|i| {
            V(
                if i & 4 == 0 { self.lo.0 } else { self.hi.0 },
                if i & 2 == 0 { self.lo.1 } else { self.hi.1 },
                if i & 1 == 0 { self.lo.2 } else { self.hi.2 },
            )
        })
    }
    // AABB.clip: entering faces only; the strict endpoint and epsilon checks matter.
    pub fn clip(self, from: V, to: V) -> Option<V> {
        let d = to.sub(from);
        let f = from.array();
        let v = d.array();
        let lo = self.lo.array();
        let hi = self.hi.array();
        let mut best = 1.;
        let mut found = false;
        for a in 0..3 {
            if v[a].abs() <= 1e-7 {
                continue;
            }
            let plane = if v[a] > 0. { lo[a] } else { hi[a] };
            let t = (plane - f[a]) / v[a];
            let b = (a + 1) % 3;
            let c = (a + 2) % 3;
            let y = f[b] + t * v[b];
            let z = f[c] + t * v[c];
            if t > 0.
                && t < best
                && y > lo[b] - 1e-7
                && y < hi[b] + 1e-7
                && z > lo[c] - 1e-7
                && z < hi[c] + 1e-7
            {
                best = t;
                found = true;
            }
        }
        found.then(|| from.add(d.scale(best)))
    }
}
fn cos_at(a: f64, c: f64) -> f64 {
    if c <= 0. { 1. } else { a / c.sqrt() }
}
fn edge(eye: V, view: V, start: V, w: V) -> f64 {
    let p = start.sub(eye);
    let a = p.dot(view);
    let b = w.dot(view);
    let c = p.len2();
    let d = p.dot(w);
    let e = w.len2();
    let mut best = cos_at(a, c).max(cos_at(a + b, c + 2. * d + e));
    let t = (a * d - b * c) / (b * d - a * e);
    if t > 0. && t < 1. {
        best = best.max(cos_at(a + b * t, c + 2. * d * t + e * t * t));
    }
    best
}
pub fn max_cos(eye: V, view: V, b: B) -> f64 {
    let mut near: f64 = 0.;
    let mut far = f64::INFINITY;
    let e = eye.array();
    let v = view.array();
    let lo = b.lo.array();
    let hi = b.hi.array();
    let mut hit = true;
    for a in 0..3 {
        if v[a] != 0. {
            let x = (lo[a] - e[a]) / v[a];
            let y = (hi[a] - e[a]) / v[a];
            near = near.max(x.min(y));
            far = far.min(x.max(y));
            if near > far {
                hit = false;
                break;
            }
        } else if e[a] < lo[a] || e[a] > hi[a] {
            hit = false;
            break;
        }
    }
    if hit {
        return 1.;
    }
    let fronts = [
        [eye.0 < b.lo.0, eye.0 > b.hi.0],
        [eye.1 < b.lo.1, eye.1 > b.hi.1],
        [eye.2 < b.lo.2, eye.2 > b.hi.2],
    ];
    let mut best: f64 = -1.;
    for axis in 0..3 {
        let a = (axis + 1) % 3;
        let c = (axis + 2) % 3;
        for i in 0..2 {
            for j in 0..2 {
                if fronts[a][i] == fronts[c][j] {
                    continue;
                }
                let mut p = lo;
                p[a] = if i == 0 { lo[a] } else { hi[a] };
                p[c] = if j == 0 { lo[c] } else { hi[c] };
                let mut w = [0.; 3];
                w[axis] = hi[axis] - lo[axis];
                best = best.max(edge(eye, view, V(p[0], p[1], p[2]), V(w[0], w[1], w[2])));
            }
        }
    }
    best
}
pub fn wrap(mut f: f32) -> f32 {
    f %= 360.;
    if f >= 180. {
        f -= 360.;
    }
    if f < -180. {
        f += 360.;
    }
    f
}
pub fn rotation(v: V) -> [f32; 2] {
    let degrees = f64::from(180. / std::f32::consts::PI);
    [
        wrap((v.2.atan2(v.0) * degrees) as f32 - 90.),
        wrap(-(v.1.atan2((v.0 * v.0 + v.2 * v.2).sqrt()) * degrees) as f32),
    ]
}
#[derive(Clone, Copy, Debug)]
pub struct Trajectory {
    pub velocity: f64,
    pub gravity: f64,
    pub drag: f64,
    pub water_drag: f64,
    pub radius: f64,
    pub copies_velocity: bool,
}
impl Trajectory {
    pub fn bow(ticks: i32) -> Option<Self> {
        let f = ticks as f32 / 20.;
        let p = ((f * f + f * 2.) / 3.).min(1.);
        if p < 0.1 {
            return None;
        }
        Some(Self {
            velocity: f64::from(p) * 3.,
            gravity: 0.05,
            drag: 0.99,
            water_drag: 0.6,
            radius: 0.5,
            copies_velocity: true,
        })
    }
    pub fn trident() -> Self {
        Self {
            velocity: 2.5,
            gravity: 0.05,
            drag: 0.99,
            water_drag: 0.99,
            radius: 0.5,
            copies_velocity: true,
        }
    }
    pub fn crossbow(firework: bool) -> Self {
        if firework {
            Self {
                velocity: 1.6,
                gravity: 0.,
                drag: 1.,
                water_drag: 1.,
                radius: 0.25,
                copies_velocity: false,
            }
        } else {
            Self {
                velocity: 3.15,
                gravity: 0.05,
                drag: 0.99,
                water_drag: 0.6,
                radius: 0.5,
                copies_velocity: false,
            }
        }
    }
}
fn direction(info: Trajectory, target: V, source: V, t: f64) -> V {
    let r = info.drag;
    let power = r.powf(t);
    let d = target.sub(source);
    let den = info.velocity * (power - 1.);
    V(
        d.0 * (r - 1.) / den,
        d.1 * (r - 1.) / den
            + info.gravity * (power - r * t + t - 1.) / (info.velocity * (r - 1.) * (power - 1.)),
        d.2 * (r - 1.) / den,
    )
}
fn minimum(mut a: f64, mut b: f64, mut f: impl FnMut(f64) -> f64) -> (f64, f64) {
    while b - a > 1e-4 {
        let m = a + (b - a) * 0.5;
        let m = if m.is_infinite() {
            a * 0.5 + b * 0.5
        } else {
            m
        };
        if m == a || m == b {
            break;
        }
        if f(a + (m - a) * 0.5) < f(m + (b - m) * 0.5) {
            b = m;
        } else {
            a = m;
        }
    }
    let t = a + (b - a) * 0.5;
    (t, f(t))
}
// JOML defaults to cosFromSin and ordinary, nested f32 products (useMathFma=false).
fn sin_cos(angle: f32) -> (f32, f32) {
    let sin = f64::from(angle).sin() as f32;
    let mut cos = f64::from(1.0f32 - sin * sin).sqrt() as f32;
    let shifted = angle + 1.5707964f32;
    let mut wrapped = shifted - (shifted / 6.2831855f32) as i32 as f32 * 6.2831855f32;
    if wrapped < 0. {
        wrapped += 6.2831855f32;
    }
    if wrapped >= 3.1415927f32 {
        cos = -cos;
    }
    (sin, cos)
}
fn matrix_y(angle: f32) -> [f32; 9] {
    let (s, c) = sin_cos(angle);
    [c, 0., -s, 0., 1., 0., s, 0., c]
}
fn matrix_z(angle: f32) -> [f32; 9] {
    let (s, c) = sin_cos(angle);
    [c, s, 0., -s, c, 0., 0., 0., 1.]
}
fn matrix_mul(a: [f32; 9], b: [f32; 9]) -> [f32; 9] {
    std::array::from_fn(|i| {
        let (column, row) = (i / 3, i % 3);
        a[row] * b[column * 3] + (a[3 + row] * b[column * 3 + 1] + a[6 + row] * b[column * 3 + 2])
    })
}
fn rotate(v: V, yaw: f32, pitch: f32, back: bool) -> V {
    let m = if back {
        matrix_mul(matrix_z(-pitch), matrix_y(yaw))
    } else {
        matrix_mul(matrix_y(-yaw), matrix_z(pitch))
    };
    let x = v.0 as f32;
    let y = v.1 as f32;
    let z = v.2 as f32;
    let (x, y, z) = (
        m[0] * x + (m[3] * y + m[6] * z),
        m[1] * x + (m[4] * y + m[7] * z),
        m[2] * x + (m[5] * y + m[8] * z),
    );
    V(f64::from(x), f64::from(y), f64::from(z))
}
pub fn projected_points(eye: V, b: B) -> Vec<V> { projected_points_limit(eye,b,128) }
pub fn projected_points_limit(eye: V, b: B, maximum: i32) -> Vec<V> {
    if b.contains(eye) {
        return vec![];
    }
    let line = b.center().sub(eye);
    let vertices = b.vertices();
    let mut origin = eye;
    let mut best = f64::INFINITY;
    for p in vertices {
        let near = eye.fma(p.sub(eye).dot(line) / line.len2(), line);
        let dist = near.sub(eye).len2();
        if dist < best {
            best = dist;
            origin = near;
        }
    }
    origin = origin.lerp(eye, 0.1);
    let len = line.len2();
    let normal = if (len - 1.).abs() < 1e-4 {
        line
    } else {
        line.normalize()
    };
    let yaw = normal.2.atan2(normal.0) as f32;
    let pitch = normal
        .1
        .atan2((normal.0 * normal.0 + normal.2 * normal.2).sqrt()) as f32;
    let mut lo_y = 0f32;
    let mut hi_y = 0f32;
    let mut lo_z = 0f32;
    let mut hi_z = 0f32;
    for p in vertices {
        let d = p.sub(eye);
        let e = d.dot(normal);
        if e.abs() < 1e-5 {
            continue;
        }
        let phi = (origin.dot(normal) - eye.dot(normal)) / e;
        let projected = eye.fma(phi, d).sub(origin);
        let v = rotate(projected, yaw, pitch, true);
        lo_y = lo_y.min(v.1 as f32);
        hi_y = hi_y.max(v.1 as f32);
        lo_z = lo_z.min(v.2 as f32);
        hi_z = hi_z.max(v.2 as f32);
    }
    // Vector3f.add(Vec3) also rounds the frame origin to float before the add.
    let base = rotate(V(0., f64::from(lo_y), f64::from(lo_z)), yaw, pitch, false);
    let base = V(
        f64::from(base.0 as f32 + origin.0 as f32),
        f64::from(base.1 as f32 + origin.1 as f32),
        f64::from(base.2 as f32 + origin.2 as f32),
    );
    let dy = rotate(V(0., f64::from(hi_y - lo_y), 0.), yaw, pitch, false);
    let dz = rotate(V(0., 0., f64::from(hi_z - lo_z)), yaw, pitch, false);
    let zero_y = dy.len2().abs() < 1e-5;
    let zero_z = dz.len2().abs() < 1e-5;
    let (ny, nz) = match (zero_y, zero_z) {
        (true, true) => (1, 1),
        (true, false) => (1, maximum),
        (false, true) => (maximum, 1),
        (false, false) => {
            let n = ((f64::from(maximum) / (dz.len() / dy.len())).sqrt().floor() as i32).clamp(1, maximum);
            (n, (maximum / n).max(1))
        }
    };
    let mut points = vec![];
    for y in 0..ny {
        for z in 0..nz {
            let p = base
                .fma((f64::from(y) + 0.5) / f64::from(ny), dy)
                .fma((f64::from(z) + 0.5) / f64::from(nz), dz);
            if let Some(hit) = b.clip(eye, p.lerp(eye, -100.)) {
                points.push(hit);
            }
        }
    }
    points
}
pub fn visible_point(eye: V, b: B, mut visible: impl FnMut(V, V) -> bool) -> Option<V> {
    let mut points = projected_points(eye, b);
    let center = b.center();
    points.sort_by(|a, b| a.sub(center).len2().total_cmp(&b.sub(center).len2()));
    for spot in points {
        let vec = spot.sub(eye);
        let target = eye.fma(2., vec);
        let hit = if b.contains(eye) {
            b.clip(target, eye)
        } else {
            b.clip(eye, target)
        };
        let Some(hit) = hit else {
            continue;
        };
        let length = vec.len2();
        let offset = if length.abs() < 1e-5 {
            V::default()
        } else {
            vec.scale(5. / length.sqrt())
        };
        if visible(hit.sub(offset), hit) {
            return Some(hit);
        }
    }
    None
}
pub fn calculate(
    info: Trajectory,
    source: V,
    dimensions: [f32; 2],
    mut position: impl FnMut(f64) -> V,
    visible: impl FnMut(V, V) -> bool,
) -> Option<[f32; 2]> {
    let base = position(0.);
    let distance = base.sub(source).len();
    if distance < 5. {
        let target = position(distance / info.velocity).sub(source);
        let h = (target.0 * target.0 + target.2 * target.2).sqrt();
        let v = 1.5f64;
        let g = 0.03;
        let d = v.powi(4) - g * (g * h * h + 2. * target.1 * v * v);
        if d < 0. {
            return None;
        }
        return Some([
            (-target.0).atan2(target.2) as f32 * (180. / std::f32::consts::PI),
            wrap(
                (-((v * v - d.sqrt()) / (g * h)).atan() * f64::from(180. / std::f32::consts::PI))
                    as f32,
            ),
        ]);
    }
    let offset = V(
        f64::from(dimensions[0]) * 0.5,
        f64::from(dimensions[1]) * 0.5,
        f64::from(dimensions[0]) * 0.5,
    );
    let (t, delta) = minimum(0., distance / info.velocity * 1.75, |t| {
        (direction(info, position(t).add(offset), source, t).len() - 1.).abs()
    });
    if delta > 0.1 {
        return None;
    }
    let at = position(t);
    let initial = direction(info, at.add(offset), source, t);
    let r = info.drag;
    let power = r.powf(t);
    let log = r.ln();
    let f = r - 1.;
    let impact = V(
        initial.0 * power * log * info.velocity / f,
        (initial.1 * f * power * log * info.velocity - info.gravity * (power * log - r + 1.))
            / f.powi(2),
        initial.2 * power * log * info.velocity / f,
    )
    .normalize();
    let virtual_eye = source.add(V(0., impact.1 * -at.sub(source).len(), 0.));
    let half = f64::from(dimensions[0]) * 0.5;
    let b = B {
        lo: at.add(V(-half, 0., -half)),
        hi: at.add(V(half, f64::from(dimensions[1]), half)),
    }
    .inflate(info.radius);
    let target = visible_point(virtual_eye, b, visible)?;
    Some(rotation(direction(
        info,
        target,
        source,
        t.round_ties_even(),
    )))
}
