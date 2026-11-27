    pub(super) fn native_client_true_sight_entity(&self,entity:i32,body:&mut u32,feature:&mut u32,apply:&mut bool) {
        if self.native_window(){mod_api::dispatch_client_true_sight_entity(self,mod_api::mods(),entity,body,feature,apply);}
    }
    pub(super) fn client_true_sight_colors(&self,entity:i32)->Option<[u32;2]> {
        let (mut body,mut feature,mut apply)=(0,0,false);self.native_client_true_sight_entity(entity,&mut body,&mut feature,&mut apply);apply.then_some([body,feature])
    }
    pub(super) fn native_client_true_sight_markers(&self,mask:&mut u8) {
        if self.native_window(){mod_api::dispatch_client_true_sight_markers(self,mod_api::mods(),mask);}
    }
