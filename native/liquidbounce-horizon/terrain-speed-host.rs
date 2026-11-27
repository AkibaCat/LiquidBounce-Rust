    /// Raw TerrainSpeed settings. Physical keys remain separate from transformed input.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn native_client_terrain_speed(&self, flags:&mut u32, mode:&mut u8,
        climb_motion:&mut f32, slipperiness:&mut f32, ice_motion:&mut f32,
        base_horizontal:&mut f32, base_vertical:&mut f32,
        sprint_horizontal:&mut f32, sprint_vertical:&mut f32, input:&mut u8, running:&mut bool) {
        mod_api::dispatch_client_terrain_speed(self,mod_api::mods(),flags,mode,climb_motion,
            slipperiness,ice_motion,base_horizontal,base_vertical,sprint_horizontal,sprint_vertical,input,running);
    }
    pub(super) fn native_terrain_snapshot(&self)->Option<horizon_client::terrain_speed_bridge::Snapshot>{
        use horizon_client::actions::Action;
        use horizon_client::terrain_speed::{ClimbMode,FastClimb,IceSpeed,TerrainSpeed,WaterSpeed};
        if !self.native_window()||self.lan_acting_uuid.is_some()
            ||!self.player.as_ref().is_some_and(|p|p.native_client_queries_allowed()){return None;}
        let(mut flags,mut mode,mut cm,mut slip,mut im,mut bh,mut bv,mut sh,mut sv,mut input,mut running)=
            (0,0,0.2872,0.6,0.5,0.44,0.44,1.0,1.0,0,false);
        self.native_client_terrain_speed(&mut flags,&mut mode,&mut cm,&mut slip,&mut im,&mut bh,&mut bv,&mut sh,&mut sv,&mut input,&mut running);
        running.then(||horizon_client::terrain_speed_bridge::Snapshot{
            settings:TerrainSpeed{
                fast_climb:FastClimb{enabled:flags&1!=0,mode:if mode==1{ClimbMode::Clip}else{ClimbMode::Motion},motion:cm},
                ice_speed:IceSpeed{enabled:flags&2!=0,slipperiness:slip,motion_enabled:flags&4!=0,horizontal_motion:im},
                water_speed:WaterSpeed{enabled:flags&8!=0,auto_swim:flags&16!=0,base_horizontal:bh,base_vertical:bv,sprint_enabled:flags&32!=0,sprint_horizontal:sh,sprint_vertical:sv},
            },forward:self.keys.held(Action::Forward),sprint:self.keys.held(Action::Sprint),
            jump:self.keys.held(Action::Jump),sneak:self.keys.held(Action::Sneak),input,
        })
    }
    pub(super) fn native_terrain_flush(&mut self){
        if let Some(p)=self.player.as_mut(){horizon_client::terrain_speed_bridge::flush(&mut p.velocity);}
        else{horizon_client::terrain_speed_bridge::discard_pending();}
        if let Some(p)=horizon_client::terrain_speed_bridge::take_snap(){self.previous_position=p;}
    }
    pub(super) fn native_terrain_player_tick(&mut self){
        let Some(s)=horizon_client::terrain_speed_bridge::snapshot()else{return};
        let Some(p)=self.player.as_mut()else{return};
        let mut velocity=[p.velocity.x,p.velocity.y,p.velocity.z];
        s.settings.water_speed.player_tick(true,horizon_client::terrain_speed::WaterContext{
            in_water:p.fluids.was_touching_water,physical_sprint:s.sprint,physical_jump:s.jump,physical_sneak:s.sneak,
            moving:p.move_vector[0]!=0.0||p.move_vector[1]!=0.0,input:s.input,yaw:self.yaw,
        },&mut velocity,|a|[horizon_core::jmath::sin(a),horizon_core::jmath::cos(a)]);
        p.velocity=horizon_core::geometry::Vec3::new(velocity[0],velocity[1],velocity[2]);
    }
    pub(super) fn native_terrain_finish_player(&mut self){
        self.native_terrain_flush();
    }
