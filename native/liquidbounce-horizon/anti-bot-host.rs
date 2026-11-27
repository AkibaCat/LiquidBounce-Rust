    pub(super) fn native_client_anti_bot_snapshot(&self)->Option<String>{
        if !self.native_client_slot_owner(){return None;}Some(self.client_anti_bot_world().to_string())
    }
    pub(super) fn native_client_anti_bot(&mut self,input:&str){
        if self.native_client_slot_owner(){mod_api::dispatch_client_anti_bot(self,mod_api::mods(),input);}
    }
