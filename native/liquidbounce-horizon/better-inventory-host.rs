    /// Query at the exact slot HEAD / itemCooldown TAIL / extractSlots AFTER
    /// points; the module actor owns all settings and click tracking.
    pub(super) fn native_client_better_inventory_render(&self, query: &str, draw: &mut String) {
        mod_api::dispatch_client_better_inventory_render(self, mod_api::mods(), query, draw);
    }
    fn client_better_inventory_query(&self, query: serde_json::Value) -> serde_json::Value {
        let mut draw = "null".to_owned();
        self.native_client_better_inventory_render(&query.to_string(), &mut draw);
        serde_json::from_str(&draw).unwrap_or(serde_json::Value::Null)
    }
    fn client_better_inventory_color(value: &serde_json::Value) -> u32 {
        let argb = value.as_i64().unwrap_or(-1) as u32;
        rgba((argb >> 16) as u8, (argb >> 8) as u8, argb as u8, (argb >> 24) as u8)
    }
    /// Like the initial JVM FORMAT locale: user's region, independently from
    /// Minecraft's selected resource language. Java caches its default too.
    fn client_better_inventory_locale() -> &'static str {
        horizon_system::initial_format_locale()
    }
    pub(super) fn client_better_inventory_begin(&mut self, ox: f32, oy: f32) {
        let mut mapping = self.client_inventory_mapping(false);
        if self.creative_tab_open() {
            for (index,slot) in mapping.iter_mut().take(45).enumerate() {*slot=Some(super::Creative::DISPLAY+index);}
        } else if self.creative_inventory_open() {
            // selectTab adds a destroy slot after InventoryMenu's 46 wrappers.
            mapping.push(Some(1023));
        }
        *self.client_better_inventory_slots.borrow_mut() = mapping;
        self.client_better_inventory_origin.set((ox, oy));
    }
    pub(super) fn client_better_inventory_slot(&self, batch: &mut HudBatch, internal: usize, x: f32, y: f32) {
        let index = self.client_better_inventory_slots.borrow().iter().position(|slot| *slot == Some(internal));
        let Some(index) = index else { return; };
        let (ox,oy)=self.client_better_inventory_origin.get();
        let draw=self.client_better_inventory_query(serde_json::json!({"kind":"slot","index":index,"x":(x-ox) as i32,"y":(y-oy) as i32}));
        if draw["kind"]=="border" {
            if let Some(rects)=draw["rects"].as_array() { for rect in rects {
                let v=|i:usize|rect[i].as_i64().unwrap_or(0) as f32;
                let (x0,y0,x1,y1)=(v(0),v(1),v(2),v(3));
                batch.fill(&self.gui.atlas,ox+x0.min(x1),oy+y0.min(y1),(x1-x0).abs(),(y1-y0).abs(),Self::client_better_inventory_color(&draw["color"]));
            }}
        } else if draw["kind"]=="texture" {
            let rect=&draw["rect"];let v=|i:usize|rect[i].as_i64().unwrap_or(0) as f32;
            batch.sprite_scaled(&self.gui.atlas,"gui/sprites/hud/hotbar_selection",ox+v(0),oy+v(1),v(2),v(3),u32::MAX);
        }
    }
    pub(super) fn client_better_inventory_cooldown(&self, batch:&mut HudBatch, stack:&ItemStack, x:f32,y:f32, progress:f32, group:&str) {
        let (ox,oy)=self.client_better_inventory_origin.get();
        let entry=self.cooldowns.render_entry(group);
        let draw=self.client_better_inventory_query(serde_json::json!({"kind":"cooldown","empty":stack.count==0,"player":self.client_auto_tool_player(),"group":group,"progress":progress.to_bits() as i32,"end":entry.map(|e|e.0),"tick":entry.map(|e|e.1),"x":(x-ox) as i32,"y":(y-oy) as i32,"locale":Self::client_better_inventory_locale()}));
        if draw["kind"]!="text" {return;}
        let text=draw["text"].as_str().unwrap_or("");
        // Font.width is an integer before centeredText divides by two.
        let width=HudBatch::text_width(&self.gui.atlas,text) as i32;
        let px=draw["centerX"].as_i64().unwrap_or(0) as i32-width/2;
        let py=draw["y"].as_i64().unwrap_or(0) as i32;
        let scale=f32::from_bits(draw["scale"].as_i64().unwrap_or(0) as u32);
        let first=batch.vertices.len();
        batch.text(&self.gui.atlas,text,ox+px as f32,oy+py as f32,Self::client_better_inventory_color(&draw["color"]),true);
        batch.scale_since(first,ox,oy,scale,scale);
    }
    /// Preserve holes and the actual length of ItemContainerContents.itemCopies.
    /// Component resolution includes item defaults and explicit removals.
    fn client_better_inventory_contents(&self, stack:&ItemStack)->Option<Vec<Option<ItemStack>>> {
        let view=horizon_client::item_render::stack_view(stack);
        let components=self.pack.item_renderer.models.components_view(&view);
        let value=components.get("minecraft:container")?;
        let entries=value.as_array()?;
        let mut contents=Vec::new();
        for entry in entries {
            let slot=entry.get("slot").and_then(serde_json::Value::as_u64)? as usize;
            if slot>255 {return None;}
            contents.resize(contents.len().max(slot+1),None);
        }
        // Use the same component codec for prototype and patched entries.
        for entry in entries {
            let slot=entry["slot"].as_u64()? as usize;
            contents[slot]=horizon_items::item_data::stack_from_hover(&entry["item"]);
        }
        Some(contents)
    }
    pub(super) fn client_better_inventory_end(&mut self,batch:&mut HudBatch) {
        let (ox,oy)=self.client_better_inventory_origin.get();
        if self.carried.is_none() {
            let hovered=self.slot_at(batch,self.cursor.0,self.cursor.1);
            let stack=hovered.and_then(|slot| {
                if self.creative_tab_open() && slot>=super::Creative::DISPLAY {
                    let first=self.creative_screen.first_row(&self.pack.names);
                    self.creative_screen.items(&self.pack.names).get(first*9+slot-super::Creative::DISPLAY).cloned()
                } else {self.slot_view(slot)}
            });
            if let Some(stack)=stack {
                let contents=self.client_better_inventory_contents(&stack);
                let nonempty=contents.as_ref().map(|items|items.iter().map(|item|item.as_ref().is_some_and(|item|item.count>0)).collect::<Vec<_>>());
                let draw=self.client_better_inventory_query(serde_json::json!({"kind":"container","carriedEmpty":true,"hovered":hovered.is_some(),"empty":stack.count==0,"stacks":nonempty,"left":ox as i32,"top":oy as i32,"mouseX":self.cursor.0 as i32,"mouseY":self.cursor.1 as i32}));
                if draw["kind"]=="container" && let Some(contents)=contents {
                    let f=|value:&serde_json::Value|f32::from_bits(value.as_i64().unwrap_or(0) as u32);
                    let (cx,cy)=(ox+f(&draw["center"][0]),oy+f(&draw["center"][1]));
                    let (width,height)=(draw["dimensions"][0].as_i64().unwrap_or(0) as f32,draw["dimensions"][1].as_i64().unwrap_or(0) as f32);
                    let scale=f(&draw["scale"]);let first=batch.vertices.len();
                    let (left,top)=(cx-width*0.5,cy-height*0.5);
                    // Nested itemCooldown draws scale about this list pose origin.
                    self.client_better_inventory_origin.set((left,top));
                    if let Some(indices)=draw["indices"].as_array() {for (i,index) in indices.iter().enumerate() {
                        let (x,y)=(left+(i%9) as f32*18.0,top+(i/9) as f32*18.0);
                        batch.sprite_scaled(&self.gui.atlas,"gui/sprites/container/slot",x,y,18.0,18.0,u32::MAX);
                        if let Some(Some(stack))=index.as_u64().and_then(|index|contents.get(index as usize)) {self.draw_item(batch,stack,x+1.0,y+1.0);}
                    }}
                    batch.scale_since(first,cx,cy,scale,scale);
                }
            }
        }
        self.client_better_inventory_origin.set((0.0,0.0));
    }
