// SPDX-License-Identifier: GPL-3.0-or-later
//! Signed monotonic nanosecond clock for SnapTap's System.nanoTime ordering.
use std::{sync::OnceLock, time::Instant};

pub fn nano_time() -> i64 {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    EPOCH.get_or_init(Instant::now).elapsed().as_nanos() as i64
}
