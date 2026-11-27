pub(super) fn native_client_extinguish_query(&mut self, text: &str) -> String {
    if !self.native_window() {
        return "null".into();
    }
    self.extinguish_query(text).unwrap_or_else(|| "null".into())
}
pub(super) fn native_client_extinguish_success(&self, position: [i32; 3]) {
    if self.native_window() {
        mod_api::dispatch_client_extinguish_success(self, mod_api::mods(), position);
    }
}
