#[path = "../liquidbounce-behaviors/src/clutch.rs"]
mod clutch;
use clutch::*;

#[test]
fn clutch_smoke_guards_follow_can_catch_not_running() {
    let mut c = Clutch::default();
    let f = Facts {
        block_count: 1,
        on_ground: true,
        ..Default::default()
    };
    assert!(!c.rotation_update(
        true,
        f,
        |_| panic!("ground short circuits"),
        |_, _| panic!("safe ground")
    ));
    assert!(c.guarding(true));
    assert!(!c.guarding(false));
    assert_eq!(c.fall(true), None);
    assert!(!c.rotation_update(
        true,
        Facts {
            block_count: 0,
            ..f
        },
        |_| panic!("no blocks"),
        |_, _| panic!("no blocks")
    ));
    assert!(!c.guarding(true));
    for f in [
        Facts {
            passenger: true,
            ..f
        },
        Facts {
            fall_flying: true,
            ..f
        },
        Facts { flying: true, ..f },
        Facts {
            scaffold_enabled: true,
            ..f
        },
    ] {
        c.rotation_update(true, f, |_| panic!("excluded"), |_, _| panic!("excluded"));
        assert!(!c.guarding(true));
    }
}

#[test]
fn clutch_smoke_fall_release_and_lifecycle() {
    let mut c = Clutch::default();
    let f = Facts {
        block_count: 2,
        in_reach: true,
        ..Default::default()
    };
    c.rotation_update(
        true,
        f,
        |_| panic!("in reach"),
        |distance, ticks| {
            assert_eq!((distance, ticks), (5., 60));
            Some(vec![[0., 9., 0.]])
        },
    );
    assert_eq!(c.fall(true), Some(&[[0., 9., 0.]][..]));
    assert!(c.fall(false).is_none());
    assert!(!c.rotation_update(false, f, |_| panic!(), |_, _| panic!()));
    assert!(c.fall(true).is_some());
    assert!(c.rotation_update(
        true,
        Facts {
            block_count: 0,
            ..f
        },
        |_| panic!(),
        |_, _| panic!()
    ));
    assert!(!c.rotation_update(true, f, |_| panic!(), |_, _| None));
    c.rotation_update(true, f, |_| panic!(), |_, _| Some(vec![[0.; 3]]));
    assert!(!c.disabled(true));
    assert!(!c.guarding(true));
    assert!(c.fall(true).is_none());
    c.rotation_update(true, f, |_| panic!(), |_, _| Some(vec![[0.; 3]]));
    assert!(c.disabled(false));
    c.rotation_update(true, f, |_| panic!(), |_, _| Some(vec![[0.; 3]]));
    c.world_change();
    assert!(!c.guarding(true));
    assert!(c.fall(true).is_none());
    assert!(!c.disabled(false));
}

#[test]
fn clutch_smoke_prediction_boundaries_and_momentum() {
    let snap = |fall, on_ground, speed| FallingSnapshot {
        position: [0., 10. - fall, 0.],
        fall_distance: fall,
        on_ground,
        horizontal_velocity_squared: speed,
    };
    assert!(fall_path(5., false, [snap(5., false, 1.)]).is_none());
    assert_eq!(
        fall_path(5., false, [snap(5., false, 1.), snap(5.001, false, 1.)])
            .unwrap()
            .len(),
        2
    );
    assert!(fall_path(5., true, [snap(0., true, 1.), snap(6., false, 1.)]).is_none());
    assert_eq!(
        fall_path(5., false, [snap(0., true, 1.), snap(6., false, 1.)])
            .unwrap()
            .len(),
        2
    );
    assert!(fall_path(5., false, [snap(0., true, 0.), snap(6., false, 1.)]).is_none());
    assert_eq!(fall_path(5., true, [snap(6., true, 0.)]).unwrap().len(), 1);
    assert!(
        fall_path(
            5.,
            false,
            (0..61).map(|i| snap(if i == 60 { 6. } else { 0. }, false, 1.))
        )
        .is_none()
    );
}
