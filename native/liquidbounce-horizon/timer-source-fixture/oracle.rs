use crate::timer::*;
use std::collections::HashMap;
use std::fmt::Write;
pub fn compare() {
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let commands = std::fs::read_to_string(directory.join("commands.txt")).unwrap();
    let expected = std::fs::read_to_string(directory.join("results.txt")).unwrap();
    let mut timer = Timer::default();
    let mut providers = HashMap::<u64, bool>::new();
    let mut same = true;
    let mut player = true;
    let mut module = ModuleTimer::default();
    let mut settings = Settings::default();
    let mut context = Context::default();
    let mut running = false;
    let mut waits = Vec::new();
    for (index, (command, expected)) in commands.lines().zip(expected.lines()).enumerate() {
        providers.insert(u64::MAX, running);
        let p: Vec<_> = command.split(' ').collect();
        let mut returned = "-".to_owned();
        let i = |at: usize| p[at].parse::<i32>().unwrap();
        let id = |at: usize| p[at].parse::<u64>().unwrap();
        let float = |at: usize| f32::from_bits(p[at].parse::<u32>().unwrap());
        let boolean = |at: usize| p[at].parse::<bool>().unwrap();
        match p[0] {
            "provider" => {
                providers.insert(id(1), boolean(2));
            }
            "same" => same = boolean(1),
            "player" => player = boolean(1),
            "request" => timer.request_speed(float(4), i(2), id(1), i(3)),
            "tick" => {
                for _ in 0..i(1) {
                    waits.retain(|&callback| {
                        module.advance_callback(callback, running, settings.mode)
                    });
                    timer.game_tick();
                    if module
                        .game_tick(running, settings, context, &mut timer, u64::MAX)
                        .is_err()
                    {
                        returned = "invalid-range".into();
                    }
                    waits.extend(module.take_wait_registrations());
                }
            }
            "scale" => {
                returned = timer
                    .scale_frame(float(1), same, &mut |id| {
                        *providers.entry(id).or_insert(true)
                    })
                    .to_bits()
                    .to_string()
            }
            "frame" => {
                returned = timer
                    .frame_ticks(i(1), i(2), same, &mut |id| {
                        *providers.entry(id).or_insert(true)
                    })
                    .to_string()
            }
            "playerTick" => timer.player_tick_final(boolean(1)),
            "hold" => timer.hold(id(1), i(2)),
            "burst" => {
                returned = timer
                    .burst(player, i(1), [i(2), i(3)])
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| "invalid-range".into())
            }
            "within" => {
                returned = timer
                    .balance_within([i(1), i(2)])
                    .map(|v| v.to_bits().to_string())
                    .unwrap_or_else(|_| "invalid-range".into())
            }
            "balanced" => {
                returned = timer
                    .request_balanced(float(4), i(2), id(1), [i(5), i(6)])
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| "invalid-range".into())
            }
            "world" => timer.world_change(),
            "reset" => timer.reset_balance(),
            "module" => {
                running = boolean(1);
                module.synchronize(running, settings.mode, &mut timer, u64::MAX);
            }
            "mode" => {
                settings.mode = Mode::from_tag(p[1]).unwrap();
                module.synchronize(running, settings.mode, &mut timer, u64::MAX);
            }
            "context" => {
                context = Context {
                    moving: boolean(1),
                    combat: boolean(2),
                    inventory_or_container: matches!(p[3], "inventory" | "container"),
                }
            }
            "set" => match (p[1], p[2]) {
                ("Classic", "Speed") => settings.classic_speed = float(3),
                ("Pulse", "NormalSpeed") => settings.normal_speed = float(3),
                ("Pulse", "NormalSpeedTicks") => settings.normal_ticks = i(3),
                ("Pulse", "BoostSpeed") => settings.pulse_boost_speed = float(3),
                ("Pulse", "BoostSpeedTicks") => settings.boost_ticks = i(3),
                ("Pulse", "OnMove") => settings.on_move = boolean(3),
                ("Boost", "BoostSpeed") => settings.boost_speed = float(3),
                ("Boost", "SlowSpeed") => settings.slow_speed = float(3),
                ("Boost", "NormalizeDuringCombat") => settings.normalize_combat = boolean(3),
                ("Boost", "Balance") => settings.balance = [i(3), i(4)],
                ("Boost", "ResetBalanceOnFlag") => settings.reset_balance_on_flag = boolean(3),
                _ => panic!("unknown setting {command}"),
            },
            "flag" => {
                if running && settings.mode == Mode::Boost && settings.reset_balance_on_flag {
                    timer.reset_balance();
                }
            }
            _ => panic!("unknown {command}"),
        }
        providers.insert(u64::MAX, running);
        let speed = timer.speed(same, &mut |id| *providers.entry(id).or_insert(true));
        let mut actual = format!(
            "{returned}|{}|{}|{}|{}|{}|{}|{}|{}|",
            timer.balance.to_bits(),
            timer.frame_speed.to_bits(),
            timer.real_ticks.to_bits(),
            timer.burst_ticks,
            timer.bursting,
            timer.held_ticks,
            timer.frame_end,
            speed.to_bits()
        );
        for back in 0..=21 {
            if back > 0 {
                actual.push(',');
            }
            write!(&mut actual, "{}", timer.held_ticks_within(back)).unwrap();
        }
        assert_eq!(actual, expected, "case {index}: {command}");
    }
}
