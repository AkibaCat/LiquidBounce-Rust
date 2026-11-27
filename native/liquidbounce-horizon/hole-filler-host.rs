pub(super) fn native_client_hole_filler_query(&mut self,text:&str)->String{if self.native_window(){self.hole_filler_query(text).unwrap_or_else(||"null".into())}else{"null".into()}}
pub(super) fn native_client_hole_filler(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_hole_filler(self,mod_api::mods(),input,output);}}
