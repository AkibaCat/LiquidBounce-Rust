// SPDX-License-Identifier: GPL-3.0-or-later
// Independent expected values checked against the original source and Java 25.

use liquidbounce_behaviors::{
    Chronometer, CustomFov, Easing, FullBright, FullBrightMode, MillisClock, NightVisionIntent,
    NoFov, NoFovMode, Zoom, ZoomContext, ZoomSettings,
};
use std::collections::VecDeque;

struct SequenceClock {
    samples: VecDeque<i64>,
    reads: usize,
}

impl SequenceClock {
    fn new(samples: impl IntoIterator<Item = i64>) -> Self {
        Self {
            samples: samples.into_iter().collect(),
            reads: 0,
        }
    }
}

impl MillisClock for SequenceClock {
    fn now_ms(&mut self) -> i64 {
        self.reads += 1;
        self.samples.pop_front().expect("unexpected clock read")
    }
}

fn at(time: i64) -> impl FnMut() -> i64 {
    move || time
}

#[test]
fn all_easings_match_independent_java_float_vectors() {
    // Values produced by Java 25 expressions with the original operation widths,
    // not by calculating expected results through this implementation.
    let at_quarter = [
        0x3e800000, 0x3d800000, 0x3ee00000, 0x3e200000, 0x3bb504f3, 0x3f52bec3, 0x3f800000,
    ];
    let at_three_quarters = [
        0x3f400000, 0x3f100000, 0x3f700000, 0x3f580000, 0x3e3504f3, 0x3f7e95f6, 0x3f800000,
    ];
    for (index, easing) in Easing::ALL.into_iter().enumerate() {
        assert_eq!(easing.transform(0.25).to_bits(), at_quarter[index]);
        assert_eq!(easing.transform(0.75).to_bits(), at_three_quarters[index]);
        assert_eq!(easing.transform(1.0), 1.0);
        assert_eq!(
            easing.transform(0.0),
            if easing == Easing::None { 1.0 } else { 0.0 }
        );
    }
    assert_eq!(Easing::QuadIn.transform(0.1).to_bits(), 0x3c23d70b);
    assert_eq!(Easing::QuadOut.transform(0.1).to_bits(), 0x3e428f60);
    assert_eq!(Easing::QuadInOut.transform(0.1).to_bits(), 0x3ce56042);
}

#[test]
fn easing_factor_preserves_nan_and_none_short_circuit() {
    assert_eq!(Easing::Linear.get_factor(100, 90, 10.0), 0.0);
    assert_eq!(Easing::Linear.get_factor(0, 20, 10.0), 1.0);
    assert!(Easing::Linear.get_factor(0, 0, 0.0).is_nan());
    assert!(Easing::QuadIn.get_factor(0, 10, f32::NAN).is_nan());
    assert_eq!(Easing::None.get_factor(100, 0, f32::NAN), 1.0);
    assert_eq!(Easing::None.transform(f32::NAN), 1.0);
    assert_eq!(Easing::Linear.get_factor(i64::MAX, i64::MIN, 2.0), 0.5);
}

#[test]
fn chronometer_keeps_signed_time_strict_boundary_and_jvm_overflow() {
    let mut timer = Chronometer::new(100);
    assert_eq!(timer.elapsed_until(90), -10);
    assert!(!timer.has_elapsed(20, &mut at(120)));
    assert!(timer.has_at_least_elapsed(20, &mut at(120)));
    timer.wait_for_at_least(20, &mut at(50));
    assert_eq!(timer.last_update(), 100);
    timer.wait_for_at_least(30, &mut at(90));
    assert_eq!(timer.last_update(), 120);
    timer.reset_at(i64::MAX);
    assert_eq!(timer.elapsed_until(i64::MIN), 1);
    assert!(timer.has_at_least_elapsed(1, &mut at(i64::MIN)));
}

