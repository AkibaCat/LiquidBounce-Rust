    pub(super) fn native_client_auto_wind_charge_snapshot(&self)->Option<serde_json::Value>{
        if !self.native_client_slot_owner(){return None;}
        let p=self.player.as_ref()?;
        let charge=|s:Option<&horizon_client::items::ItemStack>|s.is_some_and(|s|s.count>0&&horizon_client::command::args::normalize(&s.item)=="minecraft:wind_charge");
        let slot=if charge(self.offhand.as_ref()){Some(-1)}else{self.inventory.iter().take(9).position(|s|charge(s.as_ref())).map(|i|i as i32)};
        let stack=slot.and_then(|i|if i<0{self.offhand.as_ref()}else{self.inventory.get(i as usize).and_then(Option::as_ref)});
        let cooldown=stack.is_some_and(|s|{
            let view=horizon_client::item_render::stack_view(s);
            let components=self.pack.item_renderer.models.components_view(&view);
            let group=components.get("minecraft:use_cooldown").and_then(|v|v.get("cooldown_group")).and_then(serde_json::Value::as_str).unwrap_or(&s.item);
            self.cooldowns.is_on_cooldown(group)
        });
        Some(serde_json::json!({"fallFlying":p.fall_flying,"swimming":p.swimming,"liquid":p.fluids.was_touching_water||p.is_in_lava(),"jump":self.keys.held(horizon_client::actions::Action::Jump),"ground":p.ground.on_ground,"slot":slot,"cooldown":cooldown}))
    }
    pub(super) fn native_client_auto_wind_charge_query(&mut self,value:&str)->String{
        if !self.native_window()||!self.native_client_slot_owner(){return "null".into();}
        self.auto_wind_charge_query(value)
    }
