// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/step.rs"]
mod step;
use step::{JUMP_ORDER, Mode, Step};
#[test]
fn five_original_step_modes() {
    let mut s = Step::default();
    assert_eq!(s.height(true, 0.6), 1.);
    let v = s.success(true, 1., 64., |r| {
        assert_eq!(r, [0, 0]);
        2
    });
    assert_eq!(
        v.destinations,
        vec![64. + JUMP_ORDER[1], 64. + JUMP_ORDER[2]]
    );
    assert!(v.award_jump && v.full_packet);
    assert_eq!(s.height(true, 0.6), 0.6);
    s.game_tick(true);
    s.game_tick(true);
    assert_eq!(s.height(true, 0.6), 1.);
    s.settings.trim = true;
    s.settings.simulate_jump_order = [1, 8];
    assert!(
        s.success(true, 0.6, 64., |_| 0)
            .destinations
            .iter()
            .all(|v| *v <= 64.6)
    );
    s.settings.simulate_jump_order = [0, 0];
    assert!(!s.success(true, 1., 64., |_| panic!()).award_jump);
    s.set_mode(Mode::Vulcan);
    let first = s.input(true, false, [true; 3]);
    assert!(first.jump);
    let id = first.wait.unwrap();
    assert!(s.callback(id, true, false).0);
    let a = s.callback(id, true, false).1;
    assert_eq!(a.set_y, None);
    let b = s.callback(a.wait.unwrap(), true, false).1;
    assert_eq!(b.set_y, None);
    assert_eq!(
        s.callback(b.wait.unwrap(), true, false).1.set_y,
        Some(-0.17)
    );
    assert!(!s.stepping);
    let id = s.input(true, false, [true; 3]).wait.unwrap();
    assert!(s.callback(id, true, false).0);
    let a = s.callback(id, true, false).1;
    assert_eq!(a.set_y, Some(0.24680001947880004));
    assert_eq!(a.strafe, Some(0.2));
    let b = s.callback(a.wait.unwrap(), true, false).1;
    assert_eq!(b.set_y, Some(0.));
    assert_eq!(
        s.callback(b.wait.unwrap(), true, false).1.set_y,
        Some(-0.17)
    );
    s.set_mode(Mode::BlocksMc);
    assert!(!s.input(true, true, [true; 3]).jump);
    let a = s.input(true, false, [true; 3]);
    assert_eq!(a.timer, Some((3., 3)));
    assert!(s.queues_outgoing(true, false));
    let b = s.callback(a.wait.unwrap(), true, false).1;
    assert_eq!(b.set_y, Some(0.33));
    let c = s.callback(b.wait.unwrap(), true, false).1;
    assert_eq!(c.set_y, Some(0.25));
    let id = c.wait.unwrap();
    assert!(s.callback(id, true, false).0);
    let d = s.callback(id, true, false).1;
    assert!(d.subtract_y_fraction);
    assert_eq!(d.strafe, Some(0.281));
    assert_eq!(d.timer, Some((0.6, 2)));
    assert!(!s.queues_outgoing(true, false));
    s.set_mode(Mode::Hypixel);
    for (can, extra, final_wait) in [
        ([true, true, true], 0., false),
        ([false, true, true], 0.061, false),
        ([false, false, true], 0.061, true),
    ] {
        let a = s.input(true, false, can);
        assert_eq!(a.set_y, Some(0.42));
        let b = s.callback(a.wait.unwrap(), true, false).1;
        assert_eq!(b.add_y, extra);
        let id = b.wait.unwrap();
        assert!(s.callback(id, true, false).0);
        let c = s.callback(id, true, false).1;
        assert_eq!(c.add_y, if can[0] { -0.14 } else { -0.095 });
        if final_wait {
            let id = c.wait.unwrap();
            for _ in 0..4 {
                assert!(s.callback(id, true, false).0);
            }
            let d = s.callback(id, true, false).1;
            assert_eq!(d.set_y, Some(0.42));
            assert_eq!(d.strafe, Some(0.1838601407459074));
        } else {
            assert_eq!(c.strafe, Some(0.1838601407459074));
        }
        assert!(!s.stepping);
    }
    s.settings.alternate_bypass = true;
    let a = s.input(true, false, [false, false, true]);
    let b = s.callback(a.wait.unwrap(), true, false).1;
    let id = b.wait.unwrap();
    s.callback(id, true, false);
    let c = s.callback(id, true, false).1;
    let id = c.wait.unwrap();
    for _ in 0..4 {
        s.callback(id, true, false);
    }
    assert!(s.callback(id, true, false).1.ground);
    s.set_mode(Mode::Legit);
    assert!(!s.input(true, false, [true; 3]).jump);
    assert_eq!(s.height(true, 0.6), 0.6);
    assert_eq!(Mode::parse("Vulcan286"), Some(Mode::Vulcan));
    assert_eq!(Mode::parse("Hypixel"), Some(Mode::Hypixel));
}
