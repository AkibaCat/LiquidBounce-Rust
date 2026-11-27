#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_wind_charge.rs"]
mod auto_wind_charge;
#[path = "../liquidbounce-behaviors/src/no_rotate_set.rs"]
mod no_rotate_set;
#[path = "../liquidbounce-behaviors/src/random.rs"]
mod random;
#[path = "../liquidbounce-behaviors/src/rotations.rs"]
mod rotations;
use auto_wind_charge::*;
use no_rotate_set::*;
use rotations::*;
fn main() {
    let mut s = AutoWindCharge::default();
    let mut c = Context {
        jump_key: true,
        slot: Some(-1),
        on_ground: true,
        yaw: 30.,
        keys: [true, false, true, false],
        ..Default::default()
    };
    let mut random = || 0.25;
    let r = s.begin(&c, &mut random).unwrap().unwrap();
    assert_eq!(r, Rotation::new(30., 90.));
    for tick in 0..12 {
        let e = s.finish(&c, r, false, 90.);
        assert!(e.aim.is_some());
        assert_eq!(e.throw, None);
        assert_eq!(s.hold_jump, tick < 10);
    }
    s.begin(&c, &mut random).unwrap();
    assert!(!s.hold_jump);
    let e = s.finish(&c, r, false, 0.);
    assert_eq!(e.throw, Some(-1));
    assert!(s.hold_jump);
    assert_eq!(s.aim_ticks, 0);
    let mut jump = true;
    s.movement_input(true, &mut jump);
    assert!(!jump);
    c.cooldown = true;
    s.aim_ticks = 3;
    s.begin(&c, &mut random).unwrap();
    assert!(!s.hold_jump);
    assert_eq!(s.aim_ticks, 3);
    c.cooldown = false;
    c.boost_key = true;
    s.settings.pitch = FloatRange(60., 80.);
    let r = s.begin(&c, &mut random).unwrap().unwrap();
    assert_eq!(r.yaw, -195.);
    let cached = r.pitch;
    c.boost_key = false;
    assert_eq!(s.begin(&c, &mut random).unwrap().unwrap().pitch, 90.);
    c.boost_key = true;
    assert_eq!(s.begin(&c, &mut || 0.8).unwrap().unwrap().pitch, cached);
    c.jump_key = false;
    s.begin(&c, &mut random).unwrap();
    assert_eq!(s.aim_ticks, 0);
    assert_eq!(s.horizontal_pitch, Some(cached));
    c.jump_key = true;
    c.on_ground = false;
    let e = s.finish(&c, r, true, 50.);
    assert_eq!(e.aim, Some(r));
    assert_eq!(e.throw, None);
    assert!(s.finish(&c, r, false, 50.).aim.is_none());
    s.settings.rotate = false;
    assert!(s.finish(&c, r, true, 50.).aim.is_none());
    c.on_ground = true;
    assert_eq!(s.finish(&c, r, false, 179.).throw, Some(-1));
    assert_eq!(s.horizontal_pitch, None);
    c.slot = None;
    assert!(s.begin(&c, &mut random).unwrap().is_none());
    s.disabled();
    assert!(!s.hold_jump);
    assert_eq!(movement_direction(30., [false, true, true, false]), 255.);
    assert_eq!(movement_direction(30., [true, true, true, true]), 30.);
    let mut r = RotationsSettings::default();
    let at = Rotation::new(0., 0.);
    let down = Rotation::new(0., 90.);
    let mut draws = 0;
    let mut random = || {
        draws += 1;
        0.5
    };
    assert_eq!(
        calculate_ticks(&r, at, down, None, at, &mut random).unwrap(),
        0
    );
    r.linear.vertical_turn_speed = FloatRange(15., 60.);
    r.linear.horizontal_turn_speed = FloatRange(15., 60.);
    assert_eq!(
        calculate_ticks(&r, at, down, None, at, &mut random).unwrap(),
        5
    );
    r.active = SmoothMode::Sigmoid;
    let sigmoid = calculate_ticks(&r, at, down, None, at, &mut random).unwrap();
    assert!((0..=80).contains(&sigmoid));
    assert_eq!(draws, 0);
    r.active = SmoothMode::Acceleration;
    r.acceleration.acceleration_error = false;
    r.acceleration.constant_error = false;
    r.acceleration.yaw_acceleration = FloatRange(20., 20.);
    r.acceleration.pitch_acceleration = FloatRange(20., 20.);
    // A zero yaw delta and zero yaw turn create NaN, which returns zero in the source.
    assert_eq!(
        calculate_ticks(&r, at, down, None, at, &mut || 0.5).unwrap(),
        0
    );
    assert_eq!(
        calculate_ticks(&r, at, Rotation::new(90., 90.), None, at, &mut || 0.5).unwrap(),
        4
    );
    r.active = SmoothMode::Linear;
    r.linear.horizontal_turn_speed = FloatRange(0., 0.);
    r.linear.vertical_turn_speed = FloatRange(0., 0.);
    assert_eq!(
        calculate_ticks(&r, at, down, None, at, &mut || panic!(
            "factor smoothing must not draw"
        ))
        .unwrap(),
        80
    );
    let absolute = Rotation::new(1_000_000., 20.);
    r.active = SmoothMode::Acceleration;
    assert_eq!(
        calculate_ticks(
            &r,
            absolute,
            Rotation::new(1_000_090., 110.),
            None,
            absolute,
            &mut || 0.5
        )
        .unwrap(),
        4
    );
    println!(
        "AutoWindCharge: ground aim grace, jump gating, pitch cache, air landing, offhand, cooldown and all calculateTicks modes passed"
    );
}
