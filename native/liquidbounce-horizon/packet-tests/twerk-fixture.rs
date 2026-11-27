#![cfg(test)]
extern crate self as horizon_client;

#[path = "TWERK_PATH"]
mod twerk;
#[allow(dead_code)]
#[path = "NO_ENTITY_PATH"]
mod no_entity_interact;
#[allow(dead_code)]
#[path = "NO_BLOCK_PATH"]
mod no_block_interact;

use twerk::{Twerk, TwerkError};

#[test]
fn signed_int_product_and_remainder_match_the_actual_jvm() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("java-vectors.json")).unwrap();
    let vectors = vectors.as_array().unwrap();
    assert_eq!(vectors.len(), 312);
    for vector in vectors {
        let delay = i32::try_from(vector[0].as_i64().unwrap()).unwrap();
        let tick = i32::try_from(vector[1].as_i64().unwrap()).unwrap();
        for original in [false, true] {
            let mut sneak = original;
            let result = Twerk { delay }.movement_input(true, tick, &mut sneak);
            if let Some(expected) = vector[2].as_bool() {
                assert_eq!(result, Ok(()), "delay={delay}, tick={tick}");
                assert_eq!(sneak, expected, "delay={delay}, tick={tick}");
            } else {
                assert_eq!(result, Err(TwerkError::DivisionByZero));
                assert_eq!(sneak, original, "the assignment must not run on error");
            }
        }
    }
}

#[test]
fn default_cycle_live_delay_and_disabled_input_have_no_module_clock() {
    let mut module = Twerk::default();
    assert_eq!(module.delay, 2);
    for tick in 0..12 {
        for physical in [false, true] {
            let mut sneak = physical;
            module.movement_input(true, tick, &mut sneak).unwrap();
            assert_eq!(sneak, [true, true, false, false][tick as usize % 4]);
        }
    }
    module.delay = 3;
    let mut sneak = false;
    module.movement_input(true, 8, &mut sneak).unwrap();
    assert!(sneak, "the live delay applies to the actual entity age");
    module.delay = 0;
    for physical in [false, true] {
        let mut sneak = physical;
        assert_eq!(module.movement_input(false, i32::MIN, &mut sneak), Ok(()));
        assert_eq!(sneak, physical);
    }
}

// Narrow world/player storage is a fixture. The helper, native dispatch,
// generated Host implementation, input-bit assembly and damage-clock body
// inserted below are exact production source, not fixture reimplementations.
#[allow(dead_code)]
pub mod game {
    #[derive(Default)]
    struct Hurt {
        cooldown: i32,
        hurt_time: i32,
    }
    #[derive(Default)]
    struct PlayerHealth {
        game_time: i64,
        tick_count: i32,
        hurt: Hurt,
        hurt_animation_time: i32,
    }
    impl PlayerHealth {
        /* CLOCK */
    }
    #[derive(Default)]
    struct PlayerMotion {
        server_copy: bool,
        replaying: bool,
        health: PlayerHealth,
    }
    impl PlayerMotion {
        /* IDENTITY */
    }
    #[derive(Clone, Copy, Debug, Default)]
    /* KEYS */

