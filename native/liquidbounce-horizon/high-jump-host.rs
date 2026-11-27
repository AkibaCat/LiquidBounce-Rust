    pub(super) fn native_client_high_jump_state(&self)->Option<[f64;3]>{
        if !self.native_window()||self.lan_acting_uuid.is_some(){return None;}
        let p=self.player.as_ref().filter(|p|p.native_client_queries_allowed())?;
        Some([if p.ground.on_ground{1.0}else{0.0},p.fall_distance,f64::from(p.health.tick_count)])
    }
    pub(super) fn native_client_high_jump_vertical(&mut self,y:f64){
        if !self.native_window()||self.lan_acting_uuid.is_some(){return;}
        if let Some(p)=self.player.as_mut().filter(|p|p.native_client_queries_allowed()){p.velocity.y=y;}
    }
