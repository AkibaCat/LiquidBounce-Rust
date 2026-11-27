// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientDirectionalInputEvent, Host};
use liquidbounce_behaviors::auto_dodge::Settings;
use serde_json::{Value, json};
pub const TIMER_PROVIDER: u64 = 0x4155_544f_444f_4447;
#[derive(Default)]
pub struct AutoDodgeRuntime {
    pub settings: Settings,
    pub murder_mystery_disallows: bool,
    pub pending_jump: bool,
}
impl LiquidBounceHooks {
    pub(crate) fn auto_dodge_input(
        &self,
        host: &mut dyn Host,
        event: &mut ClientDirectionalInputEvent,
    ) -> bool {
        let (settings, running, inventory, scaffold, current, timer) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let a = &b.auto_dodge;
            let r = b.rotations.lock().expect("RotationManager").current;
            (
                a.settings.clone(),
                b.query_running(&s.registry, "AutoDodge")
                    && !b.query_running(&s.registry, "Blink")
                    && !a.murder_mystery_disallows,
                b.inventory.manager.inventory_open_server_side,
                b.query_running(&s.registry, "Scaffold"),
                r.map(|r| r.yaw),
                b.timer.clone(),
            )
        };
        timer.set_provider_running(TIMER_PROVIDER, running);
        if !running {
            return false;
        }
        let pending =
            std::mem::take(&mut self.state().behaviors.borrow_mut().auto_dodge.pending_jump);
        let input = json!({"mode":"movement","keys":[event.forward,event.backward,event.left,event.right,event.jump,event.shift,event.sprint],"ignore":settings.ignore,"rotation":settings.rotation,"inventoryOpen":inventory,"scaffold":scaffold,"currentYaw":current});
        let result = host.client_auto_dodge_query(&input.to_string());
        if let Ok(v) = serde_json::from_str::<Value>(&result) {
            if v["keys"].is_array() {
                [event.forward, event.backward, event.left, event.right] =
                    std::array::from_fn(|i| v["keys"][i] == true);
                if v["jump"] == true && settings.jump {
                    self.state().behaviors.borrow_mut().auto_dodge.pending_jump = true;
                }
                if v["timer"] == true && settings.timer {
                    timer.state.lock().expect("Timer").timer.request_speed(
                        settings.speed,
                        40,
                        TIMER_PROVIDER,
                        1,
                    );
                }
            }
        }
        pending
    }
}
