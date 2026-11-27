// SPDX-License-Identifier: GPL-3.0-or-later

use liquidbounce_core::{
    BindAction, CallbackError, InputBind, InputContext, InputKey, KeyAction, KeyboardInput,
    Modifiers, ModuleCallback, ModuleCallbacks, ModuleDefinition, ModuleId, ModuleRegistry,
    ModuleSyncError, MouseInput, WorldState,
};
use std::cell::{Cell, RefCell};

fn in_world() -> WorldState {
    WorldState {
        world_present: true,
        player_present: true,
    }
}

fn quiet(_registry: &mut ModuleRegistry, _callback: ModuleCallback) -> Result<(), String> {
    Ok(())
}

#[test]
fn lifecycle_callbacks_observe_old_state_and_postcommit_callbacks_observe_new_state() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let id = registry.register(ModuleDefinition::new("Test")).unwrap();
    let trace = RefCell::new(Vec::new());
    let executor = |registry: &mut ModuleRegistry, callback: ModuleCallback| {
        let state = registry.module(id).unwrap();
        assert!(state.called_since_startup());
        trace.borrow_mut().push((callback, state.enabled()));
        Ok(())
    };
    let enabled = registry.set_enabled_sync(id, true, &executor).unwrap();
    assert!(enabled.committed);
    assert_eq!(
        *trace.borrow(),
        [
            (ModuleCallback::OnEnabled(id), false),
            (
                ModuleCallback::UpdateChildren {
                    module: id,
                    enabled: true
                },
                false
            ),
            (ModuleCallback::Activation(id), false),
            (
                ModuleCallback::Notification {
                    module: id,
                    enabled: true
                },
                false
            ),
            (
                ModuleCallback::Toggle {
                    module: id,
                    enabled: true
                },
                false
            ),
            (
                ModuleCallback::ValueChanged {
                    module: id,
                    enabled: true
                },
                true
            ),
            (ModuleCallback::EnabledEffect(id), true),
            (
                ModuleCallback::PublishEnabled {
                    module: id,
                    enabled: true
                },
                true
            ),
        ]
    );
    trace.borrow_mut().clear();
    registry.set_enabled_sync(id, false, &executor).unwrap();
    assert_eq!(
        *trace.borrow(),
        [
            (ModuleCallback::CancelSequences(id), true),
            (ModuleCallback::OnDisabled(id), true),
            (
                ModuleCallback::UpdateChildren {
                    module: id,
                    enabled: false
                },
                true
            ),
            (ModuleCallback::Activation(id), true),
            (
                ModuleCallback::Notification {
                    module: id,
                    enabled: false
                },
                true
            ),
            (
                ModuleCallback::Toggle {
                    module: id,
                    enabled: false
                },
                true
            ),
            (
                ModuleCallback::ValueChanged {
                    module: id,
                    enabled: false
                },
                false
            ),
            (
                ModuleCallback::PublishEnabled {
                    module: id,
                    enabled: false
                },
                false
            ),
        ]
    );
}

struct ReentrantExecutor {
    id: ModuleId,
    depth: Cell<u32>,
    trace: RefCell<Vec<(ModuleCallback, bool)>>,
}

impl ModuleCallbacks for ReentrantExecutor {
    fn invoke(
        &self,
        registry: &mut ModuleRegistry,
        callback: ModuleCallback,
    ) -> Result<(), String> {
        self.trace
            .borrow_mut()
            .push((callback, registry.module(self.id).unwrap().enabled()));
        if callback == ModuleCallback::OnEnabled(self.id) && self.depth.get() == 0 {
            self.depth.set(1);
            registry.set_enabled_sync(self.id, true, self).unwrap();
            self.depth.set(0);
        }
        Ok(())
    }
}

#[test]
fn same_executor_reentry_has_no_lock_or_borrow_and_outer_set_does_not_recheck_equality() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let id = registry
        .register(ModuleDefinition::new("Reentrant"))
        .unwrap();
    let executor = ReentrantExecutor {
        id,
        depth: Cell::new(0),
        trace: RefCell::new(Vec::new()),
    };
    registry.set_enabled_sync(id, true, &executor).unwrap();
    let trace = executor.trace.borrow();
    assert_eq!(
        &trace[..2],
        &[
            (ModuleCallback::OnEnabled(id), false),
            (ModuleCallback::OnEnabled(id), false)
        ]
    );
    let activations: Vec<_> = trace
        .iter()
        .filter(|(callback, _)| *callback == ModuleCallback::Activation(id))
        .map(|(_, enabled)| *enabled)
        .collect();
    assert_eq!(activations, [false, true]);
    assert_eq!(
        trace
            .iter()
            .filter(|(callback, _)| matches!(callback, ModuleCallback::ValueChanged { .. }))
            .count(),
        2
    );
    assert!(registry.module(id).unwrap().enabled());
}

