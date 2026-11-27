    pub(super) fn native_client_blink_visual(&self,partial_ticks:f32)->String {
        let Some(p)=self.player.as_ref() else{return "null".into()};
        let swim=self.player_model.as_ref().map_or(0.,|m|m.extra.swim.amount(partial_ticks));
        serde_json::json!({"firstPerson":self.effective_camera_type().first_person(),"width":p.dimensions.width,"height":p.dimensions.height,"pose":p.pose.id(),"swimAmount":swim}).to_string()
    }
    pub(super) fn native_client_blink_esp(&self,input:&str,geometry:&mut String){if self.native_window(){mod_api::dispatch_client_blink_esp(self,mod_api::mods(),input,geometry);}}
