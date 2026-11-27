// SPDX-License-Identifier: GPL-3.0-or-later
//! Cross-module tests use production state machines, not substitute callbacks.
use crate::{
    anti_afk::{AntiAfk, Effect, Mode, Snapshot},
    auto_fish::{AutoFish, DelayRange, FishHand},
    coroutine_ticker::{CoroutineTicker, WaitOwner},
    derp::{Derp, Snapshot as DerpSnapshot, YawMode},
    hand_derp::{HandDerp, MainHand},
    random::ThreadRandom,
    rotations::Rotation,
};

fn register_derp(ticker: &CoroutineTicker, derp: &mut Derp) {
    for callback in derp.take_wait_registrations() {
        ticker.register(WaitOwner::Derp, callback);
    }
}
fn register_anti(ticker: &CoroutineTicker, anti: &mut AntiAfk) {
    for callback in anti.take_wait_registrations() {
        ticker.register(WaitOwner::AntiAfk, callback);
    }
}
fn register_fish(ticker: &CoroutineTicker, fish: &mut AutoFish) {
    for callback in fish.take_wait_registrations() {
        ticker.register(WaitOwner::AutoFish, callback);
    }
}
fn derp_at(yaw: f32) -> DerpSnapshot {
    DerpSnapshot {
        yaw,
        ..Default::default()
    }
}
fn anti_at(yaw: f32) -> Snapshot {
    Snapshot {
        yaw,
        ..Default::default()
    }
}

#[test]
fn real_old_anti_afk_callback_precedes_recent_derp_and_changes_its_live_yaw() {
    let ticker = CoroutineTicker::default();
    let mut derp = Derp::default();
    derp.set_yaw_mode(YawMode::Jitter);
    derp.settings.forward_ticks = 100;
    let mut anti = AntiAfk::default();
    anti.settings.mode = Mode::Old;
    let mut rng = ThreadRandom::seeded(123, 1);
    let mut yaw = 10.0;
    // Original GameTick mode registration order: Derp before AntiAFK.
    derp.start_yaw_handler(true, derp_at(yaw));
    register_derp(&ticker, &mut derp);
    assert_eq!(
        anti.start_main_handler(true, anti_at(yaw), Rotation::ZERO, &mut rng),
        Ok(None)
    );
    register_anti(&ticker, &mut anti);
    let mut trace = Vec::new();
    for tick in 1..=10 {
        let mut order = Vec::new();
        let mut batch = ticker.begin_tick().unwrap();
        while let Some(task) = batch.next_task() {
            order.push(task.owner);
            let keep = match task.owner {
                WaitOwner::Derp => {
                    let keep = derp.advance_callback(task.callback, true, derp_at(yaw));
                    register_derp(&ticker, &mut derp);
                    keep
                }
                WaitOwner::AntiAfk => {
                    let (keep, effect) = anti
                        .advance_callback(
                            task.callback,
                            true,
                            anti_at(yaw),
                            Rotation::ZERO,
                            &mut rng,
                        )
                        .unwrap();
                    register_anti(&ticker, &mut anti);
                    if let Some(effect) = effect {
                        let Effect::Yaw(value) = effect else {
                            panic!("Old mode only changes yaw")
                        };
                        yaw = value;
                        assert_eq!(
                            anti.after_effect(effect, anti_at(yaw), Rotation::ZERO, &mut rng),
                            Ok(None)
                        );
                        register_anti(&ticker, &mut anti);
                    }
                    keep
                }
                _ => panic!("unexpected owner"),
            };
            batch.finish_current(keep);
        }
        drop(batch);
        trace.push((tick, yaw, derp.jitter_yaw(), order));
        derp.start_yaw_handler(true, derp_at(yaw));
        register_derp(&ticker, &mut derp);
        anti.start_main_handler(true, anti_at(yaw), Rotation::ZERO, &mut rng)
            .unwrap();
        register_anti(&ticker, &mut anti);
    }
    assert_eq!(trace[0].3, [WaitOwner::Derp, WaitOwner::AntiAfk]);
    assert_eq!(trace[9].3, [WaitOwner::AntiAfk, WaitOwner::Derp]);
    assert_eq!((trace[9].1, trace[9].2), (190.0, 190.0));
    assert!(trace[..9].iter().all(|row| row.1 == 10.0 && row.2 == 10.0));
    if let Ok(path) = std::env::var("FIFO_RUST_TRACE") {
        let rows: Vec<_> = trace
            .iter()
            .map(|row| serde_json::json!([row.0, row.1, row.2]))
            .collect();
        std::fs::write(path, serde_json::to_vec(&rows).unwrap()).unwrap();
    }
}

