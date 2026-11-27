    /// Slots.OffhandWithHotbar visits offhand before hotbar index zero.
    pub(super) fn native_client_middle_click_pearl_slot(&self)->Option<i32>{
        if !self.native_client_slot_owner(){return None;}
        let pearl=|stack:Option<&horizon_client::items::ItemStack>|stack.is_some_and(|s|s.count>0&&horizon_client::command::args::normalize(&s.item)=="minecraft:ender_pearl");
        if pearl(self.offhand.as_ref()){return Some(-1);}
        self.inventory.iter().take(9).position(|s|pearl(s.as_ref())).map(|i|i as i32)
    }
    pub(super) fn native_client_middle_click_context(&self,friend:bool,range:f32)->Option<serde_json::Value>{
        use horizon_core::geometry::{Aabb,Vec3};
        if !self.native_client_slot_owner(){return None;}
        let player=self.player.as_ref()?;
        let mut result=serde_json::json!({"pickup":self.mapped_key_down("key.pickItem"),"pitch":self.pitch,"screenOpen":!self.client_keymapping_query_allowed(),"pearl":self.native_client_middle_click_pearl_slot(),"name":null,"visible":false});
        if !friend{return Some(result);}
        let local_id=self.remote.as_ref().map_or(0,|r|r.entity_id)as u32;
        let camera_id=self.remote_camera_id().unwrap_or(local_id);
        let camera=self.camera_entity();
        let eye=camera.map_or_else(||self.eye_position().unwrap_or([0.0;3]),|c|[c.position[0],c.position[1]+f64::from(c.eye_height),c.position[2]]);
        let bounds=if camera.is_some(){
            let entity=self.native_entity(camera_id)?;let [lo,hi]=self.native_entity_bounding_box(entity)?;
            Aabb::new(Vec3::new(lo[0],lo[1],lo[2]),Vec3::new(hi[0],hi[1],hi[2]))
        }else{player.bounds()};
        let root=if camera_id==local_id{self.riding.map(|id|self.client_reach_root_vehicle(id)).unwrap_or(local_id)}else{self.client_reach_root_vehicle(camera_id)};
        let mut names=Vec::new();let mut candidates=Vec::new();
        let mut offer=|id:u32,name:&str,position:[f64;3],bounds:Aabb|{
            if id==camera_id{return;}
            let index=names.len();names.push(name.to_owned());
            candidates.push(super::ClientReachCandidate{kind:0,index,id:id as i32,local_id:id,position,bounds,pick_radius:self.client_pick_margin(id as i32,0.0),same_root_vehicle:root==self.client_reach_root_vehicle(id),can_pick_inside:true});
        };
        if camera_id!=local_id{let name=self.remote.as_ref().map_or_else(||self.acting_player_name(),|r|r.profile_name.as_str());offer(local_id,name,[player.position.x,player.position.y,player.position.z],player.bounds());}
        for m in self.mobs.iter().filter(|m|!m.removed&&!m.extra.spectator&&horizon_client::command::args::normalize(&m.kind.id)=="minecraft:player"){
            if let Some(name)=self.remote_player_name(m.id){offer(m.id,name,[m.position.x,m.position.y,m.position.z],m.bounds());}
        }
        for guest in self.lan.iter().flat_map(|h|h.hub.guests.iter()).filter(|g|g.placed&&g.dimension==self.dim.id()&&g.game_mode!=3){offer(guest.entity_id as u32,&guest.name,guest.position,guest.bounds());}
        let dir=super::client_rotation_direction(self.yaw,self.pitch);
        let eye=Vec3::new(eye[0],eye[1],eye[2]);
        if let Some(hit)=super::client_reach_pick(candidates,eye,Vec3::new(dir[0],dir[1],dir[2]),bounds,f64::from(range)){
            result["name"]=serde_json::json!(names[hit.index]);
            result["visible"]=serde_json::json!(hit.distance_squared<=0.0||super::client_middle_click::visible(self,eye,Vec3::new(hit.point[0],hit.point[1],hit.point[2]),camera_id));
        }
        Some(result)
    }
