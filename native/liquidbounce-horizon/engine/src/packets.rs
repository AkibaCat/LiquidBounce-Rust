// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared packet pipeline and NoSwing (LiquidBounce ae0eb396).
//! Registry state is copied into atomics by the game actor. Packet dispatch
//! never calls the registry and never holds handler-list locks during callbacks.

use horizon_client::mod_api::{
    ClientboundPlayPacket as Clientbound, ServerboundPlayPacket as Serverbound,
};
use horizon_client::packet_bridge;
pub use horizon_client::packet_bridge::{
    Delivery, IncomingReplay, PacketContext, PacketEvent, PacketInterceptor, ReplayError,
    TransferOrigin,
};
pub use liquidbounce_behaviors::no_swing::{
    HideFor, NoSwing, parse_no_swing_hide_for, serialize_no_swing_hide_for,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

pub trait PacketListener: Send + Sync {
    fn incoming(&self, _event: &mut PacketEvent<'_, Clientbound>) {}
    fn incoming_final(&self,_context:PacketContext,_packet:&Clientbound,_cancelled:bool) {}
    fn outgoing_final(&self,_context:PacketContext,_packet:&Serverbound,_cancelled:bool) {}
    /// Accepted final incoming state, before the actual packet handler.
    fn incoming_accepted(&self,_context:PacketContext,_packet:&Clientbound) {}
    fn outgoing(&self, _event: &mut PacketEvent<'_, Serverbound>) {}
    /// READ_FINAL_STATE observes the final packet only after ordinary handlers
    /// accepted it. This callback cannot mutate or cancel the packet.
    fn outgoing_accepted(&self, _context: PacketContext, _packet: &Serverbound) {}
    /// MixinEntity consults NoPitchLimit.running, independently of ServerSide.
    fn bypass_pitch_limit(&self) -> bool {
        false
    }
}

/// These optional services fix the original silent-send contract. They are not
/// implementations of RotationManager, PositionPacketSeparator or PacketLogger.
#[derive(Clone, Default)]
pub struct SilentSinks {
    pub rotation: Option<Arc<dyn PacketListener>>,
    pub position_separator: Option<Arc<dyn PacketListener>>,
    pub packet_logger: Option<Arc<dyn PacketListener>>,
}

#[derive(Clone)]
struct ListenerEntry {
    id: u64,
    priority: i32,
    listener: Arc<dyn PacketListener>,
}

#[derive(Default)]
pub struct PacketPipeline {
    pub no_swing: Arc<NoSwing>,
    handlers: RwLock<Vec<ListenerEntry>>,
    next_id: AtomicU64,
    silent: RwLock<SilentSinks>,
}

impl PacketPipeline {
    pub fn with_no_swing(no_swing: Arc<NoSwing>) -> Self {
        Self {
            no_swing,
            ..Self::default()
        }
    }

    pub fn install(self: &Arc<Self>) {
        packet_bridge::set_interceptor(Some(self.clone()));
    }

    pub fn add_listener(&self, priority: i32, listener: Arc<dyn PacketListener>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut handlers = self
            .handlers
            .write()
            .expect("packet handlers lock poisoned");
        handlers.push(ListenerEntry {
            id,
            priority,
            listener,
        });
        handlers.sort_by_key(|entry| (std::cmp::Reverse(entry.priority), entry.id));
        id
    }

    pub fn remove_listener(&self, id: u64) -> bool {
        let mut handlers = self
            .handlers
            .write()
            .expect("packet handlers lock poisoned");
        let before = handlers.len();
        handlers.retain(|entry| entry.id != id);
        handlers.len() != before
    }

    pub fn set_silent_sinks(&self, sinks: SilentSinks) {
        *self.silent.write().expect("silent sink lock poisoned") = sinks;
    }

    fn handlers(&self) -> Vec<ListenerEntry> {
        self.handlers
            .read()
            .expect("packet handlers lock poisoned")
            .clone()
    }
}

impl PacketInterceptor for PacketPipeline {
    fn incoming(&self, event: &mut PacketEvent<'_, Clientbound>) {
        if event.context.delivery != Delivery::Original {
            return;
        }
        let handlers=self.handlers();
        for handler in &handlers {
            handler.listener.incoming(event);
        }
        for handler in &handlers{handler.listener.incoming_final(event.context,event.packet,event.is_cancelled());}
        if !event.is_cancelled(){for handler in handlers{handler.listener.incoming_accepted(event.context,event.packet);}}
    }

    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        match event.context.delivery {
            Delivery::Original => {
                // MixinConnection checks NoSwing before constructing PacketEvent.
                if self.no_swing.should_hide_for_server()
                    && matches!(event.packet, Serverbound::Punch(_))
                {
                    event.cancel();
                    return;
                }
                let handlers = self.handlers();
                for handler in &handlers {
                    handler.listener.outgoing(event);
                }
                for handler in &handlers{handler.listener.outgoing_final(event.context,event.packet,event.is_cancelled());}
                if !event.is_cancelled() {
                    for handler in &handlers {
                        handler.listener.outgoing_accepted(event.context, event.packet);
                    }
                }
            }
            Delivery::Silent => {
                let sinks = self
                    .silent
                    .read()
                    .expect("silent sink lock poisoned")
                    .clone();
                for sink in [
                    sinks.rotation,
                    sinks.position_separator,
                    sinks.packet_logger,
                ]
                .into_iter()
                .flatten()
                {
                    sink.outgoing(event);
                }
            }
            Delivery::IncomingReplay => {}
        }
    }

    fn cancel_local_swing(&self) -> bool {
        self.no_swing.should_hide_for_client()
    }

    fn bypass_pitch_limit(&self) -> bool {
        self.handlers()
            .iter()
            .any(|entry| entry.listener.bypass_pitch_limit())
    }
}
