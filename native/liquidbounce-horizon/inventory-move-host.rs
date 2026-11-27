    /// Real screen, focus and physical key bindings used by InventoryMove.
    pub(super) fn native_client_inventory_move_context(&mut self)->[bool;4] {
        let lectern=matches!(&self.book,Some(super::books::BookScreen::View{lectern:Some(_),..}));
        let container=self.title.is_none()&&(matches!(self.screen,Screen::Inventory|Screen::Creative)||lectern);
        let inventory=container&&self.screen==Screen::Inventory&&self.remote_menu().0==0&&self.container.is_none()&&self.station.is_none()&&!self.mount_active()&&self.craft_size==2;
        let open=self.screen!=Screen::Playing||self.title.is_some()||self.dialog_open()||self.book.is_some()||self.sign_edit.is_some()||self.remote.as_ref().is_some_and(|r|r.level_load.is_active());
        let editing=self.edit_box_focused()||self.sign_edit.is_some()||matches!(&self.book,Some(super::books::BookScreen::Edit{..}));
        horizon_client::inventory_move::observe(horizon_client::inventory_move_model::Screen{open,container,inventory,editing},self.client_inventory_screen_serial.get(),self.native_client_auto_tool_time());
        [open,container,inventory,editing]
    }
    pub(super) fn native_client_inventory_move_keys(&mut self)->[bool;7] {
        self.native_client_inventory_move_context();
        ["key.forward","key.back","key.left","key.right","key.jump","key.sneak","key.sprint"].map(|name|{
            let bound=self.settings.keys.get(name);
            if let Some(button)=horizon_client::keybinds::mouse_button(bound){self.physical_buttons.contains(&button)}
            else {horizon_client::keybinds::key_code(bound).is_some_and(|key|self.physical_keys.contains(&key))}
        })
    }
    pub(super) fn client_inventory_move_keyboard(&mut self,key:KeyCode,pressed:bool){
        let index=match key {KeyCode::KeyW=>0,KeyCode::KeyS=>1,KeyCode::KeyA=>2,KeyCode::KeyD=>3,KeyCode::Space=>4,KeyCode::ShiftLeft=>5,_=>return};
        self.native_client_inventory_move_context();
        if horizon_client::inventory_move::keyboard(index,pressed){self.native_client_inventory_close(0,false);}
    }
    pub(super) fn client_inventory_move_click(&mut self,container:i32)->bool{
        self.native_client_inventory_move_context();horizon_client::inventory_move::safe_click(container)
    }
    pub(super) fn client_inventory_move_next_loop(&mut self){
        for (session,packet) in horizon_client::inventory_move::next_loop(){let _=self.native_send_packet_silently(session,packet);}
    }
    pub(super) fn native_client_inventory_move_expired(&self)->bool{
        horizon_client::inventory_move::expired(self.native_client_auto_tool_time())
    }