#[test]
fn ordinary_swing_callbacks_change_later_live_snapshots_in_registration_order() {
    for anti_first in [false, true] {
        let ticker = CoroutineTicker::default();
        let mut derp = Derp::default();
        derp.set_yaw_mode(YawMode::Jitter);
        derp.settings.forward_ticks = 100;
        let mut anti = AntiAfk::default();
        anti.settings.mode = Mode::Custom;
        anti.settings.swing_delay = 1;
        if anti_first {
            anti.start_swing_handler(true, anti_at(10.0));
            register_anti(&ticker, &mut anti);
        }
        derp.start_yaw_handler(true, derp_at(10.0));
        register_derp(&ticker, &mut derp);
        if !anti_first {
            anti.start_swing_handler(true, anti_at(10.0));
            register_anti(&ticker, &mut anti);
        }
        let mut yaw = 10.0;
        let mut rng = ThreadRandom::seeded(123, 1);
        let mut batch = ticker.begin_tick().unwrap();
        while let Some(task) = batch.next_task() {
            let keep = match task.owner {
                WaitOwner::AntiAfk => {
                    let (keep, effect) = anti
                        .advance_callback(
                            task.callback,
                            true,
                            anti_at(yaw),
                            Rotation::ZERO,
                            &mut rng,
                        )
                        .unwrap();
                    register_anti(&ticker, &mut anti);
                    assert_eq!(effect, Some(Effect::SwingHand));
                    // A synchronous ordinary Punch listener edits player yaw.
                    yaw = 90.0;
                    assert_eq!(
                        anti.after_effect(effect.unwrap(), anti_at(yaw), Rotation::ZERO, &mut rng),
                        Ok(None)
                    );
                    register_anti(&ticker, &mut anti);
                    keep
                }
                WaitOwner::Derp => {
                    let keep = derp.advance_callback(task.callback, true, derp_at(yaw));
                    register_derp(&ticker, &mut derp);
                    keep
                }
                _ => unreachable!(),
            };
            batch.finish_current(keep);
        }
        assert_eq!(derp.jitter_yaw(), if anti_first { 90.0 } else { 10.0 });
    }
}

#[test]
fn fish_new_reel_and_recast_waiters_never_run_in_their_registration_tick() {
    let ticker = CoroutineTicker::default();
    let mut fish = AutoFish::default();
    fish.settings.reel_delay = DelayRange::new(1, 1);
    fish.settings.recast_delay = DelayRange::new(1, 1);
    fish.start_tick_handler(true, Some(FishHand::Main));
    register_fish(&ticker, &mut fish);
    assert!(fish.sound_packet(
        true,
        Some("minecraft:entity.fishing_bobber.splash"),
        Some([0.0; 3]),
        [0.0; 3]
    ));
    let mut uses = Vec::new();
    let mut sampled = Vec::new();
    for tick in 1..=3 {
        let mut batch = ticker.begin_tick().unwrap();
        let mut count = 0;
        while let Some(task) = batch.next_task() {
            count += 1;
            let hand = if tick == 3 {
                FishHand::Off
            } else {
                FishHand::Main
            };
            let (keep, effect) = fish
                .advance_callback(task.callback, true, Some(hand), true, &mut |lo, hi| {
                    sampled.push((tick, lo, hi));
                    lo
                })
                .unwrap();
            register_fish(&ticker, &mut fish);
            if let Some(hand) = effect {
                uses.push((tick, hand));
                assert_eq!(
                    fish.after_use(&mut |lo, hi| {
                        sampled.push((tick, lo, hi));
                        lo
                    }),
                    Ok(None)
                );
                register_fish(&ticker, &mut fish);
            }
            batch.finish_current(keep);
        }
        assert_eq!(count, 1);
    }
    assert_eq!(sampled, [(1, 1, 1), (2, 1, 1)]);
    assert_eq!(uses, [(2, FishHand::Main), (3, FishHand::Off)]);
    assert!(!fish.busy());
}

