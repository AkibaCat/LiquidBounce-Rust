    pub(super) fn native_client_damage_context(&self)->Option<[f64;4]> {
        if !self.native_client_slot_owner(){return None;}
        let player=self.player.as_ref()?;
        Some([player.position.x,player.position.y,player.position.z,if player.collision.horizontal{1.0}else{0.0}])
    }
    pub(super) fn native_client_damage_send(&mut self,position:[f64;3],ground:bool,collision:bool) {
        if !self.native_client_slot_owner()||self.player.is_none(){return;}
        self.send(horizon_protocol::packets::play::Serverbound::MovePlayerPos(horizon_protocol::packets::play::MovePlayerPos{x:position[0],y:position[1],z:position[2],flags:u8::from(ground)|(u8::from(collision)<<1)}));
    }
    pub(super) fn native_client_damage_velocity(&mut self,value:f64) {
        if !self.native_client_slot_owner(){return;}
        if let Some(player)=self.player.as_mut(){player.velocity.y=value;}
    }
