extern crate self as horizon_client;

pub mod mod_api {
    pub use horizon_protocol::packets::play::{
        Clientbound as ClientboundPlayPacket, Serverbound as ServerboundPlayPacket,
    };
}

#[path = "../../packet-bridge.rs"]
pub mod packet_bridge;
#[path = "../../engine/src/packets.rs"]
pub mod packets;
#[path = "../../engine/src/packet_modules.rs"]
pub mod packet_modules;
#[path = "../../slot-bridge.rs"]
pub mod slot_bridge;
#[path = "../../engine/src/slots.rs"]
pub mod slots;
#[path = "../../engine/src/friends.rs"]
pub mod friends;
