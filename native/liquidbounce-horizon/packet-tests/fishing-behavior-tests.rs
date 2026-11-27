use liquidbounce_behaviors::auto_fish::{AutoFish, DelayRange, FishHand};
const MAIN: Option<FishHand> = Some(FishHand::Main);
const OFF: Option<FishHand> = Some(FishHand::Off);
const SPLASH: Option<&str> = Some("minecraft:entity.fishing_bobber.splash");

#[test]
fn exclusive_dynamic_guards_and_exact_tick_boundaries() {
    let mut fish = AutoFish::default();
    let mut random = |first, _| first;
    fish.settings.reel_delay = DelayRange::new(2, 2);
    fish.settings.recast_delay = DelayRange::new(0, 0);
    fish.start_tick_handler(true, MAIN);
    fish.start_tick_handler(true, OFF); // Busy: DROP.
    assert!(fish.sound_packet(true, SPLASH, Some([0.0; 3]), [0.0; 3]));
    assert_eq!(fish.advance_wait(true, OFF, true, &mut random), Ok(None));
    assert!(!fish.caught_fish());
    // The already captured two ticks do not become zero after settings change.
    fish.settings.reel_delay = DelayRange::new(0, 0);
    assert_eq!(fish.advance_wait(true, MAIN, true, &mut random), Ok(None));
    assert_eq!(fish.advance_wait(true, OFF, true, &mut random), Ok(OFF));
    // No suspension means the last required hand remains valid even if the
    // real first use broke its rod. The runtime executes both actual uses.
    assert_eq!(fish.after_use(&mut random), Ok(OFF));
    assert_eq!(fish.after_use(&mut random), Ok(None));
    fish.start_tick_handler(true, MAIN);
    fish.settings.auto_cast_rod = true;
    fish.settings.auto_cast_delay = DelayRange::new(0, 0);
    assert_eq!(fish.advance_wait(true, MAIN, false, &mut random), Ok(MAIN));
    assert!(!fish.busy());
    // Existing runs finish before this tick's new tickHandler is dispatched.
    fish.start_tick_handler(true, MAIN);
    assert_eq!(fish.advance_wait(true, OFF, false, &mut random), Ok(OFF));
    // Losing all rods breaks the run; caughtFish is only reset by the module
    // when consumed or onDisabled, never by a guard interruption.
    fish.start_tick_handler(true, MAIN);
    fish.sound_packet(true, SPLASH, Some([0.0; 3]), [0.0; 3]);
    assert_eq!(fish.advance_wait(true, None, true, &mut random), Ok(None));
    assert!(!fish.busy());
    assert!(fish.caught_fish());
    fish.start_tick_handler(true, OFF);
    assert_eq!(fish.advance_wait(false, OFF, true, &mut random), Ok(None));
    assert!(fish.caught_fish());
    fish.on_disabled();
    assert!(!fish.caught_fish());
    assert!(!fish.busy());
}

#[test]
fn raw_ranges_capture_when_sampled_and_recast_reads_after_actual_use() {
    let mut fish = AutoFish::default();
    let mut draws = vec![];
    let mut random = |first, last| { draws.push((first, last)); first };
    fish.settings.auto_cast_rod = true;
    fish.settings.auto_cast_delay = DelayRange::new(-7, -3);
    fish.start_tick_handler(true, MAIN);
    assert_eq!(fish.advance_wait(true, MAIN, false, &mut random), Ok(None));
    assert_eq!(fish.advance_wait(true, OFF, true, &mut random), Ok(OFF), "negative waitTicks still suspends one tick; hook is not rechecked");
    fish.settings.reel_delay = DelayRange::new(0, 0);
    fish.start_tick_handler(true, MAIN);
    fish.sound_packet(true, SPLASH, Some([0.0; 3]), [0.0; 3]);
    assert_eq!(fish.advance_wait(true, MAIN, true, &mut random), Ok(MAIN));
    fish.settings.recast_rod = false; // Callback from actual use.
    assert_eq!(fish.after_use(&mut random), Ok(None));
    fish.settings.recast_rod = true;
    fish.settings.recast_delay = DelayRange::new(i32::MAX, i32::MAX);
    fish.start_tick_handler(true, MAIN);
    fish.sound_packet(true, SPLASH, Some([0.0; 3]), [0.0; 3]);
    assert_eq!(fish.advance_wait(true, MAIN, true, &mut random), Ok(MAIN));
    assert_eq!(fish.after_use(&mut random), Ok(None));
    assert_eq!(fish.advance_wait(true, OFF, false, &mut random), Ok(None));
    fish.on_disabled();
    fish.settings.reel_delay = DelayRange::new(9, -1);
    fish.start_tick_handler(true, MAIN);
    fish.sound_packet(true, SPLASH, Some([0.0; 3]), [0.0; 3]);
    assert!(fish.advance_wait(true, MAIN, true, &mut random).is_err());
    assert!(!fish.busy());
    assert!(!fish.caught_fish());
    assert_eq!(draws, [(-7, -3), (0, 0), (0, 0), (i32::MAX, i32::MAX)]);
}

#[test]
fn sound_registry_keys_float_square_and_imported_values() {
    let mut fish = AutoFish::default();
    assert!(!fish.sound_packet(false, SPLASH, Some([0.0; 3]), [0.0; 3]));
    assert!(!fish.sound_packet(true, SPLASH, None, [0.0; 3]));
    assert!(!fish.sound_packet(true, None, Some([0.0; 3]), [0.0; 3]));
    assert!(!fish.sound_packet(true, Some("minecraft:entity.pig.ambient"), Some([0.0; 3]), [0.0; 3]));
    assert!(fish.sound_packet(true, SPLASH, Some([0.0; 3]), [1.0, 0.0, 0.0]), "threshold equality is accepted");
    assert!(!fish.sound_packet(true, SPLASH, Some([0.0; 3]), [1.00001, 0.0, 0.0]));
    fish.on_disabled();
    fish.settings.max_distance = -2.0;
    assert!(fish.sound_packet(true, SPLASH, Some([0.0; 3]), [2.0, 0.0, 0.0]));
    fish.settings.max_distance = f32::NAN;
    assert!(fish.sound_packet(true, SPLASH, Some([0.0; 3]), [f64::MAX, 0.0, 0.0]));
    fish.settings.max_distance = 0.0;
    fish.settings.sound_distance = false;
    assert!(fish.sound_packet(true, SPLASH, Some([0.0; 3]), [f64::MAX, 0.0, 0.0]));
    fish.settings.sounds.clear();
    assert!(!fish.sound_packet(true, SPLASH, Some([0.0; 3]), [0.0; 3]));
    fish.settings.sounds.insert(None); // Unknown imported registry value.
    assert!(fish.sound_packet(true, None, Some([0.0; 3]), [0.0; 3]));
}