#[test]
fn opposite_request_inside_enable_is_a_noop_against_the_still_old_value() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let id = registry
        .register(ModuleDefinition::new("OldValue"))
        .unwrap();
    let executor = |registry: &mut ModuleRegistry, callback: ModuleCallback| {
        if callback == ModuleCallback::OnEnabled(id) {
            let nested = registry.set_enabled_sync(id, false, &quiet).unwrap();
            assert!(!nested.committed);
        }
        Ok(())
    };
    registry.set_enabled_sync(id, true, &executor).unwrap();
    assert!(registry.module(id).unwrap().enabled());
}

#[test]
fn precommit_failure_rejects_outer_commit_but_retains_nested_writes_and_startup_marker() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let outer = registry.register(ModuleDefinition::new("Outer")).unwrap();
    let nested = registry.register(ModuleDefinition::new("Nested")).unwrap();
    let executor = |registry: &mut ModuleRegistry, callback: ModuleCallback| {
        if callback == ModuleCallback::OnEnabled(outer) {
            registry.set_enabled_sync(nested, true, &quiet).unwrap();
            return Err("onEnabled failed".to_owned());
        }
        Ok(())
    };
    let report = registry.set_enabled_sync(outer, true, &executor).unwrap();
    assert!(!report.committed);
    assert_eq!(
        report.failures[0].callback,
        ModuleCallback::OnEnabled(outer)
    );
    assert!(!registry.module(outer).unwrap().enabled());
    assert!(registry.module(outer).unwrap().called_since_startup());
    assert!(registry.module(nested).unwrap().enabled());
    let panicking = |_registry: &mut ModuleRegistry, callback: ModuleCallback| {
        if callback == ModuleCallback::OnEnabled(outer) {
            panic!("source callback exception");
        }
        Ok(())
    };
    let panic_report = registry.set_enabled_sync(outer, true, &panicking).unwrap();
    assert_eq!(panic_report.failures[0].error, CallbackError::Panicked);
    assert!(!registry.module(outer).unwrap().enabled());
}

#[test]
fn cancellation_failure_is_swallowed_but_disabled_callback_failure_rejects_commit() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let id = registry.register(ModuleDefinition::new("Cancel")).unwrap();
    registry.set_enabled_sync(id, true, &quiet).unwrap();
    let disabled_seen = Cell::new(false);
    let executor = |registry: &mut ModuleRegistry, callback: ModuleCallback| {
        if callback == ModuleCallback::CancelSequences(id) {
            return Err("cancel failed".to_owned());
        }
        if callback == ModuleCallback::OnDisabled(id) {
            assert!(registry.module(id).unwrap().enabled());
            disabled_seen.set(true);
        }
        Ok(())
    };
    let report = registry.set_enabled_sync(id, false, &executor).unwrap();
    assert!(report.committed);
    assert!(disabled_seen.get());
    assert_eq!(
        report.failures[0].callback,
        ModuleCallback::CancelSequences(id)
    );
    registry.set_enabled_sync(id, true, &quiet).unwrap();
    let reject = |_registry: &mut ModuleRegistry, callback: ModuleCallback| {
        if callback == ModuleCallback::OnDisabled(id) {
            return Err("disable failed".to_owned());
        }
        Ok(())
    };
    let rejected = registry.set_enabled_sync(id, false, &reject).unwrap();
    assert!(!rejected.committed);
    assert!(registry.module(id).unwrap().enabled());
}

struct PostcommitReentry {
    id: ModuleId,
    trace: RefCell<Vec<(ModuleCallback, bool)>>,
}

impl ModuleCallbacks for PostcommitReentry {
    fn invoke(
        &self,
        registry: &mut ModuleRegistry,
        callback: ModuleCallback,
    ) -> Result<(), String> {
        self.trace
            .borrow_mut()
            .push((callback, registry.module(self.id).unwrap().enabled()));
        if callback
            == (ModuleCallback::ValueChanged {
                module: self.id,
                enabled: true,
            })
        {
            registry.set_enabled_sync(self.id, false, self).unwrap();
        }
        Ok(())
    }
}

