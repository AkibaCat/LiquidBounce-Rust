    pub(super) fn native_client_logoff_spot(&mut self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_logoff_spot(self,mod_api::mods(),input,output);}}
    pub(super) fn native_client_logoff_spot_clear(&mut self){self.client_logoff_clear();}
