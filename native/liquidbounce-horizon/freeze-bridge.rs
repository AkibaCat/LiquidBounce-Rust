// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::OnceLock;
static QUERY: OnceLock<fn(&mut bool)> = OnceLock::new();
pub fn install(callback: fn(&mut bool)) {
    let _ = QUERY.set(callback);
}
fn client_freeze_movement(cancelled: &mut bool) {
    if let Some(callback) = QUERY.get() {
        callback(cancelled);
    }
}
#[inline(never)]
pub fn native_client_freeze_movement(cancelled: &mut bool) {
    client_freeze_movement(cancelled);
}
pub fn cancel() -> bool {
    let mut cancelled = false;
    native_client_freeze_movement(&mut cancelled);
    cancelled
}
pub fn rotation(yaw: f32, pitch: f32, flags: u8) -> horizon_protocol::packets::play::Serverbound {
    horizon_protocol::packets::play::Serverbound::MovePlayerRot(
        horizon_protocol::packets::play::MovePlayerRot { yaw, pitch, flags },
    )
}
