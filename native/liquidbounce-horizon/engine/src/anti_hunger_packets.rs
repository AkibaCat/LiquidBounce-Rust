// SPDX-License-Identifier: GPL-3.0-or-later
//! All ordinary AntiHunger packets, including an already-cancelled event.
use crate::{
    anti_hunger_notifications::NotificationQueue,
    packets::{PacketEvent, PacketListener},
};
use horizon_client::mod_api::ServerboundPlayPacket as Serverbound;
use liquidbounce_behaviors::anti_hunger::{AntiHunger, Context, Packet, Source};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
#[derive(Default)]
pub struct AntiHungerService {
    pub behavior: Mutex<AntiHunger>,
    running: AtomicBool,
    pub notifications: Arc<NotificationQueue>,
}
impl AntiHungerService {
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
    }
    pub fn running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
    pub fn sprint(&self, swimming: bool, source: Source, sprint: bool) -> bool {
        self.behavior.lock().expect("AntiHunger poisoned").sprint(
            self.running(),
            swimming,
            source,
            sprint,
        )
    }
}
fn now() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(v) => v.as_millis() as i64,
        Err(v) => -(v.duration().as_millis() as i64),
    }
}
impl PacketListener for AntiHungerService {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if !event.context.original() || !self.running() {
            return;
        }
        let Some(player) = event.client_player else {
            return;
        };
        let snapshot = player.snapshot();
        let context = Context {
            swimming: snapshot.swimming,
            passenger: snapshot.passenger,
            destroying: snapshot.destroying,
            in_water: snapshot.in_water,
            under_water: snapshot.under_water,
            fall_distance: snapshot.fall_distance,
        };
        let packet = match event.packet {
            Serverbound::PlayerCommand(p) => Some(Packet::PlayerCommand {
                start_sprinting: p.action == 1,
            }),
            Serverbound::PlayerAction(p) => Some(Packet::PlayerAction {
                start_destroy: p.action == 0,
            }),
            Serverbound::MovePlayerPos(p) => {
                let mut ground = p.flags & 1 != 0;
                self.behavior.lock().expect("AntiHunger poisoned").movement(
                    true,
                    context,
                    &mut ground,
                );
                p.flags = (p.flags & !1) | u8::from(ground);
                None
            }
            Serverbound::MovePlayerPosRot(p) => {
                let mut ground = p.flags & 1 != 0;
                self.behavior.lock().expect("AntiHunger poisoned").movement(
                    true,
                    context,
                    &mut ground,
                );
                p.flags = (p.flags & !1) | u8::from(ground);
                None
            }
            Serverbound::MovePlayerRot(p) => {
                let mut ground = p.flags & 1 != 0;
                self.behavior.lock().expect("AntiHunger poisoned").movement(
                    true,
                    context,
                    &mut ground,
                );
                p.flags = (p.flags & !1) | u8::from(ground);
                None
            }
            Serverbound::MovePlayerStatus(p) => {
                let mut ground = p.flags & 1 != 0;
                self.behavior.lock().expect("AntiHunger poisoned").movement(
                    true,
                    context,
                    &mut ground,
                );
                p.flags = (p.flags & !1) | u8::from(ground);
                None
            }
            _ => None,
        };
        let Some(packet) = packet else {
            return;
        };
        let warning = self.behavior.lock().expect("AntiHunger poisoned").warning(
            true,
            snapshot.swimming,
            packet,
            now,
        );
        if let Some(warning) = warning {
            self.notifications
                .publish(warning, horizon_client::lang::code());
            self.behavior
                .lock()
                .expect("AntiHunger poisoned")
                .notification_published(now());
            // Drop every service lock before the synchronous jump re-enters
            // the generated PlayerJumpEvent and registers any HighJump wait.
            if warning.jump() {
                player.jump_from_ground();
            }
        }
    }
}
