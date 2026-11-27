// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce Value.kt, ClientModule.kt, ToggleableValueGroup.kt
// and ModuleManager.kt, Copyright (c) 2015-2026 CCBlueX.

use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Synchronous execution points, not delayed effect descriptions. Lifecycle,
/// children, activation, notification and toggle run before enabled is committed.
/// ValueChanged, EnabledEffect and PublishEnabled run after that commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleCallback {
    OnEnabled(ModuleId),
    CancelSequences(ModuleId),
    OnDisabled(ModuleId),
    UpdateChildren { module: ModuleId, enabled: bool },
    Activation(ModuleId),
    Notification { module: ModuleId, enabled: bool },
    Toggle { module: ModuleId, enabled: bool },
    ValueChanged { module: ModuleId, enabled: bool },
    EnabledEffect(ModuleId),
    PublishEnabled { module: ModuleId, enabled: bool },
}

/// Shared receiver permits genuine same-executor reentry. The mutable registry
/// is passed directly to the callback with no registry lock/borrow retained.
/// Use it for nested operations; do not reacquire an outer bridge mutex.
pub trait ModuleCallbacks {
    fn invoke(&self, registry: &mut ModuleRegistry, callback: ModuleCallback)
    -> Result<(), String>;
}

impl<F> ModuleCallbacks for F
where
    F: Fn(&mut ModuleRegistry, ModuleCallback) -> Result<(), String>,
{
    fn invoke(
        &self,
        registry: &mut ModuleRegistry,
        callback: ModuleCallback,
    ) -> Result<(), String> {
        self(registry, callback)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallbackError {
    Failed(String),
    Panicked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleCallbackFailure {
    pub callback: ModuleCallback,
    pub error: CallbackError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleSyncError {
    Registry(ModuleError),
    /// Kotlin's Value.onSuccess block is outside runCatching. These failures
    /// propagate to the caller and do not roll back the already committed value.
    AfterCommit(ModuleCallbackFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleSyncReport {
    pub module: ModuleId,
    /// This outer operation committed, independent of any nested operation.
    pub committed: bool,
    /// Startup calls onToggled directly and intentionally does not commit its
    /// return value or dispatch ValueChanged/EnabledEffect/PublishEnabled.
    pub startup: bool,
    /// Precommit exceptions are caught like Value.runCatching. Cancellation
    /// failures are recorded and swallowed even if the operation later commits.
    pub failures: Vec<ModuleCallbackFailure>,
}

impl ModuleSyncReport {
    fn new(module: ModuleId, startup: bool) -> Self {
        Self {
            module,
            committed: false,
            startup,
            failures: Vec::new(),
        }
    }
}

impl ModuleRegistry {
    pub fn loading(&self) -> bool {
        self.loading
    }

    /// Supply actual config-loading state. Return the previous value for a
    /// caller that mirrors AutoConfig.withLoading's save/restore behavior.
    pub fn set_loading(&mut self, loading: bool) -> bool {
        std::mem::replace(&mut self.loading, loading)
    }

    pub fn set_enabled_sync(
        &mut self,
        id: ModuleId,
        enabled: bool,
        callbacks: &(impl ModuleCallbacks + ?Sized),
    ) -> Result<ModuleSyncReport, ModuleSyncError> {
        let initial = self
            .module(id)
            .cloned()
            .ok_or(ModuleSyncError::Registry(ModuleError::UnknownModule(id)))?;
        let mut report = ModuleSyncReport::new(id, false);
        // Value.set only checks equality on entry, not again after reentry.
        if initial.enabled() == enabled {
            return Ok(report);
        }
        let accepted = if self.world.in_game() {
            self.mark_startup_called(id);
            match self.toggle_sync(&initial, enabled, callbacks, &mut report) {
                Ok(accepted) => accepted,
                Err(failure) => {
                    report.failures.push(failure);
                    return Ok(report);
                }
            }
        } else {
            enabled
        };
        let module = self
            .modules
            .iter_mut()
            .find(|module| module.id == id)
            .ok_or(ModuleSyncError::Registry(ModuleError::UnknownModule(id)))?;
        module.definition.enabled = accepted;
        report.committed = true;

        self.invoke_sync(
            callbacks,
            ModuleCallback::ValueChanged {
                module: id,
                enabled: accepted,
            },
        )
        .map_err(ModuleSyncError::AfterCommit)?;
        // onChanged receives the accepted argument even if a ValueChanged
        // listener already changed the live enabled value through reentry.
        if accepted {
            self.invoke_sync(callbacks, ModuleCallback::EnabledEffect(id))
                .map_err(ModuleSyncError::AfterCommit)?;
        }
        self.invoke_sync(
            callbacks,
            ModuleCallback::PublishEnabled {
                module: id,
                enabled: accepted,
            },
        )
        .map_err(ModuleSyncError::AfterCommit)?;
        Ok(report)
    }

    fn mark_startup_called(&mut self, id: ModuleId) {
        if let Some(module) = self.modules.iter_mut().find(|module| module.id == id) {
            module.called_since_startup = true;
        }
    }

    fn invoke_sync(
        &mut self,
        callbacks: &(impl ModuleCallbacks + ?Sized),
        callback: ModuleCallback,
    ) -> Result<(), ModuleCallbackFailure> {
        let error = match catch_unwind(AssertUnwindSafe(|| callbacks.invoke(self, callback))) {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(message)) => CallbackError::Failed(message),
            Err(_) => CallbackError::Panicked,
        };
        Err(ModuleCallbackFailure { callback, error })
    }

    fn toggle_sync(
        &mut self,
        initial: &ModuleState,
        enabled: bool,
        callbacks: &(impl ModuleCallbacks + ?Sized),
        report: &mut ModuleSyncReport,
    ) -> Result<bool, ModuleCallbackFailure> {
        let id = initial.id;
        if !enabled
            && let Err(failure) = self.invoke_sync(callbacks, ModuleCallback::CancelSequences(id))
        {
            report.failures.push(failure);
        }
        self.invoke_sync(
            callbacks,
            if enabled {
                ModuleCallback::OnEnabled(id)
            } else {
                ModuleCallback::OnDisabled(id)
            },
        )?;
        self.invoke_sync(
            callbacks,
            ModuleCallback::UpdateChildren {
                module: id,
                enabled,
            },
        )?;
        self.invoke_sync(callbacks, ModuleCallback::Activation(id))?;
        if initial.definition.disable_activation {
            return Ok(false);
        }
        if !self.loading {
            self.invoke_sync(
                callbacks,
                ModuleCallback::Notification {
                    module: id,
                    enabled,
                },
            )?;
        }
        self.invoke_sync(
            callbacks,
            ModuleCallback::Toggle {
                module: id,
                enabled,
            },
        )?;
        Ok(enabled)
    }

    pub fn update_world_sync(
        &mut self,
        world: WorldState,
        callbacks: &(impl ModuleCallbacks + ?Sized),
    ) -> Vec<ModuleSyncReport> {
        self.world = world;
        if !world.in_game() || self.destructed {
            return Vec::new();
        }
        let previous_loading = self.set_loading(true);
        let ids: Vec<_> = self.modules.iter().map(ModuleState::id).collect();
        let mut reports = Vec::new();
        for id in ids {
            // Examine live state for every module: preceding callbacks can
            // disable it or mark it called through their nested operations.
            let Some(initial) = self.module(id).cloned() else {
                continue;
            };
            if !initial.enabled() || initial.called_since_startup {
                continue;
            }
            self.mark_startup_called(id);
            let mut report = ModuleSyncReport::new(id, true);
            if let Err(failure) = self.toggle_sync(&initial, true, callbacks, &mut report) {
                report.failures.push(failure);
            }
            reports.push(report);
        }
        self.set_loading(previous_loading);
        reports
    }

    pub fn disconnect_sync(
        &mut self,
        callbacks: &(impl ModuleCallbacks + ?Sized),
    ) -> Vec<Result<ModuleSyncReport, ModuleSyncError>> {
        let ids: Vec<_> = self
            .modules
            .iter()
            .filter(|module| module.definition.disable_on_quit)
            .map(ModuleState::id)
            .collect();
        // Original disconnect catches a failure per module, then continues.
        ids.into_iter()
            .map(|id| self.set_enabled_sync(id, false, callbacks))
            .collect()
    }

    pub fn handle_keyboard_sync(
        &mut self,
        input: KeyboardInput,
        context: InputContext,
        callbacks: &(impl ModuleCallbacks + ?Sized),
    ) -> Result<Vec<ModuleSyncReport>, ModuleSyncError> {
        if self.destructed
            || (input.action == KeyAction::Press
                && (context.screen_open || context.debug_modifier_down))
        {
            return Ok(Vec::new());
        }
        let ids = self.own_bind_ids();
        let mut reports = Vec::new();
        for id in ids {
            let Some((bind, enabled)) = self.live_bind(id) else {
                continue;
            };
            let requested = match input.action {
                KeyAction::Press
                    if bind.key_matches(input.key) && input.modifiers.includes(bind.modifiers) =>
                {
                    Some(match bind.action {
                        BindAction::Toggle => !enabled,
                        BindAction::Hold => true,
                        BindAction::Smart => {
                            self.smart_keyboard.insert(
                                id,
                                if enabled {
                                    SmartKeyboard::PendingEnabled
                                } else {
                                    SmartKeyboard::PendingDisabled
                                },
                            );
                            true
                        }
                    })
                }
                KeyAction::Repeat
                    if bind.action == BindAction::Smart && bind.key_matches(input.key) =>
                {
                    if let Some(state) = self.smart_keyboard.get_mut(&id) {
                        *state = SmartKeyboard::Holding;
                    }
                    None
                }
                KeyAction::Release
                    if bind.key_matches(input.key)
                        || bind.modifier_released(input.released_modifier, input.modifiers) =>
                {
                    match bind.action {
                        BindAction::Toggle => None,
                        BindAction::Hold => Some(false),
                        BindAction::Smart => self
                            .smart_keyboard
                            .remove(&id)
                            .map(|state| matches!(state, SmartKeyboard::PendingDisabled)),
                    }
                }
                _ => None,
            };
            if let Some(requested) = requested {
                // Postcommit failure propagates out of this input handler, so
                // later modules do not consume this event, like Kotlin.
                reports.push(self.set_enabled_sync(id, requested, callbacks)?);
            }
        }
        Ok(reports)
    }

    pub fn handle_mouse_sync(
        &mut self,
        input: MouseInput,
        context: InputContext,
        now_ms: u64,
        callbacks: &(impl ModuleCallbacks + ?Sized),
    ) -> Result<Vec<ModuleSyncReport>, ModuleSyncError> {
        if self.destructed || (input.action == KeyAction::Press && context.screen_open) {
            return Ok(Vec::new());
        }
        let ids = self.own_bind_ids();
        let mut reports = Vec::new();
        for id in ids {
            let Some((bind, enabled)) = self.live_bind(id) else {
                continue;
            };
            let requested = match input.action {
                KeyAction::Press
                    if bind.mouse_matches(input.button)
                        && input.modifiers.includes(bind.modifiers) =>
                {
                    Some(match bind.action {
                        BindAction::Toggle => !enabled,
                        BindAction::Hold => true,
                        BindAction::Smart => {
                            self.smart_mouse.insert(
                                id,
                                SmartMouse {
                                    was_enabled: enabled,
                                    pressed_ms: now_ms,
                                },
                            );
                            true
                        }
                    })
                }
                KeyAction::Release if bind.mouse_matches(input.button) => match bind.action {
                    BindAction::Toggle => None,
                    BindAction::Hold => Some(false),
                    BindAction::Smart => self.smart_mouse.remove(&id).map(|state| {
                        now_ms.saturating_sub(state.pressed_ms) < 200 && !state.was_enabled
                    }),
                },
                _ => None,
            };
            if let Some(requested) = requested {
                reports.push(self.set_enabled_sync(id, requested, callbacks)?);
            }
        }
        Ok(reports)
    }

    fn own_bind_ids(&self) -> Vec<ModuleId> {
        self.modules
            .iter()
            .filter(|module| !module.definition.external_bind)
            .map(ModuleState::id)
            .collect()
    }

    fn live_bind(&self, id: ModuleId) -> Option<(InputBind, bool)> {
        self.module(id)
            .and_then(|module| module.definition.bind.map(|bind| (bind, module.enabled())))
    }
}
