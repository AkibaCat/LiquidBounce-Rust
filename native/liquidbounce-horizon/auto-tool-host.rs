    pub(super) fn client_auto_tool_player(&self) -> bool {
        self.native_window() && self.lan_acting.is_none() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_reach_queries_allowed())
    }
    pub(super) fn native_client_block_breaking_progress(&self, pos: [i32; 3]) {
        if self.client_auto_tool_player() {
            mod_api::dispatch_client_block_breaking_progress(self, mod_api::mods(), pos);
        }
    }
    pub(super) fn native_client_auto_tool_frame(&mut self) {
        if self.client_auto_tool_player() {
            mod_api::dispatch_client_auto_tool_frame(self, mod_api::mods());
        }
    }
    pub(super) fn native_client_inventory_observation(&self, payload: &str) {
        if self.client_auto_tool_player() {
            mod_api::dispatch_client_inventory_observation(self, mod_api::mods(), payload);
        }
    }
    pub(super) fn native_client_silent_hotbar_render(&self, visible_slot: &mut i32, swap_scale: &mut f32) {
        if self.client_auto_tool_player() {
            mod_api::dispatch_client_silent_hotbar_render(self, mod_api::mods(), visible_slot, swap_scale);
        }
    }
    pub(super) fn native_client_auto_tool_time(&self) -> i64 {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        START.get_or_init(std::time::Instant::now).elapsed().as_millis().min(i64::MAX as u128) as i64
    }
    pub(super) fn client_inventory_stack_record(&self, stack: Option<&ItemStack>) -> serde_json::Value {
        use serde_json::json;
        match stack.filter(|s| s.count > 0 && &*s.item != "minecraft:air") {
            None => json!({"identity":"empty","components":"empty","count":0}),
            Some(stack) => {
                let view=horizon_client::item_render::stack_view(stack);
                let components=self.pack.item_renderer.models.components_view(&view).to_map();
                let components=json!([&*stack.item,components]).to_string();
                json!({"identity":format!("{} {components}",stack.count),"components":components,"count":stack.count})
            },
        }
    }
    pub(super) fn native_client_auto_tool_snapshot(&self, pos: [i32; 3], scan_beds: bool) -> String {
        use serde_json::json;
        if !self.client_auto_tool_player() { return "null".to_owned(); }
        let player = self.player.as_ref().expect("player checked");
        let state = self.world.block(pos[0],pos[1],pos[2]);
        let block = &*self.res.facts().get(state).name;
        let items = self.items();
        let stacks: Vec<_> = (0..36).map(|slot| {
            let stack = self.inventory.get(slot).and_then(Option::as_ref).filter(|s|s.count>0 && &*s.item!="minecraft:air");
            let mut entry = self.client_inventory_stack_record(stack);
            entry["slot"] = json!(slot);
            entry["empty"] = json!(stack.is_none());
            // Tool.getMiningSpeed respects the stack's complete TOOL component.
            entry["destroySpeed"] = json!(items.mining_speed(stack,block));
            entry["efficiency"] = json!(stack.map_or(0,|s|s.enchantment_level("minecraft:efficiency")));
            entry["silkTouch"] = json!(stack.map_or(0,|s|s.enchantment_level("minecraft:silk_touch")));
            // getMaxDamage is independent of isDamageableItem (UNBREAKABLE
            // and an absent DAMAGE component do not erase MAX_DAMAGE).
            entry["maxDamage"] = json!(stack.map_or(0,|s|horizon_client::stations::max_damage(items,s)));
            entry["damage"] = json!(stack.map_or(0,|s| {
                if s.data().is_some_and(|d|d.other("!minecraft:damage").is_some()) {0}
                else {(s.damage.min(i32::MAX as u32) as i32).clamp(0,horizon_client::stations::max_damage(items,s).max(0))}
            }));
            entry
        }).collect();
        let mut bed_distances = Vec::new();
        if scan_beds && let Some(eye) = self.eye_position() {
            let is_head = |state:u32| {
                let state = horizon_world::registry::BlockStateId(state);
                self.res.facts().get(state).has_class("BedBlock") && self.pack.assets.registry.state(state)
                    .and_then(|s|s.properties.get("part")).is_some_and(|part|part=="head")
            };
            for (&(cx,cz),chunk) in &self.world.chunks {
                for (sy,section) in chunk.sections.iter().enumerate() {
                    if !section.blocks.maybe_has(&is_head) {continue;}
                    for index in 0..4096 {
                        if !is_head(section.get(index)) {continue;}
                        let pos=[cx*16+(index&15) as i32,chunk.min_y()+sy as i32*16+(index>>8) as i32,cz*16+((index>>4)&15) as i32];
                        bed_distances.push((0..3).map(|i|(f64::from(pos[i])+0.5-eye[i]).powi(2)).sum::<f64>());
                    }
                }
            }
        }
        let menu_id = self.remote_menu().0;
        let player_inventory = menu_id==0 && self.container.is_none() && self.station.is_none()
            && !self.mount_active() && self.craft_size==2;
        let inventory_screen = self.screen==Screen::Inventory && player_inventory;
        let cursor = if inventory_screen && !self.captured {
            self.gui_origin().ok().map(|(x,y)|[self.cursor.0-x,self.cursor.1-y])
        } else {None};
        let menu_identity=json!([menu_id,
            self.inventory.iter().map(|s|self.client_inventory_stack_record(s.as_ref())).collect::<Vec<_>>(),
            self.armor.iter().map(|s|self.client_inventory_stack_record(s.as_ref())).collect::<Vec<_>>(),
            self.client_inventory_stack_record(self.offhand.as_ref()),
            self.craft_grid.iter().map(|s|self.client_inventory_stack_record(s.as_ref())).collect::<Vec<_>>(),
            self.client_inventory_stack_record(self.craft_result().as_ref()),
            self.client_inventory_stack_record(self.carried.as_ref())]).to_string();
        json!({"block":block,"tick":player.health.tick_count,"creative":self.creative,
            "sneaking":player.shift_key_down,"serverSlot":self.client_hotbar_slot(),
            "beds":bed_distances,"stacks":stacks,"now":self.native_client_auto_tool_time(),
            "playerInventory":player_inventory,"serverControlled":self.riding_mount_with_menu() || self.riding_chest_boat(),
            "cursorEmpty":self.carried.as_ref().is_none_or(|s|s.count==0 || &*s.item=="minecraft:air"),
            "inventoryScreen":inventory_screen,"inventoryOpen":inventory_screen,
            "visibleCursor":cursor,"movement":player.move_vector,"jumping":player.jumping,
            "rotation":[player.yaw,player.pitch],
            "rotationMatches":self.client_inventory_previous_rotation.get()==Some([player.yaw,player.pitch]),
            "using":self.using.is_some(),"breaking":self.remote.as_ref().map_or(self.clicks.destroying.active,|r|r.is_destroying),
            "menuIdentity":menu_identity}).to_string()
    }
    pub(super) fn native_client_auto_tool_block_registry(&self) -> Vec<String> {
        let mut blocks=std::collections::BTreeSet::new();
        for state in 0..self.pack.assets.registry.len() {
            if let Some(state)=self.pack.assets.registry.state(horizon_world::registry::BlockStateId(state as u32)) {
                blocks.insert(state.name.clone());
            }
        }
        blocks.into_iter().collect()
    }
    pub(super) fn native_client_auto_tool_menu_provider(&self, pos:[i32;3]) -> bool {
        if !self.client_auto_tool_player() {return false;}
        let state=self.world.block(pos[0],pos[1],pos[2]);
        let name=&*self.res.facts().get(state).name;
        __MENU_PROVIDER_MATCH__
    }
    pub(super) fn native_client_auto_tool_swap(&mut self, source:i32, hotbar:i32) -> bool {
        if !self.client_auto_tool_player() || !(9..36).contains(&source) || !(0..9).contains(&hotbar) {return false;}
        let snapshot=self.native_client_auto_tool_snapshot([0;3],false);
        let Ok(snapshot)=serde_json::from_str::<serde_json::Value>(&snapshot) else {return false;};
        if snapshot["playerInventory"]!=true || snapshot["serverControlled"]==true || snapshot["cursorEmpty"]!=true {return false;}
        if self.remote.is_some() {
            self.remote_menu_ops(&[(Some(source as usize),hotbar,horizon_client::menu::Input::Swap)])
        } else {
            self.menu_click(Some(source as usize),hotbar,horizon_client::menu::Input::Swap)
        }
    }
    pub(super) fn native_client_auto_tool_close(&mut self) {
        if self.client_auto_tool_player() && self.remote.is_some() && self.remote_menu().0==0 && self.screen!=Screen::Inventory {
            self.send(horizon_protocol::packets::play::Serverbound::ContainerClose(horizon_protocol::packets::play::ContainerClose{container:0}));
        }
    }
    pub(super) fn native_client_auto_tool_move_cursor(&mut self, cursor:[f32;2]) {
        if !self.client_auto_tool_player() || self.screen!=Screen::Inventory || self.captured {return;}
        if let Ok((ox,oy))=self.gui_origin() {
            self.cursor=(ox+cursor[0],oy+cursor[1]);
            if let (Some(window),Some(renderer))=(&self.window,&self.renderer) {
                let batch=HudBatch::new([renderer.config.width,renderer.config.height]);
                let _=window.set_cursor_position(winit::dpi::PhysicalPosition::new(self.cursor.0*batch.scale,self.cursor.1*batch.scale));
            }
        }
    }
