pub(super) fn native_client_fake_lag_query(&self,text:&str)->String {
    if !self.native_window()||!self.native_client_slot_owner(){return "null".into();}
    self.fake_lag_query(text)
}
pub(super) fn native_client_fake_lag_capture(&self) {
    if self.native_window()&&self.native_client_slot_owner(){mod_api::dispatch_client_fake_lag_capture(self,mod_api::mods());}
}
