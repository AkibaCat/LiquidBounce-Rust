// SPDX-License-Identifier: GPL-3.0-or-later
//! Typed adapters for the complete packet handlers in MoreCarry and NoPitchLimit.
//! The actor owns registration and settings; these adapters only read atomics.
use crate::packets::{PacketEvent, PacketListener};
use horizon_client::mod_api::ServerboundPlayPacket as Serverbound;
pub use liquidbounce_behaviors::{more_carry::MoreCarry, no_pitch_limit::NoPitchLimit};
use std::sync::Arc;

pub struct MoreCarryListener(pub Arc<MoreCarry>);

impl PacketListener for MoreCarryListener {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if matches!(event.packet, Serverbound::ContainerClose(_)) && self.0.cancel_container_close()
        {
            event.cancel();
        }
    }
}

pub struct NoPitchLimitListener(pub Arc<NoPitchLimit>);

impl PacketListener for NoPitchLimitListener {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        let pitch = match event.packet {
            Serverbound::MovePlayerRot(packet) => &mut packet.pitch,
            Serverbound::MovePlayerPosRot(packet) => &mut packet.pitch,
            _ => return,
        };
        *pitch = self.0.outgoing_pitch(*pitch);
    }

    fn bypass_pitch_limit(&self) -> bool {
        self.0.running()
    }
}
