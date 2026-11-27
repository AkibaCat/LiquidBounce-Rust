    /// A real GUI replacement increments this even when its Screen enum is
    /// equal (closing/reopening the same kind is a different screen object).
    pub(super) fn client_inventory_screen_replaced(&self) {
        if self.client_auto_tool_player() {
            self.client_inventory_screen_serial.set(self.client_inventory_screen_serial.get().wrapping_add(1));
        }
    }
    pub(super) fn client_inventory_player_menu(&self) -> bool {
        self.remote_menu().0 == 0 && self.container.is_none() && self.station.is_none()
            && !self.mount_active() && self.craft_size == 2
    }
    pub(super) fn client_inventory_container(&self) -> i32 {
        if self.remote.is_some() { self.remote_menu().0 }
        else if self.client_inventory_player_menu() { 0 }
        else { 1 }
    }
    pub(super) fn client_inventory_drop_tick(&self) {
        if self.client_auto_tool_player() {
            let count = self.client_inventory_creative_drop_count.get();
            if count > 0 { self.client_inventory_creative_drop_count.set(count - 1); }
        }
    }
    /// Resume the render-dispatcher coroutine between frames, including its
    /// 5 ms cursor animation and arbitrary millisecond inventory delays.
    pub(super) fn native_client_inventory_wake(&mut self) {
        mod_api::dispatch_client_inventory_wake(self, mod_api::mods());
    }
    pub(super) fn native_client_inventory_schedule(&self, deadline: Option<i64>) {
        self.client_inventory_wake_at.set(deadline);
    }
    pub(super) fn client_inventory_poll_wake(&mut self) {
        if self.client_inventory_wake_at.get().is_some_and(|at| at <= self.native_client_auto_tool_time()) {
            self.client_inventory_wake_at.set(None);
            if self.client_auto_tool_player() { self.native_client_inventory_wake(); }
        }
    }
    pub(super) fn client_inventory_control_flow(&self, flow: winit::event_loop::ControlFlow) -> winit::event_loop::ControlFlow {
        use winit::event_loop::ControlFlow;
        let Some(deadline) = self.client_inventory_wake_at.get() else { return flow; };
        let until = std::time::Instant::now() + std::time::Duration::from_millis(
            deadline.saturating_sub(self.native_client_auto_tool_time()).max(0) as u64);
        match flow {
            ControlFlow::Poll => flow,
            ControlFlow::Wait => ControlFlow::WaitUntil(until),
            ControlFlow::WaitUntil(previous) => ControlFlow::WaitUntil(previous.min(until)),
        }
    }
    pub(super) fn client_inventory_stack(&self, stack: Option<&ItemStack>) -> serde_json::Value {
        let mut value = self.client_inventory_stack_record(stack);
        value["wire"] = serde_json::json!(horizon_items::item_data::canon_stack(stack));
        value
    }
    /// Extract the native menu mapping without accepting the temporary copy.
    /// Creative's ItemPickerMenu is a GUI menu; player.containerMenu is still
    /// inventoryMenu and must be the one used by game-mode container clicks.
    pub(super) fn client_inventory_mapping(&mut self, inventory: bool) -> Vec<Option<usize>> {
        let screen = self.screen;
        if inventory { self.screen = Screen::Inventory; }
        let mut mapping = Vec::new();
        self.menu_session(|menu, _, to_menu| {
            mapping.resize(menu.slot_count(), None);
            for internal in 0..1024 {
                if let Some(slot) = to_menu(internal).filter(|slot| *slot < mapping.len()) {
                    mapping[slot] = Some(internal);
                }
            }
            None
        });
        self.screen = screen;
        mapping
    }
    pub(super) fn native_client_inventory_snapshot(&mut self) -> String {
        use serde_json::json;
        if !self.client_auto_tool_player() { return "null".to_owned(); }
        let base = self.native_client_auto_tool_snapshot([0; 3], false);
        let Ok(base) = serde_json::from_str::<serde_json::Value>(&base) else { return "null".to_owned(); };
        let container = self.client_inventory_container();
        let inventory = self.client_inventory_player_menu();
        let saved_screen = self.screen;
        let mapping = self.client_inventory_mapping(inventory);
        let mut stacks = Vec::with_capacity(mapping.len());
        let mut positions = Vec::with_capacity(mapping.len());
        let mut non_player = Vec::new();
        for (slot, internal) in mapping.into_iter().enumerate() {
            let stack = internal.and_then(|internal| self.slot_view(internal));
            stacks.push(self.client_inventory_stack(stack.as_ref()));
            let position = internal.map_or((0.0, 0.0), |internal| {
                if inventory { Self::client_inventory_position(slot) }
                else { self.slot_position(internal) }
            });
            positions.push([position.0, position.1]);
            if internal.is_none_or(|internal| !(internal < 36 || (500..=504).contains(&internal))) {
                non_player.push(slot);
            }
        }
        let player_stacks: Vec<_> = (0..41).map(|slot| {
            let stack = match slot {
                0..=35 => self.inventory.get(slot).and_then(Option::as_ref),
                36..=39 => self.armor.get(39 - slot).and_then(Option::as_ref),
                _ => self.offhand.as_ref(),
            };
            self.client_inventory_stack(stack)
        }).collect();
        let handled = matches!(saved_screen, Screen::Inventory | Screen::Creative);
        let inventory_screen = saved_screen == Screen::Inventory && inventory;
        let visible_cursor = if handled && !self.captured {
            self.gui_origin().ok().map(|(x, y)| [self.cursor.0 - x, self.cursor.1 - y])
        } else { None };
        let screen_object = self.client_inventory_screen_serial.get();
        let menu_object = if inventory { 1 } else { screen_object.wrapping_add(2) };
        let menu = json!({"container":container,"stacks":stacks,"carried":self.client_inventory_stack(self.carried.as_ref())});
        // CreativeModeInventoryScreen owns ItemPickerMenu, while the player's
        // containerMenu remains InventoryMenu. Keep both geometries distinct.
        let (screen_positions, screen_non_player) = if saved_screen == Screen::Creative {
            let mapping = self.client_inventory_mapping(false);
            let positions: Vec<_> = mapping.iter().enumerate().map(|(slot, internal)| {
                let internal = internal.unwrap_or(super::Creative::DISPLAY + slot);
                let p = self.slot_position(internal); [p.0, p.1]
            }).collect();
            let non_player: Vec<_> = mapping.iter().enumerate().filter_map(|(slot, internal)|
                internal.is_none().then_some(slot)).collect();
            (positions, non_player)
        } else { (positions, non_player) };
        let screen = handled.then(|| json!({"object":screen_object,"container":container,
            "geometry":{"object":if saved_screen==Screen::Creative {screen_object.wrapping_add(2)} else {menu_object},"slots":screen_positions},"nonPlayer":screen_non_player}));
        json!({"now":self.native_client_auto_tool_time(),"creative":self.creative,"menu":menu,
            "inventoryGeometry":{"object":1,"slots":(0..46).map(|slot| {let p=Self::client_inventory_position(slot);[p.0,p.1]}).collect::<Vec<_>>()},
            "screen":screen,"inventoryScreen":inventory_screen,"containerScreen":handled && self.chest_rows().is_some(),
            "noScreen":saved_screen==Screen::Playing && !self.screen_over_playing(),
            "serverControlled":base["serverControlled"],"movement":base["movement"],"jumping":base["jumping"],
            "rotationMatches":base["rotationMatches"],"using":base["using"],"breaking":base["breaking"],
            "visibleCursor":visible_cursor,"stacks":player_stacks,
            "blockRegistrySize":super::client_vomit_blocks::CLIENT_VOMIT_BLOCKS.len(),
            "dropSpamCount":self.client_inventory_creative_drop_count.get()
        }).to_string()
    }
    /// InventoryMenu's vanilla positions, independent of a creative tab, a
    /// currently open container and a content mod's replacement screen.
    pub(super) fn client_inventory_position(slot: usize) -> (f32, f32) {
        match slot {
            0 => (154.0, 28.0),
            1..=4 => (98.0 + ((slot - 1) % 2) as f32 * 18.0, 18.0 + ((slot - 1) / 2) as f32 * 18.0),
            5..=8 => (8.0, 8.0 + (slot - 5) as f32 * 18.0),
            9..=35 => (8.0 + ((slot - 9) % 9) as f32 * 18.0, 84.0 + ((slot - 9) / 9) as f32 * 18.0),
            36..=44 => (8.0 + (slot - 36) as f32 * 18.0, 142.0),
            45 => (77.0, 62.0),
            _ => (0.0, 0.0),
        }
    }
    /// InventoryAction.Click.performAction: use the passed window/server slot
    /// directly, invoke the actual verified menu predictor and normal sender.
    pub(super) fn native_client_inventory_click(&mut self, container: i32, slot: i32, button: i32, input: i32) -> bool {
        if !self.client_auto_tool_player() { return false; }
        let current = self.client_inventory_container();
        if container != current { return true; } // game mode ignores this input
        let input = match input {
            0 => horizon_client::menu::Input::Pickup,
            1 => horizon_client::menu::Input::QuickMove,
            2 => horizon_client::menu::Input::Swap,
            3 => horizon_client::menu::Input::Clone,
            4 => horizon_client::menu::Input::Throw,
            5 => horizon_client::menu::Input::QuickCraft,
            6 => horizon_client::menu::Input::PickupAll,
            _ => return false,
        };
        let saved_screen = self.screen;
        if container == 0 { self.screen = Screen::Inventory; }
        let mapping = self.client_inventory_mapping(container == 0);
        let internal = usize::try_from(slot).ok().and_then(|slot| mapping.get(slot)).copied().flatten();
        if self.remote.is_some() {
            if slot == -999 { self.remote_menu_ops(&[(None, button, input)]); }
            else if let Some(internal) = internal { self.remote_menu_ops(&[(Some(internal), button, input)]); }
        } else if slot == -999 { self.menu_ops(&[(None, button, input)]); }
        else if let Some(internal) = internal { self.menu_ops(&[(Some(internal), button, input)]); }
        self.screen = saved_screen;
        true
    }
    /// Official game-mode Creative helpers send only. In particular an Add is
    /// never written to inventory locally and a Drop never swings the hand.
    pub(super) fn native_client_inventory_creative(&mut self, block: Option<i32>, wire: &str, count: i32, slot: Option<i32>) -> bool {
        if !self.client_auto_tool_player() { return false; }
        let (stack, flags): (Option<ItemStack>, &[&str]) = if let Some(index) = block {
            let Some((_, item, flags)) = usize::try_from(index).ok().and_then(|index| super::client_vomit_blocks::CLIENT_VOMIT_BLOCKS.get(index)) else { return false; };
            (Some(ItemStack::new(item, count.max(0) as u32)), flags)
        } else {
            let Ok(stack) = horizon_items::item_data::parse_stack(wire) else { return false; };
            let name = stack.as_ref().map_or("minecraft:air", |stack| &*stack.item);
            let flags = super::client_vomit_blocks::CLIENT_ITEM_FEATURES.binary_search_by_key(&name, |(name,_)| *name)
                .ok().map_or(&[][..], |index| super::client_vomit_blocks::CLIENT_ITEM_FEATURES[index].1);
            (stack, flags)
        };
        let enabled = self.remote.as_ref().map_or(&self.level.enabled_features, |remote| &remote.features);
        let feature_enabled = flags.iter().all(|flag| enabled.iter().any(|enabled| enabled == flag));
        if self.creative && feature_enabled {
            if let Some(slot) = slot {
                let item = self.to_network(&stack);
                self.send(horizon_protocol::packets::play::Serverbound::SetCreativeModeSlot(horizon_protocol::packets::play::SetCreativeModeSlot { slot: slot as i16, item }));
            } else if !(self.screen == Screen::Inventory) && stack.as_ref().is_some_and(|s| s.count > 0 && &*s.item != "minecraft:air") {
                let item = self.to_network(&stack);
                self.send(horizon_protocol::packets::play::Serverbound::SetCreativeModeSlot(horizon_protocol::packets::play::SetCreativeModeSlot { slot: -1, item }));
                // TickThrottler(20, 1280). The game-mode helper increments,
                // but does not consult its threshold before sending.
                self.client_inventory_creative_drop_count.set(self.client_inventory_creative_drop_count.get().wrapping_add(20));
            }
        }
        true
    }
    pub(super) fn native_client_inventory_use(&mut self, hand: i32) -> bool {
        if !self.client_auto_tool_player() { return false; }
        let hand = match hand { 0 => Hand::Main, 1 => Hand::Off, _ => return false };
        let used = if self.remote.is_some() && !self.spectator()
            && self.with_hand(hand, |game| game.held().is_none()) {
            // MultiPlayerGameMode.useItem creates the predictive packet even
            // for ItemStack.EMPTY, whose Item.use then returns PASS.
            let sequence = self.next_sequence();
            self.send(horizon_protocol::packets::play::Serverbound::UseItem(
                horizon_protocol::packets::play::UseItem {
                    hand: if matches!(hand, Hand::Off) { 1 } else { 0 },
                    sequence, yaw: self.yaw, pitch: self.pitch,
                }));
            false
        } else if self.remote.is_some() && !self.spectator() { self.remote_use_item(hand) }
        else if self.spectator() { false }
        else { self.with_hand(hand, |game| game.use_hand(None).unwrap_or(false)) };
        if used { self.heights.item_used(hand); }
        true
    }
    pub(super) fn native_client_inventory_close(&mut self, container: i32, close_screen: bool) {
        if !self.client_auto_tool_player() { return; }
        if close_screen {
            self.close_container();
            self.client_inventory_screen_replaced();
            self.screen = Screen::Playing;
            self.capture(true);
        } else {
            self.send(horizon_protocol::packets::play::Serverbound::ContainerClose(horizon_protocol::packets::play::ContainerClose { container }));
        }
    }
    pub(super) fn native_client_inventory_move_cursor(&mut self, cursor: [f32; 2]) {
        if !self.client_auto_tool_player() || !matches!(self.screen, Screen::Inventory | Screen::Creative) || self.captured { return; }
        if let Ok((ox, oy)) = self.gui_origin() {
            self.cursor = (ox + cursor[0], oy + cursor[1]);
            if let (Some(window), Some(renderer)) = (&self.window, &self.renderer) {
                let batch = HudBatch::new([renderer.config.width, renderer.config.height]);
                let _ = window.set_cursor_position(winit::dpi::PhysicalPosition::new(self.cursor.0 * batch.scale, self.cursor.1 * batch.scale));
            }
        }
    }
