// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    behaviors::current_time_millis,
    packets::{PacketEvent, PacketListener},
    runtime::{LiquidBounceHooks, raw_options},
};
use horizon_client::mod_api::{ClientboundPlayPacket as C, Host};
use liquidbounce_behaviors::ping_spoof::DelayQueue;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicI32, Ordering},
};
pub struct PingSpoofService {
    running: AtomicBool,
    alive: AtomicBool,
    pub delay: AtomicI32,
    queue: Mutex<DelayQueue<C>>,
}
impl Default for PingSpoofService {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(false),
            alive: AtomicBool::new(false),
            delay: AtomicI32::new(500),
            queue: Default::default(),
        }
    }
}
impl PingSpoofService {
    pub fn set_running(&self, value: bool) {
        self.running.store(value, Ordering::Release);
    }
    pub fn world_change(&self) {
        self.alive.store(false, Ordering::Release);
        self.queue.lock().expect("PingSpoof queue poisoned").clear();
    }
}
impl PacketListener for PingSpoofService {
    fn incoming(&self, event: &mut PacketEvent<'_, C>) {
        if !event.context.original()
            || event.is_cancelled()
            || !self.running.load(Ordering::Acquire)
            || !self.alive.load(Ordering::Acquire)
        {
            return;
        }
        let packet = match event.packet {
            C::KeepAlive(p) => C::KeepAlive(p.clone()),
            C::Ping(p) => C::Ping(p.clone()),
            _ => return,
        };
        self.queue.lock().expect("PingSpoof queue poisoned").push(
            event.context.session,
            current_time_millis(),
            packet,
        );
        event.cancel();
    }
}
impl LiquidBounceHooks {
    pub(crate) fn ping_spoof_tick(&self, host: &mut dyn Host) {
        let context = host.client_ping_spoof_context();
        let session = host.client_session().unwrap_or(0);
        let (service, mut running) = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            (
                modules.ping_spoof.clone(),
                modules.query_running(&state.registry, "PingSpoof"),
            )
        };
        service
            .alive
            .store(context.is_some_and(|c| c[0]), Ordering::Release);
        if running && context.is_some_and(|c| c[1]) {
            #[cfg(feature = "webview")]
            let message = liquidbounce_interop::localization::describe(
                &horizon_client::lang::code(),
                "liquidbounce.module.pingSpoof.messages.cantEnableInSingleplayer",
            );
            #[cfg(not(feature = "webview"))]
            let message = horizon_client::lang::get(
                "liquidbounce.module.pingSpoof.messages.cantEnableInSingleplayer",
            );
            host.chat(&message);
            let options = raw_options(host);
            let effects = {
                let mut state = self.state();
                let id = state.behaviors.borrow().query_ids["PingSpoof"];
                if let Err(error) =
                    state.with_registry(options, |r, c| r.set_enabled_sync(id, false, c))
                {
                    eprintln!("[LiquidBounce] PingSpoof: {error:?}");
                }
                state.take_effects()
            };
            crate::runtime::apply_effects(self, host, effects);
            running = false;
        }
        let packets = service
            .queue
            .lock()
            .expect("PingSpoof queue poisoned")
            .drain(
                session,
                current_time_millis(),
                service.delay.load(Ordering::Acquire),
                running,
            );
        for packet in packets {
            if let Err(error) = host.replay_client_packet(session, packet) {
                eprintln!("[LiquidBounce] PingSpoof replay: {error}");
            }
        }
    }
}
