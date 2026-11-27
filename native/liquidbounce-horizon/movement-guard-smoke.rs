// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/navigation/aabb.rs"]
pub mod aabb;
#[path = "../liquidbounce-behaviors/src/anchor.rs"]
mod anchor;
#[path = "../liquidbounce-behaviors/src/avoid_hazards.rs"]
mod avoid_hazards;
#[path = "../liquidbounce-behaviors/src/freeze.rs"]
mod freeze;
#[path = "../liquidbounce-behaviors/src/navigation/geometry.rs"]
pub mod geometry;
#[path = "../liquidbounce-behaviors/src/navigation/simulation/sine_table.rs"]
pub mod sine_table;
pub mod navigation {
    pub use crate::{aabb, geometry};
    pub mod simulation {
        pub use crate::sine_table;
    }
}
#[path = "edge-geometry.rs"]
mod edge_geometry;
#[test]
fn anchor_finds_safe_rectangles_and_pulls_toward_the_bottom() {
    use anchor::{Anchor, Block};
    let crater = |p: [i32; 3]| {
        let [x, y, z] = p;
        if (0..2).contains(&x) && (0..2).contains(&z) {
            if y == -1 {
                Block::Resistant
            } else if (0..3).contains(&y) {
                Block::Air
            } else {
                Block::Other
            }
        } else if y == 0
            && ((-1..=2).contains(&x) && (z == -1 || z == 2)
                || (-1..=2).contains(&z) && (x == -1 || x == 2))
        {
            Block::Resistant
        } else {
            Block::Other
        }
    };
    let holes = anchor::scan([-1, -1, -1], [2, 2, 2], crater);
    assert_eq!(holes.len(), 1);
    assert_eq!(holes[0].size, [2, 2]);
    assert_eq!(holes[0].center(), [1., 0., 1.]);
    let a = Anchor::default();
    let pos = [2., 2., 1.];
    let bounds = [1.7, 2., 0.7, 2.3, 3.8, 1.3];
    let goal = a.goal(&holes, pos, bounds).unwrap();
    assert_eq!(goal, [1., 0., 1.]);
    let m = a.movement(pos, bounds, [0., -0.2, 0.], goal);
    assert!((m[0] + f64::from(0.3f32) / 5f64.sqrt()).abs() < 1e-12);
    assert!((m[1] + 0.4 / 5f64.sqrt()).abs() < 1e-12);
    assert_eq!(m[2], 0.);
    let filled = anchor::scan([-1, -1, -1], [2, 2, 2], |p| {
        if p == [1, 1, 1] {
            Block::Other
        } else {
            crater(p)
        }
    });
    assert!(filled.is_empty());
}
#[test]
fn hazards_preserve_safe_input_and_respect_solid_caps_and_ladder_entry() {
    use avoid_hazards::{Flags, choose, ladder, shape};
    let forward = [true, false, false, false];
    let right = [false, false, false, true];
    let left = [false, false, true, false];
    assert_eq!(choose(forward, |k| k == forward), forward);
    assert_eq!(choose(forward, |k| k == right || k == left), right);
    assert_eq!(choose(forward, |_| false), [false; 4]);
    let below = Flags {
        ladder: true,
        facing: 2,
        ..Default::default()
    };
    let door = Flags {
        open_trapdoor: true,
        facing: 2,
        ..Default::default()
    };
    assert!(ladder(door, below));
    assert!(!ladder(Flags { facing: 1, ..door }, below));
    assert_eq!(shape(1 << 4, door, below), Some(1.));
    assert_eq!(
        shape(
            1 << 6,
            Flags::default(),
            Flags {
                magma: true,
                ..Default::default()
            }
        ),
        Some(0.25)
    );
    assert_eq!(
        shape(
            (1 << 5) | 1,
            Flags {
                hazards: (1 << 5) | 1,
                ..Default::default()
            },
            Flags::default()
        ),
        Some(1.)
    );
}
#[test]
fn freeze_releases_every_third_movement_and_refreshes_frame_holds() {
    use freeze::{Freeze, Mode, Settings};
    let mut state = Freeze::default();
    let mut settings = Settings {
        mode: Mode::TickMovement,
        interval: [3, 3],
        ..Default::default()
    };
    state.synchronize(true, settings, |r| r[0]);
    assert!(!state.holds());
    let observed = (0..6)
        .map(|_| state.cancel_movement(settings, |r| r[0]))
        .collect::<Vec<_>>();
    assert_eq!(observed, vec![true, true, false, true, true, false]);
    settings.mode = Mode::Stationary;
    state.synchronize(true, settings, |r| r[0]);
    assert!(state.holds());
    assert!(!state.cancel_movement(settings, |r| r[0]));
    settings.disable_on = 3;
    assert_eq!(Freeze::disable_reason(settings, true, [true; 5]), Some(0));
    assert_eq!(Freeze::disable_reason(settings, false, [true; 5]), Some(1));
    state.synchronize(false, settings, |r| r[0]);
    assert!(!state.holds());
    let first = state.offsets(|| 0.);
    assert_eq!(first, [0.002; 2]);
    let mut draws = [0., 0.5, 0.25].into_iter();
    let next = state.offsets(|| draws.next().unwrap());
    assert!((next[0] - 0.006).abs() < 1e-7);
    assert!((next[1] - 0.004).abs() < 1e-7);
}
#[test]
fn edge_geometry_keeps_contiguous_support_and_corrects_silent_jump_keys() {
    use aabb::Aabb;
    use geometry::Vec3 as V;
    let support = |x1, x2| Aabb::new(V::new(x1, -2., -0.3), V::new(x2, 0.55, 1.3));
    let from = V::new(0.5, -0.1, 0.5);
    let to = V::new(2., -0.1, 0.5);
    assert!(!edge_geometry::crosses_edge(
        from,
        to,
        vec![support(-0.3, 1.3), support(0.7, 2.3)]
    ));
    assert!(edge_geometry::crosses_edge(
        from,
        to,
        vec![support(-0.3, 1.3), support(1.4, 2.3)]
    ));
    assert!(!edge_geometry::crosses_edge(from, from, vec![]));
    assert_eq!(edge_geometry::inverse_input(1, 0, 90., 0.), (0, 1));
    assert_eq!(edge_geometry::inverse_input(1, 0, -90., 0.), (0, -1));
    assert_eq!(edge_geometry::inverse_input(1, -1, 0., 0.), (1, -1));
}
