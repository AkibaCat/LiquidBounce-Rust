// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/liquid_walk.rs"]
mod model;
use model::*;
#[test]
fn original_liquid_walk_policies() {
    for mode in [Mode::Vanilla, Mode::Ncp, Mode::Verus] {
        assert!(shape_allowed(Some(mode), false, 3., false, false));
        assert!(!shape_allowed(Some(mode), false, 3.01, false, false));
        assert!(!shape_allowed(Some(mode), true, 0., false, false));
        assert!(!shape_allowed(Some(mode), false, 0., true, false));
        assert!(!shape_allowed(Some(mode), false, 0., false, true));
    }
    assert!(!shape_allowed(Some(Mode::Vulcan), false, 0., false, false));
    assert!(input_jump(Some(Mode::Vanilla), false, true));
    assert!(!input_jump(Some(Mode::Vanilla), true, true));
    assert!(!input_jump(Some(Mode::Ncp), false, true));
    let mut s = LiquidWalk::default();
    s.settings.mode = Mode::Ncp;
    assert_eq!(
        s.tick(true, false, [true, false, false, false]).velocity_y,
        Some(0.08)
    );
    assert_eq!(
        s.tick(true, true, [true, false, false, false]).velocity_y,
        None
    );
    let water = [false, true, false, false];
    assert!(!s.packet(true, false, false, water).subtract_y);
    assert!(s.packet(true, false, false, water).subtract_y);
    assert_eq!(s.packet(true, true, false, water), Packet::default());
    assert!(!s.packet(true, false, false, water).subtract_y);
    assert!(cancels_jump(Some(Mode::Ncp), true));
    assert!(!cancels_jump(Some(Mode::Verus), true));
    s.settings.mode = Mode::Verus;
    assert_eq!(s.packet(true, false, false, water).ground, Some(true));
    assert_eq!(s.packet(true, false, false, water).ground, Some(false));
    assert_eq!(s.packet(true, false, true, water).ground, None);
    assert_eq!(s.packet(true, false, false, water).ground, Some(true));
    assert_eq!(
        s.packet(true, false, false, [false, true, true, false])
            .ground,
        None
    );
    s.settings.mode = Mode::Vulcan;
    let t = s.tick(true, false, [false, false, false, true]);
    assert_eq!(t.velocity_y, Some(f64::from(0.8f32)));
    assert_eq!(t.timer, Some(1.125));
    let t = s.tick(true, false, [true, true, false, false]);
    assert_eq!(t.velocity_y, None);
    assert_eq!(t.timer, Some(1.));
    for name in MODES {
        let m = Mode::parse(name).unwrap();
        assert_eq!(m.tag(), name);
        assert_eq!(Mode::from_code(m.code()), Some(m));
    }
    assert_eq!(Mode::parse("NoCheatPlus"), Some(Mode::Ncp));
    assert_eq!(Mode::parse("VerusB3901"), Some(Mode::Verus));
    assert_eq!(Mode::parse("Vulcan291"), Some(Mode::Vulcan));
}
