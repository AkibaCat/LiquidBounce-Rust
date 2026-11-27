    pub(super) fn client_smart_eat_player(&self) -> bool {
        self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed())
    }
    /// KeyMapping.isDown RETURN. Never write the logical result into the
    /// stored physical/toggle button; each caller gets its own event.
    pub(super) fn client_use_key_down(&self) -> bool {
        let mut pressed=self.right;
        if self.client_smart_eat_player() {
            self.native_client_use_key_pressed(&mut pressed);
        }
        pressed
    }
    /// CoroutineTicker.tick precedes the whole GameTickEvent, including
    /// SilentHotbar priority 1001 and CombatManager's decrement handler.
    pub(super) fn native_client_smart_eat_wait(&mut self) {
        if self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none() {
            mod_api::dispatch_client_smart_eat_wait(self,mod_api::mods());
        }
    }
    pub(super) fn native_client_use_key_pressed(&self, pressed:&mut bool) {
        if self.client_smart_eat_player() {mod_api::dispatch_client_use_key_pressed(self,mod_api::mods(),pressed);}
    }
    /// This accepts the exact Item.use result, never a consumed-action bool.
    pub(super) fn native_client_item_interacted(&mut self, pass:bool) {
        if self.client_smart_eat_player() {mod_api::dispatch_client_item_interacted(self,mod_api::mods(),pass);}
    }
    pub(super) fn native_client_smart_eat_snapshot(&self) -> String {
        use serde_json::json;
        if !self.client_smart_eat_player() {return "null".to_owned();}
        let player=self.player.as_ref().expect("player checked");
        let slots=std::iter::once((-1,self.offhand.as_ref())).chain((0..9).map(|slot|(slot as i32,self.inventory.get(slot).and_then(Option::as_ref))));
        let stacks=slots.map(|(slot,stack)| {
            let stack=stack.filter(|stack|stack.count>0 && &*stack.item!="minecraft:air");
            let Some(stack)=stack else {return json!({"slot":slot,"item":"minecraft:air","food":null,"instantHealth":false});};
            let view=horizon_client::item_render::stack_view(stack);
            let components=self.pack.item_renderer.models.components_view(&view);
            let food=components.get("minecraft:food").map(|food|json!({
                "nutrition":food.get("nutrition").and_then(serde_json::Value::as_i64).unwrap_or(0),
                "canAlwaysEat":food.get("can_always_eat").is_some_and(|v|v.as_bool().unwrap_or_else(||v.as_i64().is_some_and(|n|n!=0))),
            }));
            let contents=components.get("minecraft:potion_contents");
            let base=contents.and_then(|v|v.get("potion")).and_then(serde_json::Value::as_str)
                .and_then(horizon_client::brewing::potion).is_some_and(|p|p.effects.iter().any(|(id,_,_)|*id=="instant_health"));
            let custom=contents.and_then(|v|v.get("custom_effects")).and_then(serde_json::Value::as_array).is_some_and(|effects|effects.iter().any(|e|
                e.get("id").and_then(serde_json::Value::as_str).is_some_and(|id|id=="instant_health"||id=="minecraft:instant_health")));
            json!({"slot":slot,"item":&*stack.item,"food":food,"instantHealth":base||custom})
        }).collect::<Vec<_>>();
        let main=self.inventory.get(self.client_hotbar_slot()).and_then(Option::as_ref).filter(|stack|stack.count>0);
        let mining=main.is_some_and(|s|["minecraft:axes","minecraft:pickaxes","minecraft:shovels","minecraft:hoes"].iter().any(|tag|self.items().item_in_tag(tag,&s.item)));
        let using=self.using.and_then(|(slot,_)|self.stack_in(slot)).is_some_and(|stack| {
            // Subclasses that override Item.getUseAnimation ignore a custom
            // consumable animation. The list is read from actual MC26.3.
            let generic=match &*stack.item {__ANIMATION_OVERRIDES__ _=>true};
            let view=horizon_client::item_render::stack_view(stack);
            let components=self.pack.item_renderer.models.components_view(&view);
            let mut animation=if generic {components.get("minecraft:consumable").map_or(horizon_world::consumable::UseAnimation::None,horizon_client::first_person::consumable_animation)}else {horizon_world::consumable::UseAnimation::None};
            self.native_use_animation(stack,&mut animation);
            matches!(animation,horizon_world::consumable::UseAnimation::Eat|horizon_world::consumable::UseAnimation::Drink)
        });
        json!({"health":player.health.health(),"hunger":player.health.food.food,"canEat":player.invulnerable||player.health.food.food<20,
            "serverSlot":self.client_hotbar_slot(),"mainMiningTool":mining,"usingFoodOrDrink":using,"stacks":stacks}).to_string()
    }
    /// OverlayRenderEvent at extractHotbarAndDecorations HEAD. The module
    /// calls this from gui_layer_pre("minecraft:hotbar"), preserving F1 gates.
    pub(super) fn native_client_smart_eat_draw(&self,batch:&mut HudBatch,slot:i32,offset:i32) {
        if !self.client_smart_eat_player() {return;}
        let stack=if slot==-1 {self.offhand.as_ref()}else {usize::try_from(slot).ok().filter(|s|*s<9).and_then(|s|self.inventory.get(s)).and_then(Option::as_ref)};
        let Some(stack)=stack else {return;};
        let center=(batch.width as i32)/2;
        let x=center.wrapping_sub(117).wrapping_sub(offset) as f32;
        let y=(batch.height as i32).wrapping_sub(19) as f32;
        self.draw_item_decorations(batch,stack,x,y,None);
        self.draw_item_model(batch,stack,x,y);
        let x=center.wrapping_sub(120).wrapping_sub(offset) as f32;
        let y=(batch.height as i32).wrapping_sub(23) as f32;
        self.native_sprite(batch,"hud/hotbar_offhand_left",[0.,0.,29.,24.],[x,y,29.,24.]);
    }
