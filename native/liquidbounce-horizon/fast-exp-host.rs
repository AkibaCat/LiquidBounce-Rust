    pub(super) fn native_client_fast_exp_snapshot(&self)->Option<serde_json::Value>{
        use serde_json::json;
        if !self.native_client_slot_owner(){return None;}
        let p=self.player.as_ref()?;
        let bottle=|s:Option<&horizon_client::items::ItemStack>|s.is_some_and(|s|s.count>0&&horizon_client::command::args::normalize(&s.item)=="minecraft:experience_bottle");
        let slot=if bottle(self.offhand.as_ref()){Some(-1)}else{self.inventory.iter().take(9).position(|s|bottle(s.as_ref())).map(|i|i as i32)};
        let count=slot.and_then(|i|if i<0{self.offhand.as_ref()}else{self.inventory.get(i as usize).and_then(Option::as_ref)}).map_or(0,|s|s.count);
        let items=self.items();
        let mut repair=Vec::new();
        for stack in self.armor.iter().map(Option::as_ref).chain(std::iter::once(if slot==Some(-1){self.held()}else{self.offhand.as_ref()})).flatten(){
            if stack.enchantment_level("minecraft:mending")!=0&&self.stations().enchantments.get("minecraft:mending").is_some(){repair.push(json!({"damage":stack.damage as i32,"max":items.stack_max_damage(stack)as i32}));}
        }
        let me=self.remote.as_ref().map_or(0,|r|r.entity_id)as u32;
        let speed=p.velocity.x*p.velocity.x+p.velocity.y*p.velocity.y+p.velocity.z*p.velocity.z;
        let orbs=self.orbs.iter().any(|o|o.following==Some(me)&&o.velocity.x*o.velocity.x+o.velocity.y*o.velocity.y+o.velocity.z*o.velocity.z>speed);
        Some(json!({"valid":p.health.health()>0.,"handled":matches!(self.screen,Screen::Inventory|Screen::Creative),"slot":slot,"count":count,"items":repair,"orbs":orbs}))
    }
