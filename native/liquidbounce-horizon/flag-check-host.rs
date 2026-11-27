    pub(super) fn native_client_flag_check_snapshot(&self)->Option<serde_json::Value>{
        if !self.client_smart_eat_player(){return None;}
        let p=self.player.as_ref()?;
        Some(serde_json::json!({"ticks":p.health.tick_count,"yaw":p.head_yaw,"pitch":self.pitch,
            "health":p.health.health(),"alive":!p.health.dead&&p.health.health()>0.,"food":p.health.food.food}))
    }
    pub(super) fn native_client_flag_check_chat(&mut self,message:&str,key:&str){
        let ticks=self.visual_ticks();let focused=self.screen==Screen::Chat;
        self.ui.chat.add_client_id(&self.gui.atlas,key,Text::literal(message),ticks,focused);
    }
