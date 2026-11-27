// SPDX-License-Identifier: GPL-3.0-or-later
//! InventoryManager's accepted original-packet state, shared with the actor.
//! Packet dispatch never borrows the module registry or invokes Host.
use crate::packets::{PacketContext, PacketEvent, PacketListener};
use horizon_client::mod_api::ServerboundPlayPacket as Serverbound;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    UseOn([i32; 3]),
    MayOpen,
    Click(i32),
    Close,
}
#[derive(Default)]
pub struct InventoryPackets {
    pending: Mutex<Vec<(u64, Observation)>>,
    in_game: AtomicBool,
    open: AtomicBool,
}
impl InventoryPackets {
    pub fn publish(&self, in_game: bool, open: bool) {
        self.in_game.store(in_game, Ordering::Release);
        self.open.store(open, Ordering::Release);
    }
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
        self.publish(false, false);
    }
}
impl PacketListener for InventoryPackets {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if event.context.original()
            && self.in_game.load(Ordering::Acquire)
            && !event.is_cancelled()
            && !self.open.load(Ordering::Acquire)
            && matches!(event.packet,Serverbound::ContainerClose(packet) if packet.container==0)
        {
            event.cancel();
        }
    }
    fn outgoing_accepted(&self, context: PacketContext, packet: &Serverbound) {
        if !context.original() || !self.in_game.load(Ordering::Acquire) {
            return;
        }
        let observation = match packet {
            Serverbound::UseItemOn(packet) => Some(Observation::UseOn(packet.pos)),
            Serverbound::Interact(_) => Some(Observation::MayOpen),
            Serverbound::ContainerClick(packet) => {
                if packet.container == 0 {
                    self.open.store(true, Ordering::Release);
                }
                Some(Observation::Click(packet.container))
            }
            Serverbound::ContainerClose(_) => {
                self.open.store(false, Ordering::Release);
                Some(Observation::Close)
            }
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