#[test]
fn zoom_initial_guard_and_source_lifecycle_use_exact_clock_read_order() {
    let mut zoom = Zoom::default();
    assert!(zoom.snapshot().disable_animation_finished);
    let mut no_reads = SequenceClock::new([]);
    assert_eq!(zoom.get_fov(false, 70, &mut no_reads), 70);
    assert_eq!(zoom.progress(&mut no_reads), 1.0);
    assert_eq!(no_reads.reads, 0);

    let mut enable_clock = SequenceClock::new([100]);
    zoom.on_enabled(70, &mut enable_clock);
    let enabled = zoom.snapshot();
    assert_eq!((enabled.previous_fov, enabled.target_fov), (70, 30));
    assert_eq!(enabled.scaled_difference, 80.0);
    assert_eq!(enabled.last_update_ms, 100);
    // onEnabled does not change this flag or own external enabled state.
    assert!(enabled.disable_animation_finished);

    let mut disable_clock = SequenceClock::new([120, 121, 122]);
    zoom.on_disabled(70, &mut disable_clock);
    let disabled = zoom.snapshot();
    assert_eq!((disabled.previous_fov, disabled.target_fov), (67, 70));
    assert_eq!(disabled.scaled_difference, 6.0);
    assert_eq!(disabled.last_update_ms, 122);
    assert!(!disabled.disable_animation_finished);
    assert_eq!(disable_clock.reads, 3);
}

#[test]
fn zoom_floor_displacement_and_fraction_match_java_projection_value() {
    let mut zoom = Zoom::default();
    zoom.on_enabled(70, &mut at(100));
    assert_eq!(zoom.progress(&mut at(120)), 0.25);
    // Displacement is -2.5; Mth floors it to -3 rather than truncating to -2.
    assert_eq!(zoom.get_fov(true, 0, &mut at(120)), 67);
    assert_eq!(
        zoom.apply_fractional_fov(
            75.0,
            ZoomContext {
                in_game: true,
                running: true
            },
            &mut at(120),
        )
        .to_bits(),
        0x42971e92,
    );
    assert_eq!(
        zoom.apply_fractional_fov(
            75.0,
            ZoomContext {
                in_game: true,
                running: false
            },
            &mut SequenceClock::new([]),
        ),
        75.0,
    );
}

#[test]
fn zoom_disable_reads_default_fov_between_original_clock_resets() {
    use std::cell::RefCell;
    let mut zoom = Zoom::default();
    zoom.on_enabled(70, &mut at(0));
    let trace = RefCell::new(Vec::new());
    let mut samples = [20, 21, 22].into_iter();
    let mut clock = || {
        let time = samples.next().unwrap();
        trace.borrow_mut().push(time);
        time
    };
    zoom.on_disabled_with_default_fov(
        || {
            trace.borrow_mut().push(999);
            90
        },
        &mut clock,
    );
    assert_eq!(*trace.borrow(), [20, 21, 999, 22]);
    assert_eq!(zoom.snapshot().target_fov, 90);
}

#[test]
fn zoom_disable_finishes_only_when_integer_getter_observes_completion() {
    let mut zoom = Zoom::default();
    zoom.on_enabled(70, &mut at(0));
    zoom.on_disabled(70, &mut at(20));
    assert_eq!(zoom.snapshot().scaled_difference, 6.0);
    let context = ZoomContext {
        in_game: true,
        running: false,
    };
    assert_eq!(zoom.apply_fractional_fov(80.0, context, &mut at(26)), 80.0);
    assert!(!zoom.snapshot().disable_animation_finished);
    // This call returns the stored animation target, even if original changed.
    assert_eq!(zoom.get_fov(false, 95, &mut at(26)), 70);
    assert!(zoom.snapshot().disable_animation_finished);
    assert_eq!(zoom.get_fov(false, 95, &mut SequenceClock::new([])), 95);
}

#[test]
fn zoom_reenable_continues_from_integer_disable_animation_anchor() {
    let mut zoom = Zoom::default();
    zoom.on_enabled(70, &mut at(0));
    zoom.on_disabled(70, &mut at(20));
    // At 3/6ms QuadIn, displayed whole=67 and float=67.75.
    zoom.on_enabled(90, &mut at(23));
    let snapshot = zoom.snapshot();
    assert_eq!((snapshot.previous_fov, snapshot.target_fov), (67, 30));
    assert_eq!(snapshot.scaled_difference, 74.0);
    assert!(!snapshot.disable_animation_finished);
    assert_eq!(zoom.get_fov(true, 0, &mut at(23)), 67);
}

