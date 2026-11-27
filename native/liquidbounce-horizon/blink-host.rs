    pub(super) fn native_client_blink_control(&mut self,action:&str,position:&[f64]) {
        if !self.native_window()||self.lan_acting_uuid.is_some(){return;}
        if action=="remove" {
            if let Some(id)=self.client_blink_dummy.take(){
                self.mobs.retain(|m|m.id!=id);
                self.persist.uuids.remove(&id);
                self.client_blink_remove_identity(id as i32);
            }
            return;
        }
        if !self.native_client_slot_owner(){return;}
        if action=="reset" {
            if let Some(p)=self.player.as_mut() {if let [x,y,z]=position {p.position=horizon_core::geometry::Vec3::new(*x,*y,*z);}}
            return;
        }
        let Some(p)=self.player.as_ref()else{return;};let at=p.position;
        if action=="move" {
            if let Some(id)=self.client_blink_dummy&&let Some(m)=self.mobs.iter_mut().find(|m|m.id==id){
                m.position=at;m.yaw=self.yaw;m.head_pitch=self.pitch;
            }
            return;
        }
        if action!="create" {return;}
        self.native_client_blink_control("remove",&[]);
        let Some(kind)=self.res.player_type().clone()else{return;};
        static NEXT:std::sync::atomic::AtomicI32=std::sync::atomic::AtomicI32::new(-1);
        let id=NEXT.fetch_sub(1,std::sync::atomic::Ordering::Relaxed)as u32;
        let mut m=horizon_client::mobs::Mob::new(id,kind,at,self.yaw);
        super::remote_mob_data::client_defaults(&mut m);
        m.head_pitch=self.pitch;m.previous_head_pitch=self.pitch;
        m.head_yaw=self.player_model.as_ref().map_or(self.yaw,|p|p.head_yaw);m.previous_head_yaw=m.head_yaw;
        if let Some(own)=self.player_model.as_ref(){m.skin=own.skin.clone();m.slim=own.slim;m.skin_3d=own.skin_3d.clone();m.cape=own.cape.clone();}
        m.persistent=true;
        self.persist.uuids.insert(id,horizon_client::save::random_uuid());
        self.mobs.push(m);self.client_blink_dummy=Some(id);
        self.client_blink_add_identity(id as i32);
    }
    pub(super) fn native_client_blink_dummy(&self)->Option<i32>{self.client_blink_dummy.map(|id|id as i32)}
    pub(super) fn native_client_blink_movement(&mut self){if self.native_client_slot_owner(){mod_api::dispatch_client_blink_movement(self,mod_api::mods());}}
