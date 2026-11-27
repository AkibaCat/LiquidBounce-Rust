// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::report_helper::{
    Confirm, Player, Report, Slot, confirm_slot, format_command,
};
impl LiquidBounceHooks {
    fn report_helper_running(&self) -> bool {
        let state = self.state();
        let b = state.behaviors.borrow();
        b.query_running(&state.registry, "ReportHelper")
    }
    fn report_helper_session(&self, host: &dyn Host) {
        let identity = host.client_report_helper_session();
        self.state()
            .behaviors
            .borrow_mut()
            .report_helper
            .session(identity);
    }
    fn report_helper_send(&self, host: &mut dyn Host, name: &str) {
        let pattern = self
            .state()
            .behaviors
            .borrow()
            .report_helper
            .settings
            .pattern
            .clone();
        match format_command(&pattern, name) {
            Ok(command) => host.client_auto_account_send(&command),
            Err(error) => eprintln!("[LiquidBounce] ReportHelper format failed: {error}"),
        }
    }
    pub(crate) fn report_helper_message(&self, host: &mut dyn Host, message: &str) {
        self.report_helper_session(host);
        if !self.report_helper_running() {
            return;
        }
        // This is the actual client world player list, in insertion order;
        // it deliberately includes NPCs absent from the tab player list.
        let Some(snapshot) = host.client_report_helper_players() else {
            return;
        };
        let Some(name) = snapshot["self"].as_str() else {
            return;
        };
        let Some(players) = snapshot["players"].as_array() else {
            return;
        };
        let report = {
            let state = self.state();
            let players = players
                .iter()
                .filter_map(|p| {
                    let name = p["name"].as_str()?;
                    Some(Player {
                        name: name.into(),
                        local: p["local"].as_bool().unwrap_or(false),
                        friend: state.friends.contains(name),
                    })
                })
                .collect::<Vec<_>>();
            let mut b = state.behaviors.borrow_mut();
            b.report_helper.message(
                true,
                message,
                name,
                &players,
                liquidbounce_behaviors::random::random_i32_inclusive,
            )
        };
        match report {
            Some(Report::Wait(id)) => {
                self.state()
                    .behaviors
                    .borrow()
                    .coroutine_ticker
                    .register(WaitOwner::ReportHelperReport, id);
            }
            Some(Report::Send(name)) => self.report_helper_send(host, &name),
            None => {}
        }
    }
    pub(crate) fn report_helper_screen(&self, host: &dyn Host, screen: u64) {
        self.report_helper_session(host);
        let running = self.report_helper_running();
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        if let Some(id) = b.report_helper.screen(running, screen) {
            b.coroutine_ticker
                .register(WaitOwner::ReportHelperConfirm, id);
        }
    }
    pub(crate) fn report_helper_report_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        id: u64,
    ) -> bool {
        self.report_helper_session(host);
        let running = self.report_helper_running();
        let (keep, name) = self
            .state()
            .behaviors
            .borrow_mut()
            .report_helper
            .report_callback(id, running, in_game);
        if let Some(name) = name {
            self.report_helper_send(host, &name);
        }
        keep
    }
    pub(crate) fn report_helper_confirm_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        id: u64,
    ) -> bool {
        self.report_helper_session(host);
        let screen = self
            .state()
            .behaviors
            .borrow()
            .report_helper
            .confirm_screen(id);
        let current = screen.is_some_and(|screen| host.client_report_helper_current(screen));
        let running = self.report_helper_running();
        let step = self
            .state()
            .behaviors
            .borrow_mut()
            .report_helper
            .confirm_callback(id, running, in_game, current);
        match step {
            Confirm::Keep => true,
            Confirm::Stop => false,
            Confirm::Inspect { screen, mode } => {
                let Some(snapshot) = host.client_report_helper_screen_snapshot(screen) else {
                    return false;
                };
                let Some(container) = snapshot["container"]
                    .as_i64()
                    .and_then(|v| i32::try_from(v).ok())
                else {
                    return false;
                };
                let Some(slots) = snapshot["slots"].as_array() else {
                    return false;
                };
                let slots = slots
                    .iter()
                    .filter_map(|v| {
                        Some(Slot {
                            index: i32::try_from(v["index"].as_i64()?).ok()?,
                            item: v["item"].as_str()?.into(),
                            empty: v["empty"].as_bool()?,
                        })
                    })
                    .collect::<Vec<_>>();
                if let Some(slot) = confirm_slot(mode, &slots) {
                    // handleContainerInput uses captured screen.syncId; vanilla
                    // ignores a stale id, then clientSideCloseContainer still runs.
                    host.client_inventory_click(container, slot, 0, 0);
                    host.client_report_helper_close();
                }
                false
            }
        }
    }
}
