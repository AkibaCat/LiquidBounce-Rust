// SPDX-License-Identifier: GPL-3.0-or-later
// Compile the pinned renderer math directly, without building another game.
#[allow(dead_code, clippy::needless_range_loop)] // Matches the pinned workspace lint.
#[path = "../../../../upstream/Horizon/crates/horizon-client-render/src/camera.rs"]
mod camera;

use camera::{BobState, HurtBob};
use liquidbounce_core::{
    BindAction, InputBind, InputContext, InputKey, KeyAction, KeyboardInput, Modifiers,
    ModuleCallback, ModuleRegistry, WorldState,
};
use liquidbounce_horizon_behavior_tests::{BehaviorModules, RawOptions};
use std::cell::RefCell;

fn fixture() -> (ModuleRegistry, BehaviorModules) {
    let mut registry = ModuleRegistry::new();
    let modules = BehaviorModules::register(&mut registry);
    registry.update_world(WorldState {
        world_present: true,
        player_present: true,
    });
    (registry, modules)
}

fn input(action: KeyAction) -> KeyboardInput {
    KeyboardInput::new(11, action, Modifiers::NONE)
}

fn camera_state() -> BobState {
    BobState {
        living: Some(HurtBob {
            hurt_time: 5.25,
            hurt_duration: 10,
            hurt_dir: 47.0,
            death_time: None,
        }),
        player: Some((-3.7, 0.09)),
    }
}

// The production phase supplies the gated inputs to the same pinned functions.
fn gated_bob(
    modules: &BehaviorModules,
    registry: &ModuleRegistry,
    state: BobState,
    view: bool,
) -> glam::Mat4 {
    let (hurt, view) = modules.camera_bob_effects(registry, true, view);
    let mut filtered = state;
    if !hurt {
        filtered.living = None;
    }
    glam::Mat4::from_cols_array(&camera::bob_matrix(&filtered, 1.0, view).m)
}

#[test]
fn registered_camera_modules_have_original_defaults_and_world_lifecycle() {
    let mut registry = ModuleRegistry::new();
    let modules = RefCell::new(BehaviorModules::register(&mut registry));
    let ids = [modules.borrow().no_bob_id, modules.borrow().no_hurt_cam_id];
    assert_eq!(registry.modules().count(), modules.borrow().ids().len());
    for id in ids {
        let module = registry.module(id).unwrap();
        assert!(!module.enabled());
        assert!(!module.definition().not_activatable);
        assert!(!module.definition().disable_activation);
        assert!(!module.definition().disable_on_quit);
        assert!(!registry.running(id));
    }
    let observed = RefCell::new(Vec::new());
    let callbacks = |registry: &mut ModuleRegistry, event: ModuleCallback| {
        observed.borrow_mut().push(event);
        modules.borrow_mut().callback(
            registry,
            event,
            RawOptions {
                fov: 70,
                gamma: 0.5,
            },
        );
        Ok(())
    };
    for id in ids {
        registry.set_enabled_sync(id, true, &callbacks).unwrap();
    }
    assert_eq!(
        modules.borrow().camera_bob_effects(&registry, true, true),
        (true, true)
    );
    registry.update_world_sync(
        WorldState {
            world_present: true,
            player_present: false,
        },
        &callbacks,
    );
    assert_eq!(
        modules.borrow().camera_bob_effects(&registry, true, true),
        (true, true)
    );
    registry.update_world_sync(
        WorldState {
            world_present: true,
            player_present: true,
        },
        &callbacks,
    );
    assert_eq!(
        modules.borrow().camera_bob_effects(&registry, true, true),
        (false, false)
    );
    for id in ids {
        assert!(observed.borrow().contains(&ModuleCallback::OnEnabled(id)));
        // Original startup activation suppresses notifications while loading.
        assert!(!observed.borrow().contains(&ModuleCallback::Notification {
            module: id,
            enabled: true
        }));
        registry.set_enabled_sync(id, false, &callbacks).unwrap();
        registry.set_enabled_sync(id, true, &callbacks).unwrap();
        assert!(observed.borrow().contains(&ModuleCallback::OnDisabled(id)));
        assert!(observed.borrow().contains(&ModuleCallback::Notification {
            module: id,
            enabled: true
        }));
    }
    registry.disconnect_sync(&callbacks);
    registry.update_world_sync(WorldState::default(), &callbacks);
    for id in ids {
        assert!(registry.module(id).unwrap().enabled());
        assert!(!registry.running(id));
    }
    assert_eq!(
        modules.borrow().camera_bob_effects(&registry, true, true),
        (true, true)
    );
    registry.update_world_sync(
        WorldState {
            world_present: true,
            player_present: true,
        },
        &callbacks,
    );
    assert_eq!(
        modules.borrow().camera_bob_effects(&registry, true, true),
        (false, false)
    );
    registry.set_destructed(true);
    assert_eq!(
        modules.borrow().camera_bob_effects(&registry, true, true),
        (true, true)
    );
}

