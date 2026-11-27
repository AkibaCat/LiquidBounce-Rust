#[path = "../liquidbounce-behaviors/src/fast_exp.rs"]
mod fast_exp;
use fast_exp::*;
#[test]
fn repair_threshold_fifo_aim_throw_and_orb_smoke() {
    let mut s = FastExp::default();
    let mut c = Context {
        valid: true,
        slot: Some(-1),
        count: 64,
        items: vec![RepairItem {
            damage: 140,
            max_damage: 200,
        }],
        pitch: 86.,
        ..Default::default()
    };
    let mut random = |r: [f32; 2]| r[0];
    let e = s.start(&c, &mut random);
    assert_eq!(s.stage, Stage::BeforeOrbs(10));
    let id = e.register.unwrap();
    c.orbs = true;
    assert!(s.resume(id, &c, &mut random).0);
    c.orbs = false;
    let (_, e) = s.resume(id, &c, &mut random);
    assert!(e.pause);
    assert_eq!(s.stage, Stage::Aim);
    let id = e.register.unwrap();
    c.pitch = 85.;
    let (keep, e) = s.resume(id, &c, &mut random);
    assert!(keep && e.rotation);
    c.pitch = 86.;
    let (keep, e) = s.resume(id, &c, &mut random);
    assert!(!keep);
    assert_eq!(e.throws, 0);
    assert_eq!(s.items_to_throw, 0.5);
    s.settings.fast = true;
    s.settings.items_per_tick = [10., 10.];
    let e = s.start(&c, &mut random);
    let (_, e) = s.resume(e.register.unwrap(), &c, &mut random);
    let (_, e) = s.resume(e.register.unwrap(), &c, &mut random);
    assert_eq!(e.throws, 10);
    assert_eq!(s.stage, Stage::Appear(0));
    let id = e.register.unwrap();
    for _ in 0..9 {
        assert!(s.resume(id, &c, &mut random).0);
    }
    let (_, e) = s.resume(id, &c, &mut random);
    assert_eq!(s.stage, Stage::Absorb);
    c.orbs = true;
    assert!(s.resume(e.register.unwrap(), &c, &mut random).0);
    c.orbs = false;
    s.resume(e.register.unwrap(), &c, &mut random);
    assert_eq!(s.bottles_used, 0);
    assert!(s.repairing);
    c.items[0].damage = 10;
    assert_eq!(s.required(&c), 0);
    assert!(!s.repairing);
    let fraction = s.items_to_throw;
    s.cancel();
    assert_eq!(s.items_to_throw, fraction);
    s.disabled();
    assert_eq!(s.items_to_throw, 0.);
    s.settings.no_waste = false;
    s.settings.rotate = false;
    c.handled = true;
    assert_eq!(s.start(&c, &mut random).throws, 0);
    c.handled = false;
    assert_eq!(s.start(&c, &mut random).throws, 10);
    assert_eq!(s.stage, Stage::Idle);
}
