pub(super) fn native_client_auto_bow_query(&mut self, text: &str) -> String {
    if !self.native_window() || !self.native_client_slot_owner() {
        return "null".into();
    }
    self.auto_bow_query(text)
}
pub(super) fn native_client_auto_bow_update_use(&mut self) -> bool {
    if !self.native_client_slot_owner() {
        return false;
    }
    if self.using.is_none() {
        return false;
    }
    let _updating = super::auto_bow::begin_update();
    if self.remote.is_some() {
        self.remote_tick_using();
    } else {
        self.tick_using();
    }
    true
}