#[test]
fn toggle_and_hold_bindings_control_camera_effects_and_release_through_open_screen() {
    for hold in [false, true] {
        let (mut registry, modules) = fixture();
        for id in [modules.no_bob_id, modules.no_hurt_cam_id] {
            registry
                .set_bind(
                    id,
                    Some(InputBind::new(
                        InputKey::Keyboard(11),
                        if hold {
                            BindAction::Hold
                        } else {
                            BindAction::Toggle
                        },
                    )),
                )
                .unwrap();
        }
        let open = InputContext {
            screen_open: true,
            debug_modifier_down: false,
        };
        registry.handle_keyboard(input(KeyAction::Press), open);
        assert_eq!(
            modules.camera_bob_effects(&registry, true, true),
            (true, true)
        );
        registry.handle_keyboard(input(KeyAction::Press), InputContext::default());
        assert_eq!(
            modules.camera_bob_effects(&registry, true, true),
            (false, false)
        );
        registry.handle_keyboard(input(KeyAction::Repeat), InputContext::default());
        assert_eq!(
            modules.camera_bob_effects(&registry, true, true),
            (false, false)
        );
        registry.handle_keyboard(input(KeyAction::Release), open);
        if hold {
            assert_eq!(
                modules.camera_bob_effects(&registry, true, true),
                (true, true)
            );
        } else {
            assert_eq!(
                modules.camera_bob_effects(&registry, true, true),
                (false, false)
            );
            registry.handle_keyboard(input(KeyAction::Press), InputContext::default());
            assert_eq!(
                modules.camera_bob_effects(&registry, true, true),
                (true, true)
            );
        }
    }
}

#[test]
fn no_bob_cancels_walk_transform_but_keeps_nonzero_hurt_transform_and_history() {
    let (mut registry, modules) = fixture();
    let state = camera_state();
    let original = camera::bob_matrix(&state, 1.0, true).m;
    let hurt_only = camera::bob_matrix(&state, 1.0, false).m;
    assert_ne!(original, hurt_only);
    assert_ne!(hurt_only, glam::Mat4::IDENTITY.to_cols_array());
    registry.set_enabled(modules.no_bob_id, true).unwrap();
    assert_eq!(
        gated_bob(&modules, &registry, state, true).to_cols_array(),
        hurt_only
    );
    assert_eq!(
        gated_bob(&modules, &registry, state, false).to_cols_array(),
        hurt_only
    );
    assert_eq!(state, camera_state());
    registry.set_enabled(modules.no_bob_id, false).unwrap();
    assert_eq!(
        gated_bob(&modules, &registry, state, true).to_cols_array(),
        original
    );
    assert_eq!(
        modules.camera_bob_effects(&registry, false, false),
        (false, false)
    );
}

#[test]
fn no_hurt_cam_cancels_hurt_transform_but_keeps_nonzero_walk_transform() {
    let (mut registry, modules) = fixture();
    let state = camera_state();
    let walking = BobState {
        living: None,
        ..state
    };
    let walking_matrix = camera::bob_matrix(&walking, 1.0, true).m;
    assert_ne!(walking_matrix, glam::Mat4::IDENTITY.to_cols_array());
    assert_ne!(camera::bob_matrix(&state, 1.0, true).m, walking_matrix);
    registry.set_enabled(modules.no_hurt_cam_id, true).unwrap();
    assert_eq!(
        gated_bob(&modules, &registry, state, true).to_cols_array(),
        walking_matrix
    );
    assert_eq!(
        gated_bob(&modules, &registry, state, false),
        glam::Mat4::IDENTITY
    );
}

#[test]
fn no_hurt_cam_cancels_death_roll_without_changing_death_fov() {
    let (mut registry, modules) = fixture();
    let state = BobState {
        living: Some(HurtBob {
            hurt_time: -1.0,
            hurt_duration: 10,
            hurt_dir: 0.0,
            death_time: Some(20.0),
        }),
        player: None,
    };
    assert_ne!(
        gated_bob(&modules, &registry, state, false),
        glam::Mat4::IDENTITY
    );
    let mut view = camera::Camera::default();
    view.tick(Some(1.62), 1.0);
    view.tick(Some(1.62), 1.0);
    let death_fov = view.calculate_fov(0.5, 70, 1.0, Some(20), camera::CameraFluid::None, false);
    let alive_fov = view.calculate_fov(0.5, 70, 1.0, None, camera::CameraFluid::None, false);
    assert_ne!(death_fov, alive_fov);
    registry.set_enabled(modules.no_hurt_cam_id, true).unwrap();
    assert_eq!(
        gated_bob(&modules, &registry, state, false),
        glam::Mat4::IDENTITY
    );
    assert_eq!(
        view.calculate_fov(0.5, 70, 1.0, Some(20), camera::CameraFluid::None, false),
        death_fov
    );
}

#[test]
fn both_cancellations_preserve_nonzero_native_shake_and_independent_nausea() {
    let (mut registry, modules) = fixture();
    registry.set_enabled(modules.no_bob_id, true).unwrap();
    registry.set_enabled(modules.no_hurt_cam_id, true).unwrap();
    let state = camera_state();
    assert_ne!(
        camera::bob_matrix(&state, 1.0, true).m,
        glam::Mat4::IDENTITY.to_cols_array()
    );
    let bob = gated_bob(&modules, &registry, state, true);
    assert_eq!(bob, glam::Mat4::IDENTITY);
    let shake = glam::Mat4::from_rotation_z(0.2)
        * glam::Mat4::from_translation(glam::Vec3::new(0.03, -0.02, 0.01));
    let nausea = camera::spinning_effect_matrix(1.0, 37.0);
    assert_ne!(shake, glam::Mat4::IDENTITY);
    assert_ne!(nausea, glam::Mat4::IDENTITY);
    assert_eq!(bob * shake, shake);
    assert_eq!((bob * shake) * nausea, shake * nausea);
    assert_ne!((bob * shake) * nausea, glam::Mat4::IDENTITY);
    assert_eq!(state, camera_state());
}
