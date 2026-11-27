// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual SilentHotbar service, separate from the cosmetic SilentHotbar module.
use horizon_client::slot_bridge::{self, SlotPolicy};
pub use liquidbounce_behaviors::{
    no_slot_set::NoSlotSet,
    silent_hotbar::{HotbarSlot, HotbarState, Requester, SilentHotbar},
};
use std::sync::{
    Arc, Mutex, RwLock,
    atomic::{AtomicU64, Ordering},
};

pub struct SelectSlotEvent {
    requester: Requester,
    slot: HotbarSlot,
    cancelled: bool,
}
impl SelectSlotEvent {
    pub fn requester(&self) -> Requester {
        self.requester
    }
    pub fn slot(&self) -> HotbarSlot {
        self.slot
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}
pub trait SelectSlotListener: Send + Sync {
    fn selecting(&self, event: &mut SelectSlotEvent);
}
#[derive(Clone)]
struct SelectionListener {
    id: u64,
    priority: i32,
    listener: Arc<dyn SelectSlotListener>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireSlot {
    pub session: u64,
    pub slot: i16,
    pub silent: bool,
}
#[derive(Default)]
struct Observations {
    real_selected: Option<u8>,
    wire: Option<WireSlot>,
    server_applied: Option<u8>,
}

#[derive(Default)]
pub struct SlotService {
    pub no_slot_set: Arc<NoSlotSet>,
    hotbar: Mutex<SilentHotbar>,
    listeners: RwLock<Vec<SelectionListener>>,
    next_id: AtomicU64,
    observations: Mutex<Observations>,
}

fn real(index: Option<u8>) -> Option<HotbarSlot> {
    index.and_then(|slot| HotbarSlot::new(i32::from(slot)).ok())
}

impl SlotService {
    pub fn install(self: &Arc<Self>) {
        slot_bridge::set_policy(Some(self.clone()));
    }
    pub fn add_listener(&self, priority: i32, listener: Arc<dyn SelectSlotListener>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut listeners = self.listeners.write().expect("slot listeners poisoned");
        listeners.push(SelectionListener {
            id,
            priority,
            listener,
        });
        listeners.sort_by_key(|entry| (std::cmp::Reverse(entry.priority), entry.id));
        id
    }
    pub fn remove_listener(&self, id: u64) -> bool {
        let mut listeners = self.listeners.write().expect("slot listeners poisoned");
        let before = listeners.len();
        listeners.retain(|entry| entry.id != id);
        before != listeners.len()
    }
    /// A module calls this on the actor with a fresh real-slot supplier. It runs
    /// after all selection listeners, with no service/listener lock held.
    pub fn select_slot_silently(
        &self,
        requester: Requester,
        slot: i32,
        ticks: i32,
        real_selected: impl FnOnce() -> Option<u8>,
    ) -> Result<bool, String> {
        let slot = HotbarSlot::new(slot)?;
        let listeners = self
            .listeners
            .read()
            .expect("slot listeners poisoned")
            .clone();
        let mut event = SelectSlotEvent {
            requester,
            slot,
            cancelled: false,
        };
        for entry in listeners {
            entry.listener.selecting(&mut event);
        }
        if event.is_cancelled() {
            return Ok(false);
        }
        let real_selected = real(real_selected());
        self.hotbar
            .lock()
            .expect("SilentHotbar poisoned")
            .select_slot(requester, slot, ticks, real_selected);
        Ok(true)
    }
    /// The HotbarItemSlot overload ignores offhand/non-hotbar inventory slots.
    pub fn select_inventory_slot_silently(
        &self,
        requester: Requester,
        hotbar_index: Option<i32>,
        ticks: i32,
        real_selected: impl FnOnce() -> Option<u8>,
    ) -> Result<bool, String> {
        hotbar_index.map_or(Ok(true), |index| {
            self.select_slot_silently(requester, index, ticks, real_selected)
        })
    }
    pub fn state(&self) -> Option<HotbarState> {
        self.hotbar.lock().expect("SilentHotbar poisoned").state()
    }
    pub fn wire_slot(&self) -> Option<WireSlot> {
        self.observations
            .lock()
            .expect("slot observation poisoned")
            .wire
    }
    pub fn last_real_slot(&self) -> Option<u8> {
        self.observations
            .lock()
            .expect("slot observation poisoned")
            .real_selected
    }
    pub fn last_server_applied_slot(&self) -> Option<u8> {
        self.observations
            .lock()
            .expect("slot observation poisoned")
            .server_applied
    }
}

impl SlotPolicy for SlotService {
    fn serverside_slot(&self, real_selected: Option<u8>) -> u8 {
        self.observations
            .lock()
            .expect("slot observation poisoned")
            .real_selected = real_selected;
        self.hotbar
            .lock()
            .expect("SilentHotbar poisoned")
            .serverside_slot(real(real_selected))
            .index()
    }
    fn clientside_slot(&self, real_selected: Option<u8>) -> u8 {
        self.observations
            .lock()
            .expect("slot observation poisoned")
            .real_selected = real_selected;
        self.hotbar
            .lock()
            .expect("SilentHotbar poisoned")
            .clientside_slot(real(real_selected))
            .index()
    }
    fn no_slot_set_running(&self) -> bool {
        self.no_slot_set.running()
    }
    fn select_slot(
        &self,
        requester: u64,
        slot: i32,
        ticks: i32,
        real_selected: Option<u8>,
    ) -> Result<bool, String> {
        self.select_slot_silently(Requester::from_raw(requester), slot, ticks, || {
            real_selected
        })
    }
    fn reset_slot(&self, requester: u64) -> bool {
        self.hotbar
            .lock()
            .expect("SilentHotbar poisoned")
            .reset_slot(Requester::from_raw(requester))
    }
    fn modified(&self) -> bool {
        self.hotbar
            .lock()
            .expect("SilentHotbar poisoned")
            .is_slot_modified()
    }
    fn modified_by(&self, requester: u64) -> bool {
        self.hotbar
            .lock()
            .expect("SilentHotbar poisoned")
            .is_slot_modified_by(Requester::from_raw(requester))
    }
    fn tick(&self) {
        self.hotbar.lock().expect("SilentHotbar poisoned").tick();
    }
    fn world_change(&self) {
        self.hotbar
            .lock()
            .expect("SilentHotbar poisoned")
            .world_change();
        *self.observations.lock().expect("slot observation poisoned") = Observations::default();
    }
    fn outgoing_slot_sent(&self, session: u64, slot: i16, silent: bool) {
        self.observations
            .lock()
            .expect("slot observation poisoned")
            .wire = Some(WireSlot {
            session,
            slot,
            silent,
        });
    }
    fn server_slot_applied(&self, slot: u8) {
        let mut observed = self.observations.lock().expect("slot observation poisoned");
        observed.server_applied = Some(slot);
        observed.real_selected = Some(slot);
    }
}
