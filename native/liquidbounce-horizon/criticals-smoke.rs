#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/clicking.rs"]
mod clicking;
#[path = "../liquidbounce-behaviors/src/criticals.rs"]
mod criticals;
use clicking::Cooldown;
use criticals::{Blink, BlinkAction, Criticals, Player};
fn cooldown(ticks: f64) -> Cooldown {
    Cooldown {
        real_ticks: ticks,
        reset_ticks: 0.,
        server_tick_speed: 1.,
        attack_delay: 20.,
        has_cooldown: true,
    }
}
#[test]
fn criticals_modes_and_continuations() {
    let mut s = Criticals::default();
    let c = cooldown(20.);
    let mut p = Player {
        allows_ignoring_ground: true,
        on_ground: true,
        y: 64.,
        old_y: 64.,
        tick_count: 4,
        ..Default::default()
    };
    let expected = [
        vec![(0.2, false), (0.01, false)],
        vec![(0.11, false), (0.1100013579, false), (0.0000013579, false)],
        vec![
            (0.0625, false),
            (0.0625013579, false),
            (0.0000013579, false),
        ],
        vec![(1e-9, false), (0., false)],
        vec![(-1e-9, false)],
        vec![],
        vec![(0.0011, true), (0., false)],
    ];
    for (mode, offsets) in expected.into_iter().enumerate() {
        s.settings.packet_mode = mode;
        assert_eq!(s.offsets(true, true, p, c, false), offsets);
    }
    s.settings.packet_mode = 5;
    p.on_ground = false;
    assert_eq!(s.offsets(true, true, p, c, false), vec![(-0.000001, false)]);
    p.on_ground = true;
    s.settings.packet_mode = 1;
    p.sprinting = true;
    assert!(s.offsets(true, true, p, c, false).is_empty());
    s.settings.stop_sprinting = 0;
    assert!(!s.offsets(true, true, p, c, true).is_empty());
    assert!(s.offsets(false, true, p, c, true).is_empty());
    assert!(s.offsets(true, false, p, c, true).is_empty());
    p.sprinting = false;
    s.settings.mode = 6;
    s.rewound_ticks = 100;
    s.transition(false, true, 6);
    assert_eq!(s.rewound_ticks, 0);
    assert_eq!(s.rewound(p), None);
    assert_eq!(s.rewound(p), Some(0.003));
    assert_eq!(s.rewound(p), Some(0.001 - 3e-10));
    p.y = 65.;
    assert_eq!(s.rewound(p), None);
    assert_eq!(s.rewound_ticks, 0);
    p.y = 64.;
    s.settings.mode = 3;
    s.settings.jump_height = 0.1;
    assert!(s.jump_active(true, [false; 3]));
    s.settings.check_trigger_bot = true;
    assert!(!s.jump_active(true, [false; 3]));
    assert!(s.jump_active(true, [false, false, true]));
    let mut jump = false;
    s.jump_input(true, p, true, false, &mut jump);
    assert!(jump && s.adjust_next_jump);
    let mut honey = 0.21;
    s.jump(&mut honey);
    assert_eq!(honey, 0.21);
    assert!(s.adjust_next_jump);
    let mut normal = 0.42;
    s.jump(&mut normal);
    assert_eq!(normal, 0.1);
    assert!(!s.adjust_next_jump);
    assert!(!criticals::wait_for_jump(cooldown(0.), 0.42, None));
    assert!(criticals::wait_for_jump(cooldown(0.), 0.42, Some(20)));
    assert!(!criticals::wait_for_jump(c, 0.42, Some(20)));
    p.on_ground = false;
    p.fall_distance = 1.;
    p.sprinting = true;
    s.settings.sprint_enabled = true;
    s.settings.stop_sprinting = 1;
    assert!(s.sprint_running(true, p, c, true));
    assert!(s.stop_sprint(0, true));
    assert!(s.stop_sprint(1, true));
    assert!(!s.stop_sprint(2, true));
    s.settings.stop_sprinting = 2;
    assert!(s.stop_sprint(0, true));
    assert!(!s.stop_sprint(1, true));
    assert!(s.stop_sprint(2, true));
    assert!(!s.sprint_running(true, p, c, false));
    let mut blink = Blink::new(300);
    blink.enemy = true;
    assert_eq!(
        blink.action(true, false, 0, false, || panic!()),
        BlinkAction::Queue
    );
    blink.enqueue(7, 0, "movement");
    assert_eq!(
        blink.action(true, false, 50, true, || panic!()),
        BlinkAction::Pass
    );
    assert!(blink.in_state);
    assert_eq!(
        blink.action(true, false, 300, false, || 600),
        BlinkAction::Flush
    );
    assert_eq!(blink.next_delay, 600);
    assert!(blink.in_state);
    let queue = blink.take();
    assert_eq!(
        (queue[0].session, queue[0].millis, queue[0].packet),
        (7, 0, "movement")
    );
    blink.enqueue(8, 500, "next-session");
    assert_eq!(
        blink.action(false, false, 501, false, || panic!()),
        BlinkAction::Flush
    );
    assert!(!blink.in_state);
    assert_eq!(blink.take()[0].session, 8);
    blink.enqueue(8, 600, "discard");
    blink.disconnect();
    assert!(blink.queue.is_empty());
}
