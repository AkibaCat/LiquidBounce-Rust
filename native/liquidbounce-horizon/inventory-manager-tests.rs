use crate::{inventory_manager::*, random::ThreadRandom, vomit::Constraints};

fn state() -> Context {
    let geometry = Geometry {
        object: 1,
        slots: (0..46)
            .map(|i| Position {
                x: (i % 9) as f32 * 18.0,
                y: (i / 9) as f32 * 18.0,
            })
            .collect(),
    };
    Context {
        in_game: true,
        creative: true,
        no_screen: true,
        menu: MenuState {
            container_id: 0,
            stacks: vec![Stack::default(); 46],
            carried: Stack::default(),
        },
        inventory_menu: geometry,
        player_stacks: vec![Stack::default(); 41],
        requirements: Requirements {
            breaking: Some(false),
            rotation_matches: true,
            ..Default::default()
        },
        ..Default::default()
    }
}
fn zero() -> OwnerState {
    OwnerState {
        running: true,
        guards_pass: true,
        constraints: Constraints {
            start_delay: [0, 0],
            click_delay: [0, 0],
            close_delay: [0, 0],
            miss_chance: [0, 0],
            slot_multiplier: 0.0,
            ..Default::default()
        },
    }
}
fn stack(name: &str, count: i32) -> Stack {
    Stack {
        components: name.into(),
        count,
        wire: String::new(),
    }
}
fn set(c: &mut Context, slot: Slot, value: Stack) {
    match slot {
        Slot::Player(id) => {
            c.player_stacks[usize::from(id)] = value.clone();
            let server = slot.server_id(None).unwrap();
            c.menu.stacks[server as usize] = value;
        }
        Slot::Container(id) => c.menu.stacks[id as usize] = value,
    }
}
fn native(m: &mut Manager, c: &mut Context, effect: &Effect) -> bool {
    match effect {
        Effect::Action(action) => {
            match action {
                Action::Click {
                    screen,
                    slot,
                    input,
                    button,
                } => {
                    if *input == Input::Throw {
                        set(c, *slot, Stack::default());
                    } else if *input == Input::Swap {
                        let a = c.player_stacks[if let Slot::Player(s) = slot {
                            usize::from(*s)
                        } else {
                            panic!()
                        }]
                        .clone();
                        let b = c.player_stacks[*button as usize].clone();
                        set(c, *slot, b);
                        set(c, Slot::Player(*button as u8), a);
                    } else if *input == Input::Pickup && c.menu.carried.count > 0 {
                        let carried = std::mem::take(&mut c.menu.carried);
                        set(c, *slot, carried);
                    }
                    m.sent_click(screen.as_ref().map_or(0, |s| s.container_id));
                }
                // Official handleCreativeModeItemAdd sends only. Native
                // inventory changes wait for a real server packet.
                Action::Creative { .. } => {}
                _ => {}
            }
            true
        }
        Effect::CloseInventory => m.outgoing_close(0),
        Effect::MoveCursor(p) => {
            c.visible_cursor = *p;
            true
        }
        _ => true,
    }
}
fn drive(
    m: &mut Manager,
    c: &mut Context,
    o: &mut OwnerState,
    rng: &mut ThreadRandom,
) -> Vec<Effect> {
    let mut result = vec![];
    for _ in 0..1000 {
        let Some(emission) = m.step(c, &mut |_| o.clone(), rng).unwrap() else {
            return result;
        };
        assert!(
            m.step(c, &mut |_| o.clone(), rng).unwrap().is_none(),
            "reentrant call must not duplicate native effects"
        );
        let okay = native(m, c, &emission.effect);
        assert!(m.acknowledge(emission.ticket, okay, c));
        result.push(emission.effect);
    }
    panic!("source execution did not reach a real suspension")
}
fn create(index: i32, slot: u8) -> Action {
    Action::Creative {
        stack: CreativeStack::Block {
            registry_index: index,
            count: 64,
        },
        slot: Some(Slot::Player(slot)),
    }
}
fn throw(slot: u8) -> Action {
    Action::Click {
        screen: None,
        slot: Slot::Player(slot),
        button: 1,
        input: Input::Throw,
    }
}

