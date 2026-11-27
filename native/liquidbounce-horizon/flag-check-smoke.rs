#[path = "../liquidbounce-behaviors/src/easing.rs"]
mod easing;
#[path = "../liquidbounce-behaviors/src/numeric.rs"]
mod numeric;
pub use easing::Easing;
#[path = "../liquidbounce-behaviors/src/flag_check.rs"]
mod flag_check;
#[test]
fn correction_count_and_fade() {
    use flag_check::*;
    let mut s = State::default();
    s.sync(true, 0);
    let mut p = Player {
        ticks: 25,
        yaw: 175.,
        pitch: 0.,
        health: 20.,
        alive: true,
        food: 20,
    };
    assert!(s.position(p, [1., 2., 3.], [175., 0.], 1).is_none());
    p.ticks = 26;
    let a = s.position(p, [1., 2., 3.], [175., 0.], 1).unwrap();
    assert_eq!(a.reason, Reason::ForceRotate);
    assert_eq!(a.count, 1);
    let a = s.position(p, [4., 5., 6.], [-175., 0.], 2).unwrap();
    assert_eq!(a.reason, Reason::Lagback);
    assert_eq!(a.count, 2);
    assert_eq!(angle_delta(1080., 0.), 720.);
    assert!(s.render_colors(3, true).is_none());
    assert_eq!(s.render_colors(3, false).unwrap()[0] as u32 >> 24, 100);
    assert_eq!(s.render_colors(1252, false).unwrap()[0] as u32 >> 24, 25);
    assert!(s.render_colors(1502, false).is_none());
    s.settings.invalid = true;
    p.food = 0;
    assert_eq!(s.tick(p).unwrap().extra.as_deref(), Some("Hunger"));
    s.clock(30000);
    assert_eq!(s.count, 0);
    s.position(p, [0.; 3], [0.; 2], 30001);
    s.disconnect(p);
    assert_eq!(s.count, 0);
    s.sync(false, 30002);
    assert!(s.tick(p).is_none());
    s.sync(true, 30003);
    assert!(s.finished);
}
