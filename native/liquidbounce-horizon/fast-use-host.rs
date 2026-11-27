    pub(super) fn native_client_fast_use_context(&self)->Option<[f64;8]>{
        if !self.native_client_slot_owner(){return None;}let p=self.player.as_ref()?;
        let using=self.using.and_then(|(slot,ticks)|self.stack_in(slot).map(|stack|(stack,ticks)));
        let (consumable,ticks,crossbow)=using.map_or((false,0,false),|(stack,ticks)|{
            let view=horizon_client::item_render::stack_view(stack);let components=self.pack.item_renderer.models.components_view(&view);
            (components.get("minecraft:consumable").is_some(),self.ticks_held(stack,ticks),&*stack.item=="minecraft:crossbow")
        });
        Some([u8::from(p.ground.on_ground)as f64,u8::from(p.move_vector!=[0.0,0.0])as f64,u8::from(self.potions.effects.get("regeneration").is_some())as f64,u8::from(self.using.is_some())as f64,u8::from(consumable)as f64,f64::from(ticks),u8::from(crossbow)as f64,f64::from(p.health.tick_count)])
    }
    pub(super) fn native_client_fast_use_release(&mut self){
        if !self.native_client_slot_owner(){return;}
        if let Some((slot,ticks))=self.using&&let Some(stack)=self.stack_in(slot).cloned(){self.release_used_item(&stack,ticks,false);}
    }
    pub(super) fn native_client_fast_use_crossbow(&mut self){
        if !self.native_client_slot_owner(){return;}let Some((slot,_))=self.using else{return;};
        let hand=if slot==super::shield::OFFHAND_SLOT{1}else{0};
        let menu_slot=self.selected as i32+36;
        self.native_client_inventory_click(0,menu_slot,40,2);
        self.native_client_use_item(1-hand,self.yaw,self.pitch);
        self.native_client_inventory_click(0,menu_slot,40,2);
        self.native_client_use_item(hand,self.yaw,self.pitch);
    }
    pub(super) fn native_client_fast_use_key(&self)->bool{self.native_client_slot_owner()&&self.client_use_key_down()}
