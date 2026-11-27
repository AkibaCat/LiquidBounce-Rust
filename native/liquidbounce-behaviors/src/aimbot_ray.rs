// SPDX-License-Identifier: GPL-3.0-or-later
//! RotationFinding.raytraceBox with LeastDifferenceToLastPoint, exact strict
//! range checks, visible-before-hidden preference and all surface samples.
use crate::auto_bow_math::{self as math, B, V};
pub fn first_hit(b: B, from: V, to: V) -> Option<V> {
    if b.contains(from) {
        b.clip(to, from)
    } else {
        b.clip(from, to)
    }
}
fn difference(base: [f32; 2], angle: [f32; 2]) -> f32 {
    let y = math::wrap(angle[0] - base[0]);
    let p = angle[1] - base[1];
    y * y + p * p
}
pub fn raytrace(
    eyes: V,
    b: B,
    point: V,
    range: f64,
    walls: f64,
    mut visible: impl FnMut(V, V) -> bool,
) -> Option<[f32; 2]> {
    let range = range * range;
    let walls = walls * walls;
    if let Some(hit) = first_hit(b, eyes, point) {
        let distance = hit.sub(eyes).len2();
        let seen = visible(eyes, hit);
        if distance < if seen { range } else { walls } {
            return Some(math::rotation(hit.sub(eyes)));
        }
    }
    let base = math::rotation(point.sub(eyes));
    let mut best: [Option<([f32; 2], f32)>; 2] = [None, None];
    let mut consider = |spot: V| {
        let Some(hit) = first_hit(b, eyes, eyes.fma(2., spot.sub(eyes))) else {
            return;
        };
        let distance = hit.sub(eyes).len2();
        let seen = visible(eyes, hit);
        if !(distance < if seen { range } else { walls }) {
            return;
        }
        let angle = math::rotation(spot.sub(eyes));
        let score = difference(base, angle);
        let i = usize::from(!seen);
        if best[i].is_none_or(|(_, old)| java_float_compare(old, score).is_gt()) {
            best[i] = Some((angle, score));
        }
    };
    consider(point);
    let nearest = V(
        eyes.0.clamp(b.lo.0, b.hi.0),
        eyes.1.clamp(b.lo.1, b.hi.1),
        eyes.2.clamp(b.lo.2, b.hi.2),
    );
    consider(nearest);
    if !b.contains(eyes) {
        for p in math::projected_points_limit(eyes, b, 256) {
            consider(p)
        }
    } else {
        // Direction.entries: DOWN, UP, NORTH, SOUTH, WEST, EAST.
        for (axis, high) in [
            (1, false),
            (1, true),
            (2, false),
            (2, true),
            (0, false),
            (0, true),
        ] {
            let mut lo = b.lo.array();
            let mut hi = b.hi.array();
            let c = if high { hi[axis] } else { lo[axis] };
            lo[axis] = c;
            hi[axis] = c;
            let dims = std::array::from_fn::<_, 3, _>(|i| hi[i] - lo[i]);
            let (a, z) = if dims[0] == 0. {
                (1, 2)
            } else if dims[1] == 0. {
                (0, 2)
            } else {
                (0, 1)
            };
            let area = dims[0] * dims[1] + dims[1] * dims[2] + dims[0] * dims[2];
            let spacing = 0.1f64.max((area / 100.).sqrt());
            let na = (dims[a] / spacing).ceil().max(1.) as i32;
            let nz = (dims[z] / spacing).ceil().max(1.) as i32;
            for x in 0..na {
                for y in 0..nz {
                    let mut p = lo;
                    p[a] = dims[a].mul_add((f64::from(x) + 0.5) / f64::from(na), lo[a]);
                    p[z] = dims[z].mul_add((f64::from(y) + 0.5) / f64::from(nz), lo[z]);
                    consider(V(p[0], p[1], p[2]));
                }
            }
        }
    }
    best[0].or(best[1]).map(|(r, _)| r)
}
fn java_float_compare(a: f32, b: f32) -> std::cmp::Ordering {
    if a < b {
        return std::cmp::Ordering::Less;
    }
    if a > b {
        return std::cmp::Ordering::Greater;
    }
    let bits = |v: f32| {
        if v.is_nan() {
            0x7fc00000i32
        } else {
            v.to_bits() as i32
        }
    };
    bits(a).cmp(&bits(b))
}
