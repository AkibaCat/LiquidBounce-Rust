// SPDX-License-Identifier: MIT
//! Horizon jmath::exact sine/cosine for the original Windows target.
//! Copied without arithmetic changes; large angles retain its platform fallback.
// MIT License
//
// Copyright (c) 2026 Ludovic Marie and the Horizon contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
/// A value `hi + lo` with |lo| <= ulp(hi) / 2.
#[derive(Clone, Copy)]
struct Dd(f64, f64);

/// `a + b` exactly, as a sum and its rounding error (Knuth).
fn two_sum(a: f64, b: f64) -> Dd {
    let s = a + b;
    let v = s - a;
    Dd(s, (a - (s - v)) + (b - v))
}

/// `a + b` exactly when |a| >= |b| (Dekker).
fn fast_two_sum(a: f64, b: f64) -> Dd {
    let s = a + b;
    Dd(s, b - (s - a))
}

/// `a * b` exactly, as a product and its rounding error (Dekker's split, no fused multiply-add).
fn two_prod(a: f64, b: f64) -> Dd {
    fn split(a: f64) -> (f64, f64) {
        let t = 134_217_729.0 * a; // 2^27 + 1
        let hi = t - (t - a);
        (hi, a - hi)
    }
    let p = a * b;
    let (ah, al) = split(a);
    let (bh, bl) = split(b);
    Dd(p, ((ah * bh - p) + ah * bl + al * bh) + al * bl)
}

impl Dd {
    fn add(self, b: Dd) -> Dd {
        let s = two_sum(self.0, b.0);
        let t = two_sum(self.1, b.1);
        let s = fast_two_sum(s.0, s.1 + t.0);
        fast_two_sum(s.0, s.1 + t.1)
    }
    fn mul(self, b: Dd) -> Dd {
        let p = two_prod(self.0, b.0);
        fast_two_sum(p.0, p.1 + (self.0 * b.1 + self.1 * b.0))
    }
    /// Divided by a small whole number (exact in an `f64`).
    fn div(self, d: f64) -> Dd {
        let q = self.0 / d;
        let p = two_prod(q, d);
        let r = ((self.0 - p.0) - p.1 + self.1) / d;
        fast_two_sum(q, r)
    }
    fn neg(self) -> Dd {
        Dd(-self.0, -self.1)
    }
}

/// pi/2 = P1 + P2 + P3 + P4 to 2^-160: the first three have 33 significant bits, so `k * Pn` is
/// exact for |k| < 2^20.
const P1: f64 = f64::from_bits(0x3FF9_21FB_5440_0000);
const P2: f64 = f64::from_bits(0x3DD0_B461_1A60_0000);
const P3: f64 = f64::from_bits(0x3BA3_198A_2E00_0000);
const P4: f64 = f64::from_bits(0x397B_839A_2520_49C1);

/// `x - k * pi/2` and the quadrant `k mod 4`, for |x| <= 2^20.
fn reduce(x: f64) -> (Dd, u32) {
    let k = (x * std::f64::consts::FRAC_2_PI).round();
    let r = Dd(x - k * P1, 0.0)
        .add(Dd(-(k * P2), 0.0))
        .add(Dd(-(k * P3), 0.0))
        .add(two_prod(k, P4).neg());
    (r, (k as i64 & 3) as u32)
}

/// `sum(xs)` rounded once, exactly (Shewchuk's partials, with the half-way fix-up of a final
/// rounding that a running sum can't see).
fn exact_sum(xs: &[f64]) -> f64 {
    let mut partials = [0.0f64; 64];
    let mut len = 0;
    for &x0 in xs {
        let mut x = x0;
        let mut i = 0;
        for j in 0..len {
            let mut y = partials[j];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
            let lo = y - (hi - x);
            if lo != 0.0 {
                partials[i] = lo;
                i += 1;
            }
            x = hi;
        }
        partials[i] = x;
        len = i + 1;
    }
    if len == 0 {
        return 0.0;
    }
    let mut n = len - 1;
    let mut hi = partials[n];
    let mut lo = 0.0;
    while n > 0 {
        let x = hi;
        n -= 1;
        let y = partials[n];
        hi = x + y;
        lo = y - (hi - x);
        if lo != 0.0 {
            break;
        }
    }
    if n > 0 && ((lo < 0.0 && partials[n - 1] < 0.0) || (lo > 0.0 && partials[n - 1] > 0.0)) {
        let y = lo * 2.0;
        let x = hi + y;
        if y == x - hi {
            hi = x;
        }
    }
    hi
}

