// SPDX-License-Identifier: GPL-3.0-or-later
// Based on LiquidBounce ClientModule.kt, ModuleManager.kt, Value.kt and Toggleable.kt.

use crate::input::{BindAction, InputBind, InputContext, KeyAction, KeyboardInput, MouseInput};
use std::collections::HashMap;

mod synchronous;
pub use synchronous::{
    CallbackError, ModuleCallback, ModuleCallbackFailure, ModuleCallbacks, ModuleSyncError,
    ModuleSyncReport,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleId(u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldState {
    pub world_present: bool,
    pub player_present: bool,
}

impl WorldState {
    pub fn in_game(self) -> bool {
        self.world_present && self.player_present
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleDefinition {
    pub name: String,
    pub enabled: bool,
    pub bind: Option<InputBind>,
    pub not_activatable: bool,
    pub disable_activation: bool,
    pub disable_on_quit: bool,
    pub external_bind: bool,
}

impl ModuleDefinition {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: false,
            bind: None,
            not_activatable: false,
            disable_activation: false,
            disable_on_quit: false,
            external_bind: false,
        }
    }
    pub fn passive(name: impl Into<String>) -> Self {
        Self {
            not_activatable: true,
            disable_activation: true,
            ..Self::new(name)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleState {
    id: ModuleId,
    definition: ModuleDefinition,
    called_since_startup: bool,
}

impl ModuleState {
    pub fn id(&self) -> ModuleId {
        self.id
    }
    pub fn definition(&self) -> &ModuleDefinition {
        &self.definition
    }
    pub fn enabled(&self) -> bool {
        self.definition.enabled
    }
    pub fn called_since_startup(&self) -> bool {
        self.called_since_startup
    }
}

/// Ordered lifecycle work for the integrating runtime. These effects are not
/// gameplay implementations. In particular, no effect can imply a network action
/// or a module has been ported. The bridge executes actual module callbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleEffect {
    OnEnabled(ModuleId),
    CancelSequences(ModuleId),
    OnDisabled(ModuleId),
    Activation(ModuleId),
    Toggle { module: ModuleId, enabled: bool },
    ValueChanged { module: ModuleId, enabled: bool },
    EnabledEffect(ModuleId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleError {
    /// The verified built-in names are ASCII. Unicode add-on name collation has
    /// not been ported from Java's CASE_INSENSITIVE_ORDER.
    InvalidName,
    DuplicateName(String),
    UnknownModule(ModuleId),
}

#[derive(Clone, Copy)]
enum SmartKeyboard {
    PendingEnabled,
    PendingDisabled,
    Holding,
}

#[derive(Clone, Copy)]
struct SmartMouse {
    was_enabled: bool,
    pressed_ms: u64,
}

/// Empty by default. Register only modules whose real implementation exists in
/// the bridge. It owns no world, packets, player or placeholder module list.
#[derive(Default)]
pub struct ModuleRegistry {
    modules: Vec<ModuleState>,
    next_id: u64,
    world: WorldState,
    destructed: bool,
    loading: bool,
    smart_keyboard: HashMap<ModuleId, SmartKeyboard>,
    smart_mouse: HashMap<ModuleId, SmartMouse>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn world(&self) -> WorldState {
        self.world
    }
    pub fn modules(&self) -> impl Iterator<Item = &ModuleState> {
        self.modules.iter()
    }
    pub fn module(&self, id: ModuleId) -> Option<&ModuleState> {
        self.modules.iter().find(|module| module.id == id)
    }
    pub fn set_destructed(&mut self, destructed: bool) {
        self.destructed = destructed;
    }
    /// Modules with an explicit pre-world running override still honor SelfDestruct.
    pub fn enabled_without_world(&self, id: ModuleId) -> bool {
        !self.destructed && self.module(id).is_some_and(|module| module.enabled())
    }
    pub fn running(&self, id: ModuleId) -> bool {
        !self.destructed
            && self.world.in_game()
            && self
                .module(id)
                .is_some_and(|module| module.enabled() || module.definition.not_activatable)
    }

    pub fn register(&mut self, definition: ModuleDefinition) -> Result<ModuleId, ModuleError> {
        if definition.name.is_empty() || !definition.name.is_ascii() {
            return Err(ModuleError::InvalidName);
        }
        if self.modules.iter().any(|module| {
            module
                .definition
                .name
                .eq_ignore_ascii_case(&definition.name)
        }) {
            return Err(ModuleError::DuplicateName(definition.name));
        }
        let id = ModuleId(self.next_id);
        self.next_id += 1;
        let key = definition.name.to_ascii_lowercase();
        let index = self
            .modules
            .partition_point(|module| module.definition.name.to_ascii_lowercase() < key);
        self.modules.insert(
            index,
            ModuleState {
                id,
                definition,
                called_since_startup: false,
            },
        );
        Ok(id)
    }

    pub fn unregister(&mut self, id: ModuleId) -> bool {
        let Some(index) = self.modules.iter().position(|module| module.id == id) else {
            return false;
        };
        self.modules.remove(index);
        self.smart_keyboard.remove(&id);
        self.smart_mouse.remove(&id);
        true
    }

    pub fn set_bind(&mut self, id: ModuleId, bind: Option<InputBind>) -> Result<(), ModuleError> {
        let module = self
            .modules
            .iter_mut()
            .find(|module| module.id == id)
            .ok_or(ModuleError::UnknownModule(id))?;
        module.definition.bind = bind;
        Ok(())
    }

    /// Feed observed runtime state. Pending startup activation waits until both
    /// the world and player actually exist, and runs once per module lifetime.
    /// Nested value-group callbacks, persistence and coroutine execution remain
    /// responsibilities of the bridge, not simulated here.
    pub fn update_world(&mut self, world: WorldState) -> Vec<ModuleEffect> {
        self.world = world;
        if !world.in_game() || self.destructed {
            return Vec::new();
        }
        let mut effects = Vec::new();
        for module in &mut self.modules {
            if module.enabled() && !module.called_since_startup {
                module.called_since_startup = true;
                Self::toggle_work(module, true, &mut effects);
            }
        }
        effects
    }

    pub fn set_enabled(
        &mut self,
        id: ModuleId,
        enabled: bool,
    ) -> Result<Vec<ModuleEffect>, ModuleError> {
        let in_game = self.world.in_game();
        let module = self
            .modules
            .iter_mut()
            .find(|module| module.id == id)
            .ok_or(ModuleError::UnknownModule(id))?;
        if module.enabled() == enabled {
            return Ok(Vec::new());
        }
        let mut effects = Vec::new();
        let accepted = if in_game {
            module.called_since_startup = true;
            Self::toggle_work(module, enabled, &mut effects)
        } else {
            enabled
        };
        module.definition.enabled = accepted;
        effects.push(ModuleEffect::ValueChanged {
            module: id,
            enabled: accepted,
        });
        if accepted {
            effects.push(ModuleEffect::EnabledEffect(id));
        }
        Ok(effects)
    }

    fn toggle_work(module: &ModuleState, enabled: bool, effects: &mut Vec<ModuleEffect>) -> bool {
        if enabled {
            effects.push(ModuleEffect::OnEnabled(module.id));
        } else {
            effects.push(ModuleEffect::CancelSequences(module.id));
            effects.push(ModuleEffect::OnDisabled(module.id));
        }
        effects.push(ModuleEffect::Activation(module.id));
        if module.definition.disable_activation {
            return false;
        }
        effects.push(ModuleEffect::Toggle {
            module: module.id,
            enabled,
        });
        enabled
    }

    /// Apply DisconnectEvent's disableOnQuit while preserving input state,
    /// exactly as upstream does. Observed world absence is fed separately.
    pub fn disconnect(&mut self) -> Vec<ModuleEffect> {
        let ids: Vec<_> = self
            .modules
            .iter()
            .filter(|module| module.definition.disable_on_quit)
            .map(ModuleState::id)
            .collect();
        let mut effects = Vec::new();
        for id in ids {
            effects.extend(self.set_enabled(id, false).expect("registered module"));
        }
        effects
    }

    pub fn handle_keyboard(
        &mut self,
        input: KeyboardInput,
        context: InputContext,
    ) -> Vec<ModuleEffect> {
        if self.destructed {
            return Vec::new();
        }
        if input.action == KeyAction::Press && (context.screen_open || context.debug_modifier_down)
        {
            return Vec::new();
        }
        let bindings: Vec<_> = self
            .modules
            .iter()
            .filter(|module| !module.definition.external_bind)
            .filter_map(|module| {
                module
                    .definition
                    .bind
                    .map(|bind| (module.id, bind, module.enabled()))
            })
            .collect();
        let mut effects = Vec::new();
        for (id, bind, enabled) in bindings {
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
                effects.extend(self.set_enabled(id, requested).expect("registered module"));
            }
        }
        effects
    }

    pub fn handle_mouse(
        &mut self,
        input: MouseInput,
        context: InputContext,
        now_ms: u64,
    ) -> Vec<ModuleEffect> {
        if self.destructed || (input.action == KeyAction::Press && context.screen_open) {
            return Vec::new();
        }
        let bindings: Vec<_> = self
            .modules
            .iter()
            .filter(|module| !module.definition.external_bind)
            .filter_map(|module| {
                module
                    .definition
                    .bind
                    .map(|bind| (module.id, bind, module.enabled()))
            })
            .collect();
        let mut effects = Vec::new();
        for (id, bind, enabled) in bindings {
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
                effects.extend(self.set_enabled(id, requested).expect("registered module"));
            }
        }
        effects
    }
}
