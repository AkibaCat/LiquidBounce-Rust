    // Located in remote.rs to read the real level and player-info profile maps.
    pub(super) fn client_murder_mystery_uuid(&self,entity:i32)->Option<String>{
        if let Some(r)=&self.remote{
            if entity==r.entity_id{return Some(format!("{:032x}",r.profile));}
            return r.level_players.get(&entity).map(|id|format!("{id:032x}"));
        }
        if entity==0{return Some(format!("{:032x}",horizon_protocol::crypto::offline_uuid(&self.own_profile_name())));}
        self.lan.iter().flat_map(|h|h.hub.guests.iter()).find(|g|g.placed&&g.dimension==self.dim.id()&&g.entity_id==entity).map(|g|format!("{:032x}",g.uuid))
    }
    pub(super) fn client_murder_mystery_players(&self)->Vec<serde_json::Value>{
        use serde_json::json;
        let mut out=vec![];let local=self.remote.as_ref().map_or(0,|r|r.entity_id);
        let make=|id:i32,name:String,position:[f64;3],main:&str,off:&str,using:bool|self.client_murder_mystery_uuid(id).map(|uuid|json!({"id":id,"uuid":uuid,"name":name,"position":position,"hasArrow":main=="minecraft:arrow"||off=="minecraft:arrow","usingBow":using}));
        if let Some(p)=self.player.as_ref(){
            let main=self.inventory.get(self.client_hotbar_slot()).and_then(Option::as_ref).filter(|s|s.count>0).map_or("minecraft:air",|s|&*s.item);
            let off=self.offhand.as_ref().filter(|s|s.count>0).map_or("minecraft:air",|s|&*s.item);
            let using=self.remote.as_ref().and_then(|r|r.use_item.as_deref()).map_or_else(||self.using.and_then(|(slot,_)|self.stack_in(slot)).is_some_and(|s|&*s.item=="minecraft:bow"),|s|s=="minecraft:bow");
            if let Some(v)=make(local,self.own_profile_name(),[p.position.x,p.position.y,p.position.z],main,off,using){out.push(v);}
        }
        for row in self.client_report_helper_world_players(){
            let Some(id)=row["id"].as_i64().and_then(|n|i32::try_from(n).ok()).filter(|&id|id!=local)else{continue;};
            let Some(m)=self.mobs.iter().find(|m|m.id as i32==id&&!m.removed&&&*m.kind.id=="minecraft:player")else{continue;};
            let main=m.extra.main_hand.as_deref().unwrap_or("minecraft:air");let off=m.extra.off_hand.as_deref().unwrap_or("minecraft:air");
            let using=m.extra.using_item>0&&if m.extra.using_off_hand{off}else{main}=="minecraft:bow";
            if let Some(v)=make(id,row["name"].as_str().unwrap_or("").into(),[m.position.x,m.position.y,m.position.z],main,off,using){out.push(v);}
        }
        for h in self.lan.iter(){for g in h.hub.guests.iter().filter(|g|g.placed&&g.dimension==self.dim.id()&&g.entity_id!=local){
            if out.iter().any(|p|p["id"].as_i64()==Some(i64::from(g.entity_id))){continue;}
            let main=g.inventory.get(g.selected).and_then(Option::as_ref).filter(|s|s.count>0).map_or("minecraft:air",|s|&*s.item);let off=g.offhand.as_ref().filter(|s|s.count>0).map_or("minecraft:air",|s|&*s.item);
            let using=h.play.get(&g.conn).and_then(|p|p.using()).is_some_and(|(slot,_)|if slot==super::shield::OFFHAND_SLOT{off}else{g.inventory.get(slot).and_then(Option::as_ref).map_or("minecraft:air",|s|&*s.item)}=="minecraft:bow");
            if let Some(v)=make(g.entity_id,g.name.clone(),g.position,main,off,using){out.push(v);}
        }}out
    }
    pub(super) fn client_murder_mystery_online(&self)->Vec<serde_json::Value>{
        self.remote.as_ref().map(|r|r.players.iter().map(|(uuid,name)|serde_json::json!({"uuid":format!("{uuid:032x}"),"name":name})).collect()).unwrap_or_default()
    }
    pub(super) fn client_murder_mystery_packet(&mut self,packet:&C){
        use serde_json::json;
        if !self.client_auto_tool_player(){return;}
        let v=match packet{
            C::Login(_)|C::Respawn(_)=>json!({"kind":"reset"}),
            C::SetEquipment(p)=>{
                let Some(player)=self.client_murder_mystery_players().into_iter().find(|v|v["id"].as_i64()==Some(i64::from(p.id)))else{return;};
                let items=p.slots.iter().filter(|(slot,stack)|*slot<=1&&stack.as_ref().is_some_and(|s|s.count>0)).map(|(_,s)|{let id=s.as_ref().map(|s|self.item_name(s.item));self.client_murder_mystery_item(id.as_deref().unwrap_or("minecraft:air"))}).collect::<Vec<_>>();
                json!({"kind":"equipment","player":player,"items":items})
            },
            C::Sound(p)if self.remote_sound_name(&p.sound).as_deref()==Some("minecraft:block.note_block.basedrum")=>{
                let Some(own)=self.player.as_ref().map(|p|[p.position.x,p.position.y,p.position.z])else{return;};
                json!({"kind":"sound","players":self.client_murder_mystery_players(),"own":own,"volume":p.volume})
            },_=>return,
        };
        self.client_murder_mystery_process(&v.to_string());
    }
