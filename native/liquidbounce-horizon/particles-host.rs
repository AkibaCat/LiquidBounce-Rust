    pub(super) fn native_client_particles(&self,input:&str,geometry:&mut String){if self.native_window(){mod_api::dispatch_client_particles(self,mod_api::mods(),input,geometry);}}
    pub(super) fn native_client_particles_direction(&self,yaw:f32,pitch:f32)->[f64;3]{let v=horizon_core::geometry::Vec3::direction_from_rotation(pitch,yaw);[v.x,v.y,v.z]}
    pub(super) fn native_client_particles_context(&self,entity:i32)->String{
        if !self.native_window(){return "null".into();}
        let Some(p)=self.player.as_ref().filter(|p|p.native_client_queries_allowed())else{return "null".into();};
        let local_id=self.remote.as_ref().map_or(0,|r|r.entity_id);
        let target=if entity==local_id{Some([p.position.x,p.position.y,p.position.z])}else if let Some(g)=self.lan.iter().flat_map(|h|h.hub.guests.iter()).find(|g|g.placed&&g.entity_id==entity&&g.dimension==self.dim.id()){Some(g.position)}else{self.native_entity(entity as u32).and_then(|e|self.native_entity_position(e))};
        let Some(target)=target else{return "null".into();};
        let dx=(p.position.x-target[0])as f32;let dy=(p.position.y-target[1])as f32;let dz=(p.position.z-target[2])as f32;
        serde_json::json!({"eye":[p.position.x,p.position.y+f64::from(p.dimensions.eye_height),p.position.z],"distance":(dx*dx+dy*dy+dz*dz).sqrt(),"yaw":self.yaw,"pitch":self.pitch,"firstPerson":self.effective_camera_type().first_person()}).to_string()
    }
