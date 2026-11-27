    pub(super) fn native_client_auto_buff_query(&mut self,value:&str)->String{if !self.native_window()||!self.native_client_slot_owner(){return "null".into();}self.auto_buff_query(value)}
    pub(super) fn native_client_auto_buff_packet(&mut self,yaw:f32,pitch:f32){
        if !self.native_client_slot_owner(){return;}let Some(p)=self.player.as_ref()else{return;};let at=p.position;let flags=u8::from(p.ground.on_ground)|(u8::from(p.collision.horizontal)<<1);
        self.send(horizon_protocol::packets::play::Serverbound::MovePlayerPosRot(horizon_protocol::packets::play::MovePlayerPosRot{x:at.x,y:at.y,z:at.z,yaw,pitch,flags}));
    }
    pub(super) fn native_client_auto_buff_drop(&mut self){if !self.native_client_slot_owner(){return;}let selected=self.selected;self.selected=self.client_hotbar_slot();if self.remote.is_some(){self.remote_drop(true);}else{self.drop_held(true);}self.selected=selected;}
