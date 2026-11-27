// SPDX-License-Identifier: GPL-3.0-or-later
//! Committed local-player fishing state for the socket-reader PacketEvent.
//! No Game pointer, actor request, module callback or writer lock crosses here.

use horizon_protocol::registry::Builtin;
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Clone, Debug, Default)]
pub struct FishingSnapshot {
    pub session: u64,
    pub hook: Option<[f64; 3]>,
    pub builtin: Option<Arc<Builtin>>,
}

impl FishingSnapshot {
    /// MappedRegistry.getKey uses object identity. A direct inline SoundEvent
    /// has no registered key, even when its location spells a selected sound.
    pub fn registered_sound(&self, sound: &(i32, Option<(String, Option<f32>)>)) -> Option<&str> {
        if sound.1.is_some() { return None; }
        self.builtin.as_ref()?.get("minecraft:sound_event")?.name(sound.0)
    }
}

fn slot() -> &'static RwLock<FishingSnapshot> {
    static SNAPSHOT: OnceLock<RwLock<FishingSnapshot>> = OnceLock::new();
    SNAPSHOT.get_or_init(|| RwLock::new(FishingSnapshot::default()))
}

pub fn snapshot() -> FishingSnapshot {
    slot().read().expect("fishing snapshot poisoned").clone()
}

pub fn publish(state: FishingSnapshot) {
    *slot().write().expect("fishing snapshot poisoned") = state;
}

pub fn clear() {
    publish(FishingSnapshot::default());
}