#[test]
fn postcommit_reentry_keeps_outer_changed_listener_argument_even_when_live_value_changes() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let id = registry
        .register(ModuleDefinition::new("Postcommit"))
        .unwrap();
    let executor = PostcommitReentry {
        id,
        trace: RefCell::new(Vec::new()),
    };
    registry.set_enabled_sync(id, true, &executor).unwrap();
    assert!(!registry.module(id).unwrap().enabled());
    let trace = executor.trace.borrow();
    assert_eq!(
        trace[trace.len() - 2..],
        [
            (ModuleCallback::EnabledEffect(id), false),
            (
                ModuleCallback::PublishEnabled {
                    module: id,
                    enabled: true
                },
                false
            ),
        ]
    );
}

#[test]
fn postcommit_failure_propagates_without_rollback_and_stops_remaining_input_modules() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let mut first = ModuleDefinition::new("AFirst");
    first.bind = Some(InputBind::new(InputKey::Keyboard(54), BindAction::Toggle));
    let first = registry.register(first).unwrap();
    let mut second = ModuleDefinition::new("BSecond");
    second.bind = Some(InputBind::new(InputKey::Keyboard(54), BindAction::Toggle));
    let second = registry.register(second).unwrap();
    let executor = |_registry: &mut ModuleRegistry, callback: ModuleCallback| {
        if callback
            == (ModuleCallback::ValueChanged {
                module: first,
                enabled: true,
            })
        {
            return Err("postcommit failure".to_owned());
        }
        Ok(())
    };
    let error = registry
        .handle_keyboard_sync(
            KeyboardInput::new(54, KeyAction::Press, Modifiers::NONE),
            InputContext::default(),
            &executor,
        )
        .unwrap_err();
    assert!(matches!(error, ModuleSyncError::AfterCommit(_)));
    assert!(registry.module(first).unwrap().enabled());
    assert!(!registry.module(second).unwrap().enabled());
}

#[test]
fn input_reads_later_modules_live_after_prior_callback_changes_binding_and_enabled_state() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let mut ids = Vec::new();
    for name in ["AFirst", "BSecond", "CThird"] {
        let mut definition = ModuleDefinition::new(name);
        definition.bind = Some(InputBind::new(InputKey::Keyboard(54), BindAction::Toggle));
        ids.push(registry.register(definition).unwrap());
    }
    let executor = |registry: &mut ModuleRegistry, callback: ModuleCallback| {
        if callback == ModuleCallback::OnEnabled(ids[0]) {
            registry.set_enabled_sync(ids[1], true, &quiet).unwrap();
            registry
                .set_bind(
                    ids[2],
                    Some(InputBind::new(InputKey::Keyboard(55), BindAction::Toggle)),
                )
                .unwrap();
        }
        Ok(())
    };
    registry
        .handle_keyboard_sync(
            KeyboardInput::new(54, KeyAction::Press, Modifiers::NONE),
            InputContext::default(),
            &executor,
        )
        .unwrap();
    assert!(registry.module(ids[0]).unwrap().enabled());
    // Already enabled by A's callback, so the same TOGGLE press disables B.
    assert!(!registry.module(ids[1]).unwrap().enabled());
    assert!(!registry.module(ids[2]).unwrap().enabled());
}

#[test]
fn startup_runs_direct_lifecycle_under_loading_and_does_not_commit_passive_return_value() {
    let mut registry = ModuleRegistry::new();
    let mut passive = ModuleDefinition::passive("Passive");
    passive.enabled = true;
    let id = registry.register(passive).unwrap();
    let trace = RefCell::new(Vec::new());
    let executor = |registry: &mut ModuleRegistry, callback: ModuleCallback| {
        assert!(registry.loading());
        assert!(registry.module(id).unwrap().enabled());
        trace.borrow_mut().push(callback);
        Ok(())
    };
    let reports = registry.update_world_sync(in_world(), &executor);
    assert_eq!(reports.len(), 1);
    assert!(reports[0].startup);
    assert!(!reports[0].committed);
    assert_eq!(
        *trace.borrow(),
        [
            ModuleCallback::OnEnabled(id),
            ModuleCallback::UpdateChildren {
                module: id,
                enabled: true
            },
            ModuleCallback::Activation(id)
        ]
    );
    assert!(registry.module(id).unwrap().enabled());
    assert!(!registry.loading());
    assert!(registry.update_world_sync(in_world(), &executor).is_empty());
}