#[test]
fn earlier_swing_can_break_a_later_fish_live_guard() {
    let ticker = CoroutineTicker::default();
    let mut anti = AntiAfk::default();
    let mut fish = AutoFish::default();
    anti.settings.mode = Mode::Custom;
    anti.settings.swing_delay = 1;
    anti.start_swing_handler(true, anti_at(0.0));
    register_anti(&ticker, &mut anti);
    fish.start_tick_handler(true, Some(FishHand::Main));
    register_fish(&ticker, &mut fish);
    let mut rod = Some(FishHand::Main);
    let mut rng = ThreadRandom::seeded(123, 1);
    let mut batch = ticker.begin_tick().unwrap();
    while let Some(task) = batch.next_task() {
        let keep = match task.owner {
            WaitOwner::AntiAfk => {
                let (keep, effect) = anti
                    .advance_callback(task.callback, true, anti_at(0.0), Rotation::ZERO, &mut rng)
                    .unwrap();
                register_anti(&ticker, &mut anti);
                assert_eq!(effect, Some(Effect::SwingHand));
                rod = None;
                keep
            }
            WaitOwner::AutoFish => {
                let (keep, effect) = fish
                    .advance_callback(task.callback, true, rod, false, &mut |_, _| {
                        panic!("broken guard must precede predicate/delay")
                    })
                    .unwrap();
                register_fish(&ticker, &mut fish);
                assert_eq!(effect, None);
                keep
            }
            _ => unreachable!(),
        };
        batch.finish_current(keep);
    }
    assert!(!fish.busy());
    assert!(fish.take_wait_registrations().is_empty());
}

#[test]
fn cancelled_local_tasks_cannot_advance_or_cancel_their_replacements() {
    let mut derp = Derp::default();
    derp.set_yaw_mode(YawMode::Jitter);
    derp.start_yaw_handler(true, derp_at(1.0));
    let old = derp.take_wait_registrations()[0];
    derp.cancel_sequences();
    derp.start_yaw_handler(true, derp_at(2.0));
    let fresh = derp.take_wait_registrations()[0];
    assert_ne!(old, fresh);
    assert!(!derp.advance_callback(old, false, derp_at(3.0)));
    assert!(derp.busy());
    assert!(!derp.advance_callback(fresh, true, derp_at(4.0)));
    assert_eq!(derp.jitter_yaw(), 4.0);
    let mut fish = AutoFish::default();
    fish.start_tick_handler(true, Some(FishHand::Main));
    let old = fish.take_wait_registrations()[0];
    fish.cancel_sequences();
    fish.start_tick_handler(true, Some(FishHand::Main));
    let fresh = fish.take_wait_registrations()[0];
    assert_ne!(old, fresh);
    assert_eq!(
        fish.advance_callback(old, false, None, false, &mut |_, _| unreachable!()),
        Ok((false, None))
    );
    assert_eq!(
        fish.advance_callback(
            fresh,
            true,
            Some(FishHand::Main),
            true,
            &mut |_, _| unreachable!()
        ),
        Ok((true, None))
    );
    let mut hand = HandDerp::default();
    hand.initialize(MainHand::Right);
    hand.settings.delay = 2;
    hand.start_delay(true, || 10);
    hand.cancel_sequences();
    hand.start_delay(true, || 11);
    assert_eq!(hand.advance_callback(10, false), (false, None));
    assert_eq!(hand.advance_callback(11, true), (true, None));
    assert_eq!(
        hand.advance_callback(11, true),
        (false, Some(MainHand::Left))
    );
    let mut anti = AntiAfk::default();
    anti.settings.mode = Mode::Old;
    let mut rng = ThreadRandom::seeded(123, 1);
    anti.start_main_handler(true, anti_at(0.0), Rotation::ZERO, &mut rng)
        .unwrap();
    let old = anti.take_wait_registrations()[0];
    anti.cancel_sequences();
    anti.start_main_handler(true, anti_at(0.0), Rotation::ZERO, &mut rng)
        .unwrap();
    let fresh = anti.take_wait_registrations()[0];
    assert_ne!(old, fresh);
    assert_eq!(
        anti.advance_callback(old, false, anti_at(0.0), Rotation::ZERO, &mut rng),
        Ok((false, None))
    );
    assert_eq!(
        anti.advance_callback(fresh, true, anti_at(0.0), Rotation::ZERO, &mut rng),
        Ok((true, None))
    );
}

