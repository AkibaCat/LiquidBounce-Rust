// SPDX-License-Identifier: GPL-3.0-or-later
//! Thread-safe policy seam; game slot operations stay on the client actor.
//! Ordinary/silent send observations contain actual accepted wire slots only.
use horizon_protocol::packets::play::Serverbound;
use std::sync::{Arc, OnceLock, RwLock};

pub trait SlotPolicy: Send + Sync {
    fn serverside_slot(&self, real_selected: Option<u8>) -> u8;
    fn clientside_slot(&self, real_selected: Option<u8>) -> u8;
    fn no_slot_set_running(&self) -> bool;
    fn select_slot(
        &self,
        requester: u64,
        slot: i32,
        ticks: i32,
        real_selected: Option<u8>,
    ) -> Result<bool, String>;
    fn reset_slot(&self, requester: u64) -> bool;
    fn modified(&self) -> bool;
    fn modified_by(&self, requester: u64) -> bool;
    fn tick(&self);
    fn world_change(&self);
    fn outgoing_slot_sent(&self, session: u64, slot: i16, silent: bool);
    fn server_slot_applied(&self, slot: u8);
}

type Policy = Arc<dyn SlotPolicy>;
static POLICY: OnceLock<RwLock<Option<Policy>>> = OnceLock::new();
fn slot() -> &'static RwLock<Option<Policy>> {
    POLICY.get_or_init(|| RwLock::new(None))
}
fn snapshot() -> Option<Policy> {
    slot().read().expect("slot bridge poisoned").clone()
}
pub fn set_policy(policy: Option<Policy>) -> Option<Policy> {
    std::mem::replace(&mut *slot().write().expect("slot bridge poisoned"), policy)
}
pub fn serverside_slot(real: Option<u8>) -> u8 {
    snapshot().map_or_else(|| real.unwrap_or(0), |p| p.serverside_slot(real))
}
pub fn clientside_slot(real: Option<u8>) -> u8 {
    snapshot().map_or_else(|| real.unwrap_or(0), |p| p.clientside_slot(real))
}
pub fn no_slot_set_running() -> bool {
    snapshot().is_some_and(|p| p.no_slot_set_running())
}
pub fn select_slot(
    requester: u64,
    index: i32,
    ticks: i32,
    real: Option<u8>,
) -> Result<bool, String> {
    snapshot()
        .ok_or_else(|| "SilentHotbar service is not installed".to_owned())?
        .select_slot(requester, index, ticks, real)
}
pub fn reset_slot(requester: u64) -> bool {
    snapshot().is_some_and(|p| p.reset_slot(requester))
}
pub fn modified() -> bool {
    snapshot().is_some_and(|p| p.modified())
}
pub fn modified_by(requester: u64) -> bool {
    snapshot().is_some_and(|p| p.modified_by(requester))
}
pub fn tick() {
    if let Some(p) = snapshot() {
        p.tick();
    }
}
pub fn world_change() {
    if let Some(p) = snapshot() {
        p.world_change();
    }
}
pub fn outgoing_sent(session: u64, packet: &Serverbound, silent: bool) {
    if let Serverbound::SetCarriedItem(packet) = packet
        && let Some(p) = snapshot()
    {
        p.outgoing_slot_sent(session, packet.slot, silent);
    }
}
pub fn server_slot_applied(index: u8) {
    if let Some(p) = snapshot() {
        p.server_slot_applied(index);
    }
}
