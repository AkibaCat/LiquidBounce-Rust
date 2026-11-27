    pub(super) fn native_client_mob_owners(&self,mask:&mut u8){if self.native_window(){mod_api::dispatch_client_mob_owners(self,mod_api::mods(),mask);}}
    pub(super) fn client_mob_owners_begin(&self){let mut mask=0;self.native_client_mob_owners(&mut mask);horizon_client::mob_owners::begin(mask);}
