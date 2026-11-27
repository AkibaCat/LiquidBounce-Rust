pub(super) fn native_client_bed_defender_query(&mut self,text:&str)->String{if self.native_window(){self.bed_defender_query(text).unwrap_or_else(||"null".into())}else{"null".into()}}
pub(super) fn native_client_bed_defender(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_bed_defender(self,mod_api::mods(),input,output);}}