#[test]
fn loading_suppresses_notification_and_passive_value_set_still_publishes_false() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    registry.set_loading(true);
    let id = registry.register(ModuleDefinition::new("Loading")).unwrap();
    let trace = RefCell::new(Vec::new());
    let executor = |_registry: &mut ModuleRegistry, callback: ModuleCallback| {
        trace.borrow_mut().push(callback);
        Ok(())
    };
    registry.set_enabled_sync(id, true, &executor).unwrap();
    assert!(
        !trace
            .borrow()
            .iter()
            .any(|callback| matches!(callback, ModuleCallback::Notification { .. }))
    );
    let passive = registry
        .register(ModuleDefinition::passive("Passive"))
        .unwrap();
    trace.borrow_mut().clear();
    registry.set_enabled_sync(passive, true, &executor).unwrap();
    assert!(!registry.module(passive).unwrap().enabled());
    assert_eq!(
        *trace.borrow(),
        [
            ModuleCallback::OnEnabled(passive),
            ModuleCallback::UpdateChildren {
                module: passive,
                enabled: true
            },
            ModuleCallback::Activation(passive),
            ModuleCallback::ValueChanged {
                module: passive,
                enabled: false
            },
            ModuleCallback::PublishEnabled {
                module: passive,
                enabled: false
            },
        ]
    );
}

#[test]
fn smart_mouse_sync_preserves_boundary_and_release_callback_sees_enabled() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let mut definition = ModuleDefinition::new("Mouse");
    definition.bind = Some(InputBind::new(InputKey::Mouse(1), BindAction::Smart));
    let id = registry.register(definition).unwrap();
    let disabled_seen = Cell::new(false);
    let executor = |registry: &mut ModuleRegistry, callback: ModuleCallback| {
        if callback == ModuleCallback::OnDisabled(id) {
            assert!(registry.module(id).unwrap().enabled());
            disabled_seen.set(true);
        }
        Ok(())
    };
    registry
        .handle_mouse_sync(
            MouseInput::new(1, KeyAction::Press, Modifiers::NONE),
            InputContext::default(),
            1000,
            &executor,
        )
        .unwrap();
    registry
        .handle_mouse_sync(
            MouseInput::new(1, KeyAction::Release, Modifiers::NONE),
            InputContext {
                screen_open: true,
                debug_modifier_down: false,
            },
            1199,
            &executor,
        )
        .unwrap();
    assert!(registry.module(id).unwrap().enabled());
    assert!(!disabled_seen.get());
    registry
        .handle_mouse_sync(
            MouseInput::new(1, KeyAction::Press, Modifiers::NONE),
            InputContext::default(),
            2000,
            &executor,
        )
        .unwrap();
    registry
        .handle_mouse_sync(
            MouseInput::new(1, KeyAction::Release, Modifiers::NONE),
            InputContext {
                screen_open: true,
                debug_modifier_down: false,
            },
            2200,
            &executor,
        )
        .unwrap();
    assert!(!registry.module(id).unwrap().enabled());
    assert!(disabled_seen.get());
}

#[test]
fn disconnect_catches_each_postcommit_failure_and_continues_to_next_module() {
    let mut registry = ModuleRegistry::new();
    registry.update_world(in_world());
    let mut ids = Vec::new();
    for name in ["AFirst", "BSecond"] {
        let mut definition = ModuleDefinition::new(name);
        definition.disable_on_quit = true;
        let id = registry.register(definition).unwrap();
        registry.set_enabled_sync(id, true, &quiet).unwrap();
        ids.push(id);
    }
    let executor = |_registry: &mut ModuleRegistry, callback: ModuleCallback| {
        if callback
            == (ModuleCallback::ValueChanged {
                module: ids[0],
                enabled: false,
            })
        {
            return Err("disconnect callback failed".to_owned());
        }
        Ok(())
    };
    let reports = registry.disconnect_sync(&executor);
    assert!(matches!(reports[0], Err(ModuleSyncError::AfterCommit(_))));
    assert!(reports[1].as_ref().unwrap().committed);
    assert!(
        ids.iter()
            .all(|id| !registry.module(*id).unwrap().enabled())
    );
}
