// SPDX-License-Identifier: GPL-3.0-or-later
use crate::anchor::Anchor;
use horizon_core::geometry::Vec3;
use std::cell::Cell;
#[derive(Clone, Copy)]
pub struct Snapshot {
    pub settings: Anchor,
    pub goal: [f64; 3],
}
thread_local! {static STATE:Cell<Option<Snapshot>>=const{Cell::new(None)};}
pub fn set(value: Option<Snapshot>) {
    STATE.set(value);
}
pub fn movement(position: Vec3, bounds: horizon_core::geometry::Aabb, movement: Vec3) -> Vec3 {
    let Some(s) = STATE.get() else {
        return movement;
    };
    let lo = bounds.min_position();
    let hi = bounds.max_position();
    let r = s.settings.movement(
        [position.x, position.y, position.z],
        [lo.x, lo.y, lo.z, hi.x, hi.y, hi.z],
        [movement.x, movement.y, movement.z],
        s.goal,
    );
    Vec3::new(r[0], r[1], r[2])
}
