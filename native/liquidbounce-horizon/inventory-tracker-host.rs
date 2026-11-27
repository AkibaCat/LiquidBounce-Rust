    pub(super) fn native_client_inventory_tracker(&mut self, input: &str, output: &mut String) {
        if self.client_auto_tool_player() {
            mod_api::dispatch_client_inventory_tracker(self, mod_api::mods(), input, output);
        }
    }
    pub(super) fn native_client_inventory_tracker_query(&mut self, request: &str) -> String {
        self.client_inventory_tracker_query(request)
    }
