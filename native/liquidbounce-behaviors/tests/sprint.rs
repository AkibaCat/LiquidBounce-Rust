use liquidbounce_behaviors::rotations::{
    DirectionalInput, MovementCorrection, Rotation, RotationManager, modern_mouse_gcd,
};
use liquidbounce_behaviors::sprint::{Context, Source, Sprint, SprintMode, movement_direction};

fn trig(radians: f64) -> (f32, f32) {
    (radians.sin() as f32, radians.cos() as f32)
}

#[test]
fn source_defaults_and_phase_order_preserve_input_and_network() {
    let sprint = Sprint::default();
    assert_eq!(sprint.mode, SprintMode::Legit);
    assert!(sprint.stop_on.ground && sprint.stop_on.air);
    assert!(sprint.stop_on.sneaking && sprint.stop_on.using_item);
    assert!(!sprint.ignore.blindness && !sprint.ignore.hunger && !sprint.ignore.collision);
    let input = DirectionalInput::FORWARDS;
    let stopped = sprint.movement_override(
        true,
        input,
        Context {
            slow_due_to_item: true,
            ..Context::default()
        },
        false,
        trig,
    );
    assert_eq!(stopped, Some(false));
    assert!(sprint.event(true, Source::Input, input, false, stopped));
    assert!(!sprint.event(true, Source::MovementTick, input, true, stopped));
    assert!(!sprint.event(true, Source::Network, input, false, stopped));
    assert!(sprint.event(true, Source::Network, input, true, stopped));
    assert_eq!(
        sprint.movement_override(false, input, Context::default(), false, trig),
        None
    );
}

#[test]
fn opposing_flags_and_direction_follow_source_formula() {
    let both = DirectionalInput {
        forwards: true,
        backwards: true,
        left: true,
        right: true,
    };
    assert!(!both.is_moving());
    assert_eq!(movement_direction(17.0, both), 17.0);
    let sprint = Sprint::default();
    assert_eq!(
        sprint.movement_override(true, both, Context::default(), false, trig),
        None
    );
    for (input, expected) in [
        (
            DirectionalInput {
                left: true,
                ..DirectionalInput::FORWARDS
            },
            -45.0,
        ),
        (
            DirectionalInput {
                backwards: true,
                left: true,
                ..DirectionalInput::default()
            },
            225.0,
        ),
        (
            DirectionalInput {
                right: true,
                ..DirectionalInput::default()
            },
            90.0,
        ),
        (
            DirectionalInput {
                forwards: true,
                backwards: true,
                left: true,
                right: false,
            },
            -90.0,
        ),
    ] {
        assert_eq!(movement_direction(0.0, input), expected);
    }
}

#[test]
fn rotation_prevention_requires_selected_state_and_correction_off() {
    let mut sprint = Sprint::default();
    let input = DirectionalInput::FORWARDS;
    let mut context = Context {
        on_ground: true,
        forward: 1.0,
        current_rotation: Some(Rotation::new(180.0, 0.0)),
        movement_correction: Some(MovementCorrection::Off),
        ..Context::default()
    };
    assert_eq!(
        sprint.movement_override(true, input, context, false, trig),
        Some(false)
    );
    context.movement_correction = Some(MovementCorrection::Silent);
    assert_eq!(
        sprint.movement_override(true, input, context, false, trig),
        Some(true)
    );
    context.movement_correction = Some(MovementCorrection::Off);
    sprint.stop_on.ground = false;
    assert_eq!(
        sprint.movement_override(true, input, context, false, trig),
        Some(true)
    );
    context.on_ground = false;
    assert_eq!(
        sprint.movement_override(true, input, context, false, trig),
        Some(false)
    );
    assert_eq!(
        sprint.movement_override(true, input, context, true, trig),
        Some(true)
    );
    sprint.mode = SprintMode::Omnidirectional;
    assert_eq!(
        sprint.movement_override(true, input, context, false, trig),
        Some(true)
    );
    context.shift_key_down = true;
    assert_eq!(
        sprint.movement_override(true, input, context, false, trig),
        Some(false)
    );
}

#[test]
fn omni_thresholds_scaffold_precedence_and_jump_yaw_match_source() {
    let sprint = Sprint {
        mode: SprintMode::Omnidirectional,
        ..Sprint::default()
    };
    assert!(sprint.has_forward_impulse(true, false, false, [-0.5, 0.0]));
    assert!(!sprint.can_start_forward_impulse(true, false, true, [-0.5, 0.0], false));
    assert!(sprint.can_start_forward_impulse(true, false, false, [-0.5, 0.0], true));
    assert!(sprint.can_start_forward_impulse(true, false, false, [-0.8, 0.0], false));
    assert!(!sprint.has_forward_impulse(true, false, true, [1.0e-5, 0.0]));
    assert!(Sprint::default().omnidirectional(false, true));
    let input = DirectionalInput {
        left: true,
        ..DirectionalInput::default()
    };
    assert_eq!(sprint.jump_yaw(true, 44.0, 30.0, input), -60.0);
    assert_eq!(sprint.jump_yaw(false, 44.0, 30.0, input), 44.0);
}

#[test]
fn omnirotation_request_runs_after_manager_update_and_uses_previous_input() {
    let sprint = Sprint {
        mode: SprintMode::Omnirotational,
        ..Sprint::default()
    };
    let player = Rotation::new(0.0, 23.0);
    let target = sprint
        .rotation_target(
            true,
            player,
            DirectionalInput {
                left: true,
                ..DirectionalInput::default()
            },
        )
        .unwrap();
    assert_eq!(target.rotation, Rotation::new(-90.0, 23.0));
    assert_eq!(target.movement_correction, MovementCorrection::Silent);
    assert_eq!(
        (
            target.ticks_until_reset,
            target.reset_threshold,
            target.consider_inventory
        ),
        (5, 2.0, false)
    );
    let mut manager = RotationManager::default();
    let gcd = modern_mouse_gcd(0.5);
    manager.update(player, gcd, true, false, |_| true).unwrap();
    manager.request(17, -20, target, true);
    assert_eq!(manager.current, None);
    manager.update(player, gcd, true, false, |_| true).unwrap();
    assert!(manager.current.unwrap().yaw < -89.0);
    assert_eq!(manager.current.unwrap().pitch, 23.0);
    assert_eq!(
        Sprint::default().rotation_target(true, player, DirectionalInput::FORWARDS),
        None
    );
}
