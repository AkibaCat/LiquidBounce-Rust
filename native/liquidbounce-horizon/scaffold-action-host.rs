// SPDX-License-Identifier: GPL-3.0-or-later
    pub(super) fn native_client_scaffold_action(&mut self, request: &str) -> String {
        if !self.native_window() || !self.native_client_slot_owner() { return "null".into(); }
        self.scaffold_action_query(request).unwrap_or_else(|| "null".into())
    }
    pub(super) fn native_client_scaffold_success(&mut self, placed: [i32;3], simulated: bool) {
        if !self.native_window() || !self.native_client_slot_owner() { return; }
        mod_api::dispatch_client_scaffold_success(self, mod_api::mods(), placed, simulated);
    }
