// SPDX-License-Identifier: GPL-3.0-or-later
use crate::rotations::DirectionalInput;
use crate::snap_tap::{Action, BACK, FORWARD, LEFT, RIGHT, SnapTap};

fn input(bits: u8) -> DirectionalInput {
    DirectionalInput {
        left: bits & LEFT != 0,
        right: bits & RIGHT != 0,
        backwards: bits & BACK != 0,
        forwards: bits & FORWARD != 0,
    }
}

fn press(state: &mut SnapTap, matches: u8, time: i64) {
    state.keyboard(true, matches, Action::Press, || time);
}

fn movement(state: &SnapTap, bits: u8) -> DirectionalInput {
    let mut value = input(bits);
    state.movement_input(true, &mut value);
    value
}

#[test]
fn last_press_wins_in_both_orders_on_each_axis() {
    for (first, last) in [
        (LEFT, RIGHT),
        (RIGHT, LEFT),
        (BACK, FORWARD),
        (FORWARD, BACK),
    ] {
        let mut state = SnapTap::default();
        press(&mut state, first, 100);
        press(&mut state, last, 101);
        assert_eq!(movement(&state, first | last), input(last));
    }
}

#[test]
fn equal_nano_time_prefers_right_and_forward_even_when_negative_was_pressed_last() {
    let mut state = SnapTap::default();
    for key in [RIGHT, LEFT, FORWARD, BACK] {
        press(&mut state, key, 42);
    }
    assert_eq!(movement(&state, 15), input(RIGHT | FORWARD));
}

#[test]
fn timestamp_comparison_retains_jvm_signed_long_semantics() {
    let mut state = SnapTap::default();
    press(&mut state, LEFT, i64::MAX);
    press(&mut state, RIGHT, i64::MIN);
    assert_eq!(movement(&state, 15), input(LEFT | BACK | FORWARD));
}

#[test]
fn repeat_does_not_update_timestamps_or_create_a_hold() {
    let mut state = SnapTap::default();
    state.keyboard(true, LEFT, Action::Repeat, || panic!("repeat read clock"));
    press(&mut state, RIGHT, 5);
    assert_eq!(movement(&state, 0), input(0));
    press(&mut state, LEFT, 10);
    press(&mut state, RIGHT, 11);
    state.keyboard(true, LEFT, Action::Repeat, || panic!("repeat read clock"));
    assert_eq!(movement(&state, 0), input(RIGHT));
}

#[test]
fn release_clears_one_hold_without_reading_clock_or_editing_other_axis() {
    let mut state = SnapTap::default();
    for (time, key) in [LEFT, RIGHT, BACK, FORWARD].into_iter().enumerate() {
        press(&mut state, key, time as i64);
    }
    state.keyboard(true, RIGHT, Action::Release, || {
        panic!("release read clock")
    });
    assert_eq!(
        movement(&state, LEFT | RIGHT),
        input(LEFT | RIGHT | FORWARD)
    );
    state.keyboard(true, BACK, Action::Release, || panic!("release read clock"));
    assert_eq!(movement(&state, 15), input(15));
}

#[test]
fn zero_or_single_hold_preserves_all_incoming_direction_combinations() {
    for held in [0, LEFT, RIGHT, BACK, FORWARD] {
        let mut state = SnapTap::default();
        if held != 0 {
            press(&mut state, held, 1);
        }
        for bits in 0..16 {
            assert_eq!(movement(&state, bits), input(bits));
        }
    }
}

#[test]
fn internal_both_hold_guard_overrides_flags_changed_by_an_earlier_handler() {
    let mut state = SnapTap::default();
    press(&mut state, LEFT, 1);
    press(&mut state, RIGHT, 2);
    press(&mut state, FORWARD, 3);
    press(&mut state, BACK, 4);
    for bits in 0..16 {
        assert_eq!(movement(&state, bits), input(RIGHT | BACK));
    }
}

#[test]
fn disabled_keyboard_events_are_ignored_and_reenable_keeps_stale_holds() {
    let mut state = SnapTap::default();
    press(&mut state, LEFT, 1);
    press(&mut state, RIGHT, 2);
    state.keyboard(false, LEFT, Action::Release, || panic!("disabled clock"));
    state.keyboard(false, RIGHT, Action::Release, || panic!("disabled clock"));
    state.keyboard(false, BACK, Action::Press, || panic!("disabled clock"));
    let mut value = input(15);
    state.movement_input(false, &mut value);
    assert_eq!(value, input(15));
    assert_eq!(movement(&state, 0), input(RIGHT));
    state.keyboard(true, RIGHT, Action::Release, || panic!("release clock"));
    assert_eq!(movement(&state, 0), input(0));
}

#[test]
fn conflicting_bindings_use_left_then_right_then_back_then_forward() {
    let mut state = SnapTap::default();
    press(&mut state, RIGHT, 1);
    press(&mut state, LEFT | RIGHT | BACK | FORWARD, 2);
    assert_eq!(movement(&state, 0), input(LEFT));
    // Releasing a conflicted event releases only left; right remains held.
    state.keyboard(true, 15, Action::Release, || panic!("release clock"));
    press(&mut state, LEFT, 0);
    assert_eq!(movement(&state, 0), input(RIGHT));
    let mut state = SnapTap::default();
    press(&mut state, LEFT, 1);
    press(&mut state, RIGHT | BACK | FORWARD, 2);
    assert_eq!(movement(&state, 0), input(RIGHT));
    let mut state = SnapTap::default();
    press(&mut state, FORWARD, 1);
    press(&mut state, BACK | FORWARD, 2);
    assert_eq!(movement(&state, 0), input(BACK));
}

#[test]
fn unmatched_key_does_not_read_clock_and_duplicate_press_refreshes_time() {
    let mut state = SnapTap::default();
    state.keyboard(true, 0, Action::Press, || panic!("unmatched clock"));
    press(&mut state, LEFT, 10);
    press(&mut state, RIGHT, 11);
    press(&mut state, LEFT, 12);
    assert_eq!(movement(&state, 0), input(LEFT));
}

#[test]
fn nanosecond_clock_is_monotonic_with_equal_values_allowed() {
    let first = crate::clock::nano_time();
    let second = crate::clock::nano_time();
    assert!(second >= first);
}
