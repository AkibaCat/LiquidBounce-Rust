// SPDX-License-Identifier: GPL-3.0-or-later
//! JDK 25 ThreadLocalRandom sampling used by Kotlin Random.Default on the JVM.
//! Seeded fixtures reproduce Java sequences; live Rust threads choose independent
//! entropy and identifiers, so process initialization is intentionally not identical.

use std::{
    cell::RefCell,
    hash::{BuildHasher, RandomState},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Copy, Debug)]
pub struct ThreadRandom {
    seed: u64,
    thread_id: u64,
}
impl ThreadRandom {
    pub fn seeded(seed: u64, thread_id: u64) -> Self {
        Self { seed, thread_id }
    }
    fn next_seed(&mut self) -> u64 {
        self.seed = self
            .seed
            .wrapping_add(self.thread_id.wrapping_shl(1))
            .wrapping_add(0x9e3779b97f4a7c15);
        self.seed
    }
    pub fn next_i32(&mut self) -> i32 {
        let seed = self.next_seed();
        let z = (seed ^ (seed >> 33)).wrapping_mul(0xff51afd7ed558ccd);
        ((z ^ (z >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53) >> 32) as i32
    }
    /// JDK mix64 consumes one seed, independently of mix32.
    pub fn next_i64(&mut self) -> i64 {
        let mut z = self.next_seed();
        z = (z ^ (z >> 33)).wrapping_mul(0xff51afd7ed558ccd);
        z = (z ^ (z >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53);
        (z ^ (z >> 33)) as i64
    }
    pub fn next_bool(&mut self) -> bool {
        self.next_i32() < 0
    }
    pub fn next_unit_f64(&mut self) -> f64 {
        // JDK25 ThreadLocalRandom delegates to Random.nextDouble(), which
        // invokes next(26) and next(27), hence two independent mix32 seeds.
        let high = u64::from((self.next_i32() as u32) >> 6);
        let low = u64::from((self.next_i32() as u32) >> 5);
        ((high << 27) + low) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }
    pub fn next_f64_between(&mut self, origin: f64, bound: f64) -> f64 {
        assert!(
            origin.is_finite() && bound.is_finite() && origin < bound,
            "random double range must be finite and nonempty"
        );
        bounded_f64(self.next_unit_f64(), origin, bound)
    }
    pub fn next_unit(&mut self) -> f32 {
        ((self.next_i32() as u32) >> 8) as f32 * (1.0 / 16_777_216.0)
    }
    /// Random.nextInt(bound), used by Kotlin collection.randomOrNull. JDK25
    /// ThreadLocalRandom inherits Random's multiply-high power-of-two path;
    /// it differs from the origin/bound RandomSupport overload below.
    pub fn next_i32_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "random bound must be positive");
        let mut bits = ((self.next_i32() as u32) >> 1) as i32;
        let mask = bound - 1;
        if bound & mask == 0 {
            return ((i64::from(bound) * i64::from(bits)) >> 31) as i32;
        }
        loop {
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(mask) >= 0 {
                return value;
            }
            bits = ((self.next_i32() as u32) >> 1) as i32;
        }
    }
    /// JDK nextInt(origin,bound), including signed overflow and rejection draws.
    pub fn next_i32_between(&mut self, origin: i32, bound: i32) -> i32 {
        assert!(origin < bound, "random range must be nonempty");
        let mut r = self.next_i32();
        let n = bound.wrapping_sub(origin);
        let m = n.wrapping_sub(1);
        if n & m == 0 {
            r = (r & m).wrapping_add(origin);
        } else if n > 0 {
            let mut u = ((r as u32) >> 1) as i32;
            loop {
                r = u % n;
                if u.wrapping_add(m).wrapping_sub(r) >= 0 {
                    break;
                }
                u = ((self.next_i32() as u32) >> 1) as i32;
            }
            r = r.wrapping_add(origin);
        } else {
            while r < origin || r >= bound {
                r = self.next_i32();
            }
        }
        r
    }
    /// Kotlin Random.nextInt(IntRange), including singleton ranges (one draw).
    pub fn next_i32_inclusive(&mut self, first: i32, last: i32) -> i32 {
        assert!(first <= last, "random range must be nonempty");
        if last < i32::MAX {
            self.next_i32_between(first, last + 1)
        } else if first > i32::MIN {
            self.next_i32_between(first - 1, last) + 1
        } else {
            self.next_i32()
        }
    }
    pub fn next_f32_between(&mut self, origin: f32, bound: f32) -> f32 {
        assert!(
            origin.is_finite() && bound.is_finite() && origin < bound,
            "random float range must be finite and nonempty"
        );
        bounded_f32(self.next_unit(), origin, bound)
    }
}

/// JDK RandomSupport.boundedNextDouble; the upper endpoint is exclusive.
pub fn bounded_f64(unit: f64, origin: f64, bound: f64) -> f64 {
    let result = if (bound - origin).is_finite() {
        unit * (bound - origin) + origin
    } else {
        let half_origin = 0.5 * origin;
        (unit * (0.5 * bound - half_origin) + half_origin) * 2.0
    };
    if result >= bound {
        bound.next_down()
    } else {
        result
    }
}

/// RandomSupport.boundedNextFloat arithmetic, including overflowing span.
pub fn bounded_f32(unit: f32, origin: f32, bound: f32) -> f32 {
    let result = if (bound - origin).is_finite() {
        unit * (bound - origin) + origin
    } else {
        let half_origin = 0.5 * origin;
        (unit * (0.5 * bound - half_origin) + half_origin) * 2.0
    };
    if result >= bound {
        bound.next_down()
    } else {
        result
    }
}

fn new_thread_random() -> ThreadRandom {
    static THREAD_IDS: AtomicU64 = AtomicU64::new(1);
    let id = THREAD_IDS.fetch_add(1, Ordering::Relaxed);
    ThreadRandom::seeded(RandomState::new().hash_one(id), id)
}
thread_local! { static LOCAL_RANDOM: RefCell<ThreadRandom> = RefCell::new(new_thread_random()); }
pub fn with_thread_random<T>(sample: impl FnOnce(&mut ThreadRandom) -> T) -> T {
    LOCAL_RANDOM.with(|random| sample(&mut random.borrow_mut()))
}
pub fn random_unit_f32() -> f32 {
    LOCAL_RANDOM.with(|random| random.borrow_mut().next_unit())
}
pub fn random_i32_inclusive(first: i32, last: i32) -> i32 {
    LOCAL_RANDOM.with(|random| random.borrow_mut().next_i32_inclusive(first, last))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn singleton_range_consumes_a_draw_and_full_range_uses_all_signed_bits() {
        let mut a = ThreadRandom::seeded(123, 1);
        let mut b = a;
        assert_eq!(a.next_i32_inclusive(7, 7), 7);
        b.next_i32();
        assert_eq!(a.next_i32(), b.next_i32());
        assert_eq!(a.next_i32_inclusive(i32::MIN, i32::MAX), b.next_i32());
    }
}
