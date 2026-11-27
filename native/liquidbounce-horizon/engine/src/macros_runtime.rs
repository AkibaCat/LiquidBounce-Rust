// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{Host, KeyCode};
use liquidbounce_behaviors::macros::{COUNT, Slot, Start};
pub(crate) const REQUESTER: u64 = 0x4d41_4352_4f53_0001;
fn random(a: i32, b: i32) -> i32 {
    liquidbounce_behaviors::random::random_i32_inclusive(a, b)
}
impl LiquidBounceHooks {
    fn macros_running(&self) -> bool {
        let s = self.state();
        let b = s.behaviors.borrow();
        b.query_running(&s.registry, "Macros")
    }
    fn macros_schedule(&self, host: &dyn Host) {
        host.client_macros_schedule(self.state().behaviors.borrow().macros.next_wake());
    }
    fn macros_chat_run(&self, host: &mut dyn Host, id: u64) {
        loop {
            let message = self.state().behaviors.borrow_mut().macros.message(id);
            let (message, use_client) = match message {
                Ok(Some(v)) => v,
                Ok(None) => break,
                Err(error) => {
                    self.state().behaviors.borrow_mut().macros.failed(id);
                    eprintln!("[LiquidBounce] Macros: {error}");
                    break;
                }
            };
            let prefix = self.client_command_prefix();
            let result = if use_client && message.starts_with(&prefix) {
                self.client_command_execute(host, &message[prefix.len()..])
            } else {
                host.client_macros_send(&message);
                Ok(())
            };
            if let Err(error) = result {
                self.state().behaviors.borrow_mut().macros.failed(id);
                eprintln!("[LiquidBounce] Macros: {error}");
                break;
            }
            let sent = self.state().behaviors.borrow_mut().macros.sent(
                id,
                host.client_auto_tool_time(),
                random,
            );
            match sent {
                Ok(true) => {}
                Ok(false) => break,
                Err(error) => {
                    self.state().behaviors.borrow_mut().macros.failed(id);
                    eprintln!("[LiquidBounce] Macros: {error}");
                    break;
                }
            }
        }
    }
    pub(crate) fn macros_key(&self, host: &mut dyn Host, key: KeyCode) {
        if !self.macros_running() {
            return;
        }
        #[cfg(feature = "webview")]
        if self
            .browser
            .as_ref()
            .is_some_and(|b| b.lock().expect("Browser poisoned").visible)
        {
            return;
        }
        let Some(key) = horizon_client::keybinds::key_name(key) else {
            return;
        };
        let Some(v) = host.client_macros_snapshot() else {
            return;
        };
        if v["screen"].as_bool() != Some(false) {
            return;
        }
        let Some(now) = v["now"].as_i64() else {
            return;
        };
        let event = self.state().behaviors.borrow_mut().macros.new_event();
        for i in 0..COUNT * 2 {
            if !self.macros_running() {
                break;
            }
            // Each item action observes changes made by earlier macros in this event.
            let slots = if i >= COUNT {
                host.client_macros_snapshot()
                    .and_then(|v| {
                        v["slots"].as_array().map(|a| {
                            a.iter()
                                .filter_map(|v| {
                                    Some(Slot {
                                        slot: i32::try_from(v["slot"].as_i64()?).ok()?,
                                        item: v["item"].as_str()?.into(),
                                        name: v["name"].as_str()?.into(),
                                    })
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let action = self
                .state()
                .behaviors
                .borrow_mut()
                .macros
                .begin(i, event, key, now, &slots, random);
            let action = match action {
                Ok(action) => action,
                Err(error) => {
                    self.state()
                        .behaviors
                        .borrow_mut()
                        .macros
                        .failed_event(event);
                    eprintln!("[LiquidBounce] Macros: {error}");
                    break;
                }
            };
            match action {
                Some(Start::Chat(id)) => self.macros_chat_run(host, id),
                Some(Start::Item {
                    slot,
                    hold,
                    action,
                    swing,
                }) => {
                    if slot >= 0 {
                        host.client_select_slot_silently(REQUESTER, slot, hold);
                    }
                    let [yaw, pitch] = host.client_player_rotation();
                    let rotation = {
                        let s = self.state();
                        let b = s.behaviors.borrow();
                        let rotation = b
                            .rotations
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .current;
                        rotation
                    };
                    let (yaw, pitch) = rotation.map_or((yaw, pitch), |r| (r.yaw, r.pitch));
                    host.client_macros_item(
                        i32::from(slot == -1),
                        action == 1,
                        swing == 1,
                        yaw,
                        pitch,
                    );
                }
                None => {}
            }
        }
        self.macros_schedule(host);
    }
    pub(crate) fn macros_wake(&self, host: &mut dyn Host) {
        if !self.macros_running() {
            self.state().behaviors.borrow_mut().macros.cancel();
        } else {
            let due = self
                .state()
                .behaviors
                .borrow_mut()
                .macros
                .due(host.client_auto_tool_time());
            for id in due {
                self.macros_chat_run(host, id);
            }
        }
        self.macros_schedule(host);
    }
}
