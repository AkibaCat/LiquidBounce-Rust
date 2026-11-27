// SPDX-License-Identifier: GPL-3.0-or-later
//! Conversion through the exact target's published InputConstants tables.

use horizon_client::mod_api::KeyCode;
use horizon_client::{key_names, keybinds};
use liquidbounce_core::{InputKey, Modifier, Modifiers};

/// Winit physical keys are not SDL scan codes or GLFW key IDs. Translate via
/// Horizon's vanilla key name and then its generated 26.3 SDL table.
pub fn keyboard_scan_code(code: KeyCode) -> Option<i32> {
    let name = keybinds::key_name(code)?;
    key_names::KEYBOARD
        .iter()
        .find_map(|(scan, key)| (*key == name).then_some(*scan))
}

pub fn configured_bind(name: &str) -> Option<InputKey> {
    key_names::KEYBOARD
        .iter()
        .find_map(|(scan, key)| (*key == name).then_some(InputKey::Keyboard(*scan)))
        .or_else(|| {
            key_names::MOUSE
                .iter()
                .find_map(|(button, key)| (*key == name).then_some(InputKey::Mouse(*button)))
        })
}

/// Raw Horizon mouse hooks expose GLFW's zero-based left/right/middle ordering;
/// the pinned LiquidBounce InputConstants uses SDL's one-based ordering.
pub fn mouse_button(button: i32) -> Option<i32> {
    match button {
        0 => Some(1),
        1 => Some(3),
        2 => Some(2),
        3..=7 => Some(button + 1),
        _ => None,
    }
}

pub fn modifiers(bits: u32) -> Modifiers {
    let mut held = Vec::with_capacity(4);
    for (bit, modifier) in [
        (1, Modifier::Shift),
        (2, Modifier::Control),
        (4, Modifier::Alt),
        (8, Modifier::Super),
    ] {
        if bits & bit != 0 {
            held.push(modifier);
        }
    }
    Modifiers::new(&held)
}

pub fn physical_modifier(code: KeyCode) -> Option<Modifier> {
    match code {
        KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(Modifier::Shift),
        KeyCode::ControlLeft | KeyCode::ControlRight => Some(Modifier::Control),
        KeyCode::AltLeft | KeyCode::AltRight => Some(Modifier::Alt),
        KeyCode::SuperLeft | KeyCode::SuperRight => Some(Modifier::Super),
        _ => None,
    }
}
