// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/reverse_step.rs"]
mod reverse_step;
#[test]
fn reverse_step_protects_jumps_and_replays_only_successful_landings() {
    use reverse_step::*;
    let mut state = ReverseStep::default();
    let mut s = Settings::default();
    state.observe(true, -0.1, false);
    assert!(state.eligible(s, false, 1., false, true));
    assert!(!state.eligible(s, false, 1.01, false, true));
    assert!(!state.eligible(s, false, 0., true, true));
    assert!(!state.eligible(s, false, 0., false, false));
    state.jump();
    assert!(!state.eligible(s, false, 0., false, true));
    state.observe(true, -0.1, false);
    assert!(state.initiated_jump);
    state.observe(true, -0.1, true);
    assert!(!state.initiated_jump);
    state.observe(true, 0.1, false);
    assert!(state.initiated_jump);
    s.mode = Mode::Strict;
    s.motion = 2.;
    assert_eq!(velocity(s, [0.3, -0.2, 0.4]), Some([0.3, -2., 0.4]));
    s.mode = Mode::Accelerator;
    s.factor = 2.;
    assert_eq!(velocity(s, [0.3, -0.2, 0.4]), Some([0., -0.4, 0.]));
    assert_eq!(velocity(s, [0., 0., 0.]), None);
    struct Replay {
        tick: i32,
        land: i32,
    }
    impl Simulation for Replay {
        fn sample(&self) -> Sample {
            Sample {
                position: [1., 10. - f64::from(self.tick), 2.],
                on_ground: self.tick >= self.land,
                horizontal_collision: self.tick == 1,
            }
        }
        fn tick(&mut self) {
            self.tick += 1;
        }
    }
    s.mode = Mode::Instant;
    s.ticks = 2;
    s.simulate_falling = true;
    let landed = instant(s, &mut Replay { tick: 0, land: 2 }).unwrap();
    assert_eq!(landed.position, [1., 8., 2.]);
    assert_eq!(landed.packets.len(), 2);
    assert!(landed.packets[0].horizontal_collision);
    assert!(landed.packets[1].on_ground);
    assert_eq!(instant(s, &mut Replay { tick: 0, land: 3 }), None);
    s.simulate_falling = false;
    assert!(
        instant(s, &mut Replay { tick: 0, land: 2 })
            .unwrap()
            .packets
            .is_empty()
    );
}
