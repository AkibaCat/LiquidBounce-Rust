// SPDX-License-Identifier: GPL-3.0-or-later
//! PacketMine state holder. Game hooks feed real block facts into the model;
//! keeping the state in the actor makes enable/disable and config updates safe.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::packet_mine::{PacketMine, Settings};

pub const REQUESTER: u64 = 0x5041_434b_4d49_4e45;

#[derive(Clone, Debug)]
pub struct PacketMineRuntime {
    pub behavior: PacketMine,
}

impl Default for PacketMineRuntime {
    fn default() -> Self {
        Self {
            behavior: PacketMine::new(0),
        }
    }
}

impl PacketMineRuntime {
    pub fn reset(&mut self) {
        self.behavior.disabled();
    }
    pub fn settings(&self) -> &Settings {
        &self.behavior.settings
    }
}

impl LiquidBounceHooks {
    pub(crate) fn packet_mine_tick(&self, _host: &mut dyn Host) {
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if !modules.query_running(&state.registry, "PacketMine") {
            return;
        }
        // Event-specific game facts are supplied by the native hand hook. The
        // model remains idle until a real target has been selected there.
        if modules.packet_mine.behavior.target.is_none() {
            return;
        }
    }
}
