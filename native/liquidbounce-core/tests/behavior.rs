// SPDX-License-Identifier: GPL-3.0-or-later

use liquidbounce_core::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn mutable_events_keep_priority_and_registration_order_after_cancellation() {
    let dispatcher = EventDispatcher::new();
    let listener = dispatcher.listener();
    for (priority, label) in [(0, "first"), (10, "high"), (0, "second"), (-2, "low")] {
        dispatcher.on(
            listener,
            priority,
            || true,
            move |event: &mut Event<Vec<&str>>| {
                assert!(!event.is_completed());
                event.payload.push(label);
                if label == "high" {
                    event.cancel().unwrap();
                }
            },
        );
    }
    let mut event = Event::cancellable(Vec::<&'static str>::new());
    let report = dispatcher.dispatch(&mut event);
    assert_eq!(event.payload, ["high", "first", "second", "low"]);
    assert!(event.is_cancelled());
    assert!(event.is_completed());
    assert_eq!(report.invoked, 4);
    assert_eq!(event.cancel(), Err(CancelError::Completed));
}

#[test]
fn dispatch_snapshot_survives_unregister_and_new_handlers_wait_until_next_call() {
    let dispatcher = Arc::new(EventDispatcher::new());
    let listener = dispatcher.listener();
    let removed = Arc::new(Mutex::new(None));
    let once = Arc::new(AtomicBool::new(false));
    let inner = dispatcher.clone();
    let target = removed.clone();
    dispatcher.on(
        listener,
        10,
        || true,
        move |event: &mut Event<Vec<&str>>| {
            event.payload.push("control");
            if !once.swap(true, Ordering::Relaxed) {
                inner.unregister(target.lock().unwrap().unwrap());
                inner.on(
                    listener,
                    0,
                    || true,
                    |event: &mut Event<Vec<&str>>| event.payload.push("new"),
                );
            }
        },
    );
    *removed.lock().unwrap() = Some(dispatcher.on(
        listener,
        0,
        || true,
        |event: &mut Event<Vec<&str>>| event.payload.push("old"),
    ));
    let mut first = Event::new(Vec::<&'static str>::new());
    dispatcher.dispatch(&mut first);
    assert_eq!(first.payload, ["control", "old"]);
    let mut second = Event::new(Vec::<&'static str>::new());
    dispatcher.dispatch(&mut second);
    assert_eq!(second.payload, ["control", "new"]);
}

#[test]
fn handlers_recheck_running_and_recover_from_ordinary_panics() {
    let dispatcher = EventDispatcher::new();
    let listener = dispatcher.listener();
    let running = Arc::new(AtomicBool::new(true));
    let change = running.clone();
    dispatcher.on(
        listener,
        10,
        || true,
        move |_: &mut Event<usize>| {
            change.store(false, Ordering::Relaxed);
            panic!("ordinary handler failure");
        },
    );
    dispatcher.on(
        listener,
        0,
        move || running.load(Ordering::Relaxed),
        |event: &mut Event<usize>| event.payload += 10,
    );
    dispatcher.on(
        listener,
        -1,
        || true,
        |event: &mut Event<usize>| event.payload += 1,
    );
    let mut event = Event::new(0_usize);
    let report = dispatcher.dispatch(&mut event);
    assert_eq!(event.payload, 1);
    assert_eq!(report.panicked, 1);
    assert_eq!(report.skipped, 1);
    assert!(event.is_completed());
}

#[test]
fn unknown_and_suppressed_events_stay_untouched_but_known_empty_events_complete() {
    let dispatcher = EventDispatcher::new();
    let mut event = Event::cancellable(42_i32);
    assert!(!dispatcher.dispatch(&mut event).known_event);
    assert!(!event.is_completed());
    dispatcher.register_event::<i32>();
    dispatcher.set_suppressed(true);
    assert!(dispatcher.dispatch(&mut event).suppressed);
    assert!(!event.is_completed());
    dispatcher.set_suppressed(false);
    assert!(dispatcher.dispatch(&mut event).known_event);
    assert!(event.is_completed());
    assert_eq!(event.payload, 42);
}

fn in_world() -> WorldState {
    WorldState {
        world_present: true,
        player_present: true,
    }
}

fn module_with_bind(registry: &mut ModuleRegistry, action: BindAction, key: InputKey) -> ModuleId {
    let mut definition = ModuleDefinition::new("TestModule");
    definition.bind = Some(InputBind::new(key, action));
    registry.register(definition).unwrap()
}

fn keyboard(registry: &mut ModuleRegistry, action: KeyAction) {
    registry.handle_keyboard(
        KeyboardInput::new(54, action, Modifiers::NONE),
        InputContext::default(),
    );
}

#[test]
fn configured_modules_wait_for_real_player_and_world_and_start_once() {
    let mut registry = ModuleRegistry::new();
    assert_eq!(registry.modules().count(), 0);
    let mut definition = ModuleDefinition::new("Configured");
    definition.enabled = true;
    let id = registry.register(definition).unwrap();
    assert!(!registry.running(id));
    assert!(
        registry
            .update_world(WorldState {
                world_present: true,
                player_present: false
            })
            .is_empty()
    );
    assert!(
        registry
            .update_world(WorldState {
                world_present: false,
                player_present: true
            })
            .is_empty()
    );
    assert_eq!(
        registry.update_world(in_world()),
        [
            ModuleEffect::OnEnabled(id),
            ModuleEffect::Activation(id),
            ModuleEffect::Toggle {
                module: id,
                enabled: true
            }
        ]
    );
    assert!(registry.running(id));
    assert!(registry.module(id).unwrap().called_since_startup());
    registry.update_world(WorldState::default());
    assert!(registry.update_world(in_world()).is_empty());
    registry.set_destructed(true);
    assert!(!registry.running(id));
}

#[test]
fn disconnect_only_disables_flagged_modules_and_preserves_lifecycle_order() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let keep = registry.register(ModuleDefinition::new("Keep")).unwrap();
    let mut definition = ModuleDefinition::new("Quit");
    definition.disable_on_quit = true;
    let quit = registry.register(definition).unwrap();
    registry.set_enabled(keep, true).unwrap();
    registry.set_enabled(quit, true).unwrap();
    assert_eq!(
        registry.disconnect(),
        [
            ModuleEffect::CancelSequences(quit),
            ModuleEffect::OnDisabled(quit),
            ModuleEffect::Activation(quit),
            ModuleEffect::Toggle {
                module: quit,
                enabled: false
            },
            ModuleEffect::ValueChanged {
                module: quit,
                enabled: false
            }
        ]
    );
    assert!(registry.module(keep).unwrap().enabled());
    assert!(!registry.module(quit).unwrap().enabled());
    let passive = registry
        .register(ModuleDefinition::passive("Passive"))
        .unwrap();
    assert!(registry.running(passive));
    let effects = registry.set_enabled(passive, true).unwrap();
    assert!(!registry.module(passive).unwrap().enabled());
    assert_eq!(
        effects,
        [
            ModuleEffect::OnEnabled(passive),
            ModuleEffect::Activation(passive),
            ModuleEffect::ValueChanged {
                module: passive,
                enabled: false
            }
        ]
    );
}

#[test]
fn toggle_ignores_repeat_and_release_and_hold_releases_inside_a_screen() {
    let mut registry = ModuleRegistry::new();
    let id = module_with_bind(&mut registry, BindAction::Toggle, InputKey::Keyboard(54));
    keyboard(&mut registry, KeyAction::Press);
    keyboard(&mut registry, KeyAction::Repeat);
    keyboard(&mut registry, KeyAction::Release);
    assert!(registry.module(id).unwrap().enabled());
    registry.handle_keyboard(
        KeyboardInput::new(54, KeyAction::Press, Modifiers::NONE),
        InputContext {
            screen_open: true,
            debug_modifier_down: false,
        },
    );
    registry.handle_keyboard(
        KeyboardInput::new(54, KeyAction::Press, Modifiers::NONE),
        InputContext {
            screen_open: false,
            debug_modifier_down: true,
        },
    );
    assert!(registry.module(id).unwrap().enabled());
    registry
        .set_bind(
            id,
            Some(InputBind::new(InputKey::Keyboard(54), BindAction::Hold)),
        )
        .unwrap();
    registry.handle_keyboard(
        KeyboardInput::new(54, KeyAction::Release, Modifiers::NONE),
        InputContext {
            screen_open: true,
            debug_modifier_down: true,
        },
    );
    assert!(!registry.module(id).unwrap().enabled());
    registry
        .set_bind(
            id,
            Some(InputBind::new(InputKey::Keyboard(0), BindAction::Toggle)),
        )
        .unwrap();
    registry.handle_keyboard(
        KeyboardInput::new(0, KeyAction::Press, Modifiers::NONE),
        InputContext::default(),
    );
    assert!(!registry.module(id).unwrap().enabled());
}

#[test]
fn keyboard_smart_toggles_short_presses_and_holds_on_repeat_even_in_a_screen() {
    let mut registry = ModuleRegistry::new();
    let id = module_with_bind(&mut registry, BindAction::Smart, InputKey::Keyboard(54));
    keyboard(&mut registry, KeyAction::Press);
    keyboard(&mut registry, KeyAction::Release);
    assert!(registry.module(id).unwrap().enabled());
    keyboard(&mut registry, KeyAction::Press);
    keyboard(&mut registry, KeyAction::Release);
    assert!(!registry.module(id).unwrap().enabled());
    keyboard(&mut registry, KeyAction::Press);
    registry.handle_keyboard(
        KeyboardInput::new(54, KeyAction::Repeat, Modifiers::NONE),
        InputContext {
            screen_open: true,
            debug_modifier_down: false,
        },
    );
    keyboard(&mut registry, KeyAction::Release);
    assert!(!registry.module(id).unwrap().enabled());
    keyboard(&mut registry, KeyAction::Release); // no remembered press
    assert!(!registry.module(id).unwrap().enabled());
}

#[test]
fn mouse_smart_uses_the_exact_200_ms_threshold_for_both_initial_states() {
    let mut registry = ModuleRegistry::new();
    let id = module_with_bind(&mut registry, BindAction::Smart, InputKey::Mouse(1));
    for (initial, duration, expected) in [
        (false, 199, true),
        (false, 200, false),
        (true, 199, false),
        (true, 200, false),
    ] {
        registry.set_enabled(id, initial).unwrap();
        registry.handle_mouse(
            MouseInput::new(1, KeyAction::Press, Modifiers::NONE),
            InputContext::default(),
            1000,
        );
        assert!(registry.module(id).unwrap().enabled());
        registry.handle_mouse(
            MouseInput::new(1, KeyAction::Release, Modifiers::NONE),
            InputContext {
                screen_open: true,
                debug_modifier_down: true,
            },
            1000 + duration,
        );
        assert_eq!(registry.module(id).unwrap().enabled(), expected);
    }
}

#[test]
fn required_modifier_release_waits_for_both_sides_and_external_binds_are_untouched() {
    let mut registry = ModuleRegistry::new();
    let id = module_with_bind(&mut registry, BindAction::Hold, InputKey::Keyboard(54));
    registry
        .set_bind(
            id,
            Some(
                InputBind::new(InputKey::Keyboard(54), BindAction::Hold)
                    .with_modifiers(Modifiers::new(&[Modifier::Shift])),
            ),
        )
        .unwrap();
    keyboard(&mut registry, KeyAction::Press);
    assert!(!registry.module(id).unwrap().enabled());
    registry.handle_keyboard(
        KeyboardInput::new(
            54,
            KeyAction::Press,
            Modifiers::new(&[Modifier::Shift, Modifier::Alt]),
        ),
        InputContext::default(),
    );
    let mut release =
        KeyboardInput::new(225, KeyAction::Release, Modifiers::new(&[Modifier::Shift]));
    release.released_modifier = Some(Modifier::Shift);
    registry.handle_keyboard(release, InputContext::default());
    assert!(registry.module(id).unwrap().enabled());
    release.modifiers = Modifiers::NONE;
    registry.handle_keyboard(release, InputContext::default());
    assert!(!registry.module(id).unwrap().enabled());
    let mut external = ModuleDefinition::new("External");
    external.external_bind = true;
    external.bind = Some(InputBind::new(InputKey::Keyboard(54), BindAction::Toggle));
    let external = registry.register(external).unwrap();
    keyboard(&mut registry, KeyAction::Press);
    assert!(!registry.module(external).unwrap().enabled());
}