#[test]
fn zero_rounded_scroll_still_reanchors_resets_and_cancels() {
    let mut zoom = Zoom::new(ZoomSettings {
        scroll_speed: 0.5,
        ..Default::default()
    });
    zoom.on_enabled(70, &mut at(0));
    let outcome = zoom.on_hotbar_scroll(1, true, &mut at(20));
    assert!(outcome.cancel_hotbar_scroll);
    let snapshot = zoom.snapshot();
    assert_eq!((snapshot.previous_fov, snapshot.target_fov), (67, 30));
    assert_eq!(snapshot.scaled_difference, 74.0);
    assert_eq!(zoom.get_fov(true, 0, &mut at(20)), 67);
    assert_eq!(
        zoom.apply_fractional_fov(
            67.0,
            ZoomContext {
                in_game: true,
                running: true
            },
            &mut at(20)
        ),
        67.0
    );
}

#[test]
fn scroll_ties_to_even_clamps_targets_and_respects_running_child_gate() {
    let mut zoom = Zoom::new(ZoomSettings {
        scroll_speed: 1.5,
        duration_factor: 0.0,
        ..Default::default()
    });
    zoom.on_enabled(70, &mut at(0));
    zoom.on_hotbar_scroll(1, true, &mut at(1));
    assert_eq!(zoom.snapshot().target_fov, 28);
    zoom.on_hotbar_scroll(-1, true, &mut at(2));
    assert_eq!(zoom.snapshot().target_fov, 30);
    zoom.settings.scroll_speed = 2.5;
    zoom.on_hotbar_scroll(1, true, &mut at(3));
    assert_eq!(zoom.snapshot().target_fov, 28);
    zoom.on_hotbar_scroll(100, true, &mut at(4));
    assert_eq!(zoom.snapshot().target_fov, 1);
    zoom.on_hotbar_scroll(-100, true, &mut at(5));
    assert_eq!(zoom.snapshot().target_fov, 179);
    let unchanged = zoom.snapshot();
    assert!(
        !zoom
            .on_hotbar_scroll(1, false, &mut SequenceClock::new([]))
            .cancel_hotbar_scroll
    );
    zoom.settings.scroll_enabled = false;
    assert!(
        !zoom
            .on_hotbar_scroll(1, true, &mut SequenceClock::new([]))
            .cancel_hotbar_scroll
    );
    assert_eq!(zoom.snapshot(), unchanged);
}

#[test]
fn duration_is_sampled_at_reset_but_transition_updates_current_animation() {
    let mut zoom = Zoom::default();
    zoom.on_enabled(70, &mut at(0));
    zoom.settings.duration_factor = 0.0;
    assert_eq!(zoom.progress(&mut at(20)), 0.25);
    assert_eq!(zoom.get_fov(true, 0, &mut at(20)), 67);
    zoom.settings.transition = Easing::None;
    assert_eq!(zoom.get_fov(true, 0, &mut at(20)), 30);
    zoom.on_hotbar_scroll(1, true, &mut at(20));
    assert_eq!(zoom.progress(&mut SequenceClock::new([])), 1.0);
}

#[test]
fn invalid_duration_zero_difference_and_backwards_clock_follow_source_guards() {
    for factor in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let mut zoom = Zoom::new(ZoomSettings {
            duration_factor: factor,
            ..Default::default()
        });
        zoom.on_enabled(70, &mut at(100));
        assert_eq!(zoom.progress(&mut SequenceClock::new([])), 1.0);
        assert_eq!(zoom.get_fov(true, 0, &mut SequenceClock::new([])), 30);
    }
    let mut zoom = Zoom::default();
    zoom.on_enabled(30, &mut at(100));
    assert_eq!(zoom.progress(&mut SequenceClock::new([])), 1.0);
    zoom.on_enabled(70, &mut at(100));
    assert_eq!(zoom.progress(&mut at(50)), 0.0);
    assert_eq!(zoom.get_fov(true, 0, &mut at(50)), 70);
}

#[test]
fn fractional_fov_guards_absent_world_and_zero_integer_base() {
    let mut zoom = Zoom::new(ZoomSettings {
        zoom: 0,
        duration_factor: 0.0,
        ..Default::default()
    });
    zoom.on_enabled(70, &mut at(0));
    assert_eq!(
        zoom.apply_fractional_fov(
            60.0,
            ZoomContext {
                in_game: false,
                running: true
            },
            &mut SequenceClock::new([])
        ),
        60.0
    );
    assert_eq!(
        zoom.apply_fractional_fov(
            60.0,
            ZoomContext {
                in_game: true,
                running: true
            },
            &mut SequenceClock::new([])
        ),
        60.0
    );
}

