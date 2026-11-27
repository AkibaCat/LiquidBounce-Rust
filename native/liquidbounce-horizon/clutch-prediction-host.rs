    pub(super) fn native_client_clutch_query(&mut self, request: &str) -> String {
        if !self.native_window() || !self.native_client_slot_owner() {
            return "null".into();
        }
        self.clutch_query(request)
    }
