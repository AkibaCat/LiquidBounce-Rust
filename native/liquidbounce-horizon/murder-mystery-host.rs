    pub(super) fn native_client_murder_mystery(&mut self,input:&str,output:&mut String){
        if self.client_auto_tool_player(){mod_api::dispatch_client_murder_mystery(self,mod_api::mods(),input,output);}
    }
    pub(super) fn client_murder_mystery_item(&self,id:&str)->serde_json::Value{
        let tag=|t:&str|self.items().item_in_tag(t,id);
        serde_json::json!({"id":id,"sword":tag("minecraft:swords"),"mining":(["minecraft:axes","minecraft:pickaxes","minecraft:shovels","minecraft:hoes"].iter().any(|t|tag(t))),"boat":client_weapon_item_facts::item_is(id,"BoatItem")})
    }
    pub(super) fn client_murder_mystery_process(&mut self,input:&str)->serde_json::Value{
        let mut output=String::new();self.native_client_murder_mystery(input,&mut output);
        let v=serde_json::from_str::<serde_json::Value>(&output).unwrap_or(serde_json::Value::Null);
        if let Some(prefixes)=v["prefixes"].as_array(){for p in prefixes{
            let Some(name)=p["name"].as_str()else{continue;};
            let team=self.score.board.players_team(name).map(|t|t.name.clone());
            if let Some(team)=team.and_then(|t|self.score.board.team_mut(&t)){
                team.prefix=serde_json::json!({"text":p["text"],"color":p["color"]});
            }
        }}
        v
    }
    pub(super) fn client_murder_mystery_tick(&mut self){
        if !self.client_auto_tool_player(){return;}
        let main=self.inventory.get(self.client_hotbar_slot()).and_then(Option::as_ref).filter(|s|s.count>0).map_or("minecraft:air",|s|&*s.item);
        let off=self.offhand.as_ref().filter(|s|s.count>0).map_or("minecraft:air",|s|&*s.item);
        let stack=self.inventory.get(3).and_then(Option::as_ref).filter(|s|s.count>0);
        let is_map=stack.is_some_and(|s|&*s.item=="minecraft:filled_map");let id=stack.and_then(super::maps::map_id);
        let colors=id.and_then(|id|self.maps.data.get(&id)).map(|m|&m.colors);
        let input=serde_json::json!({"kind":"tick","players":self.client_murder_mystery_players(),"hands":[self.client_murder_mystery_item(main),self.client_murder_mystery_item(off)],"map":{"isMap":is_map,"id":id,"colors":colors},"online":self.client_murder_mystery_online()}).to_string();
        self.client_murder_mystery_process(&input);
    }
    pub(super) fn client_murder_mystery_frame(&mut self,alpha:f64,primitives:&mut horizon_client::world_primitives::WorldPrimitives){
        use horizon_client::sound_engine::{SoundInstance,Sounds};
        let v=self.client_murder_mystery_process("{\"kind\":\"frame\"}");if v["active"]!=true{return;}
        for (flag,sound)in [("hurt","minecraft:entity.villager.hurt"),("bow","minecraft:item.crossbow.shoot")]{if v[flag]==true{
            self.audio.play_instance(SoundInstance::for_ui(sound,1.,0.25,horizon_client::random_source::create()),false);
        }}
        for a in self.armor_stands.iter().filter(|a|a.invisible&&a.equipment[0].as_ref().is_some_and(|s|s.count>0&&&*s.item=="minecraft:bow")){
            let p=[a.position.x,a.position.y,a.position.z];let prev=[a.previous.x,a.previous.y,a.previous.z];let at=std::array::from_fn::<_,3,_>(|i|prev[i]+(p[i]-prev[i])*alpha);
            let min=[at[0]-0.6,at[1],at[2]-0.6];let max=[at[0]+0.6,at[1]+2.5,at[2]+0.6];
            primitives.filled_box(min,max,0x647fffd4).through_walls();primitives.box_outline(min,max,0xff00ffff,1.).through_walls();
        }
    }
