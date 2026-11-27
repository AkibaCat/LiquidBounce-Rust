    pub(super) fn native_client_mace_kill_context(&self)->Option<[f64;5]>{
        if !self.native_client_slot_owner(){return None;}
        let player=self.player.as_ref()?;
        let mace=self.inventory.get(self.client_hotbar_slot()).and_then(Option::as_ref).is_some_and(|s|s.count>0&&&*s.item=="minecraft:mace");
        Some([player.position.x,player.position.y,player.position.z,u8::from(player.ground.on_ground)as f64,u8::from(mace)as f64])
    }
    pub(super) fn native_client_mace_kill_height(&self,maximum:i32)->i32{
        use horizon_core::geometry::Vec3;
        use horizon_world::{block_collision_scan::BlockCollisionScan,collision_context::{CollisionContext,CollisionEntity,CollisionItem,EntityContextKind},entity_obstruction::EntityIdentity};
        if !self.native_client_slot_owner(){return 0;}
        let Some(player)=self.player.as_ref()else{return 0;};
        // Capture the actual player box once, as determineHeight does. Only
        // block collisions count: entities and the world border are excluded.
        let bounds=player.bounds();
        let held=self.inventory.get(self.client_hotbar_slot()).and_then(Option::as_ref);
        let context=CollisionContext::for_entity(CollisionEntity{
            identity:EntityIdentity(self.remote.as_ref().map_or(0,|r|r.entity_id as u32)as u64),kind:EntityContextKind::Player,
            y:player.position.y,descending:player.shift_key_down,
            main_hand:Some(held.map_or(CollisionItem::EMPTY,|s|CollisionItem{item:&s.item,count:s.count as i32})),
        },false);
        horizon_world::spider::player(player.position.y,player.shift_key_down,player.ground.on_ground);
        let mut scene=horizon_client::collisions::SceneCollisions{world:&self.world,binding:self.collision,subject:Some(player.powder_snow_subject())};
        super::mace_kill_height::highest_clear(maximum,|height|{
            let at=bounds.move_by(Vec3::new(0.0,f64::from(height),0.0));
            BlockCollisionScan::any_collision(&mut scene,context,at,false,1_000_000).map(|hit|!hit)
        }).unwrap_or_else(|e|{eprintln!("[LiquidBounce] MaceKill collision query: {e}");0})
    }
    pub(super) fn native_client_mace_kill_warp(&mut self,position:Option<[f64;3]>,ground:bool){
        use horizon_protocol::packets::play::{self,Serverbound};
        if !self.native_client_slot_owner(){return;}
        if let Some(vehicle)=self.riding{
            if let Some(packet)=self.client_mace_kill_vehicle(vehicle,position){self.send(Serverbound::MoveVehicle(packet));}
            return;
        }
        let Some(player)=self.player.as_ref()else{return;};
        let flags=u8::from(ground)|(u8::from(player.collision.horizontal)<<1);
        let packet=match position{
            Some([x,y,z])=>Serverbound::MovePlayerPos(play::MovePlayerPos{x,y,z,flags}),
            None=>Serverbound::MovePlayerStatus(play::MovePlayerStatus{flags}),
        };
        self.send(packet);
    }
