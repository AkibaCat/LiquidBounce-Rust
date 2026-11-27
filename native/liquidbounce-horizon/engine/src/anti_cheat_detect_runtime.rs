// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    packets::{PacketContext, PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::{ClientboundPlayPacket as C, Host};
use liquidbounce_behaviors::anti_cheat_detect::Observation;
use std::sync::Mutex;
#[derive(Default)]
pub struct AntiCheatDetectService {
    pub observation: Mutex<Observation>,
    completed: Mutex<Vec<Observation>>,
}
impl AntiCheatDetectService {
    pub fn disconnect(&self) {
        self.observation
            .lock()
            .expect("ServerObserver")
            .disconnect();
        self.completed.lock().expect("ServerObserver queue").clear();
    }
}
impl PacketListener for AntiCheatDetectService {
    fn incoming(&self, event: &mut PacketEvent<'_, C>) {
        if !event.context.original() {
            return;
        }
        let mut o = self.observation.lock().expect("ServerObserver");
        match event.packet {
            C::Login(_) => o.login(),
            C::KeepAlive(p) => o.keep_alive(p.id),
            C::Ping(p) if o.ping(p.id) => self
                .completed
                .lock()
                .expect("ServerObserver queue")
                .push(o.clone()),
            _ => {}
        }
    }
    fn incoming_accepted(&self, _context: PacketContext, packet: &C) {
        if let C::CustomPayload(p) = packet
            && matches!(
                p.channel.as_str(),
                "minecraft:register" | "minecraft:unregister" | "register" | "unregister"
            )
        {
            self.observation.lock().expect("ServerObserver").channels(
                matches!(p.channel.as_str(), "minecraft:register" | "register"),
                &p.data,
            );
        }
    }
}
impl LiquidBounceHooks {
    fn anti_cheat_detect_report(&self, host: &mut dyn Host, observation: Observation) {
        let address = host.client_state().and_then(|s| s.server_address);
        let Some(guess) = observation.guess(address.as_deref()) else {
            return;
        };
        host.client_anti_cheat_detect_chat(guess.tag, guess.certainty);
    }
    pub(crate) fn anti_cheat_detect_enabled(&self, host: &mut dyn Host) {
        let observation = self
            .state()
            .behaviors
            .borrow()
            .anti_cheat_detect
            .observation
            .lock()
            .expect("ServerObserver")
            .clone();
        self.anti_cheat_detect_report(host, observation);
    }
    pub(crate) fn anti_cheat_detect_tick(&self, host: &mut dyn Host) {
        let (service, enabled) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let id = b.query_ids["AntiCheatDetect"];
            (
                b.anti_cheat_detect.clone(),
                state.registry.module(id).is_some_and(|m| m.enabled()),
            )
        };
        let completed =
            std::mem::take(&mut *service.completed.lock().expect("ServerObserver queue"));
        if enabled {
            for observation in completed {
                self.anti_cheat_detect_report(host, observation);
            }
        }
    }
}
