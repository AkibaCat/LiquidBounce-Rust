// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce, Copyright (c) 2015-2026 CCBlueX.

//! State and event primitives ported from the pinned LiquidBounce source.
//! Game data and module implementations are supplied by the integrating runtime.

pub mod event;
pub mod input;
pub mod module;

pub use event::{CancelError, DispatchReport, Event, EventDispatcher, HandlerId, ListenerId};
pub use input::{
    BindAction, InputBind, InputContext, InputKey, KeyAction, KeyboardInput, Modifier, Modifiers,
    MouseInput,
};
pub use module::{
    CallbackError, ModuleCallback, ModuleCallbackFailure, ModuleCallbacks, ModuleDefinition,
    ModuleEffect, ModuleError, ModuleId, ModuleRegistry, ModuleState, ModuleSyncError,
    ModuleSyncReport, WorldState,
};
