    pub(super) fn native_terrain_entity_exists(&self,id:u32)->bool{
        self.remote.as_ref().is_some_and(|r|r.entities.contains_key(&(id as i32)))
    }
    fn native_terrain_tick_entity(&mut self,id:u32){
        use horizon_client::terrain_speed_order::Selection;
        if !self.remote.as_ref().is_some_and(|r|r.entities.contains_key(&(id as i32))){return;}
        let _selection=horizon_client::terrain_speed_bridge::select(Selection::Entity(id));
        self.tick_remote_entities(true);
        self.native_terrain_tick_controlled_vehicle(id);
        self.native_terrain_flush();
        if self.boats.iter().any(|b|b.id==id){self.native_terrain_entity_push(id);}
        self.tick_own_rockets();
    }
    fn native_terrain_reset_passenger_velocity(&mut self,id:u32){
        if self.remote.as_ref().is_some_and(|r|id==r.entity_id as u32)&&let Some(p)=self.player.as_mut(){p.velocity=Vec3::ZERO;}
        if let Some(m)=self.mobs.iter_mut().find(|m|m.id==id){m.velocity=Vec3::ZERO;}
        if let Some(e)=self.dropped.iter_mut().find(|e|e.id==id){e.velocity=Vec3::ZERO;}
        if let Some(e)=self.orbs.iter_mut().find(|e|e.id==id){e.velocity=Vec3::ZERO;}
        if let Some(e)=self.boats.iter_mut().find(|e|e.id==id){e.velocity=Vec3::ZERO;}
    }
    /// One Entity.push call at that entity's actual turn, never a sum applied
    /// to a later move. Client-side pushableBy only selects the local player.
    fn native_terrain_entity_push(&mut self,id:u32){
        use horizon_client::mobs::{Species,Who};
        let collisions=self.player_entity_collisions();
        let climbable=if let Some(p)=self.player.as_mut(){
            MotionWorld{entities:&collisions,player:p,scene:&mut self.world,assets:&self.pack.assets,
                collision_binding:self.collision,resources:self.motion,health:&self.health_resources}.climbable().unwrap_or(false)
        }else{return;};
        let(Some(p),Some(r))=(self.player.as_ref(),self.remote.as_ref())else{return};
        if p.health.dead_or_dying()||p.spectator||p.sleeping
            ||r.passenger_vehicles.values().any(|&parent|parent==r.entity_id as u32)
            ||climbable{return;}
        let root=|mut who:u32|{
            let mut seen=std::collections::HashSet::new();
            while seen.insert(who){if let Some(&parent)=r.passenger_vehicles.get(&who){who=parent;}else{break;}}
            who
        };
        if root(id)==root(r.entity_id as u32){return;}
        let bounds=p.bounds();let board=&self.score.board;let player_team=board.players_team(&r.profile_name);
        let impulse=if let Some(m)=self.mobs.iter().find(|m|m.id==id){
            // MixinClientLevel.getPushableEntities can suppress LivingEntity
            // queries. AbstractBoat.tick uses getEntities and remains allowed.
            if !self.native_client_airjump_push_policy()[3]{return;}
            let listed=r.level_players.get(&(id as i32)).and_then(|uuid|r.tab_players.get(uuid));
            let teams=self.mob_teams();
            let team=listed.map_or_else(||teams.of(Who::Mob(id)),|t|board.players_team(&t.name));
            if !m.alive()||m.species()==Species::Bat||!m.bounds().intersects(bounds)
                ||listed.is_some_and(|t|t.game_mode==SPECTATOR)
                ||!horizon_client::scoreboard::pushable_by(team,player_team,true){return;}
            super::boat_passengers::push_impulse(p.position,m.position).map(|v|Vec3::new(-v.x,0.0,-v.z))
        }else if let Some(b)=self.boats.iter().find(|b|b.id==id){
            let area=Vec3::new(f64::from(0.2_f32),f64::from(-0.01_f32),f64::from(0.2_f32));let bb=b.bounds();
            if !bounds.intersects(bb.inflate(area))
                ||!horizon_client::scoreboard::pushable_by(None,player_team,true)
                ||!super::boat_passengers::boat_pushes(bb,bounds,false){return;}
            super::boat_passengers::push_impulse(b.position,p.position)
        }else{return;};
        if let Some(v)=impulse.filter(|v|v.x.is_finite()&&v.y.is_finite()&&v.z.is_finite())
            &&let Some(p)=self.player.as_mut(){p.velocity=p.velocity.add_vector(v);}
    }
