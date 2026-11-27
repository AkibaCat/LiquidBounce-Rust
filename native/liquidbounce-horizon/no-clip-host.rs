    pub(super) fn native_client_no_clip_vehicle(&self)->Option<u32> {
        let id=self.riding?;
        if self.boats.iter().any(|b|b.id==id) && self.remote.as_ref().is_none_or(|r|r.first_passenger) {return Some(id);}
        if self.remote.is_some() {return self.driven_mount().map(|m|m.id);}
        let mob=self.mobs.iter().find(|m|m.id==id)?;
        let steer=super::mounts::steering_item(mob.species(),self.held().map(|s|s.item.as_ref()),self.offhand.as_ref().map(|s|s.item.as_ref()));
        let equine=horizon_client::mobs::equine::moved_by_rider(mob.species())&&mob.is_saddled()
            && (!matches!(mob.species(),horizon_client::mobs::Species::Pig|horizon_client::mobs::Species::Strider)||steer);
        let ghast=mob.species()==horizon_client::mobs::Species::HappyGhast&&horizon_client::mobs::happy_ghast::harnessed_adult(mob)&&!horizon_client::mobs::happy_ghast::on_still_timeout(mob);
        (mob.alive()&&(equine||ghast)).then_some(id)
    }
    pub(super) fn native_client_no_clip_state(&self)->Option<[f64;4]> {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() || !self.player.as_ref().is_some_and(|p|p.native_client_queries_allowed()) {return None;}
        Some([f64::from(self.yaw),if self.native_client_no_clip_vehicle().is_some(){1.0}else{0.0},if self.keys.held(horizon_client::actions::Action::Jump){1.0}else{0.0},if self.keys.held(horizon_client::actions::Action::Sneak){1.0}else{0.0}])
    }
    /// Clear only the current controlled vehicle, exactly like onDisabled.
    pub(super) fn native_client_no_clip_flags(&mut self,enabled:bool) {
        if self.native_client_no_clip_state().is_none() {return;}
        let vehicle=self.native_client_no_clip_vehicle();
        if let Some(player)=self.player.as_mut() {player.client_no_physics=enabled;if enabled {player.fall_distance=0.0;player.ground.on_ground=false;}}
        if let Some(id)=vehicle {
            if let Some(mob)=self.mobs.iter_mut().find(|m|m.id==id) {mob.client_no_physics=enabled;}
            if let Some(boat)=self.boats.iter_mut().find(|b|b.id==id) {boat.client_no_physics=enabled;}
        }
    }
    pub(super) fn native_client_no_clip_velocity(&mut self,vehicle:bool,moving:bool,velocity:[f64;3]) {
        if self.native_client_no_clip_state().is_none() {return;}
        let apply=|old:horizon_core::geometry::Vec3|horizon_core::geometry::Vec3::new(if moving{old.x*0.0+velocity[0]}else{0.0},velocity[1],if moving{old.z*0.0+velocity[2]}else{0.0});
        if vehicle {if let Some(id)=self.native_client_no_clip_vehicle() {
            if let Some(mob)=self.mobs.iter_mut().find(|m|m.id==id) {mob.velocity=apply(mob.velocity);}
            if let Some(boat)=self.boats.iter_mut().find(|b|b.id==id) {boat.velocity=apply(boat.velocity);}
        }} else if let Some(player)=self.player.as_mut() {player.velocity=apply(player.velocity);}
    }