/// sin(r) or cos(r) for |r| <= pi/4 + a little: the Taylor series to below 2^-110, each term
/// kept as a double-double and the terms summed exactly. The leading terms are exact for a small
/// `x` (r = x), where cos x and sin x come closest to half-way between two doubles.
fn series(r: Dd, cos: bool) -> f64 {
    let r2 = r.mul(r);
    let mut parts = [0.0f64; 32];
    let mut len = 0;
    let (mut term, first) = if cos { (Dd(1.0, 0.0), 1.0) } else { (r, 2.0) };
    parts[0] = term.0;
    parts[1] = term.1;
    len += 2;
    for n in 1..=14 {
        let k = 2.0 * f64::from(n) + first - 1.0;
        term = term.mul(r2).div(k * (k - 1.0)).neg();
        parts[len] = term.0;
        parts[len + 1] = term.1;
        len += 2;
    }
    exact_sum(&parts[..len])
}

/// Taylor coefficients 1/n! with alternating sign, for the f64 tails: sin from r^7, cos from r^8.
const SIN_TAIL: [f64; 9] = [
    -1.0 / 5040.0,
    1.0 / 362880.0,
    -1.0 / 39916800.0,
    1.0 / 6227020800.0,
    -1.0 / 1307674368000.0,
    1.0 / 355687428096000.0,
    -1.0 / 121645100408832000.0,
    1.0 / 51090942171709440000.0,
    -1.0 / 25852016738884976640000.0,
];
const COS_TAIL: [f64; 8] = [
    1.0 / 40320.0,
    -1.0 / 3628800.0,
    1.0 / 479001600.0,
    -1.0 / 87178291200.0,
    1.0 / 20922789888000.0,
    -1.0 / 6402373705728000.0,
    1.0 / 2432902008176640000.0,
    -1.0 / 1124000727777607680000.0,
];

fn horner(c: &[f64], x: f64) -> f64 {
    c.iter().rev().fold(0.0, |acc, &k| acc * x + k)
}

/// sin(r) or cos(r) to about 2^-64: the first terms in double-double, the small rest in f64.
fn fast(r: Dd, cos: bool) -> Dd {
    let r2 = r.mul(r);
    let x2 = r2.0;
    if cos {
        let t2 = r2.div(2.0).neg();
        let t4 = r2.mul(r2).div(24.0);
        let t6 = t4.mul(r2).div(30.0).neg();
        let tail = x2 * x2 * x2 * x2 * horner(&COS_TAIL, x2);
        Dd(1.0, 0.0).add(t2).add(t4).add(t6).add(Dd(tail, 0.0))
    } else {
        let t3 = r2.mul(r).div(6.0).neg();
        let t5 = t3.mul(r2).div(20.0).neg();
        let tail = r.0 * x2 * x2 * x2 * horner(&SIN_TAIL, x2);
        r.add(t3).add(t5).add(Dd(tail, 0.0))
    }
}

/// Rounds `v` when its error bound can't change the rounding, else `None`.
fn rounded(v: Dd) -> Option<f64> {
    let e = v.0.abs() * f64::from_bits(0x3C30_0000_0000_0000); // 2^-60
    let (a, b) = (v.0 + (v.1 - e), v.0 + (v.1 + e));
    (a == b).then_some(a)
}

/// sin (`cos == false`) or cos of a reduced angle in quadrant `q`.
fn eval(r: Dd, q: u32, cos: bool) -> f64 {
    // sin(x) = sin r, cos r, -sin r, -cos r in quadrants 0..3; cos(x) is a quadrant later.
    let q = (q + u32::from(cos)) & 3;
    let want_cos = q & 1 == 1;
    let v = rounded(fast(r, want_cos)).unwrap_or_else(|| series(r, want_cos));
    if q >= 2 { -v } else { v }
}

/// `sin(x)` rounded to nearest; beyond |x| = 2^20 the platform's.
pub fn sin(x: f64) -> f64 {
    if x.abs() < f64::from_bits(0x3E40_0000_0000_0000) {
        // |x| < 2^-27: sin x rounds to x (0 included).
        return x;
    }
    if x.is_nan() || x.abs() > 1_048_576.0 {
        return x.sin();
    }
    let (r, q) = reduce(x);
    eval(r, q, false)
}

/// `cos(x)` rounded to nearest; beyond |x| = 2^20 the platform's.
pub fn cos(x: f64) -> f64 {
    if x.abs() < f64::from_bits(0x3E40_0000_0000_0000) {
        // |x| < 2^-27: cos x rounds to 1.
        return 1.0;
    }
    if x.is_nan() || x.abs() > 1_048_576.0 {
        return x.cos();
    }
    let (r, q) = reduce(x);
    eval(r, q, true)
}