#[test]
fn random_direction_outer_wait_is_a_fresh_callback_even_with_same_lane() {
    let ticker = CoroutineTicker::default();
    let mut anti = AntiAfk::default();
    let mut rng = ThreadRandom::seeded(123, 1);
    anti.settings.interactions = 1 << 5;
    anti.settings.delay = [1, 1];
    anti.start_main_handler(true, anti_at(0.0), Rotation::ZERO, &mut rng)
        .unwrap();
    register_anti(&ticker, &mut anti);
    let mut ids = Vec::new();
    for _ in 0..2 {
        let mut batch = ticker.begin_tick().unwrap();
        let task = batch.next_task().unwrap();
        ids.push(task.callback);
        let (keep, effect) = anti
            .advance_callback(task.callback, true, anti_at(0.0), Rotation::ZERO, &mut rng)
            .unwrap();
        register_anti(&ticker, &mut anti);
        assert!(!keep);
        assert_eq!(effect, None);
        batch.finish_current(keep);
        assert!(batch.next_task().is_none());
    }
    assert_ne!(ids[0], ids[1]);
    assert!(!anti.busy());
}

#[test]
fn native_depth_scope_skips_nested_tick_after_outer_batch_has_finished() {
    use crate::native_scope::{CoroutineTickScope, Game};
    let game = Game;
    assert!(!game.native_client_coroutine_outermost());
    let ticker = CoroutineTicker::default();
    let outer = CoroutineTickScope::enter();
    assert!(game.native_client_coroutine_outermost());
    {
        let mut batch = ticker.begin_tick().unwrap();
        assert!(batch.next_task().is_none());
    }
    // This registration happens in a later ordinary outer tick handler.
    let ticket = ticker.register(WaitOwner::HandDerp, 0);
    {
        let _nested = CoroutineTickScope::enter();
        assert!(!game.native_client_coroutine_outermost());
        // Runtime uses the actual Host helper before calling begin_tick.
        if game.native_client_coroutine_outermost() {
            panic!("nested tick must not drain pending")
        }
    }
    assert!(game.native_client_coroutine_outermost());
    drop(outer);
    assert!(!game.native_client_coroutine_outermost());
    let _next = CoroutineTickScope::enter();
    assert!(game.native_client_coroutine_outermost());
    let mut batch = ticker.begin_tick().unwrap();
    assert_eq!(batch.next_task().unwrap().ticket, ticket);
    batch.finish_current(false);
    assert!(batch.next_task().is_none());
}
