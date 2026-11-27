    pub(super) fn native_client_potion_spoof_duration(&self,id:&str)->Option<i32> {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() || !self.player.as_ref().is_some_and(|p|p.native_client_queries_allowed()) {return None;}
        self.potions.effects.get(id).map(|e|e.duration)
    }
    /// action: 0 addEffect, 1 explicit addAttributeModifiers, 2 removeEffect,
    /// 3 explicit removeAttributeModifiers. These calls retain original order.
    pub(super) fn native_client_potion_spoof_apply(&mut self,id:&str,amplifier:i32,action:u8) {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() || !self.player.as_ref().is_some_and(|p|p.native_client_queries_allowed()) {return;}
        let Some(effect)=horizon_client::brewing::effect(id) else {return;};
        let id=effect.id;
        match action {
            0=>{self.add_player_effect(horizon_client::effects::EffectInstance::new(id,0,amplifier));},
            2=>{
                let existed=self.potions.effects.get(id).is_some();
                self.potions.effects.list.retain(|effect|effect.id!=id);
                // LivingEntity.onEffectsRemoved only removes modifiers on the server.
                if existed && self.remote.is_none() {self.native_client_potion_spoof_apply(id,0,3);}
                self.apply_effect_status();
            },
            1|3=>{
                if let Some((attribute,key,amount,operation))=horizon_client::effects::effect_modifier(id)
                    && let Some(player)=self.player.as_mut() {
                    let _=player.set_modifier_unclamped(attribute,key,amount*f64::from(amplifier+1),operation,action==1);
                    let _=player.clamp_health_to_max();
                }
            },_=>{}
        }
    }
