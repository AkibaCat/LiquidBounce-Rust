#[path = "../../native/liquidbounce-behaviors/src/super_knockback.rs"]
mod super_knockback;
use super_knockback::*;
struct R {
    ticks: i32,
    draws: usize,
}
impl Random for R {
    fn unit(&mut self) -> f32 {
        self.draws += 1;
        0.25
    }
    fn ticks(&mut self, range: [i32; 2]) -> Result<i32, String> {
        if range[0] > range[1] {
            Err("range".into())
        } else {
            Ok(self.ticks)
        }
    }
}
#[test]
fn three_modes_shared_waits_critical_and_server_cooldown() {
    let mut m = SuperKnockback::default();
    let mut r = R { ticks: 1, draws: 0 };
    let mut p = Player {
        forward: 1.0,
        sideways: 0.0,
        sprinting: true,
        was_sprinting: true,
        on_ground: true,
        water_or_bubble: false,
    };
    let mut t = Target {
        living: true,
        hurt_time: 10,
        facing: true,
        would_critical: false,
    };
    assert_eq!(
        m.attack(true, p, t, &mut r).unwrap().packets,
        [false, true, false, true]
    );
    assert_eq!(r.draws, 0);
    p.sprinting = false;
    assert_eq!(
        m.attack(true, p, t, &mut r).unwrap().packets,
        [true, false, true]
    );
    p.sprinting = true;
    p.forward = -1.0;
    assert!(m.attack(true, p, t, &mut r).unwrap().mark_sprint);
    p.sideways = 1.0;
    assert!(!m.attack(true, p, t, &mut r).unwrap().mark_sprint);
    p.sideways = 0.0;
    t.would_critical = true;
    assert!(!m.attack(true, p, t, &mut r).unwrap().mark_sprint);
    t.would_critical = false;
    m.set_mode(Mode::SprintTap);
    let id = m.attack(true, p, t, &mut r).unwrap().wait.unwrap();
    assert!(m.stop_sprint(true));
    assert!(m.callback(id, true, Some(p), &mut r).unwrap().0);
    p.sprinting = false;
    assert!(m.callback(id, true, Some(p), &mut r).unwrap().0);
    p.was_sprinting = false;
    let (keep, step) = m.callback(id, true, Some(p), &mut r).unwrap();
    assert!(!keep);
    let next = step.wait.unwrap();
    assert_ne!(id, next);
    assert!(m.cancel_sprint);
    assert!(!m.callback(next, true, Some(p), &mut r).unwrap().0);
    assert!(!m.cancel_sprint);
    p.sprinting = true;
    p.was_sprinting = true;
    p.forward = 1.0;
    m.set_mode(Mode::WTap);
    let id = m.attack(true, p, t, &mut r).unwrap().wait.unwrap();
    assert!(m.in_sequence && !m.cancel_movement);
    let next = m
        .callback(id, true, Some(p), &mut r)
        .unwrap()
        .1
        .wait
        .unwrap();
    assert!(m.block_movement(true));
    assert_ne!(id, next);
    assert!(m.callback(next, true, Some(p), &mut r).unwrap().0);
    p.forward = 0.0;
    let allow = m
        .callback(next, true, Some(p), &mut r)
        .unwrap()
        .1
        .wait
        .unwrap();
    assert!(m.cancel_movement);
    m.callback(allow, true, Some(p), &mut r).unwrap();
    assert!(!m.in_sequence && !m.cancel_movement);
    r.ticks = 0;
    p.forward = 1.0;
    let id = m.attack(true, p, t, &mut r).unwrap().wait.unwrap();
    assert!(m.cancel_movement);
    m.cancel();
    assert!(!m.callback(id, true, Some(p), &mut r).unwrap().0);
    m.set_mode(Mode::SprintTap);
    let id = m.attack(true, p, t, &mut r).unwrap().wait.unwrap();
    p.sprinting = false;
    p.was_sprinting = false;
    m.callback(id, true, Some(p), &mut r).unwrap();
    assert!(!m.cancel_sprint);
    let mut cd = cooldown::Cooldown::default();
    assert_eq!(cd.progress(20.0, 20.0, 5.0), 4.1);
    cd.outgoing(10.0);
    assert_eq!(cd.progress(10.0, 20.0, 5.0), 0.1);
    cd.set_time(1000);
    cd.set_time(2000);
    assert!(cd.tps.is_nan());
    cd.drain(20.0);
    assert_eq!(cd.tps, 20.0);
    cd.set_time(4000);
    cd.drain(20.0);
    assert!((cd.tps - 13.3333333333).abs() < 1.0e-8);
    assert_eq!(cd.progress(20.0, 20.0, 5.0), 1.5);
    cd.respawn();
    cd.drain(21.0);
    assert_eq!(cd.reset_ticks, 21.0);
    cd.world_change(22.0);
    assert_eq!(cd.reset_ticks, 22.0);
    cd.disconnect(23.0);
    assert!(cd.tps.is_nan());
    assert_eq!(Mode::parse("sprinttap"), Some(Mode::SprintTap));
}
