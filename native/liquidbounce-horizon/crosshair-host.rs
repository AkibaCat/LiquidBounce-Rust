    pub(super)fn native_client_crosshair(&self,input:&str,geometry:&mut String){if self.native_window(){mod_api::dispatch_client_crosshair(self,mod_api::mods(),input,geometry);}}
