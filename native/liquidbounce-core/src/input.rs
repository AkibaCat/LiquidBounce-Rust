// SPDX-License-Identifier: GPL-3.0-or-later
// Based on LiquidBounce utils/input/InputBind.kt.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindAction {
    Toggle,
    Hold,
    Smart,
}

/// Keyboard numbers are the pinned upstream SDL scan codes. The runtime bridge
/// must map its physical keys to these numbers; the core never invents a key map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputKey {
    Unbound,
    Keyboard(i32),
    Mouse(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Repeat,
    Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Modifier {
    Shift = 1,
    Control = 2,
    Alt = 4,
    Super = 8,
}

/// Platform-independent logical modifier groups. A group is active while either
/// its left or right physical key is down. Extra held modifiers are allowed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub fn new(modifiers: &[Modifier]) -> Self {
        Self(
            modifiers
                .iter()
                .fold(0, |bits, modifier| bits | *modifier as u8),
        )
    }
    pub fn contains(self, modifier: Modifier) -> bool {
        self.0 & modifier as u8 != 0
    }
    pub fn includes(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputBind {
    pub key: InputKey,
    pub action: BindAction,
    pub modifiers: Modifiers,
}

impl InputBind {
    pub fn new(key: InputKey, action: BindAction) -> Self {
        Self {
            key,
            action,
            modifiers: Modifiers::NONE,
        }
    }
    pub fn with_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }
    pub(crate) fn key_matches(self, key: i32) -> bool {
        matches!(self.key, InputKey::Keyboard(bound) if bound != 0 && bound == key)
    }
    pub(crate) fn mouse_matches(self, button: i32) -> bool {
        self.key == InputKey::Mouse(button)
    }
    pub(crate) fn modifier_released(self, modifier: Option<Modifier>, held: Modifiers) -> bool {
        modifier
            .is_some_and(|modifier| self.modifiers.contains(modifier) && !held.contains(modifier))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputContext {
    pub screen_open: bool,
    pub debug_modifier_down: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardInput {
    pub key: i32,
    pub action: KeyAction,
    pub modifiers: Modifiers,
    /// The bridge supplies the logical modifier of a released physical key.
    /// `modifiers` must reflect both physical sides after this release.
    pub released_modifier: Option<Modifier>,
}

impl KeyboardInput {
    pub fn new(key: i32, action: KeyAction, modifiers: Modifiers) -> Self {
        Self {
            key,
            action,
            modifiers,
            released_modifier: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseInput {
    pub button: i32,
    pub action: KeyAction,
    pub modifiers: Modifiers,
}

impl MouseInput {
    pub fn new(button: i32, action: KeyAction, modifiers: Modifiers) -> Self {
        Self {
            button,
            action,
            modifiers,
        }
    }
}
