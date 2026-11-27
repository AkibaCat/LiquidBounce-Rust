    pub(super) fn native_client_combine_mobs_frame(&self){if self.native_window(){mod_api::dispatch_client_combine_mobs_frame(self,mod_api::mods());}}
    pub(super) fn native_client_combine_mobs(&self,kind:&str,group:u8,baby:bool,x:f64,y:f64,z:f64,action:u8,skip:&mut bool,count:&mut i32){if self.native_window(){mod_api::dispatch_client_combine_mobs(self,mod_api::mods(),kind,group,baby,x,y,z,action,skip,count);}}
    pub(super) fn client_combine_mobs_skip(&self,kind:&str,group:u8,baby:bool,position:horizon_core::geometry::Vec3)->bool {
        let (mut skip,mut count)=(false,1);self.native_client_combine_mobs(kind,group,baby,position.x,position.y,position.z,0,&mut skip,&mut count);skip
    }
