// SPDX-License-Identifier: GPL-3.0-or-later
//! RotationManager's READ_FINAL_STATE listener at LiquidBounce ae0eb396.
//! Install last in the ordinary pipeline and first in the explicit silent path.
use crate::packets::{Delivery, PacketEvent, PacketListener};
use horizon_client::mod_api::{
    ClientboundPlayPacket as Clientbound, ServerboundPlayPacket as Serverbound,
};
pub use liquidbounce_behaviors::rotations::RotationManager;
use std::sync::{Arc, Mutex};

// Relative.Y_ROT / X_ROT in the pinned protocol's PlayerPosition relatives.
// The adapter depends only on horizon-client's public packet aliases.
const Y_ROT: i32 = 8;
const X_ROT: i32 = 16;

pub struct RotationListener(pub Arc<Mutex<RotationManager>>);

impl PacketListener for RotationListener {
    fn incoming(&self, event: &mut PacketEvent<'_, Clientbound>) {
        if event.context.delivery != Delivery::Original {
            return;
        }
        let (yaw, pitch, relative_yaw, relative_pitch) = match event.packet {
            Clientbound::PlayerPosition(packet) => (
                packet.yaw,
                packet.pitch,
                packet.relatives & Y_ROT != 0,
                packet.relatives & X_ROT != 0,
            ),
            Clientbound::PlayerRotation(packet) => (
                packet.yaw,
                packet.pitch,
                packet.relative_yaw,
                packet.relative_pitch,
            ),
            _ => return,
        };
        // Server corrections already changed the server, even if the client
        // cancels them. Relative values resolve against actual_server.
        self.0
            .lock()
            .expect("RotationManager poisoned")
            .inbound_rotation(yaw, pitch, relative_yaw, relative_pitch);
    }

    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if event.context.delivery == Delivery::IncomingReplay {
            return;
        }
        let (yaw, pitch) = match event.packet {
            Serverbound::MovePlayerRot(packet) => (packet.yaw, packet.pitch),
            Serverbound::MovePlayerPosRot(packet) => (packet.yaw, packet.pitch),
            Serverbound::UseItem(packet) => (packet.yaw, packet.pitch),
            _ => return,
        };
        // NetworkUtils.sendPacketSilently sends unconditionally after its
        // tracking sinks; its event's cancellation cannot suppress the write.
        let cancelled = event.context.delivery != Delivery::Silent && event.is_cancelled();
        self.0
            .lock()
            .expect("RotationManager poisoned")
            .outbound_rotation(yaw, pitch, cancelled);
    }
}
