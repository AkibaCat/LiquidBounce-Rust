#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/clock.rs"]
mod clock;
#[path = "../liquidbounce-behaviors/src/fake_lag.rs"]
mod fake_lag;
use fake_lag::{Context, FakeLag, PacketKind as P};
fn queues(s: &mut FakeLag, now: i64, oldest: Option<i64>, c: Context, p: P) -> bool {
    s.queues(true, true, p, now, oldest, c, |_| 450)
}
fn main() {
    let mut s = FakeLag::new(300);
    let mut c = Context::default();
    assert_eq!(s.settings.range, [2., 5.]);
    assert_eq!(s.settings.delay, [300, 600]);
    assert_eq!(s.settings.flush_on, 7);
    assert!(!queues(&mut s, 250, None, c, P::Other));
    s.enemy_nearby = true;
    assert!(!queues(&mut s, 249, None, c, P::Other));
    assert!(queues(&mut s, 250, None, c, P::Other));
    for outgoing in [true, false] {
        assert!(
            !s.queues(false, outgoing, P::Other, 1000, None, c, |_| panic!(
                "disabled"
            ))
        );
    }
    assert!(
        !s.queues(true, false, P::ResourcePack, 1000, None, c, |_| panic!(
            "incoming guard"
        ))
    );
    assert_eq!(s.chronometer.last_update(), 0);
    c.old_position = true;
    assert!(!queues(&mut s, 1000, None, c, P::Other));
    c.has_entities = true;
    c.server_distance = 4.;
    c.client_distance = 3.;
    assert!(queues(&mut s, 1000, None, c, P::Other));
    c.server_distance = 3.;
    assert!(queues(&mut s, 1000, None, c, P::Other));
    c.server_distance = 2.999;
    assert!(!queues(&mut s, 1000, None, c, P::Other));
    c.server_distance = 4.;
    c.intersects = true;
    assert!(!queues(&mut s, 1000, None, c, P::Other));
    c = Context::default();
    s.settings.constant = true;
    s.enemy_nearby = false;
    assert!(queues(&mut s, 1000, None, c, P::Other));
    for guard in 0..4 {
        let mut guarded = c;
        match guard {
            0 => guarded.dead = true,
            1 => guarded.in_water = true,
            2 => guarded.screen = true,
            _ => guarded.using_consumable = true,
        };
        assert!(!queues(&mut s, 1000, None, guarded, P::Other));
    }
    for p in [
        P::EntityInteract,
        P::BlockInteract,
        P::Action,
        P::ResourcePack,
    ] {
        assert!(!queues(&mut s, 1000, None, c, p));
        assert_eq!(s.chronometer.last_update(), 1000);
        assert!(!queues(&mut s, 1249, None, c, P::Other));
        assert!(queues(&mut s, 1250, None, c, P::Other));
        s.chronometer.reset_at(0);
    }
    s.settings.flush_on = 0;
    assert!(queues(&mut s, 1000, None, c, P::EntityInteract));
    assert!(!queues(&mut s, 1000, None, c, P::ResourcePack));
    s.chronometer.reset_at(0);
    assert!(queues(&mut s, 1000, Some(701), c, P::Other));
    assert!(!queues(&mut s, 1000, Some(700), c, P::Other));
    assert_eq!(s.next_delay, 450);
    // Delay expiration returns before FlushOn, so this packet must not reset recoil.
    assert!(!queues(&mut s, 1000, Some(550), c, P::ResourcePack));
    assert_eq!(s.chronometer.last_update(), 0);
    s.settings.constant = false;
    c.shoot_constant = true;
    c.no_shoot_target = true;
    assert!(queues(&mut s, 1000, None, c, P::Other));
    c.no_shoot_target = false;
    assert!(!queues(&mut s, 1000, None, c, P::Other));
    s.enemy_nearby = true;
    s.chronometer.reset_at(600);
    s.disabled();
    assert!(!s.enemy_nearby);
    assert_eq!(s.next_delay, 450);
    assert_eq!(s.chronometer.last_update(), 600);
    c = Context::default();
    s.settings.constant = true;
    s.chronometer.reset_at(i64::MAX - 10);
    assert!(!queues(&mut s, i64::MIN + 100, None, c, P::Other));
    println!(
        "FakeLag guards, dynamic geometry, constant-lag consumer, packet flush groups, clocks, delay sampling and lifecycle passed"
    );
}
