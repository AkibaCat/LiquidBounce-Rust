// SPDX-License-Identifier: GPL-3.0-or-later
// SilentHotbar.kt / SelectHotbarSlotSilentlyEvent at LiquidBounce ae0eb396.
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HotbarSlot(u8);

impl HotbarSlot {
    pub fn new(slot: i32) -> Result<Self, String> {
        if (0..9).contains(&slot) {
            Ok(Self(slot as u8))
        } else {
            Err(format!("Invalid hotbar slot: {slot}"))
        }
    }
    pub const fn index(self) -> u8 {
        self.0
    }
}

/// Identity, rather than equality of a module's name. Zero represents the
/// original nullable requester; allocate once for each independent owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requester(u64);
impl Requester {
    pub const NONE: Self = Self(0);
    pub fn allocate() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let value = NEXT.fetch_add(1, Ordering::Relaxed);
        assert_ne!(value, 0, "SilentHotbar requester identity exhausted");
        Self(value)
    }
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HotbarState {
    pub enforced_slot: HotbarSlot,
    pub requester: Requester,
    pub ticks_until_reset: i32,
    pub clientside_slot: HotbarSlot,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SilentHotbar {
    state: Option<HotbarState>,
    ticks_since_last_update: i32,
}

impl SilentHotbar {
    pub fn serverside_slot(&self, real_selected: Option<HotbarSlot>) -> HotbarSlot {
        self.state
            .map(|state| state.enforced_slot)
            .or(real_selected)
            .unwrap_or(HotbarSlot(0))
    }
    pub fn clientside_slot(&self, real_selected: Option<HotbarSlot>) -> HotbarSlot {
        self.state
            .map(|state| state.clientside_slot)
            .or(real_selected)
            .unwrap_or(HotbarSlot(0))
    }
    /// Called after the cancellable selection event. The existing visible slot
    /// survives replacement by a different owner, as in the original service.
    pub fn select_slot(
        &mut self,
        requester: Requester,
        slot: HotbarSlot,
        ticks_until_reset: i32,
        real_selected: Option<HotbarSlot>,
    ) {
        let clientside_slot = self.clientside_slot(real_selected);
        self.state = Some(HotbarState {
            enforced_slot: slot,
            requester,
            ticks_until_reset,
            clientside_slot,
        });
        self.ticks_since_last_update = 0;
    }
    pub fn reset_slot(&mut self, requester: Requester) -> bool {
        if self.state.is_some_and(|state| state.requester == requester) {
            self.state = None;
            true
        } else {
            false
        }
    }
    pub fn is_slot_modified(&self) -> bool {
        self.state.is_some()
    }
    pub fn is_slot_modified_by(&self, requester: Requester) -> bool {
        // Kotlin's hotbarState?.requester === requester compares null with
        // null when there is no state, so the nullable owner still matches.
        self.state
            .map(|state| state.requester)
            .unwrap_or(Requester::NONE)
            == requester
    }
    /// GameTickEvent priority 1001: compare before increment. A duration of 1
    /// survives one tick and resets at the start of the second tick.
    pub fn tick(&mut self) {
        let Some(state) = self.state else {
            return;
        };
        if self.ticks_since_last_update >= state.ticks_until_reset {
            self.state = None;
        } else {
            self.ticks_since_last_update += 1;
        }
    }
    pub fn world_change(&mut self) {
        self.state = None;
        self.ticks_since_last_update = 0;
    }
    pub fn state(&self) -> Option<HotbarState> {
        self.state
    }
    pub fn ticks_since_last_update(&self) -> i32 {
        self.ticks_since_last_update
    }
}
