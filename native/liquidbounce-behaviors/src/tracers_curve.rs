// SPDX-License-Identifier: GPL-3.0-or-later
// CurveUtil.transformNormalized, LiquidBounce ae0eb396, CCBlueX 2015-2026.
pub fn sample(data: &[[f32; 2]], x: f32) -> f32 {
    if data.is_empty() {
        return 0.;
    }
    if data.len() == 1 {
        return data[0][1];
    }
    let first = data[0];
    let last = data[data.len() - 1];
    if x <= first[0] {
        return first[1];
    }
    if x >= last[0] {
        return last[1];
    }
    if data.len() == 2 {
        return first[1] + (last[1] - first[1]) * ((x - first[0]) / (last[0] - first[0]));
    }
    if let Some(p) = data.iter().find(|p| p[0] == x) {
        return p[1];
    }
    fn points(data: &[[f32; 2]], x: f32) -> ([f32; 2], [f32; 2], [f32; 2]) {
        let adjusted = if x == data[0][0] { x + 1. } else { x };
        let n = if adjusted == data[data.len() - 1][0] {
            data.len() - 1
        } else {
            data.iter()
                .position(|p| p[0] >= adjusted)
                .unwrap_or(data.len() - 1)
        };
        (
            data[n.saturating_sub(2)],
            data[n.saturating_sub(1)],
            data[n],
        )
    }
    fn controls(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> ([f32; 2], [f32; 2]) {
        let dist = |p: [f32; 2], q: [f32; 2]| {
            ((p[0] - q[0]) * (p[0] - q[0]) + (p[1] - q[1]) * (p[1] - q[1])).sqrt()
        };
        let d01 = dist(a, b);
        let d12 = dist(b, c);
        let total = d01 + d12;
        if total == 0. {
            return (b, b);
        }
        let fa = 0.4 * d01 / total;
        let fb = 0.4 * d12 / total;
        (
            std::array::from_fn(|i| b[i] - (c[i] - a[i]) * fa),
            std::array::from_fn(|i| b[i] + (c[i] - a[i]) * fb),
        )
    }
    let (prev, p, next) = points(data, x);
    let t = if x == next[0] {
        1.
    } else if next[0] - p[0] == 0. {
        0.
    } else {
        ((x - p[0]) / (next[0] - p[0])).clamp(0., 1.)
    };
    let p1 = controls(prev, p, next).1;
    let p2 = if let Some(n) = data.iter().position(|v| v[0] > next[0]) {
        let segment = data[n - 1];
        let after = data[n];
        let (a, b, c) = points(data, segment[0] + (after[0] - segment[0]));
        controls(a, b, c).0
    } else {
        next
    };
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1. - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    mt3 * p[1] + 3. * mt2 * t * p1[1] + 3. * mt * t2 * p2[1] + t3 * next[1]
}
