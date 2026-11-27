// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/navigation/walk_jump/java_math.rs"]
mod java_math;
mod navigation {
    pub mod walk_jump {
        pub(crate) use crate::java_math;
        pub fn vector_yaw(x: f64, z: f64) -> f32 {
            (crate::java_math::fdlibm_atan2(-x, z) as f32) * f32::from_bits(0x4265_2ee0)
        }
        pub fn yaw(x: f64, z: f64) -> f32 {
            (crate::java_math::fdlibm_atan2(z, x) * f64::from(f32::from_bits(0x4265_2ee0)) - 90.)
                as f32
        }
    }
}
#[path = "../liquidbounce-behaviors/src/auto_dodge.rs"]
mod model;
#[test]
fn arrow_forecast_and_evasion() {
    use model::*;
    let settings = Settings::default();
    assert!(!settings.rotation && !settings.timer && settings.jump && settings.ignore == 7);
    let arrow = Arrow {
        id: 3,
        position: [-6., 1., 0.],
        velocity: [2., 0., 0.],
        stopped: false,
    };
    let hit = inflicted(std::iter::repeat_n([0.; 3], 8), &[arrow], 0.7, |_, _| None).unwrap();
    assert_eq!((hit.id, hit.tick), (3, 2));
    assert!(
        inflicted(
            std::iter::repeat_n([0.; 3], 8),
            &[arrow],
            0.7,
            |from, to| clip_box(from, to, [-4.5, 0., -1.], [-4., 3., 1.])
        )
        .is_none()
    );
    assert!(
        inflicted(
            std::iter::repeat_n([0.; 3], 8),
            &[Arrow {
                stopped: true,
                ..arrow
            }],
            0.7,
            |_, _| None
        )
        .is_none()
    );
    let second = Arrow { id: 4, ..arrow };
    assert_eq!(
        inflicted([[0.; 3]; 8], &[second, arrow], 0.7, |_, _| None)
            .unwrap()
            .id,
        4
    );
    assert_eq!(
        avoiding(&[[0.; 3], [0.1, 0., 0.], [2., 0., 0.]], [0.; 3], |p| {
            if p[0] < 1. { Some(hit) } else { None }
        }),
        Some((2, false))
    );
    assert_eq!(
        avoiding(&[[0.; 3], [0.2, 0., 0.]], [0.; 3], |_| Some(hit)),
        None
    );
    assert_eq!(yaw([1., 0., 0.]), -90.);
    assert_eq!(
        directional(wrap(yaw([0., 0., -1.]))),
        [false, true, false, false]
    );
    let direction = |yaw: f32| {
        [
            -f64::from(yaw).to_radians().sin(),
            0.,
            f64::from(yaw).to_radians().cos(),
        ]
    };
    let p = plan(
        [0.; 3],
        [0.; 3],
        0.,
        0.,
        true,
        Hit { tick: 0, ..hit },
        |_, _| 10.,
        direction,
    )
    .unwrap();
    assert_eq!(p.keys, [true, false, false, false]);
    assert!(p.yaw.is_some() && p.timer && !p.jump);
    let p = plan(
        [0., 0., 4.],
        [0.; 3],
        0.,
        0.,
        false,
        hit,
        |_, _| 10.,
        direction,
    );
    assert!(p.is_none());
}
