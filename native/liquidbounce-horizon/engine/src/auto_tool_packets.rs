// SPDX-License-Identifier: GPL-3.0-or-later
//! Final accepted original outgoing packets. Game-dependent tests run on actor.
use crate::packets::{PacketContext, PacketListener};
use horizon_client::mod_api::ServerboundPlayPacket as Serverbound;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    UseOn([i32; 3]),
    MayOpen,
    Click,
    Close,
}
#[derive(Default)]
pub struct InventoryPackets {
    pending: Mutex<Vec<(u64, Observation)>>,
}
impl InventoryPackets {
    pub fn drain(&self, session: u64) -> Vec<Observation> {
        self.pending
            .lock()
            .expect("inventory observations poisoned")
            .drain(..)
            .filter_map(|(owner, observation)| (owner == session).then_some(observation))
            .collect()
    }
    pub fn world_change(&self) {
        self.pending
            .lock()
            .expect("inventory observations poisoned")
            .clear();
    }
}
impl PacketListener for InventoryPackets {
    fn outgoing_accepted(&self, context: PacketContext, packet: &Serverbound) {
        if !context.original() {
            return;
        }
        let observation = match packet {
            Serverbound::UseItemOn(packet) => Some(Observation::UseOn(packet.pos)),
            Serverbound::Interact(_) => Some(Observation::MayOpen),
            Serverbound::ContainerClick(packet) if packet.container == 0 => {
                Some(Observation::Click)
            }
            Serverbound::ContainerClose(_) => Some(Observation::Close),
            _ => None,
        };
        if let Some(observation) = observation {
            self.pending
                .lock()
                .expect("inventory observations poisoned")
                .push((context.session, observation));
        }
    }
}