#[test]
fn gamma_captures_real_option_accumulates_float_ticks_and_never_decreases() {
    let mut bright = FullBright::new();
    assert_eq!(bright.gamma.gamma(), 0.0);
    bright.on_enabled(0.0);
    for _ in 0..10 {
        assert_eq!(bright.on_player_post_tick(true), None);
    }
    assert_eq!(bright.gamma.gamma().to_bits(), 0x3f800001);
    bright.gamma.brightness = 1;
    bright.on_player_post_tick(true);
    assert_eq!(bright.gamma.gamma().to_bits(), 0x3f800001);
    assert_eq!(
        bright.brightness_override(true),
        Some(f32::from_bits(0x3f800001))
    );
    assert_eq!(bright.brightness_override(false), None);
    assert_eq!(bright.on_disabled(), None);
    bright.on_enabled(0.95);
    bright.on_player_post_tick(true);
    assert_eq!(bright.gamma.gamma(), 1.0);
    bright.on_player_post_tick(false);
    assert_eq!(bright.gamma.gamma(), 1.0);
}

#[test]
fn night_vision_reapplies_exact_effect_and_removes_on_disable_and_mode_change() {
    let mut bright = FullBright::new();
    let mut effects = Vec::new();
    bright.switch_mode(FullBrightMode::NightVision, true, 0.5, |intent| {
        effects.push(intent)
    });
    assert!(effects.is_empty());
    let expected = Some(NightVisionIntent::Apply {
        duration_ticks: 1337,
        amplifier: 0,
        ambient: false,
        visible: true,
        show_icon: true,
    });
    assert_eq!(bright.on_player_post_tick(true), expected);
    assert_eq!(bright.on_player_post_tick(true), expected);
    assert_eq!(bright.on_player_post_tick(false), None);
    assert_eq!(bright.brightness_override(true), None);
    assert_eq!(bright.on_disabled(), Some(NightVisionIntent::Remove));
    bright.switch_mode(FullBrightMode::Gamma, true, 0.75, |intent| {
        effects.push(intent)
    });
    assert_eq!(effects, [NightVisionIntent::Remove]);
    assert_eq!(bright.gamma.gamma(), 0.75);
    bright.switch_mode(FullBrightMode::Gamma, true, 0.25, |_| {
        panic!("same mode must do nothing")
    });
    assert_eq!(bright.gamma.gamma(), 0.75);
}

#[test]
fn inactive_fullbright_mode_switch_does_not_run_lifecycle_or_effects() {
    let mut bright = FullBright::new();
    bright.switch_mode(FullBrightMode::NightVision, false, 0.9, |_| {
        panic!("inactive")
    });
    bright.switch_mode(FullBrightMode::Gamma, false, 0.9, |_| panic!("inactive"));
    assert_eq!(bright.gamma.gamma(), 0.0);
    bright.on_enabled(f64::NAN);
    bright.on_player_post_tick(true);
    assert!(bright.gamma.gamma().is_nan());
}

#[test]
fn nofov_constant_and_custom_preserve_distinct_integer_and_multiplier_behavior() {
    let mut no_fov = NoFov::default();
    assert_eq!(no_fov.get_fov(70), 90);
    assert_eq!(no_fov.get_fov_multiplier(f32::NAN), Ok(1.0));
    no_fov.mode = NoFovMode::Custom;
    assert_eq!(no_fov.get_fov(70), 70);
    assert_eq!(no_fov.get_fov_multiplier(2.0), Ok(1.5));
    no_fov.custom.base_fov = 0.5;
    no_fov.custom.multiplier = 0.5;
    assert_eq!(no_fov.get_fov_multiplier(1.0), Ok(0.5));
    assert_eq!(no_fov.get_fov_multiplier(-2.0), Ok(0.0));
    assert_eq!(no_fov.get_fov_multiplier(4.0), Ok(1.5));
}

#[test]
fn nofov_float_range_preserves_nan_signed_zero_and_rejects_empty_ranges() {
    let mut custom = CustomFov::default();
    assert!(custom.get_fov_multiplier(f32::NAN).unwrap().is_nan());
    custom.limit = (2.0, 1.0);
    assert!(custom.get_fov_multiplier(1.0).is_err());
    custom.limit = (f32::NAN, 1.5);
    assert!(custom.get_fov_multiplier(1.0).is_err());
    custom.limit = (0.0, 1.5);
    custom.base_fov = -0.0;
    custom.multiplier = 0.0;
    assert_eq!(
        custom.get_fov_multiplier(0.0).unwrap().to_bits(),
        (-0.0_f32).to_bits()
    );
}
