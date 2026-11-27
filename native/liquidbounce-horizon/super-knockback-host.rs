    pub(super) fn native_client_super_knockback_context(&self,target:u32)->Option<serde_json::Value>{
        use horizon_core::geometry::Axis;
        if !self.native_client_slot_owner(){return None;}
        let p=self.player.as_ref()?;
        let local=self.remote.as_ref().map_or(0,|r|r.entity_id)as u32;
        let handle=self.native_entity(target);
        let living=handle.and_then(|e|self.native_living(e)).is_some();
        let hurt=if target==local{p.health.hurt.hurt_time}else if let Some(m)=self.mobs.iter().find(|m|m.id==target){m.hurt_time}else if let Some(g)=self.lan.iter().flat_map(|h|h.hub.guests.iter()).find(|g|g.entity_id as u32==target){g.motion.as_ref().map_or(0,|p|p.health.hurt.hurt_time)}else if let Some(d)=self.dragon.as_ref().filter(|d|d.id==target){d.hurt_time}else{0};
        let facing=handle.and_then(|h|Some((self.native_entity_position(h)?,self.native_entity_rotation(h)?))).is_some_and(|(position,[yaw,pitch])|{
            let direction=super::client_rotation_direction(yaw,pitch);
            direction[0]*(p.position.x-position[0])+direction[1]*(p.position.y-position[1])+direction[2]*(p.position.z-position[2])>=0.0
        });
        let at=p.position;let block=self.res.facts().get(self.world.block(at.x.floor()as i32,at.y.floor()as i32,at.z.floor()as i32));
        let water=p.fluids.was_touching_water;let lava=p.is_in_lava();
        let bounds=p.bounds();let mut web=false;
        for x in bounds.min(Axis::X).floor()as i32..=bounds.max(Axis::X).floor()as i32{for y in bounds.min(Axis::Y).floor()as i32..=bounds.max(Axis::Y).floor()as i32{for z in bounds.min(Axis::Z).floor()as i32..=bounds.max(Axis::Z).floor()as i32{
            if &*self.res.facts().get(self.world.block(x,y,z)).name=="minecraft:cobweb"{web=true;}
        }}}
        let effects=["levitation","blindness","slow_falling"].into_iter().any(|id|self.potions.effects.get(id).is_some());
        // PlayerMotion's LivingMotionWorld.no_gravity is false for this actual
        // local player. Remote entity NoGravity metadata is unrelated.
        let critical_allowed=!water&&!lava&&self.riding.is_none()&&!web&&!effects&&!p.on_climbable&&!self.hands_busy()&&!p.flying&&!p.ground.on_ground;
        let delay=(1.0/self.main_hand_attack_speed()*20.0)as f32;
        Some(serde_json::json!({"forward":p.move_vector[1],"sideways":p.move_vector[0],"sprinting":p.sprinting,"wasSprinting":self.remote.as_ref().map_or(p.client_super_knockback_was_sprinting,|r|r.was_sprinting),"onGround":p.ground.on_ground,"waterOrBubble":water||&*block.name=="minecraft:bubble_column","living":living,"hurtTime":hurt,"facing":facing,"criticalAllowed":critical_allowed,"fallDistance":p.fall_distance,"legacy":self.legacy_rules(),"attackDelay":delay,"tickRate":self.clock.rate()}))
    }
    pub(super) fn native_client_super_knockback_packet(&mut self,start:bool){
        if !self.native_client_slot_owner(){return;}
        let id=self.remote.as_ref().map_or(0,|r|r.entity_id);
        self.send(horizon_protocol::packets::play::Serverbound::PlayerCommand(horizon_protocol::packets::play::PlayerCommand{id,action:if start{1}else{2},data:0}));
    }
    pub(super) fn native_client_super_knockback_mark_sprint(&mut self){
        if !self.native_client_slot_owner(){return;}
        if let Some(p)=self.player.as_mut(){let _=p.set_sprinting(true);p.client_super_knockback_was_sprinting=true;}
        if let Some(r)=self.remote.as_mut(){r.was_sprinting=true;}
    }
    pub(super) fn sync_native_super_knockback_sprint(&mut self){
        if self.remote.is_some()||!self.native_client_slot_owner(){return;}
        let mut sprinting=self.player.as_ref().is_some_and(|p|p.sprinting);
        self.native_client_sprint_network(&mut sprinting);
        if let Some(p)=self.player.as_mut(){p.client_super_knockback_was_sprinting=sprinting;}
    }
