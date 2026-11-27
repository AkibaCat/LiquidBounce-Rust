// SPDX-License-Identifier: GPL-3.0-or-later
//! Native input/edge geometry shared by the live Parkour and SafeWalk adapter.
use crate::navigation::{aabb::Aabb, geometry::Vec3 as V};
pub(super) fn inverse_input(
    forward: i32,
    strafe: i32,
    physical_yaw: f32,
    managed_yaw: f32,
) -> (i32, i32) {
    let delta = (physical_yaw - managed_yaw) * f32::from_bits(0x3c8efa35);
    let sin = crate::navigation::simulation::sine_table::sin(delta);
    let cos = crate::navigation::simulation::sine_table::cos(delta);
    let round = |v: f32| v.floor() as i32 + i32::from(v - v.floor() >= 0.5);
    for z in -1..=1 {
        for x in -1..=1 {
            if round(x as f32 * cos - z as f32 * sin) == strafe
                && round(z as f32 * cos + x as f32 * sin) == forward
            {
                return (z, x);
            }
        }
    }
    (forward, strafe)
}
pub(super) fn crosses_edge(from: V, to: V, mut boxes: Vec<Aabb>) -> bool {
    let d = V::new(to.x - from.x, to.y - from.y, to.z - from.z);
    if d.x * d.x + d.y * d.y + d.z * d.z <= 1e-12 {
        return false;
    }
    let extended_from = V::new(
        from.x - 1000. * d.x,
        from.y - 1000. * d.y,
        from.z - 1000. * d.z,
    );
    let extended_to = V::new(to.x + 1000. * d.x, to.y + 1000. * d.y, to.z + 1000. * d.z);
    let mut at = from;
    loop {
        let containing = boxes
            .iter()
            .copied()
            .filter(|b| b.contains(at))
            .collect::<Vec<_>>();
        if containing.is_empty() {
            return true;
        }
        if containing.iter().any(|b| b.contains(to)) {
            return false;
        }
        let next = containing
            .iter()
            .map(|b| {
                b.clip(extended_to, extended_from)
                    .expect("edge crossing ray")
            })
            .min_by(|a, b| a.distance_sqr(to).total_cmp(&b.distance_sqr(to)))
            .expect("edge containing support");
        boxes.retain(|b| !containing.contains(b));
        at = next;
    }
}
