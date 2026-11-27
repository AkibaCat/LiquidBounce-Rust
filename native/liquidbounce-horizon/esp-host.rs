    pub(super) fn native_client_esp(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_esp(self,mod_api::mods(),input,output);}}
