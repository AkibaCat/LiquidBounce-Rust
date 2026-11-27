// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::{ClientLogoffSpotEvent, Host, ServerboundPlayPacket as S};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

pub struct LogoffSpotService {
    pub send_in_chat: AtomicBool,
    running: AtomicBool,
    ids: Mutex<BTreeSet<i32>>,
}
impl Default for LogoffSpotService {
    fn default() -> Self {
        Self {
            send_in_chat: AtomicBool::new(true),
            running: AtomicBool::new(false),
            ids: Mutex::default(),
        }
    }
}
impl LogoffSpotService {
    pub fn send_in_chat(&self) -> bool {
        self.send_in_chat.load(Ordering::Acquire)
    }
    pub fn set_send_in_chat(&self, value: bool) {
        self.send_in_chat.store(value, Ordering::Release);
    }
    pub fn set_running(&self, value: bool) {
        self.running.store(value, Ordering::Release);
    }
    pub fn clear(&self) {
        self.ids.lock().expect("LogoffSpot IDs").clear();
    }
    fn replace(&self, values: impl IntoIterator<Item = i32>) {
        *self.ids.lock().expect("LogoffSpot IDs") = values.into_iter().collect();
    }
    pub fn is_fake(&self, id: i32) -> bool {
        self.running.load(Ordering::Acquire)
            && self.ids.lock().expect("LogoffSpot IDs").contains(&id)
    }
}
impl PacketListener for LogoffSpotService {
    fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
        let target = match event.packet {
            S::Attack(p) => Some(p.id),
            S::Interact(p) => Some(p.id),
            S::SpectatorAction(p) => p.target,
            _ => None,
        };
        if target.is_some_and(|id| self.is_fake(id)) {
            event.cancel();
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn logoff_spot(&self, host: &mut dyn Host, event: &mut ClientLogoffSpotEvent<'_>) {
        let Ok(v) = serde_json::from_str::<Value>(event.input) else {
            return;
        };
        let (service, active) = {
            let state = self.state();
            let m = state.behaviors.borrow();
            (
                m.logoff_spot.clone(),
                m.query_running(&state.registry, "LogoffSpot"),
            )
        };
        service.set_running(active);
        match v["action"].as_str() {
            Some("options") => {
                *event.output =
                    json!({"running":active,"sendInChat":service.send_in_chat()}).to_string()
            }
            Some("ids") => service.replace(
                v["ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_i64().map(|i| i as i32)),
            ),
            Some("reset") => service.clear(),
            Some("message") if active && service.send_in_chat() => {
                let kind = v["kind"].as_str().unwrap_or("");
                if !matches!(kind, "disappeared" | "reappeared" | "unloaded") {
                    return;
                }
                let key = format!("liquidbounce.module.logoffSpot.messages.{kind}");
                let mut message = liquidbounce_interop::localization::describe(
                    &horizon_client::lang::code(),
                    &key,
                );
                for arg in v["args"].as_array().into_iter().flatten() {
                    let text = arg
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| arg.to_string());
                    message = message.replacen("%s", &text, 1);
                }
                host.chat(&message);
            }
            _ => {}
        }
    }
    pub(crate) fn logoff_spot_disabled(&self, host: &mut dyn Host) {
        self.state().behaviors.borrow().logoff_spot.clear();
        host.client_logoff_spot_clear();
    }
}
