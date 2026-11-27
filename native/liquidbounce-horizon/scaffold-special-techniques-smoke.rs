#[cfg(test)]
mod special_smoke {
    use super::rules::*;

    #[test]
    fn source_rotation_rounding_and_ground_side_are_retained() {
        assert_eq!(moving_yaw(-157.5), 0.0);
        assert_eq!(moving_yaw(-112.5), 90.0);
        assert_eq!(no_input_rotation(Some([-0.1, 20.0])), Some([-45.0, 75.0]));
        assert_eq!(no_input_rotation(None), None);
        let mut god = GodBridge::default();
        assert_eq!(god.rotation(0.0, true, [0.8, 1.0, 0.2], [1.0, 0.0], false, false), [45.0, 75.7]);
        assert_eq!(god.rotation(90.0, false, [0.0; 3], [0.0, 1.0], false, false), [135.0, 75.7]);
        assert_eq!(god.rotation(45.0, true, [0.0; 3], [0.0; 2], true, true), [45.0, 75.6]);
        assert!(god.on_right_side);
        assert_eq!(Breezily::rotation(90.0), [90.0, 80.0]);
    }

    #[test]
    fn ledge_uses_strict_count_and_only_source_random_branches() {
        let mut draws = Vec::new();
        let configured = [LedgeMode::Jump, LedgeMode::StopInput];
        assert_eq!(ledge_mode(2, 3, &configured, 0.42, |n| { draws.push(n); 0 }), LedgeMode::Sneak);
        assert!(draws.is_empty());
        assert_eq!(ledge_mode(3, 3, &configured, 0.42, |n| { draws.push(n); 0 }), LedgeMode::Jump);
        assert_eq!(draws, [2]);
        draws.clear();
        assert_eq!(ledge_mode(3, 3, &configured, 0.8, |n| { draws.push(n); 0 }), LedgeMode::StopInput);
        assert_eq!(draws, [2, 1]);
        draws.clear();
        assert_eq!(ledge_mode(3, 3, &[LedgeMode::Jump], 0.8, |n| { draws.push(n); 0 }), LedgeMode::Sneak);
        assert_eq!(draws, [1]);
        assert!(jump_apex(0.42) < 2.0 && jump_apex(0.8) >= 2.0);
        let mut count = 0;
        assert_eq!(action(LedgeMode::Backwards, || { count += 1; 2 }), LedgeAction { step_back: true, ..Default::default() });
        assert_eq!(count, 0);
        assert_eq!(action(LedgeMode::Sneak, || { count += 1; 2 }).sneak_time, 2);
        assert_eq!(count, 1);
    }

    #[test]
    fn breezily_keeps_side_in_center_and_resamples_only_on_changes() {
        let mut state = Breezily::default();
        assert_eq!(state.current_edge_distance.to_bits(), 0.45f64.to_bits());
        let mut draws = 0;
        let input = [true, true, false, false];
        assert_eq!(state.input(input, false, true, 10, [0.8, 2.0, 0.5], 0.0, || { draws += 1; 0.5 }), [true, true, false, true]);
        assert_eq!(state.input(input, false, true, 20, [0.5, 2.0, 0.5], 0.0, || { draws += 1; 0.5 }), [true, true, false, true]);
        assert_eq!(state.input(input, false, true, 30, [0.2, 2.0, 0.5], 0.0, || { draws += 1; 0.5 }), [true, true, true, false]);
        assert_eq!(draws, 2);
    }

    #[test]
    fn breezily_grace_is_inclusive_and_input_guards_preserve_state() {
        let mut state = Breezily::default();
        let forward = [true, false, false, false];
        let at = [0.8, 2.0, 0.5];
        state.input(forward, false, true, 100, at, 0.0, || 0.5);
        assert_eq!(state.input(forward, false, false, 600, at, 0.0, || panic!("same side")), [true, false, false, true]);
        assert_eq!(state.input(forward, false, false, 601, at, 0.0, || panic!("expired")), forward);
        let before = state.clone();
        assert_eq!(state.input(forward, true, true, 1000, at, 0.0, || panic!("shift guard")), forward);
        assert_eq!(state, before);
    }

    #[test]
    fn cardinal_mapping_handles_negative_and_wrapped_yaw() {
        assert_eq!(cardinal(-45.0), Cardinal::South);
        assert_eq!(cardinal(45.0), Cardinal::West);
        assert_eq!(cardinal(180.0), Cardinal::North);
        assert_eq!(cardinal(-90.0), Cardinal::East);
        assert_eq!(cardinal_offset(450.0), [-1, 0, 0]);
    }
}