#[test]
fn non_player_chains_then_priority_then_fifo_are_selected_atomically() {
    let mut m = Manager::default();
    let mut c = state();
    let mut o = zero();
    let mut rng = ThreadRandom::seeded(123, 1);
    for (id, priority, actions) in [
        (1, 99, vec![throw(10)]),
        (2, 0, vec![create(4, 9), create(5, 10)]),
        (3, 5, vec![create(6, 11)]),
        (4, 5, vec![create(7, 12)]),
    ] {
        m.submit(Request::new(id, id, priority, actions, &c, &m));
    }
    // Creative adds do not change local menu contents, so all pending plans
    // still match; non-player chains precede the higher-priority player click.
    m.begin_tick(&c);
    let effects = drive(&mut m, &mut c, &mut o, &mut rng);
    let actions = effects
        .iter()
        .filter_map(|e| {
            if let Effect::Outcome(Outcome::Performed { request, .. }) = e {
                Some(*request)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(actions, [3, 4, 2, 2, 1]);
    assert!(!effects.iter().any(|effect| matches!(effect,Effect::Outcome(Outcome::Cancelled{..}))));
}
#[test]
fn active_chain_finishes_when_owner_stops_and_other_plans_revalidate() {
    let mut m = Manager::default();
    let mut c = state();
    let mut o = zero();
    let mut rng = ThreadRandom::seeded(4, 7);
    m.submit(Request::new(
        1,
        8,
        0,
        vec![create(4, 9), create(5, 10), throw(9), throw(10)],
        &c,
        &m,
    ));
    m.begin_tick(&c);
    let first = m.step(&c, &mut |_| o.clone(), &mut rng).unwrap().unwrap();
    assert!(matches!(
        first.effect,
        Effect::Action(Action::Creative { .. })
    ));
    native(&mut m, &mut c, &first.effect);
    m.acknowledge(first.ticket, true, &c);
    o.running = false;
    o.guards_pass = false;
    let effects = drive(&mut m, &mut c, &mut o, &mut rng);
    assert_eq!(
        effects
            .iter()
            .filter(|e| matches!(e, Effect::Outcome(Outcome::Performed { .. })))
            .count(),
        4
    );
    assert!(effects.contains(&Effect::Outcome(Outcome::Completed { request: 1 })));
    assert!(c.player_stacks[9].empty() && c.player_stacks[10].empty());
    assert!(!m.busy());
}
#[test]
fn request_cannot_start_after_external_menu_or_locked_count_change() {
    let mut m = Manager::default();
    let mut c = state();
    let mut o = zero();
    let mut rng = ThreadRandom::seeded(4, 7);
    m.submit(Request::new(1, 8, 0, vec![create(4, 9)], &c, &m));
    set(&mut c, Slot::Player(1), stack("external", 1));
    m.begin_tick(&c);
    let effects = drive(&mut m, &mut c, &mut o, &mut rng);
    assert_eq!(
        effects,
        [Effect::Outcome(Outcome::Cancelled { request: 1 })]
    );
    assert!(!m.busy());
}
#[test]
fn recovery_waits_twice_and_old_tick_callback_never_advances_new_wait() {
    let mut m = Manager::default();
    let mut c = state();
    c.creative = false;
    c.menu.carried = stack("cursor", 3);
    let mut o = zero();
    let mut rng = ThreadRandom::seeded(4, 7);
    m.begin_tick(&c);
    assert_eq!(
        drive(&mut m, &mut c, &mut o, &mut rng),
        [Effect::OpenInventory]
    );
    let first = m.take_wait_registrations()[0];
    assert!(drive(&mut m, &mut c, &mut o, &mut rng).is_empty());
    m.advance_callback(first);
    let effects = drive(&mut m, &mut c, &mut o, &mut rng);
    assert!(matches!(
        effects[0],
        Effect::Action(Action::Click {
            input: Input::Pickup,
            ..
        })
    ));
    let second = m.take_wait_registrations()[0];
    m.advance_callback(first);
    assert!(drive(&mut m, &mut c, &mut o, &mut rng).is_empty());
    m.advance_callback(second);
    drive(&mut m, &mut c, &mut o, &mut rng);
    assert!(c.menu.carried.empty());
    assert_eq!(c.player_stacks[9].count, 3);
    assert!(!m.busy());
}
#[test]
fn actual_click_pause_is_in_milliseconds_and_constraints_are_live_after_suspend() {
    let mut m = Manager::default();
    let mut c = state();
    let mut o = zero();
    o.constraints.click_delay = [7, 7];
    let mut rng = ThreadRandom::seeded(4, 7);
    m.submit(Request::new(
        1,
        8,
        0,
        vec![create(4, 9), create(5, 10)],
        &c,
        &m,
    ));
    m.begin_tick(&c);
    let effects = drive(&mut m, &mut c, &mut o, &mut rng);
    assert_eq!(
        effects
            .iter()
            .filter(|e| matches!(e, Effect::Action(_)))
            .count(),
        1
    );
    c.now = 6;
    assert!(drive(&mut m, &mut c, &mut o, &mut rng).is_empty());
    c.now = 7;
    o.constraints.click_delay = [3, 3];
    let effects = drive(&mut m, &mut c, &mut o, &mut rng);
    assert_eq!(
        effects
            .iter()
            .filter(|e| matches!(e, Effect::Action(_)))
            .count(),
        1
    );
    assert!(effects.contains(&Effect::Outcome(Outcome::Completed { request: 1 })));
    c.now = 9;
    assert!(m.busy());
    c.now = 10;
    drive(&mut m, &mut c, &mut o, &mut rng);
    assert!(!m.busy());
}
#[test]
fn throw_probability_rng_draws_happen_even_though_throw_cannot_miss() {
    let mut m = Manager::default();
    let mut c = state();
    set(&mut c, Slot::Player(9), stack("stone", 64));
    let mut o = zero();
    o.constraints.miss_chance = [100, 100];
    m.set_open_server_side(true);
    let mut rng = ThreadRandom::seeded(123, 1);
    let mut expected = rng;
    // recent start, miss range, probability, two position floats, travel range,
    // click delay, close delay, maximum close delay has no extra random draw.
    expected.next_i32_inclusive(0, 0);
    expected.next_i32_inclusive(100, 100);
    expected.next_i32_bound(100);
    expected.next_unit();
    expected.next_unit();
    expected.next_i32_inclusive(0, 0);
    expected.next_i32_inclusive(0, 0);
    expected.next_i32_inclusive(0, 0);
    m.submit(Request::new(1, 1, 0, vec![throw(9)], &c, &m));
    m.begin_tick(&c);
    let effects = drive(&mut m, &mut c, &mut o, &mut rng);
    assert_eq!(
        effects
            .iter()
            .filter(|e| matches!(e, Effect::Action(_)))
            .count(),
        1
    );
    assert_eq!(rng.next_i32(), expected.next_i32());
}
#[test]
fn server_restore_is_judged_after_response_then_blocks_and_locks_after_repeated_rejection() {
    let mut m = Manager::default();
    m.response_time = 10;
    m.lock_after_rejections = 2;
    let mut c = state();
    let mut o = zero();
    let mut rng = ThreadRandom::seeded(4, 7);
    let item = stack("stone{components}", 64);
    for id in 1..=2 {
        c.now = (id - 1) * 30;
        set(&mut c, Slot::Player(9), item.clone());
        m.submit(Request::new(
            id as u64,
            id as u64,
            0,
            vec![throw(9)],
            &c,
            &m,
        ));
        m.begin_tick(&c);
        drive(&mut m, &mut c, &mut o, &mut rng);
        m.resync_slot(0, 9, &item, c.now + 1);
        assert!(m.revert_pending(c.now + 10));
        m.poll_timers(c.now + 11);
        assert!(m.revert_pending(c.now + 20));
        m.poll_timers(c.now + 21);
        assert!(!m.revert_pending(c.now + 21));
    }
    assert_eq!(m.locked_count(), 1);
    assert!(m.is_locked(&item));
    assert!(!m.is_locked(&stack("different", 64)));
    m.world_change();
    assert_eq!(m.locked_count(), 0);
}
#[test]
fn moving_reject_and_confirmed_resync_do_not_lock_items() {
    let mut m = Manager::default();
    m.response_time = 10;
    m.lock_after_rejections = 1;
    let mut c = state();
    c.requirements.movement = [1.0, 0.0];
    let mut o = zero();
    let mut rng = ThreadRandom::seeded(4, 7);
    let item = stack("stone", 64);
    set(&mut c, Slot::Player(9), item.clone());
    m.submit(Request::new(1, 1, 0, vec![throw(9)], &c, &m));
    m.begin_tick(&c);
    drive(&mut m, &mut c, &mut o, &mut rng);
    m.resync_slot(0, 9, &item, 1);
    m.poll_timers(11);
    assert_eq!(m.locked_count(), 0);
    c.now = 25;
    set(&mut c, Slot::Player(9), item);
    m.submit(Request::new(2, 2, 0, vec![throw(9)], &c, &m));
    m.begin_tick(&c);
    drive(&mut m, &mut c, &mut o, &mut rng);
    m.resync_slot(0, 9, &Stack::default(), 26);
    m.poll_timers(100);
    assert!(!m.revert_pending(100));
    assert_eq!(m.locked_count(), 0);
}
#[test]
fn raw_invalid_range_cancels_current_chain_and_all_failed_actions_still_complete() {
    let mut m = Manager::default();
    let mut c = state();
    let mut o = zero();
    o.constraints.start_delay = [1, 0];
    let mut rng = ThreadRandom::seeded(4, 7);
    m.opened();
    m.submit(Request::new(1, 1, 0, vec![create(1, 9)], &c, &m));
    m.begin_tick(&c);
    assert!(m.step(&c, &mut |_| o.clone(), &mut rng).is_err());
    assert!(!m.busy());
    o = zero();
    drive(&mut m, &mut c, &mut o, &mut rng);
    m.submit(Request::new(2, 1, 0, vec![create(1, 9)], &c, &m));
    m.begin_tick(&c);
    let emission = m.step(&c, &mut |_| o.clone(), &mut rng).unwrap().unwrap();
    m.acknowledge(emission.ticket, false, &c);
    let effects = drive(&mut m, &mut c, &mut o, &mut rng);
    assert!(effects.contains(&Effect::Outcome(Outcome::Completed { request: 2 })));
}
