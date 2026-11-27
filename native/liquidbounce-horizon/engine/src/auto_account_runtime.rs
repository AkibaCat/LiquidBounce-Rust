// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::auto_account::Operation;
impl LiquidBounceHooks {
    fn auto_account_running(&self) -> bool {
        let state = self.state();
        let modules = state.behaviors.borrow();
        state
            .registry
            .enabled_without_world(modules.query_ids["AutoAccount"])
    }
    pub(crate) fn auto_account_message(&self, source: u8, message: &str) {
        let running = self.auto_account_running();
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if let Some(id) = modules.auto_account.message(running, source, message) {
            modules
                .coroutine_ticker
                .register(WaitOwner::AutoAccount, id);
        }
    }
    fn auto_account_send(&self, host: &mut dyn Host, operation: Operation) {
        let command = self
            .state()
            .behaviors
            .borrow()
            .auto_account
            .command(operation);
        let chinese = horizon_client::lang::code()
            .to_ascii_lowercase()
            .replace('-', "_");
        let text = match (chinese.as_str(), operation) {
            ("zh_cn" | "zh_sg" | "zh_hans", Operation::Login) => "登录",
            ("zh_cn" | "zh_sg" | "zh_hans", Operation::Register) => "注册",
            ("zh_tw" | "zh_hk" | "zh_mo" | "zh_hant", Operation::Login) => "登入",
            ("zh_tw" | "zh_hk" | "zh_mo" | "zh_hant", Operation::Register) => "註冊",
            (_, Operation::Login) => "login",
            (_, Operation::Register) => "register",
        };
        host.chat(text);
        host.client_auto_account_send(&command);
    }
    pub(crate) fn auto_account_callback(&self, host: &mut dyn Host, id: u64) -> bool {
        let running = self.auto_account_running();
        let connected = host.client_auto_account_connected();
        let step = self.state().behaviors.borrow_mut().auto_account.callback(
            id,
            running,
            connected,
            liquidbounce_behaviors::random::random_i32_inclusive,
        );
        if let Some(id) = step.wait {
            self.state()
                .behaviors
                .borrow()
                .coroutine_ticker
                .register(WaitOwner::AutoAccount, id);
        }
        if let Some(operation) = step.operation {
            self.auto_account_send(host, operation);
            self.state().behaviors.borrow_mut().auto_account.sent();
        }
        step.keep
    }
    pub(crate) fn auto_account_command(&self, host: &mut dyn Host, message: &str) -> bool {
        let mut parts = message.split_whitespace();
        if parts.next() != Some(".autoaccount") {
            return false;
        }
        let operation = match (parts.next(), parts.next()) {
            (Some("login"), None) => Some(Operation::Login),
            (Some("register"), None) => Some(Operation::Register),
            _ => None,
        };
        let in_game = host
            .client_state()
            .is_some_and(|c| c.world.is_some() && c.logged_in && c.eye_position.is_some());
        if let Some(operation) = operation.filter(|_| in_game) {
            self.auto_account_send(host, operation);
        } else {
            host.chat(".autoaccount login | register");
        }
        true
    }
}