    #[derive(Default)]
    pub struct Game<'a> {
        window: bool,
        lan_acting_uuid: Option<u128>,
        player: Option<PlayerMotion>,
        marker: std::marker::PhantomData<&'a ()>,
    }
    impl Game<'_> {
        fn native_window(&self) -> bool {
            self.window
        }
        fn input_packet_bits(&self, mut keys: Keys) -> (Keys, u8) {
            /* INPUT */
            (keys, input)
        }
    }
    mod native_hooks {
        use super::Game;
        use crate::mod_api;
        impl Game<'_> {
            /* NATIVE */
        }
    }
    mod generated_host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs"));
    }

    #[test]
    fn real_helper_generated_api_clock_phase_priority_and_packet_bit() {
        use crate::mod_api::{Host, RUNTIME};
        use crate::{Twerk, TwerkError};
        let mut game = Game {
            window: true,
            player: Some(PlayerMotion::default()),
            ..Default::default()
        };
        *RUNTIME.lock().unwrap() = crate::mod_api::Runtime {
            running: true,
            ..Default::default()
        };
        let original = Keys {
            forward: true,
            left: true,
            jump: true,
            sprint: true,
            analog: Some([0.2, 0.7]),
            ..Default::default()
        };
        for tick in 1..=10 {
            assert_eq!(Host::client_player_input_tick_count(&game), Some(tick));
            let (keys, bits) = game.input_packet_bits(original);
            let expected = tick % 4 < 2;
            assert_eq!(keys.shift, expected);
            assert_eq!(bits & 0x20 != 0, expected);
            assert_eq!(bits & !0x20, 0x55, "other input bits retain their values");
            assert_eq!(keys.analog, original.analog);
            let player = game.player.as_mut().unwrap();
            assert_eq!(player.health.tick_count, tick - 1, "input has not advanced the clock");
            player.health.advance_clock();
            assert_eq!(player.health.tick_count, tick, "actual imminent entity tick");
        }
        assert_eq!(RUNTIME.lock().unwrap().observed_ticks, (1..=10).collect::<Vec<_>>());

        // Query the player afresh after replacement and keep Java overflow.
        game.player = Some(PlayerMotion {
            health: PlayerHealth {
                game_time: i64::MAX,
                tick_count: i32::MAX,
                hurt: Hurt { cooldown: 2, hurt_time: 1 },
                hurt_animation_time: 3,
            },
            ..Default::default()
        });
        assert_eq!(Host::client_player_input_tick_count(&game), Some(i32::MIN));
        game.input_packet_bits(original);
        let player = game.player.as_mut().unwrap();
        player.health.advance_clock();
        assert_eq!(player.health.tick_count, i32::MIN);
        assert_eq!(player.health.game_time, i64::MIN);
        assert_eq!((player.health.hurt.cooldown, player.health.hurt.hurt_time, player.health.hurt_animation_time), (1, 0, 2));

        // Real native identity guards exclude other players and prediction.
        game.window = false;
        assert_eq!(Host::client_player_input_tick_count(&game), None);
        game.window = true;
        game.lan_acting_uuid = Some(9);
        assert_eq!(Host::client_player_input_tick_count(&game), None);
        game.lan_acting_uuid = None;
        game.player.as_mut().unwrap().server_copy = true;
        assert_eq!(Host::client_player_input_tick_count(&game), None);
        game.player.as_mut().unwrap().server_copy = false;
        game.player.as_mut().unwrap().replaying = true;
        assert_eq!(Host::client_player_input_tick_count(&game), None);
        game.player = None;
        assert_eq!(Host::client_player_input_tick_count(&game), None);

        // The fixture models the original handler priority: Twerk (10) then
        // production NoBlockInteract (0), including continuation after errors.
        game.player = Some(PlayerMotion::default());
        game.player.as_mut().unwrap().health.tick_count = 1;
        {
            let mut runtime = RUNTIME.lock().unwrap();
            runtime.no_block_running = true;
            runtime.no_block.start_sneaking();
        }
        let (keys, bits) = game.input_packet_bits(original);
        assert!(keys.shift && bits & 0x20 != 0, "NoBlock overrides Twerk's false");
        assert!(RUNTIME.lock().unwrap().no_block.interacting);
        RUNTIME.lock().unwrap().twerk.delay = 0;
        let (keys, _) = game.input_packet_bits(original);
        assert!(keys.shift, "a Twerk ArithmeticException does not skip NoBlock");
        assert_eq!(RUNTIME.lock().unwrap().errors, [TwerkError::DivisionByZero]);
        {
            let mut runtime = RUNTIME.lock().unwrap();
            runtime.no_block.finish_retry();
            runtime.no_block_running = false;
            runtime.running = false;
        }
        let (keys, _) = game.input_packet_bits(Keys { shift: true, ..original });
        assert!(keys.shift, "disabled module retains physical sneak even with invalid delay");
        {
            let mut runtime = RUNTIME.lock().unwrap();
            runtime.running = true;
            runtime.twerk = Twerk::default();
        }
        let (keys, _) = game.input_packet_bits(Keys { shift: true, ..original });
        assert!(!keys.shift, "resuming uses this player's existing age");
    }
}

pub mod mod_api {
    use crate::{Twerk, TwerkError, no_block_interact::NoBlockInteract};
    use std::sync::{LazyLock, Mutex};
    pub trait RenderHooks: Send + Sync {}
    #[allow(unused_imports)]
    mod generated_host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs"));
    }
    pub use generated_host::*;
    #[allow(clippy::too_many_arguments)]
    mod generated {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs"));
    }
    pub use generated::*;
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(Box::as_ref)
    }
    #[derive(Default)]
    pub struct Runtime {
        pub twerk: Twerk,
        pub running: bool,
        pub no_block: NoBlockInteract,
        pub no_block_running: bool,
        pub errors: Vec<TwerkError>,
        pub observed_ticks: Vec<i32>,
    }
    pub static RUNTIME: LazyLock<Mutex<Runtime>> = LazyLock::new(|| Mutex::new(Runtime::default()));
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: LazyLock<Vec<Box<dyn Hooks>>> = LazyLock::new(|| vec![Box::new(Recorder)]);
        &MODS
    }
    struct Recorder;
    impl Hooks for Recorder {
        fn client_directional_input(&self, host: &dyn Host, event: &mut ClientDirectionalInputEvent) {
            let tick = host.client_player_input_tick_count();
            let mut runtime = RUNTIME.lock().unwrap();
            if let Some(tick) = tick {
                runtime.observed_ticks.push(tick);
                if let Err(error) = runtime.twerk.movement_input(runtime.running, tick, &mut event.shift) {
                    runtime.errors.push(error);
                }
            }
            let running = runtime.no_block_running;
            runtime.no_block.movement_input(running, &mut event.shift);
        }
    }
}
