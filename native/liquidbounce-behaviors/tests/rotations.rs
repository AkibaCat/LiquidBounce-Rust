// SPDX-License-Identifier: GPL-3.0-or-later
// Independent expected bits: native/liquidbounce-horizon/rotation-reference.java
// on Java 25 + Kotlin stdlib 2.4.21. Saved in research/rotation-source-vectors.json.
use liquidbounce_behaviors::rotations::{
    DefaultRotationTarget, DirectionalInput, MovementCorrection, RequestHandler, Rotation,
    RotationError, RotationManager, modern_mouse_gcd, movement_yaw,
};

#[test]
fn gcd_and_normalization_match_jvm_operation_widths_and_wrapped_yaw() {
    let sensitivities = [0.0, 0.1, 0.5, 0.9, 1.0];
    let gcd_bits = [
        0x3f83_a92a_4000_0000,
        0x3f95_98ee_a000_0000,
        0x3fc3_3333_6000_0000,
        0x3fdf_1f07_6000_0000,
        0x3fe3_a92a_4000_0000,
    ];
    for (sensitivity, expected) in sensitivities.into_iter().zip(gcd_bits) {
        assert_eq!(modern_mouse_gcd(sensitivity).to_bits(), expected);
    }
    let vectors = [
        ([180.0, 45.0], [0.0, 0.0], [0xc334_0002, 0x4234_0002]),
        ([-179.92, 90.0], [179.95, -90.0], [0x4334_1999, 0x42b4_0000]),
        ([10.0, -10.0], [350.0, 15.0], [0x43b8_f99a, 0xc120_ccd0]),
        ([0.001, -0.001], [0.0, 0.0], [0, 0]),
        (
            [179.99, 89.99],
            [-179.99, -89.99],
            [0xc333_fd71, 0x42b4_0000],
        ),
    ];
    for ([yaw, pitch], [current_yaw, current_pitch], expected) in vectors {
        let normalized = Rotation::new(yaw, pitch)
            .normalize(
                Rotation::new(current_yaw, current_pitch),
                modern_mouse_gcd(0.5),
            )
            .unwrap();
        assert_eq!(
            [normalized.yaw.to_bits(), normalized.pitch.to_bits()],
            expected
        );
        assert!(normalized.is_normalized);
    }
    // Kotlin rounds both signs of a tie toward positive infinity.
    assert_eq!(
        Rotation::new(-1.0, 1.0)
            .normalize(Rotation::ZERO, 2.0)
            .unwrap(),
        Rotation {
            yaw: 0.0,
            pitch: 2.0,
            is_normalized: true,
        }
    );
    assert_eq!(
        Rotation::new(f32::NAN, 0.0).normalize(Rotation::ZERO, 1.0),
        Err(RotationError::NanRounding)
    );
    // A previously normalized rotation is returned verbatim, even under
    // NoPitchLimit and without reading a usable sensitivity/GCD.
    let raw = Rotation {
        yaw: 720.5,
        pitch: 140.0,
        is_normalized: true,
    };
    assert_eq!(raw.normalize(Rotation::ZERO, f64::NAN), Ok(raw));
}

fn flags(mask: u8) -> DirectionalInput {
    DirectionalInput {
        forwards: mask & 1 != 0,
        backwards: mask & 2 != 0,
        left: mask & 4 != 0,
        right: mask & 8 != 0,
    }
}

fn mask(input: DirectionalInput) -> u8 {
    u8::from(input.forwards)
        | u8::from(input.backwards) << 1
        | u8::from(input.left) << 2
        | u8::from(input.right) << 3
}

