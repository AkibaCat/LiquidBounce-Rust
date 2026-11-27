pub(super) fn native_client_kill_aura_query(&mut self, text: &str) -> String {
    if !self.native_window() || !self.native_client_slot_owner() {
        return "null".into();
    }
    self.kill_aura_query(text)
}
pub(super) fn native_client_kill_aura_state(&self, enforced: &mut i32, pose: &mut bool) {
    if !self.native_window() || !self.native_client_slot_owner() {
        return;
    }
    mod_api::dispatch_client_kill_aura_state(self, mod_api::mods(), enforced, pose);
}
