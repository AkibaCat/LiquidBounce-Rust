"""Keep a cancelled native Screen's menu separate from player.containerMenu."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def function(text, name):
    start=text.index(name);brace=text.index('{',start);end=brace+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return start,brace,end

def wrap(text, once, name, arguments, scope, wrapper_signature=None):
    start,brace,end=function(text,name)
    signature=text[start:brace]
    inner_name=name.rsplit('(',1)[0]+'_gui_closer_inner('
    wrapper=wrapper_signature or signature
    call=name.split('fn ',1)[1].split('(',1)[0]+'_gui_closer_inner'
    body=wrapper+'{ self.'+scope+'(|game| game.'+call+'('+arguments+')) }\n    '+signature.replace(name,inner_name)+text[brace:end]
    return text[:start]+body+text[end:]

def apply(edit,once):
    def game(text):
        text=once(text,'mod native_hooks;','mod native_hooks;\nmod gui_closer_view;')
        text=once(text,'    craft_size: usize,','''    craft_size: usize,
    client_gui_closer_view: Option<gui_closer_view::ContainerView>,
    client_gui_closer_other: Option<gui_closer_view::ContainerView>,
    client_gui_closer_in_view: bool,
    client_gui_closer_display_remote: Option<gui_closer_view::RemoteView>,
    client_gui_closer_display_title: Option<(Option<String>, horizon_client::text::Text)>,''')
        text=once(text,'            craft_size: 2,','''            craft_size: 2,
            client_gui_closer_view: None,
            client_gui_closer_other: None,
            client_gui_closer_in_view: false,
            client_gui_closer_display_remote: None,
            client_gui_closer_display_title: None,''')
        text=once(text,'        if &*name == "minecraft:crafting_table" {\n            self.craft_size = 3;','        self.client_gui_closer_prepare_menu();\n        if &*name == "minecraft:crafting_table" {\n            self.craft_size = 3;')
        text=once(text,'                    "{} is not available yet",','                    "{} is not available yet",')
        start,brace,end=function(text,'    fn open_menu(');part=text[start:end]
        part=once(part,'            return;\n        }\n        { let client_gui_closer_next','            self.client_gui_closer_abandon_menu();\n            return;\n        }\n        { let client_gui_closer_next')
        text=text[:start]+part+text[end:]
        text=wrap(text,once,'fn open_menu(', 'pos','with_gui_closer_active')
        text=wrap(text,once,'fn close_container(', '','with_gui_closer_active')
        text=wrap(text,once,'fn hud_layers(', 'size','with_gui_closer_view')
        text=once(text,'        self.tick_enchanting_book();','        self.with_gui_closer_view(|game| game.tick_enchanting_book());')
        return text
    edit('crates/horizon-client/src/game.rs',game)
    edit('crates/horizon-client/src/game/gui_closer_view.rs',lambda text:(HERE/'gui-closer-view.rs').read_text(encoding='utf8'))
    def native(text):
        # The prospective title reads the active new menu. AutoF5 reads the
        # previous Screen title, which remains a component across language changes.
        text=once(text,'        self.client_container_menu_title(self.screen)\n    }','        self.client_gui_closer_display_title().or_else(|| self.client_container_menu_title(self.screen))\n    }')
        name='    fn client_container_menu_title('
        start,brace,end=function(text,name);old=text[start:end]
        new=old.replace('fn client_container_menu_title(', 'pub(super) fn client_container_menu_text(').replace('Option<(Option<String>, String)>','Option<(Option<String>, horizon_client::text::Text)>')
        new=new.replace('        if !matches!(screen,','        use horizon_client::text::Text;\n        if !matches!(screen,',1)
        new=new.replace('Some((None, String::new()))','Some((None, Text::empty()))')
        new=new.replace('map_or_else(String::new, |m| {\n                m.custom_name.clone().unwrap_or_else(|| {','map_or_else(Text::empty, |m| {\n                m.custom_name.clone().map(Text::literal).unwrap_or_else(|| {')
        new=new.replace('                    self.translate(&key, &key)','                    Text::translate_with_fallback(&key, &key, &[])')
        new=new.replace('self.remote_menu_title().map_or_else(String::new, |t| t.plain())','self.remote_menu_title().unwrap_or_else(Text::empty)')
        new=new.replace('self.auto_f5_storage_menu()', 'self.client_gui_closer_storage_text()')
        new=new.replace('merchant.name.clone().unwrap_or_else(|| {','merchant.name.clone().map(Text::literal).unwrap_or_else(|| {')
        new=new.replace('} else { self.translate(station.kind.title_key(), "") };','} else { Text::translate_with_fallback(station.kind.title_key(), "", &[]) };')
        new=new.replace('self.translate("container.crafting", "Crafting")','Text::translate_with_fallback("container.crafting", "Crafting", &[])')
        wrapper='''    fn client_container_menu_title(&self, screen: Screen) -> Option<(Option<String>, String)> {
        self.client_container_menu_text(screen).map(|(menu, title)| (menu, title.plain()))
    }
'''
        text=text[:start]+wrapper+new+text[end:]
        # Screen geometry is the retained Screen's, while menu/stacks/carried
        # are always the actual active player.containerMenu.
        name='pub(super) fn native_client_inventory_snapshot('
        start,brace,end=function(text,name);old=text[start:end]
        wrapper='''pub(super) fn native_client_inventory_snapshot(&mut self) -> String {
        let active = self.with_gui_closer_active(|game| game.native_client_inventory_snapshot_gui_closer_inner());
        if self.client_gui_closer_view.is_none() && !self.client_gui_closer_in_view { return active; }
        let visible = self.with_gui_closer_view(|game| game.native_client_inventory_snapshot_gui_closer_inner());
        let (Ok(mut active), Ok(visible)) = (serde_json::from_str::<serde_json::Value>(&active), serde_json::from_str::<serde_json::Value>(&visible)) else { return active; };
        if active.is_object() && visible.is_object() {
            for key in ["screen", "inventoryScreen", "containerScreen", "visibleCursor"] { active[key] = visible[key].clone(); }
        }
        active.to_string()
    }
    '''
        text=text[:start]+wrapper+old.replace('fn native_client_inventory_snapshot(', 'fn native_client_inventory_snapshot_gui_closer_inner(')+text[end:]
        for name,args in [('click','container, slot, button, input'),('creative','block, wire, count, slot'),('use','hand'),('close','container, close_screen')]:
            text=wrap(text,once,'pub(super) fn native_client_inventory_'+name+'(',args,'with_gui_closer_active')
        text=wrap(text,once,'pub(super) fn native_client_inventory_move_cursor(','cursor','with_gui_closer_view')
        return text
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    def storage(text):
        start,brace,end=function(text,'    pub(super) fn spectator_open_menu(');part=text[start:end]
        part=once(part,'        self.container = Some(key);','        self.client_gui_closer_prepare_menu();\n        self.container = Some(key);')
        text=text[:start]+part+text[end:]
        text=wrap(text,once,'pub(super) fn spectator_open_menu(','pos','with_gui_closer_active')
        start,brace,end=function(text,'    pub(super) fn auto_f5_storage_menu(');old=text[start:end]
        new=old.replace('fn auto_f5_storage_menu(', 'fn client_gui_closer_storage_text(').replace('Option<(String, String)>','Option<(String, Text)>')
        new=new.replace('self.container_title(self.translate("container.shulkerBox", "Shulker Box"))','self.container_parts().iter().find_map(|(key,_)| self.block_name_plain(*key)).map(Text::literal).unwrap_or_else(|| Text::translate_with_fallback("container.shulkerBox", "Shulker Box", &[]))')
        new=new.replace('self.chest_title(rows).plain()', '''{
                if let Some((_, title)) = self.remote_chest_menu() { title }
                else {
                    let (key, fallback) = match self.container_block().as_deref() {
                        Some("minecraft:barrel") => ("container.barrel", "Barrel"),
                        Some("minecraft:ender_chest") => ("container.enderchest", "Ender Chest"),
                        _ if self.open_cart_kind().is_some() => ("entity.minecraft.chest_minecart", "Minecart with Chest"),
                        _ if rows > 3 => ("container.chestDouble", "Large Chest"),
                        _ => ("container.chest", "Chest"),
                    };
                    self.container_parts().iter().find_map(|(key,_)| self.block_name_plain(*key)).map(|name| Text::literal(name).resolve_legacy()).unwrap_or_else(|| Text::translate_with_fallback(key, fallback, &[]))
                }
            }''')
        new=new.replace('self.remote_menu_title().map_or_else(String::new, |t| t.plain())','self.remote_menu_title().unwrap_or_else(Text::empty)')
        new=new.replace('horizon_client::text::Text::literal(self.container_title(self.translate(key, key))).resolve_legacy().plain()','self.container_parts().iter().find_map(|(key,_)| self.block_name_plain(*key)).map(|name| Text::literal(name).resolve_legacy()).unwrap_or_else(|| Text::translate_with_fallback(key, key, &[]))')
        return text[:start]+'''    pub(super) fn auto_f5_storage_menu(&self) -> Option<(String, String)> {
        self.client_gui_closer_storage_text().map(|(menu, title)| (menu, title.plain()))
    }
'''+new+text[end:]
    edit('crates/horizon-client/src/game/storage.rs',storage)
    def remote(text):
        # Construction of supported menus is active even when setScreen cancels.
        text=once(text,'        let title = menu_title(&p.title);\n        if let Some(r)', '        self.client_gui_closer_prepare_menu();\n        let title = menu_title(&p.title);\n        if let Some(r)')
        before='''        if !self.client_gui_closer_accept_screen(Screen::Inventory) { return; }
        self.recipe_ui.reset_screen();
        if let Some(entity) = entity {'''
        text=once(text,before,'        if let Some(entity) = entity {')
        # Build the new menu's actual backing container before the event;
        # native screen initialization belongs only to the accepted Screen.
        text=once(text,'            self.open_native_screen(key, target, p.container, &menu, extra);','''        }
        if !self.client_gui_closer_accept_screen(Screen::Inventory) { return; }
        self.recipe_ui.reset_screen();
        if let Some(key) = self.container {
            self.open_native_screen(key, target, p.container, &menu, extra);''')
        # The old Screen must be captured before mount construction and before
        # merchant metadata changes. Helpers invoked afterward are idempotent.
        start,brace,end=function(text,'    fn remote_open_merchant(');part=text[start:end]
        part=once(part,'        if let Some(r) = self.remote.as_mut() {','        self.client_gui_closer_prepare_menu();\n        if let Some(r) = self.remote.as_mut() {')
        text=text[:start]+part+text[end:]
        start,brace,end=function(text,'    fn remote_open_mount(');part=text[start:end]
        part=once(part,'        let columns =','        self.client_gui_closer_prepare_menu();\n        let columns =')
        part=once(part,'        if !self.open_remote_mount_menu(p.entity as u32, columns) {','        if !self.open_remote_mount_menu(p.entity as u32, columns) {\n            self.client_gui_closer_abandon_menu();')
        text=text[:start]+part+text[end:]
        text=once(text,'        match self.remote.as_ref().and_then(|r| r.menu.clone()) {','''        let menu = if self.client_gui_closer_in_view {
            self.client_gui_closer_display_remote.as_ref().and_then(|v| v.menu.clone())
        } else { self.remote.as_ref().and_then(|r| r.menu.clone()) };
        match menu {''')
        text=once(text,'        Some(self.remote.as_ref()?.menu.as_ref()?.1.as_str())','''        if self.client_gui_closer_in_view { return Some(self.client_gui_closer_display_remote.as_ref()?.menu.as_ref()?.1.as_str()); }
        Some(self.remote.as_ref()?.menu.as_ref()?.1.as_str())''')
        text=once(text,'        self.remote.as_ref()?.menu_title.clone()','''        if self.client_gui_closer_in_view { return self.client_gui_closer_display_remote.as_ref()?.title.as_ref().map(|title| horizon_client::scoreboard::comp_text(&title.component()).resolve_legacy()); }
        self.remote.as_ref()?.menu_title.clone()''')
        start,brace,end=function(text,'    pub(super) fn remote_storage_open(')
        part=text[start:end].replace('self.remote.as_ref().and_then(|r| r.menu.as_ref())','self.remote_menu_type()').replace('Some((_, name))','Some(name)')
        text=text[:start]+part+text[end:]
        start,brace,end=function(text,'    pub(super) fn remote_chest_menu(')
        text=text[:start]+'''    pub(super) fn remote_chest_menu(&self) -> Option<(usize, horizon_client::text::Text)> {
        let rows = self.remote_menu_type()?.strip_prefix("minecraft:generic_9x")?.parse().ok()?;
        Some((rows, self.remote_menu_title().unwrap_or_else(horizon_client::text::Text::empty)))
    }'''+text[end:]
        for name,result in [('remote_menu_ops','true'),('remote_quick_move_unpredicted','true'),('remote_quick_move_all',''),('remote_throw','')]:
            start,brace,end=function(text,'fn '+name+'(')
            text=text[:brace+1]+'\n        if self.client_gui_closer_stale_input() { return '+result+'; }'+text[brace+1:]
        text=wrap(text,once,'pub(super) fn remote_close_container(','','with_gui_closer_active')
        start,brace,end=function(text,'    pub(super) fn handle_packet(')
        # Preserve the original parameter signature; Result propagates normally.
        text=wrap(text,once,'pub(super) fn handle_packet(','packet','with_gui_closer_active')
        return text
    edit('crates/horizon-client/src/game/remote.rs',remote)
    def remote_stations(text):
        start,brace,end=function(text,'    pub(super) fn remote_open_station(')
        text=text[:brace+1]+'\n        self.client_gui_closer_prepare_menu();'+text[brace+1:]
        return text
    edit('crates/horizon-client/src/game/remote_stations.rs',remote_stations)
    def mount(text):
        start,brace,end=function(text,'    pub(super) fn open_mount_menu(');part=text[start:end]
        part=once(part,'        self.mount_menu.open = Some(Open {','        let client_gui_closer_mount = Open {')
        part=once(part,'        });\n        self.show_mount_menu();','        };\n        self.client_gui_closer_prepare_menu();\n        self.mount_menu.open = Some(client_gui_closer_mount);\n        self.show_mount_menu();')
        text=text[:start]+part+text[end:]
        text=once(text,'        self.mount_menu.open = Some(Open {\n            mob: entity,','        self.client_gui_closer_prepare_menu();\n        self.mount_menu.open = Some(Open {\n            mob: entity,')
        text=wrap(text,once,'pub(super) fn open_mount_menu(','index','with_gui_closer_active')
        return wrap(text,once,'pub(super) fn open_remote_mount_menu(','entity, columns','with_gui_closer_active')
    edit('crates/horizon-client/src/game/mount_menu.rs',mount)
    def trading(text):
        text=once(text,'        let mut s = Station::new_kind(Kind::Merchant, [0, 0, 0]);','        self.client_gui_closer_prepare_menu();\n        let mut s = Station::new_kind(Kind::Merchant, [0, 0, 0]);')
        return wrap(text,once,'fn open_merchant_state(','state','with_gui_closer_active')
    edit('crates/horizon-client/src/game/trading.rs',trading)
    for path,name,arg,anchor in [('boats','open_chest_boat_container','i','        let b = &self.boats[i];'),('minecarts','open_cart_container','i','        self.unpack_cart_loot(i);')]:
        def opening(text,name=name,arg=arg,anchor=anchor):
            start,brace,end=function(text,'    pub(super) fn '+name+'(');part=text[start:end]
            part=once(part,anchor,'        self.client_gui_closer_prepare_menu();\n'+anchor)
            return wrap(text[:start]+part+text[end:],once,'pub(super) fn '+name+'(',arg,'with_gui_closer_active')
        edit('crates/horizon-client/src/game/'+path+'.rs',opening)
    def window(text):
        name='pub(super) fn handle_window_event('
        start,brace,end=function(text,name);signature=text[start:brace].replace('mut event: WindowEvent','event: WindowEvent')
        return wrap(text,once,name,'event_loop, id, event, origin','with_gui_closer_view',signature)
    edit('crates/horizon-client/src/game/window_handler.rs',window)
    edit('crates/horizon-client/src/game/pad_menu.rs',lambda text:wrap(text,once,'pub(super) fn pad_menu_frame(','event_loop','with_gui_closer_view'))
    def craft(text):
        return once(text,'            return r.craft_result.clone();','            return if self.client_gui_closer_in_view { self.client_gui_closer_display_remote.as_ref().and_then(|v| v.result.clone()) } else { r.craft_result.clone() };')
    edit('crates/horizon-client/src/game/inventory_slots.rs',craft)
    def recipes(text):
        # MC26.3 handlePlaceRecipe validates the active container id first,
        # then fills the current Screen's RecipeUpdateListener (possibly old).
        text=wrap(text,once,'pub(super) fn remote_ghost_recipe(','p','with_gui_closer_view')
        start,brace,end=function(text,'    pub(super) fn remote_ghost_recipe(')
        text=text[:brace+1]+'\n        if p.container != self.remote_menu_id() { return; }'+text[brace+1:]
        # The retained Screen can belong to a different window, so the active
        # check above replaces the duplicate check in the UI implementation.
        start,brace,end=function(text,'    pub(super) fn remote_ghost_recipe_gui_closer_inner(');part=text[start:end]
        part=once(part,'        let open = self.remote_menu_id();\n','')
        part=once(part,'        if p.container != open {\n            return;\n        }\n','')
        # p otherwise only supplied the window id; the selected entry still
        # comes from Horizon's synced recipe provider.
        part=part.replace('        p: horizon_protocol::packets::recipes::PlaceGhostRecipe,','        _p: horizon_protocol::packets::recipes::PlaceGhostRecipe,')
        return text[:start]+part+text[end:]
    edit('crates/horizon-client/src/game/recipe_book.rs',recipes)
    def menu(text):
        start,brace,end=function(text,'    pub(super) fn menu_ops(')
        return text[:brace+1]+'\n        if self.client_gui_closer_stale_input() { return true; }'+text[brace+1:]
    edit('crates/horizon-client/src/game/menu_clicks.rs',menu)
