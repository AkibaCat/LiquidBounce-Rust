    pub(super) fn native_client_stronghold_finder(&self,input:&str,output:&mut String){
        if self.native_window(){mod_api::dispatch_client_stronghold_finder(self,mod_api::mods(),input,output);}
    }
