// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
};
use horizon_client::{
    inventory_move as bridge,
    mod_api::{ClientDirectionalInputEvent, Host, ServerboundPlayPacket as S},
};
use liquidbounce_behaviors::inventory_move::Settings;
use std::sync::Mutex;
pub const TIMER_PROVIDER: u64 = 0x494e_564d_4f56;
#[derive(Default)]
pub struct InventoryMoveService {
    settings: Mutex<Settings>,
}
impl InventoryMoveService {
    pub fn settings(&self) -> Settings {
        *self.settings.lock().expect("InventoryMove")
    }
    pub fn update(&self, f: impl FnOnce(&mut Settings)) {
        f(&mut self.settings.lock().expect("InventoryMove"));
    }
    pub fn publish(&self, running: bool, server_open: bool) {
        let s = self.settings();
        bridge::configure(
            running,
            [
                i32::from(s.behavior),
                i32::from(s.passthrough_sneak),
                i32::from(s.sprint_control),
                i32::from(s.client_sprint),
                i32::from(s.server_sprint),
                i32::from(s.sneak_control),
                i32::from(s.client_sneak),
                i32::from(s.timer),
                s.speed.to_bits() as i32,
                i32::from(s.blink),
                s.maximum_time,
            ],
            server_open,
        );
    }
}
impl PacketListener for InventoryMoveService {
    fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
        if event.context.original() && bridge::delay(event.context.session, event.packet) {
            event.cancel();
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn inventory_move_input(
        &self,
        host: &mut dyn Host,
        e: &mut ClientDirectionalInputEvent,
    ) {
        let keys = host.client_inventory_move_keys();
        let mut input = [
            e.forward, e.backward, e.left, e.right, e.jump, e.shift, e.sprint,
        ];
        for i in 0..7 {
            if bridge::handles(i) {
                input[i] = keys[i];
            }
        }
        [
            e.forward, e.backward, e.left, e.right, e.jump, e.shift, e.sprint,
        ] = input;
    }
    pub(crate) fn inventory_move_final(&self, e: &mut ClientDirectionalInputEvent) {
        [
            e.forward, e.backward, e.left, e.right, e.jump, e.shift, e.sprint,
        ] = bridge::final_input([
            e.forward, e.backward, e.left, e.right, e.jump, e.shift, e.sprint,
        ]);
    }
    pub(crate) fn inventory_move_tick(&self, host: &mut dyn Host) {
        host.client_inventory_move_context();
        let (screen, running, timer, speed, start, time) = bridge::snapshot();
        let timer_service = self.state().behaviors.borrow().timer.clone();
        timer_service.set_provider_running(TIMER_PROVIDER, running);
        if running && timer && screen.container {
            timer_service
                .state
                .lock()
                .expect("Timer")
                .timer
                .request_speed(speed, 30, TIMER_PROVIDER, 1);
        }
        if start {
            let duration = format!("{}s", time / 1000);
            self.inventory_move_notify(
                &format!("You are now lagging for {duration}."),
                &format!("你将开始 {duration} 的网络滞后。"),
                &format!("你將開始 {duration} 的網路滯後。"),
            );
        }
        if host.client_inventory_move_expired() {
            host.client_inventory_close(0, true);
            host.client_inventory_move_context();
            self.inventory_move_notify(
                "You are no longer lagging and we closed your screen.",
                "你不再滞后，我们关闭了你的背包界面。",
                "你不再滯後，我們關閉了你的背包畫面。",
            );
        }
    }
    fn inventory_move_notify(&self, en: &str, cn: &str, tw: &str) {
        let lang = horizon_client::lang::code();
        let normalized = lang.to_ascii_lowercase().replace('-', "_");
        let message = match normalized.as_str() {
            "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => tw,
            "zh_cn" | "zh_sg" | "zh_hans" => cn,
            _ => en,
        };
        #[cfg(feature = "webview")]
        {
            let title =
                liquidbounce_interop::localization::display_module_name(&lang, "InventoryMove");
            self.state().events.borrow_mut().push((
                "notification".into(),
                serde_json::json!({"title":title,"message":message,"severity":"INFO"}),
            ));
        }
        #[cfg(not(feature = "webview"))]
        eprintln!("[LiquidBounce] InventoryMove: {message}");
    }
}
