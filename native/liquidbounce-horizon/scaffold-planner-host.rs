// SPDX-License-Identifier: GPL-3.0-or-later
    pub(super) fn native_client_scaffold_planner(&mut self, request: &str) -> String {
        if !self.native_window() || !self.native_client_slot_owner() { return "null".into(); }
        self.scaffold_planner_query(request).unwrap_or_else(|| "null".into())
    }
