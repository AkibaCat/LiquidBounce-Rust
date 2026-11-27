    pub(super)fn native_client_hit_fx(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_hit_fx(self,mod_api::mods(),input,output);}}
    pub(super)fn native_client_hit_fx_context(&self,id:i32)->String{
        if !self.native_window(){return "null".into();}
        let Ok(entity)=serde_json::from_str::<serde_json::Value>(&self.native_client_targeting_entity(id))else{return "null".into();};
        let local=self.remote.as_ref().map_or(0,|r|r.entity_id);
        let position=if id==local{self.player.as_ref().map(|p|[p.position.x,p.position.y,p.position.z])}else{self.native_entity(id as u32).and_then(|e|self.native_entity_position(e))};
        let Some(position)=position else{return "null".into();};
        serde_json::json!({"position":position,"alive":entity["alive"]}).to_string()
    }
