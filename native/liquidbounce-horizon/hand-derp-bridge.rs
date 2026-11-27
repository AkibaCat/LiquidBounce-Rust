// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual current options shared with packet callbacks. The original main arm
//! is captured at the window Game's construction, before any world/module use.
use horizon_protocol::packets::{configuration::ClientInformation, play::Serverbound};
use std::sync::{OnceLock, RwLock};

#[derive(Clone, Debug)]
pub struct OptionsSnapshot { pub original_hand: i32, pub information: ClientInformation }
fn slot() -> &'static RwLock<Option<OptionsSnapshot>> {
    static OPTIONS: OnceLock<RwLock<Option<OptionsSnapshot>>> = OnceLock::new();
    OPTIONS.get_or_init(|| RwLock::new(None))
}
pub fn publish(information: ClientInformation) {
    let mut state = slot().write().expect("HandDerp options poisoned");
    let original_hand = state.as_ref().map_or(information.main_hand, |old| old.original_hand);
    *state = Some(OptionsSnapshot { original_hand, information });
}
pub fn snapshot() -> Option<OptionsSnapshot> { slot().read().expect("HandDerp options poisoned").clone() }
pub fn packet(hand: i32) -> Option<Serverbound> {
    let mut information = snapshot()?.information;
    information.main_hand = hand;
    Some(Serverbound::ClientInformation(information))
}
