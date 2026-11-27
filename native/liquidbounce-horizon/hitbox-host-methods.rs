    pub(super) fn native_client_entity_margin(&self, entity: i32, margin: &mut f32) {
        if !self.client_reach_player() { return; }
        mod_api::dispatch_client_entity_margin(self, mod_api::mods(), entity, margin);
    }
    pub(super) fn native_client_debug_hitbox_margin(&self, entity: i32, margin: &mut f32) {
        if !self.client_reach_player() { return; }
        mod_api::dispatch_client_debug_hitbox_margin(self, mod_api::mods(), entity, margin);
    }
    pub(super) fn native_client_hitbox_component_margin(&self, margin: &mut f32) {
        if !self.client_reach_player() { return; }
        mod_api::dispatch_client_hitbox_component_margin(self, mod_api::mods(), margin);
    }
    pub(super) fn client_pick_margin(&self, entity: i32, original: f32) -> f64 {
        let mut margin=original;
        self.native_client_entity_margin(entity, &mut margin);
        f64::from(margin)
    }
    pub(super) fn client_debug_hitbox_margin(&self, entity: i32) -> f32 {
        let mut margin=0.0;
        self.native_client_debug_hitbox_margin(entity, &mut margin);
        margin
    }
    // An unsent resident dragon has no network id yet. Give its debug/render
    // lookup a distinct handle from the resident player's id 0.
    pub(super) fn client_targeting_dragon_id(&self) -> i32 {
        self.dragon.as_ref().map_or(i32::MIN+1,|d|if d.id==0 {i32::MIN+1}else{d.id as i32})
    }
    /// Live client entity facts shared by combat/visual target filters. Java
    /// inheritance classification remains in the actual-JAR facts table.
    pub(super) fn native_client_targeting_entity(&self, entity: i32) -> String {
        if !self.client_reach_player() { return "null".into(); }
        let local_id=self.remote.as_ref().map_or(0,|r|r.entity_id);
        let (kind,position,alive,invisible,sleeping,is_local)=if entity==local_id {
            let Some(p)=self.player.as_ref() else {return "null".into();};
            ("minecraft:player".to_owned(),p.position,p.health.health()>0.0,self.own_invisible(),p.sleeping,true)
        } else if let Some(m)=self.mobs.iter().find(|m|m.id as i32==entity) {
            (horizon_client::command::args::normalize(&m.kind.id),m.position,!m.removed && m.alive(),m.invisible || m.has_effect("minecraft:invisibility"),m.extra.avatar_pose==horizon_client::pose::Pose::Sleeping,false)
        } else if let Some(a)=self.armor_stands.iter().find(|a|a.id as i32==entity) {
            ("minecraft:armor_stand".to_owned(),a.position,a.health>0.0,a.invisible,false,false)
        } else if let Some(d)=self.dragon.as_ref().filter(|_|entity==self.client_targeting_dragon_id()) {
            ("minecraft:ender_dragon".to_owned(),d.position,d.alive(),false,false,false)
        } else {
            let Some(handle)=u32::try_from(entity).ok().and_then(|id|self.native_entity(id)) else {return "null".into();};
            let (Some(kind),Some(position))=(self.native_entity_kind(handle),self.native_entity_position(handle)) else {return "null".into();};
            (kind,horizon_core::geometry::Vec3::new(position[0],position[1],position[2]),true,false,false,false)
        };
        // isWithinWorldBorder uses BlockPos, not the raw double position.
        let within_border=self.border.geometry().is_within_bounds(f64::from(position.x.floor() as i32),f64::from(position.z.floor() as i32),0.0);
        serde_json::json!({"kind":kind,"alive":alive,"invisible":invisible,"sleeping":sleeping,
            "is_local":is_local,"has_local_passenger":!is_local && u32::try_from(entity).ok().is_some_and(|id|self.riding==Some(id)),
            "within_border":within_border,"player_name":self.remote_player_name(entity as u32)}).to_string()
    }
