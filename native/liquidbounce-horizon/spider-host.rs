    pub(super) fn native_client_spider_context(&self)->Option<[bool;2]>{
        if !self.native_client_slot_owner(){return None;}let p=self.player.as_ref()?;Some([p.collision.horizontal,p.on_climbable])
    }
    pub(super) fn native_client_spider_velocity(&mut self,value:[f64;3],all:bool){
        if !self.native_client_slot_owner(){return;}if let Some(p)=self.player.as_mut(){if all{p.velocity=horizon_core::geometry::Vec3::new(value[0],value[1],value[2]);}else{p.velocity.y=value[1];}}
    }
    pub(super) fn native_client_spider_player_sync(&self){
        self.sync_native_spider_player();
    }
    pub(super) fn sync_native_spider_player(&self){
        if !self.native_client_slot_owner(){return;}if let Some(p)=self.player.as_ref(){horizon_world::spider::player(p.position.y,p.shift_key_down,p.ground.on_ground);}
    }
