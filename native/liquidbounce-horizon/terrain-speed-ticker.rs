struct TerrainEntityTicker<'g,'r>{game:&'g mut Game<'r>,input:Option<u8>}
impl horizon_client::terrain_speed_order::EntityTicks for TerrainEntityTicker<'_,'_>{
    type Error=Box<dyn std::error::Error>;
    fn roots(&self)->Vec<u32>{self.game.remote.as_ref().map(|r|r.terrain_order.roots(r.entity_id as u32)).unwrap_or_default()}
    fn removed(&self,id:u32)->bool{
        self.game.remote.as_ref().is_none_or(|r|if id==r.entity_id as u32{self.game.player.is_none()}else{!r.entities.contains_key(&(id as i32))})
    }
    fn vehicle(&self,id:u32)->Option<u32>{self.game.remote.as_ref().and_then(|r|r.passenger_vehicles.get(&id).copied())}
    fn ticking(&self,id:u32)->bool{!self.removed(id)}
    fn player(&self,id:u32)->bool{self.game.remote.as_ref().is_some_and(|r|id==r.entity_id as u32||r.level_players.contains_key(&(id as i32)))}
    fn passengers(&self,id:u32)->Vec<u32>{self.game.remote.as_ref().map(|r|r.terrain_order.passengers(id)).unwrap_or_default()}
    fn detach(&mut self,id:u32){
        if let Some(r)=self.game.remote.as_mut(){r.passenger_vehicles.remove(&id);if id==r.entity_id as u32{self.game.riding=None;}}
    }
    fn tick(&mut self,id:u32,passenger:bool)->Result<()>{
        let own=self.game.remote.as_ref().is_some_and(|r|id==r.entity_id as u32);
        let _selection=horizon_client::terrain_speed_bridge::select(horizon_client::terrain_speed_order::Selection::Entity(id));
        if own&&let Some(p)=self.game.player.as_ref(){self.game.previous_position=p.position;}
        if passenger{self.game.native_terrain_reset_passenger_velocity(id);}
        if own{
            self.input=self.game.native_terrain_tick_player(Vec3::ZERO)?;
        }else{self.game.native_terrain_tick_entity(id);}
        // Entity.rideTick seats only after the entire tick, and does not clear
        // deltaMovement a second time. Nested passengers recurse afterwards.
        if passenger&&self.vehicle(id).is_some(){
            if own{self.game.native_terrain_position_player_ride()?;}
            else{self.game.position_riders();}
        }
        Ok(())
    }
}
