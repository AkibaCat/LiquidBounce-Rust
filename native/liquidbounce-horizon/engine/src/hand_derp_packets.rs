// SPDX-License-Identifier: GPL-3.0-or-later
//! HandDerp priority=1 packet handlers. No state lock crosses nested sends.
use crate::packets::{Delivery, PacketContext, PacketEvent, PacketListener};
use horizon_client::{
    hand_derp_bridge,
    mod_api::{ClientboundPlayPacket as Clientbound, ServerboundPlayPacket as Serverbound},
};
pub use liquidbounce_behaviors::hand_derp::{HandDerp, MainHand};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
struct AcceptedPackets {
    information: u64,
    punches: u64,
    last_information: Option<serde_json::Value>,
}
#[derive(Default)]
pub struct HandDerpService {
    pub behavior: Mutex<HandDerp>,
    running: AtomicBool,
    accepted: Mutex<AcceptedPackets>,
}
impl HandDerpService {
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
    }
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
    pub fn initialize(&self) {
        if let Some(snapshot) = hand_derp_bridge::snapshot() {
            self.behavior
                .lock()
                .expect("HandDerp poisoned")
                .initialize(MainHand::from_left(snapshot.original_hand == 0));
        }
    }
    /// READ_FINAL_STATE packet evidence, independent of pure flip intentions.
    /// Counts all accepted ordinary packets, including normal option broadcasts;
    /// compare deltas around a controlled action. Acceptance precedes transport.
    pub fn accepted_packets(&self) -> serde_json::Value {
        let state = self
            .accepted
            .lock()
            .expect("HandDerp observations poisoned");
        serde_json::json!({"clientInformation":state.information,"punches":state.punches,"lastClientInformation":state.last_information})
    }
}
impl PacketListener for HandDerpService {
    fn outgoing_accepted(&self, context: PacketContext, packet: &Serverbound) {
        if context.delivery != Delivery::Original {
            return;
        }
        match packet {
            Serverbound::Punch(_) => {
                self.accepted
                    .lock()
                    .expect("HandDerp observations poisoned")
                    .punches += 1;
            }
            Serverbound::ClientInformation(info) => {
                let mut state = self
                    .accepted
                    .lock()
                    .expect("HandDerp observations poisoned");
                state.information += 1;
                state.last_information = Some(
                    serde_json::json!({"session":context.session,"language":info.language,"viewDistance":info.view_distance,"chatVisibility":info.chat_visibility,"chatColors":info.chat_colors,"modelCustomisation":info.model_customisation,"mainHand":info.main_hand,"textFiltering":info.text_filtering,"allowsListing":info.allows_listing,"particleStatus":info.particle_status}),
                );
            }
            _ => {}
        }
    }
    fn incoming(&self, event: &mut PacketEvent<'_, Clientbound>) {
        if event.context.delivery != Delivery::Original || !self.is_running() {
            return;
        }
        let Clientbound::SetEntityData(packet) = event.packet else {
            return;
        };
        let Some(options) = hand_derp_bridge::snapshot() else {
            return;
        };
        let cancel = self
            .behavior
            .lock()
            .expect("HandDerp poisoned")
            .cancel_metadata(
                true,
                MainHand::from_left(options.information.main_hand == 0),
                packet.values.iter().map(|value| value.0),
            );
        if cancel {
            event.cancel();
        }
    }
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if event.context.delivery != Delivery::Original
            || !self.is_running()
            || !matches!(event.packet, Serverbound::Punch(_))
        {
            return;
        }
        self.initialize();
        let hand = self.behavior.lock().expect("HandDerp poisoned").punch(true);
        if let Some(packet) = hand.and_then(|hand| hand_derp_bridge::packet(hand.id())) {
            event.send(packet);
        }
    }
}
