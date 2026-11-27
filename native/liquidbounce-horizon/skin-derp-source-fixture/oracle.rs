#[test]
fn production_skin_derp_matches_unchanged_kotlin_with_real_ticker_and_rng() {
    use crate::skin_derp::{SkinDerp, part_index};
    let source: serde_json::Value =
        serde_json::from_str(include_str!("source-vectors.json")).unwrap();
    let mut state = SkinDerp::default();
    let mut actual = 127u8;
    let mut enabled = false;
    let mut in_game = true;
    let mut pending = Vec::new();
    let mut rng = crate::random::ThreadRandom::seeded(
        source["seed"].as_u64().unwrap(),
        source["thread"].as_u64().unwrap(),
    );
    let mut seed = source["seed"].as_u64().unwrap();
    let step = source["thread"]
        .as_u64()
        .unwrap()
        .wrapping_shl(1)
        .wrapping_add(0x9e3779b97f4a7c15);
    for (row, line) in include_str!("commands.txt").lines().enumerate() {
        let p: Vec<_> = line.split_whitespace().collect();
        let mut random = || {
            seed = seed.wrapping_add(step);
            rng.next_bool()
        };
        match p[0] {
            "actual" => actual = p[1].parse().unwrap(),
            "enabled" => {
                let value = p[1].parse().unwrap();
                if enabled != value && in_game {
                    if value {
                        state.on_enabled(actual);
                    } else {
                        actual = state.on_disabled();
                    }
                }
                enabled = value;
            }
            "inGame" => in_game = p[1].parse().unwrap(),
            "delay" => state.settings.delay = p[1].parse().unwrap(),
            "sync" => state.settings.sync = p[1].parse().unwrap(),
            "parts" => {
                state.settings.parts = 0;
                for tag in &p[1..] {
                    if let Some(index) = part_index(tag) {
                        state.settings.parts |= 1 << index;
                    }
                }
            }
            "tick" => {
                let mut retained = Vec::new();
                let mut next = Vec::new();
                for callback in std::mem::take(&mut pending) {
                    let (keep, effect) =
                        state.advance_callback(callback, enabled && in_game, actual, &mut random);
                    next.extend(state.take_wait_registrations());
                    if let Some(parts) = effect {
                        actual = parts;
                    }
                    if keep {
                        retained.push(callback);
                    }
                }
                if let Some(parts) =
                    state.start_tick_handler(enabled && in_game, actual, &mut random)
                {
                    actual = parts;
                }
                next.extend(state.take_wait_registrations());
                retained.extend(next);
                pending = retained;
            }
            _ => panic!("unknown line {line}"),
        }
        let expected = &source["rows"][row];
        assert_eq!(
            u64::from(actual),
            expected[0].as_u64().unwrap(),
            "actual row {row}: {line}"
        );
        assert_eq!(
            u64::from(state.previous_parts()),
            expected[1].as_u64().unwrap(),
            "previous row {row}: {line}"
        );
        assert_eq!(
            seed.to_string(),
            expected[2].as_str().unwrap(),
            "random seed/draws row {row}: {line}"
        );
    }
}
