// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
};
use horizon_client::mod_api::{ClientDirectionalInputEvent, Host, ServerboundPlayPacket as S};
use liquidbounce_behaviors::sneak::Sneak;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};
#[derive(Default)]
pub struct SneakService {
    pub settings: Mutex<Sneak>,
    pub moving: AtomicBool,
    running: AtomicBool,
}
impl SneakService {
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
    }
    pub fn settings(&self) -> Sneak {
        *self.settings.lock().expect("Sneak settings")
    }
}
impl PacketListener for SneakService {
    fn outgoing(&self, e: &mut PacketEvent<'_, S>) {
        if let S::PlayerInput(p) = e.packet {
            p.flags = self.settings().force_packet(
                self.running.load(Ordering::Acquire),
                self.moving.load(Ordering::Acquire),
                p.flags,
            );
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn sneak_input(&self, host: &mut dyn Host, e: &mut ClientDirectionalInputEvent) {
        let (service, running) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            (b.sneak.clone(), b.query_running(&state.registry, "Sneak"))
        };
        let Some([moving, immune]) = host.client_sneak_context() else {
            return;
        };
        service.moving.store(moving, Ordering::Release);
        let settings = service.settings();
        if settings.mode != 0 || !settings.allowed(running, moving) {
            return;
        }
        let on_magma = !settings.magma_only
            || (!immune
                && host.client_sneak_magma([
                    e.forward, e.backward, e.left, e.right, e.jump, e.shift, e.sprint,
                ]));
        if settings.legit(running, moving, immune, on_magma) {
            e.shift = true;
        }
    }
    pub(crate) fn sneak_protocol_warning(&self) {
        let language = horizon_client::lang::code()
            .to_ascii_lowercase()
            .replace('-', "_");
        let (title, message) = match language.as_str() {
            "zh_cn" | "zh_sg" | "zh_hans" => {
                ("协议错误", "此模式只能在早于 1.21.6 的服务器版本上使用。")
            }
            "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => {
                ("協定錯誤", "此模式只能在早於 1.21.6 的伺服器版本上使用。")
            }
            _ => (
                "Protocol Error",
                "This mode can only be used on server with version earlier than 1.21.6.",
            ),
        };
        #[cfg(feature = "webview")]
        self.state().events.borrow_mut().push((
            "notification".into(),
            serde_json::json!({"title":title,"message":message,"severity":"ERROR"}),
        ));
        #[cfg(not(feature = "webview"))]
        eprintln!("[LiquidBounce] {title}: {message}");
    }
}
