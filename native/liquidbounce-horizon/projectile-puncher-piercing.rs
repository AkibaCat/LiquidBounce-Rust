    /// CombatExtensions.attackEntity's piercing-weapon early branch. This lives
    /// with remote_spear_attack so it shares the actual look/pvp/action service.
    pub(super) fn client_projectile_puncher_piercing(&mut self)->bool {
        if self.spectator(){return false;}
        let Some(stack)=self.held()else{return false;};
        let view=horizon_client::item_render::stack_view(stack);
        let components=self.pack.item_renderer.models.components_view(&view);
        let Some(component)=components.get("minecraft:piercing_weapon")else{return false;};
        let sound=component.get("sound").and_then(|v|v.as_str().or_else(||v["sound_id"].as_str())).map(str::to_owned);
        let kinetic=horizon_client::spear::Kinetic::of(&stack.item);
        let animation=self.client_projectile_puncher_animation();
        if self.remote.is_some(){
            self.remote_sync_look();
            if let Some((i,distance,_))=self.mob_hit()&&self.before_block(distance)&&self.held_range_allows(distance){self.pvp_attacked(self.mobs[i].id as i32,distance);}
            self.send_action(8);
        }else{
            let old=self.client_projectile_puncher_hide_implicit;self.client_projectile_puncher_hide_implicit=true;let _=self.jab();self.client_projectile_puncher_hide_implicit=old;
        }
        self.hand_strength.reset_attack();
        self.swing_hand(Hand::Main,animation);
        if let Some(sound)=sound.as_deref().or_else(||kinetic.as_ref().map(|k|k.attack_sound)){self.play_at_feet(sound);}
        true
    }
