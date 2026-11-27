// SPDX-License-Identifier: GPL-3.0-or-later
pub(super) fn native_client_anti_void_query(&mut self, text: &str) -> Option<String> {
    self.anti_void_query(text)
}
pub(super) fn native_client_anti_void_clear(&self, level: i32) -> bool {
    self.anti_void_clear(level)
}
pub(super) fn native_client_anti_void_raise(&mut self, height: f64) {
    self.anti_void_raise(height);
}
pub(super) fn native_client_anti_void_network(&mut self, pre: bool, y: &mut f64) {
    if self.native_client_slot_owner() {
        mod_api::dispatch_client_anti_void_network(self, mod_api::mods(), pre, y);
    }
}
