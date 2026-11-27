    pub(super) fn native_client_auto_mob_heal_query(&mut self, text: &str) -> String {
        if !self.native_client_slot_owner() { return "null".into(); }
        self.auto_mob_heal_query(text)
    }
