// SPDX-License-Identifier: GPL-3.0-or-later
pub(super) fn native_client_no_slow_context(&self) -> Option<[f64; 15]> {
    self.no_slow_context()
}
pub(super) fn native_client_no_slow_send(&mut self, value: &str) {
    self.no_slow_send(value);
}
pub(super) fn native_client_no_slow_slowness(&mut self, value: f32) {
    self.no_slow_slowness(value);
}
pub(super) fn native_client_no_slow_network(&mut self, pre: bool) {
    if self.native_client_slot_owner() {
        horizon_client::no_slow_bridge::set_context(self.no_slow_context());
        mod_api::dispatch_client_no_slow_network(self, mod_api::mods(), pre);
    }
}
