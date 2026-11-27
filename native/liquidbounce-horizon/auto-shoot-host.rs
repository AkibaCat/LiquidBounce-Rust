pub(super) fn native_client_auto_shoot_query(&mut self, text: &str) -> String {
    if !self.native_window() || !self.native_client_slot_owner() { return "null".into(); }
    self.auto_shoot_query(text)
}
