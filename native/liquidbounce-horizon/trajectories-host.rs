    pub(super) fn native_client_trajectories(&self,input:&str,output:&mut String){
        if self.native_window(){mod_api::dispatch_client_trajectories(self,mod_api::mods(),input,output);}
    }
