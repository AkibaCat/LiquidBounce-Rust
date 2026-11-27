    pub(super) fn native_client_block_outline(&self,input:&str,geometry:&mut String,active:&mut bool){if self.native_window(){mod_api::dispatch_client_block_outline(self,mod_api::mods(),input,geometry,active);}}
    pub(super) fn client_block_outline_active(&self)->bool{let mut active=false;self.native_client_block_outline("",&mut String::new(),&mut active);active}