#[test]
fn silent_input_matches_java_for_all_independent_flags_and_quantized_trig() {
    // Mth table values include its asymmetric +/-30 and +/-45 samples.
    // Replacing these with native sin/cos can change diagonal boundary keys.
    let vectors = [
        (
            0x250d3132,
            0xbf800000,
            [0, 2, 1, 0, 8, 10, 9, 8, 4, 6, 5, 4, 0, 2, 1, 0],
        ),
        (
            0xbf3504f3,
            0xbf3504f3,
            [0, 6, 9, 0, 10, 2, 8, 10, 5, 4, 1, 5, 0, 6, 9, 0],
        ),
        (
            0xbf800000,
            0x00000000,
            [0, 4, 8, 0, 2, 6, 10, 2, 1, 5, 9, 1, 0, 4, 8, 0],
        ),
        (
            0xbf3504f3,
            0x3f350082,
            [0, 5, 10, 0, 6, 4, 2, 6, 9, 1, 8, 9, 0, 5, 10, 0],
        ),
        (
            0xbefffc5f,
            0x3f5db1bf,
            [0, 1, 2, 0, 4, 4, 2, 4, 8, 1, 8, 8, 0, 1, 2, 0],
        ),
        (
            0x00000000,
            0x3f800000,
            [0, 1, 2, 0, 4, 5, 6, 4, 8, 9, 10, 8, 0, 1, 2, 0],
        ),
        (
            0x3efffc5f,
            0x3f5db4e3,
            [0, 1, 2, 0, 4, 1, 4, 4, 8, 8, 2, 8, 0, 1, 2, 0],
        ),
        (
            0x3f3504f3,
            0x3f3504f3,
            [0, 9, 6, 0, 5, 1, 4, 5, 10, 8, 2, 10, 0, 9, 6, 0],
        ),
        (
            0x3f800000,
            0x250d3132,
            [0, 8, 4, 0, 1, 9, 5, 1, 2, 10, 6, 2, 0, 8, 4, 0],
        ),
        (
            0x3f3504f3,
            0xbf3504f3,
            [0, 10, 5, 0, 9, 8, 1, 9, 6, 2, 4, 6, 0, 10, 5, 0],
        ),
    ];
    for (sin, cos, expected) in vectors {
        for (raw, expected) in expected.into_iter().enumerate() {
            assert_eq!(
                mask(
                    flags(raw as u8)
                        .silent_correct_with_sin_cos(f32::from_bits(sin), f32::from_bits(cos))
                ),
                expected
            );
        }
    }
    let original = flags(15); // W+S and A+D are independently held.
    assert!(!original.is_moving());
    assert_eq!(original.axes(), [0.0, 0.0]);
    assert_eq!(mask(original), 15); // Original packet snapshot remains intact.
    assert_eq!(mask(original.silent_correct_with_sin_cos(0.0, 1.0)), 0);
    for correction in [
        None,
        Some(MovementCorrection::Off),
        Some(MovementCorrection::Strict),
    ] {
        assert_eq!(
            original.transform(Some(30.0), Some(Rotation::ZERO), correction, |_| panic!(
                "no trig query outside SILENT correction"
            )),
            original
        );
    }
    let input = DirectionalInput::FORWARDS.transform(
        Some(0.0),
        Some(Rotation::new(90.0, 0.0)),
        Some(MovementCorrection::Silent),
        |angle| {
            assert_eq!(
                angle.to_bits(),
                (-90.0 * f32::from_bits(0x3c8e_fa35)).to_bits()
            );
            (-1.0, 0.0)
        },
    );
    assert_eq!(mask(input), 4);
    // Exactly +/-0.5 use Math.round's asymmetric ties.
    assert_eq!(
        mask(DirectionalInput::FORWARDS.silent_correct_with_sin_cos(0.5, 0.5)),
        1
    );
}

#[test]
fn requests_expire_on_the_fifth_tick_replace_provider_and_follow_java_ties() {
    let mut requests = RequestHandler::default();
    requests.request(1, -20, 5, "sprint");
    requests.request(2, 30, 2, "combat");
    assert_eq!(requests.active_value(|_| true), Some(&"combat"));
    requests.tick(2);
    assert_eq!(requests.active_value(|_| true), Some(&"sprint"));
    requests.request(1, -20, 5, "renewed");
    requests.tick(4);
    assert_eq!(requests.active_value(|_| true), Some(&"renewed"));
    requests.tick(1);
    assert_eq!(requests.peek_off_thread(), Some(&"renewed"));
    assert_eq!(requests.active_value(|_| true), None);
    requests.request(1, 60, 5, "disabled");
    requests.request(2, 0, 5, "active");
    assert_eq!(
        requests.active_value(|provider| provider != 1),
        Some(&"active")
    );
    requests.clear();
    assert_eq!(requests.current_tick(), 0);
    assert_eq!(requests.peek_off_thread(), None);
    let mut requests = RequestHandler::default();
    for provider in 1..=5 {
        requests.request(provider, 0, 5, format!("p{provider}"));
    }
    requests.request(3, 0, 5, "new3".to_owned());
    let mut trace = Vec::new();
    let mut active = std::collections::BTreeSet::from([1, 2, 3, 4, 5]);
    while let Some(value) = requests
        .active_value(|provider| active.contains(&provider))
        .cloned()
    {
        let provider = if value == "new3" {
            3
        } else {
            value[1..].parse().unwrap()
        };
        trace.push(value);
        active.remove(&provider);
    }
    assert_eq!(trace.join(","), "p1,new3,p5,p4,p2");
}

