    pub(super) fn native_client_old_regen_context(&mut self) -> Option<[f64;11]> {
        if !self.native_client_slot_owner() {return None;}
        let player=self.player.as_ref()?;
        let at=player.position;
        let top=player.bounds().max(horizon_core::geometry::Axis::Y);
        let mut context=[
            f64::from(player.health.health()),
            u8::from(self.creative) as f64,u8::from(player.health.dead_or_dying()) as f64,
            u8::from(player.ground.on_ground) as f64,u8::from(player.move_vector != [0.0,0.0]) as f64,
            u8::from(self.potions.effects.get("regeneration").is_some()) as f64,
            f64::from(player.health.food.food),u8::from(player.fire_ticks>0) as f64,
            u8::from(player.fluids.was_touching_water) as f64,0.0,0.0,
        ];
        if let Some(duration)=self.potions.effects.list.iter().filter(|effect|effect.duration != -1 && horizon_client::brewing::effect(effect.id).is_some_and(|definition|definition.category==horizon_client::brewing::Category::Harmful)).map(|effect|effect.duration).max() {
            context[9]=1.0;context[10]=f64::from(duration);
        }
        if context[8]==0.0 && self.in_rain(at,top) {context[8]=1.0;}
        Some(context)
    }
    pub(super) fn native_client_old_regen_send(&mut self,kind:u8) {
        use horizon_protocol::packets::play::{self,Serverbound};
        if !self.native_client_slot_owner() {return;}
        let Some(player)=self.player.as_ref() else {return;};
        let flags=u8::from(player.ground.on_ground)|(u8::from(player.collision.horizontal)<<1);
        let p=player.position;let yaw=player.yaw;let pitch=player.pitch;
        let packet=match kind {
            0=>Serverbound::MovePlayerStatus(play::MovePlayerStatus{flags}),
            1=>Serverbound::MovePlayerPos(play::MovePlayerPos{x:p.x,y:p.y,z:p.z,flags}),
            2=>Serverbound::MovePlayerRot(play::MovePlayerRot{yaw,pitch,flags}),
            _=>Serverbound::MovePlayerPosRot(play::MovePlayerPosRot{x:p.x,y:p.y,z:p.z,yaw,pitch,flags}),
        };
        self.send(packet);
    }
