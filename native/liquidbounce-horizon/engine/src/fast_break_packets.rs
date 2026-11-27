// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete FastBreak AbortAnother PacketEvent, ordinary nested send at priority 0.
use crate::packets::{PacketEvent, PacketListener};
use horizon_client::{fast_break_bridge, mod_api::ServerboundPlayPacket as Serverbound};
pub use liquidbounce_behaviors::fast_break::FastBreak;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};
#[derive(Default)]
pub struct FastBreakService {
    pub behavior: Mutex<FastBreak>,
    running: AtomicBool,
}
impl FastBreakService {
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
    }
    pub fn running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}
impl PacketListener for FastBreakService {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if !event.context.original() || !self.running() {
            return;
        }
        let Serverbound::PlayerAction(packet) = event.packet else {
            return;
        };
        let Some(context) = fast_break_bridge::snapshot(event.context.session) else {
            return;
        };
        let position = self
            .behavior
            .lock()
            .expect("FastBreak poisoned")
            .abort_above(true, context.mining_tool, packet.action == 3, packet.pos);
        let mut abort = packet.clone();
        if let Some(pos) = position {
            // The three-argument original constructor defaults sequence to zero.
            // Drop the behavior lock before recursive PacketEvent dispatch.
            abort.action = 2;
            abort.pos = pos;
            abort.sequence = 0;
            event.send(Serverbound::PlayerAction(abort));
        }
    }
}
