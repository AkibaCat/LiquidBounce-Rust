// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoFish's ordinary PacketEvent handler, including cancelled events.

use crate::packets::{Delivery, PacketEvent, PacketListener};
use horizon_client::{fish_bridge, mod_api::ClientboundPlayPacket as Clientbound};
pub use liquidbounce_behaviors::auto_fish::{AutoFish, AutoFishSettings, DelayRange, FishHand};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
pub struct AutoFishService {
    pub behavior: Mutex<AutoFish>,
    running: AtomicBool,
    ticker: crate::coroutine_ticker::CoroutineTicker,
}

impl AutoFishService {
    pub fn with_ticker(ticker: crate::coroutine_ticker::CoroutineTicker) -> Self {
        Self {
            ticker,
            ..Default::default()
        }
    }

    pub fn register_waits(&self, fish: &mut AutoFish) {
        for callback in fish.take_wait_registrations() {
            self.ticker
                .register(crate::coroutine_ticker::WaitOwner::AutoFish, callback);
        }
    }
    /// Publish on registry/world state changes, before callbacks can dispatch.
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}

impl PacketListener for AutoFishService {
    fn incoming(&self, event: &mut PacketEvent<'_, Clientbound>) {
        if event.context.delivery != Delivery::Original || !self.is_running() {
            return;
        }
        let Clientbound::Sound(packet) = event.packet else {
            return;
        };
        let snapshot = fish_bridge::snapshot();
        if snapshot.session == 0 || snapshot.session != event.context.session {
            return;
        }
        let mut behavior = self.behavior.lock().expect("AutoFish poisoned");
        behavior.sound_packet(
            true,
            snapshot.registered_sound(&packet.sound),
            snapshot.hook,
            [
                f64::from(packet.x) / 8.0,
                f64::from(packet.y) / 8.0,
                f64::from(packet.z) / 8.0,
            ],
        );
        self.register_waits(&mut behavior);
    }
}