#[test]
fn managed_rotation_resets_after_request_expiry_and_blends_the_f5_model() {
    let mut manager = RotationManager::default();
    let player = Rotation::new(0.0, 10.0);
    let target = DefaultRotationTarget::sprint(Rotation::new(90.0, 10.0));
    assert_eq!(target.ticks_until_reset, 5);
    assert_eq!(target.reset_threshold, 2.0);
    assert!(!target.consider_inventory);
    assert_eq!(target.movement_correction, MovementCorrection::Silent);
    manager.request(1, -20, target, true);
    for tick in 0..5 {
        let update = manager.update(player, 1.0, true, true, |_| true).unwrap();
        assert_eq!(manager.current.unwrap().yaw, 90.0, "tick {tick}");
        assert_eq!(update.reset_player_yaw, None);
    }
    assert_eq!(manager.model_yaw(0.5, 0.0, true), 90.0);
    let reset = manager.update(player, 1.0, true, false, |_| true).unwrap();
    assert_eq!(reset.reset_player_yaw, Some(0.0));
    assert_eq!(manager.current, None);
    assert_eq!(manager.last, None);
    assert_eq!(manager.last_model.unwrap().yaw, 90.0);
    assert_eq!(manager.model_yaw(0.5, 0.0, true), 45.0);
    manager.update(player, 1.0, true, false, |_| true).unwrap();
    assert_eq!(manager.last_model, None);
    // A pending default plan keeps resetting after disable, using the same
    // linear processor rather than snapping a 180-degree residual angle.
    let mut slow = target;
    slow.horizontal_turn_speed = 10.0;
    manager.request(1, -20, slow, true);
    manager.update(player, 1.0, true, false, |_| true).unwrap();
    assert_eq!(manager.current.unwrap().yaw, 10.0);
    manager.update(player, 1.0, true, false, |_| false).unwrap();
    assert_eq!(manager.current, None);
}

#[test]
fn packet_final_state_keeps_actual_theoretical_and_relative_corrections_distinct() {
    let mut manager = RotationManager::default();
    manager.outbound_rotation(370.0, 270.0, false);
    assert_eq!(manager.actual_server.yaw, 10.0);
    assert_eq!(manager.actual_server.pitch, -90.0);
    manager.outbound_rotation(160.0, 100.0, true);
    assert_eq!(manager.server_rotation(false, false).yaw, 10.0);
    assert_eq!(manager.server_rotation(true, false).yaw, 160.0);
    assert_eq!(manager.server_rotation(false, true).pitch, 90.0);
    manager.inbound_rotation(30.0, 50.0, true, true);
    assert_eq!(manager.actual_server.yaw, 40.0);
    assert_eq!(manager.actual_server.pitch, -40.0);
    assert_eq!(manager.actual_server, manager.theoretical_server);
    // Incoming corrections do not yaw-wrap, unlike outbound server parsing.
    manager.inbound_rotation(720.0, 20.0, false, false);
    assert_eq!(manager.actual_server.yaw, 720.0);
    assert_eq!(
        movement_yaw(
            15.0,
            Some(manager.actual_server),
            Some(MovementCorrection::Off)
        ),
        15.0
    );
    assert_eq!(
        movement_yaw(
            15.0,
            Some(manager.actual_server),
            Some(MovementCorrection::Silent)
        ),
        720.0
    );
    assert_eq!(
        movement_yaw(
            15.0,
            Some(Rotation::new(f32::NAN, 0.0)),
            Some(MovementCorrection::Strict)
        ),
        15.0
    );
    manager.reset_world();
    assert_eq!(manager, RotationManager::default());
}
