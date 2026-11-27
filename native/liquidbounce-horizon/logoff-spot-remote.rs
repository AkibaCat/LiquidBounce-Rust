    // Keep profile maps private to this module, just like the native Blink bridge.
    pub(super) fn client_logoff_add_identity(&mut self,original:i32,id:i32,profile:u128,name:String){
        if let Some(r)=self.remote.as_mut(){
            let textures=r.level_player_textures.get(&original).cloned();
            r.level_players.insert(id,profile);
            r.level_player_names.insert(id,name);
            if let Some(textures)=textures{r.level_player_textures.insert(id,textures);}
        }
    }
    pub(super) fn client_logoff_remove_identity(&mut self,id:i32){
        if let Some(r)=self.remote.as_mut(){r.level_players.remove(&id);r.level_player_names.remove(&id);r.level_player_textures.remove(&id);}
    }
