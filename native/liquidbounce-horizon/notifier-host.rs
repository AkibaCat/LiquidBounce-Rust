pub(super) fn native_client_notifier_snapshot(&self) -> Option<String> {
    if !self.client_auto_tool_player() {
        return None;
    }
    Some(self.client_notifier_snapshot().to_string())
}
pub(super) fn native_client_notifier(&mut self, input: &str) {
    if self.native_window() {
        mod_api::dispatch_client_notifier(self, mod_api::mods(), input);
    }
}
