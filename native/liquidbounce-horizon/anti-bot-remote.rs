    // Keep complete profile-property presence and PlayerInfo availability.
    // MC 26.3 PlayerInfo starts in DEFAULT_MODE (survival), unlike a raw
    // PlayerInfoUpdate.Entry whose absent mode remains null. These are
    // observations of accepted vanilla handlers, independent of module state.
    pub(super) fn client_anti_bot_keep_info(&mut self,p:&play::PlayerInfoUpdate){
        let n=&mut self.client_anti_bot;
        for e in &p.entries{
            let known=self.remote.as_ref().is_some_and(|r|r.tab_players.contains_key(&e.uuid));
            if p.actions&1!=0&&e.name.is_some(){n.profiles.entry(e.uuid).or_insert((e.properties.is_empty(),Some(0)));}
            if (known||p.actions&1!=0)&&let Some(mode)=e.game_mode{if let Some(v)=n.profiles.get_mut(&e.uuid){v.1=Some(mode);}}
        }
    }
    pub(super) fn client_anti_bot_online(&self)->Vec<serde_json::Value>{
        self.remote.as_ref().map(|r|r.tab_players.iter().map(|(uuid,p)|serde_json::json!({"uuid":format!("{uuid:032x}"),"name":p.name})).collect()).unwrap_or_else(||{
            let mut out=vec![];if self.player.is_some(){out.push(serde_json::json!({"uuid":self.client_murder_mystery_uuid(0),"name":self.own_profile_name()}));}
            for g in self.lan.iter().flat_map(|h|h.hub.guests.iter()).filter(|g|g.placed){out.push(serde_json::json!({"uuid":format!("{:032x}",g.uuid),"name":g.name}));}out
        })
    }
    pub(super) fn client_anti_bot_player(&self,id:i32)->Option<serde_json::Value>{
        use serde_json::json;
        let local=self.remote.as_ref().map_or(0,|r|r.entity_id);
        let uuid=self.client_murder_mystery_uuid(id)?;
        let raw=u128::from_str_radix(&uuid,16).ok()?;
        let profile=self.client_anti_bot.profiles.get(&raw);
        let info=self.remote.as_ref().and_then(|r|r.tab_players.get(&raw));
        let online=self.remote.as_ref().is_none_or(|r|r.tab_players.contains_key(&raw));
        let listed=self.remote.as_ref().is_none_or(|r|r.listed.contains(&raw));
        let mode=if self.remote.is_none(){Some(if self.spectator(){3}else if self.creative{1}else{0})}else{profile.and_then(|p|p.1)};
        let make=|token:u64,name:String,ticks:i32,position:[f64;3],previous:[f64;3],ground:bool,pitch:f32,health:f32,scale:f64,armor:[serde_json::Value;4],empty:bool,mode:Option<i32>,ping:i32|json!({"id":id,"token":token,"uuid":uuid,"name":name,"local":id==local,"ticks":ticks,"position":position,"previous":previous,"onGround":ground,"pitchBits":pitch.to_bits(),"healthBits":health.to_bits(),"scaleBits":scale.to_bits().to_string(),"armor":armor,"propertiesEmpty":empty,"online":online,"listed":listed,"mode":mode,"ping":ping});
        if id==local{
            let p=self.player.as_ref()?;return Some(make(0,self.own_profile_name(),p.health.tick_count,[p.position.x,p.position.y,p.position.z],[self.previous_position.x,self.previous_position.y,self.previous_position.z],p.ground.on_ground,self.pitch,p.health.health(),p.attributes.computed_value_of("minecraft:scale").unwrap_or(1.),[3,2,1,0].map(|i|self.client_anti_bot_armor(self.armor[i].as_ref())),profile.is_none_or(|p|p.0),mode,info.map_or(0,|p|p.latency)));
        }
        if let Some(m)=self.mobs.iter().find(|m|m.id as i32==id&&!m.removed&&m.kind.id.as_str()=="minecraft:player"){
            use horizon_world::player_inventory::EquipmentSlot as S;
            let armor=[S::Head,S::Chest,S::Legs,S::Feet].map(|s|self.client_anti_bot_armor(horizon_client::mobs::finalize::equipment_stack(m,s).as_ref()));
            let scale=self.client_anti_bot.scales.get(&id).copied().unwrap_or_else(||if m.extra.model_scale==0.{1.}else{f64::from(m.extra.model_scale)});
            return Some(make(m.join.get(),self.remote_player_name(m.id).unwrap_or_default().to_owned(),m.age as i32,[m.position.x,m.position.y,m.position.z],[m.previous.x,m.previous.y,m.previous.z],m.on_ground,m.head_pitch,m.health,scale,armor,self.client_anti_bot.entities.get(&id).copied().unwrap_or(true),mode,info.map_or(0,|p|p.latency)));
        }
        let g=self.lan.iter().flat_map(|h|h.hub.guests.iter()).find(|g|g.placed&&g.dimension==self.dim.id()&&g.entity_id==id)?;
        let motion=g.motion.as_deref()?;
        Some(make((1u64<<63)|u64::from(g.conn),g.name.clone(),motion.health.tick_count,g.position,g.previous_position,g.on_ground,g.pitch,g.health,motion.attributes.computed_value_of("minecraft:scale").unwrap_or(1.),[3,2,1,0].map(|i|self.client_anti_bot_armor(g.armor[i].as_ref())),g.properties.is_empty(),Some(g.game_mode),g.latency.load(std::sync::atomic::Ordering::Relaxed)))
    }
    pub(super) fn client_anti_bot_world(&self)->serde_json::Value{
        let mut ids=self.client_report_helper_world_players().into_iter().filter_map(|p|p["id"].as_i64().map(|i|i as i32)).collect::<Vec<_>>();
        let local=self.remote.as_ref().map_or(0,|r|r.entity_id);if self.player.is_some()&&!ids.contains(&local){ids.insert(0,local);}
        for g in self.lan.iter().flat_map(|h|h.hub.guests.iter()).filter(|g|g.placed&&g.dimension==self.dim.id()){if !ids.contains(&g.entity_id){ids.push(g.entity_id);}}
        let players=ids.into_iter().filter_map(|id|self.client_anti_bot_player(id)).collect::<Vec<_>>();
        let p=self.player.as_ref();let own=p.map(|p|[p.position.x,p.position.y,p.position.z]);let max=p.map_or(20.,|p|p.attributes.computed_value_of("minecraft:max_health").unwrap_or(20.)as f32);
        serde_json::json!({"players":players,"online":self.client_anti_bot_online(),"own":own,"maxHealthBits":max.to_bits()})
    }
    pub(super) fn client_anti_bot_query(&self,id:i32)->serde_json::Value{
        serde_json::json!({"player":self.client_anti_bot_player(id),"online":self.client_anti_bot_online(),"maxHealthBits":self.player.as_ref().map_or(20.,|p|p.attributes.computed_value_of("minecraft:max_health").unwrap_or(20.)as f32).to_bits()})
    }
    pub(super) fn client_anti_bot_packet(&mut self,packet:&C,time:i64){
        use serde_json::json;
        if !self.native_window(){return;}
        let mut v=match packet{
            C::PlayerInfoUpdate(p)=>json!({"kind":"info","actions":p.actions,"time":time,"online":self.client_anti_bot_online(),"entries":p.entries.iter().map(|e|json!({"uuid":format!("{:032x}",e.uuid),"name":e.name,"propertiesEmpty":e.properties.is_empty(),"mode":e.game_mode,"latency":e.latency.unwrap_or(0)})).collect::<Vec<_>>()}),
            C::PlayerInfoRemove(p)=>json!({"kind":"removeProfiles","ids":p.uuids.iter().map(|id|format!("{id:032x}")).collect::<Vec<_>>()}),
            C::MoveEntityPos(play::MoveEntityPos(p))|C::MoveEntityPosRot(play::MoveEntityPosRot(p))=>{
                let local=self.remote.as_ref().map_or(0,|r|r.entity_id);
                let state=if p.id==local{self.player.as_ref().map(|m|(m.ground.on_ground,self.previous_position.y,m.position.y))}else{self.mobs.iter().find(|m|m.id as i32==p.id).map(|m|(m.on_ground,m.previous.y,m.position.y))};
                let Some((ground,previous,y))=state else{return;};json!({"kind":"ground","id":p.id,"ground":ground,"previous":previous,"y":y})
            },
            C::UpdateAttributes(p)=>json!({"kind":"attributes","id":p.id}),
            C::SetEntityData(p)=>json!({"kind":"metadata","id":p.id,"size":p.values.len()}),
            C::Animate(p)=>json!({"kind":"animate","id":p.id,"action":p.action}),
            C::SwingAnimation(p)=>json!({"kind":"swing","id":p.id}),
            C::RemoveEntities(p)=>json!({"kind":"removeEntities","ids":p.ids,"world":self.client_anti_bot_world()}),
            _=>return,
        };
        self.native_client_anti_bot(&v.take().to_string());
    }
