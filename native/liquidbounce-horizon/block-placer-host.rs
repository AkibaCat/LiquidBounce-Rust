pub(super) fn native_client_block_placer_query(&mut self,text:&str)->String{
    if !self.native_window(){return "null".into();}
    self.block_placer_query(text).unwrap_or_else(||"null".into())
}
pub(super) fn native_client_liquid_filler_query(&mut self,text:&str)->String{
    if !self.native_window(){return "null".into();}
    self.liquid_filler_query(text).unwrap_or_else(||"null".into())
}
pub(super) fn native_client_block_placer_success(&self,requester:u64,position:[i32;3]){
    if !self.native_window(){return;}
    // Horizon 26.3's event schema carries only requester and position. The
    // behavior runtime uses a unit block bounds fallback for post-place state
    // updates; placement itself already succeeded against the real shape.
    mod_api::dispatch_client_block_placer_success(self,mod_api::mods(),requester,position);
}
pub(super) fn native_client_block_placer_after_move(&mut self){
    if self.native_window(){mod_api::dispatch_client_block_placer_after_move(self,mod_api::mods());}
}
pub(super) fn native_client_block_placer_removed(&mut self,id:i32){
    if self.native_window(){mod_api::dispatch_client_block_placer_removed(self,mod_api::mods(),id);}
}
