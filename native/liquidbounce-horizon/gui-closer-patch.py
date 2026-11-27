#!/usr/bin/env python3
"""GUICloser's ScreenEvent at actual container setScreen HEAD.

Compose after AutoF5, inventory lifecycle, AutoBreak and ItemScroller. Each
accepted branch retains their original assignment and initialization bodies.
No cancellation flag survives the call or affects another capture operation.
"""
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent

def assignment(screen):
    return 'self.screen = { let client_next_screen = { self.client_inventory_screen_replaced(); '+screen+' }; self.client_keymapping_screen(client_next_screen) }'

def guarded(text, once, screen, tail='', count=1, accepted_head=''):
    # The caller's mouse-use release belongs to the accepted setScreen path.
    if tail.endswith('self.capture(false);'):
        candidate=assignment(screen)+tail
        if text.count(candidate+'\n        self.right = false;')==count:
            tail+='\n        self.right = false;'
    before=assignment(screen)+tail
    if text.count(before)!=count:raise ValueError(('GUICloser boundary',screen,text.count(before),count))
    after='{ let client_gui_closer_next = '+screen+'; if self.client_gui_closer_accept_screen(client_gui_closer_next) { '+accepted_head+before+' } }'
    return text.replace(before,after,count)

def player_inventory(text):
    # Both keyboard paths choose a screen then perform its initialization. The
    # event must precede the entire branch so capture and creative init cancel too.
    pattern=re.compile(r'(?P<indent> *)if self\.creative \{\n(?P<body> *'+re.escape(assignment('Screen::Creative'))+r';\n.*?\n *)(?:\} else \{)\n *'+re.escape(assignment('Screen::Inventory'))+r';\n *\}\n(?: *self\.craft_size = 2;\n)? *self\.capture\(false\);',re.S)
    matches=list(pattern.finditer(text))
    if len(matches)!=1:raise ValueError(('GUICloser player inventory',len(matches)))
    m=matches[0];whole=m.group(0);indent=m['indent'];start=m.start();end=m.end()
    reset=indent+'self.recipe_ui.reset_screen();\n'
    if text[:start].endswith(reset):start-=len(reset);whole=reset+whole
    release=re.match(r'(?:\n *//[^\n]*)*\n *self\.release_use_for_keybind_screen\(\);',text[end:])
    if release:whole+=release.group(0);end+=len(release.group(0))
    replacement=indent+'let client_gui_closer_next = if self.creative { Screen::Creative } else { Screen::Inventory };\n'+indent+'if self.client_gui_closer_accept_screen(client_gui_closer_next) {\n'+whole+'\n'+indent+'}'
    return text[:start]+replacement+text[end:]

def native_player_inventory(text,once):
    # The mapped-key handler is appended after the inventory lifecycle patch.
    # These are real screen replacements; unlike the two temporary inventory
    # contexts used by mapping/click prediction, they need one observer call.
    before='''                self.recipe_ui.reset_screen();
                if self.creative {
                    self.screen = { let client_next_screen = super::Screen::Creative; self.client_keymapping_screen(client_next_screen) };
                    self.refresh_creative_tabs();
                    self.creative_screen.reopen();
                    self.refresh_saved_hotbars();
                } else { self.screen = { let client_next_screen = super::Screen::Inventory; self.client_keymapping_screen(client_next_screen) }; }
                self.craft_size = 2;
                self.capture(false);
                self.release_use_for_keybind_screen();'''
    after='''                let client_gui_closer_next = if self.creative { Screen::Creative } else { Screen::Inventory };
                if self.client_gui_closer_accept_screen(client_gui_closer_next) {
                    self.client_inventory_screen_replaced();
'''+before+'''
                }'''
    return once(text,before,after)

def apply(edit, once):
    def native(text):
        anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        text=once(text,anchor,(HERE/'gui-closer-host.rs').read_text(encoding='utf8')+'\n'+anchor)
        text=once(text,'''    fn auto_f5_menu_title(&self) -> Option<(Option<String>, String)> {
        if self.title.is_some() || !matches!(self.screen, Screen::Inventory | Screen::Creative) {
            return None;
        }
        if self.screen == Screen::Creative { return Some((None, String::new())); }
''','''    fn auto_f5_menu_title(&self) -> Option<(Option<String>, String)> {
        if self.title.is_some() { return None; }
        self.client_container_menu_title(self.screen)
    }
    fn client_container_menu_title(&self, screen: Screen) -> Option<(Option<String>, String)> {
        if !matches!(screen, Screen::Inventory | Screen::Creative) { return None; }
        if screen == Screen::Creative { return Some((None, String::new())); }
''')
        return native_player_inventory(text,once)
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    # Ordinary container openings have their menu/title constructed beforehand,
    # as MenuScreens.create/LocalPlayer.openHorseInventory do in vanilla.
    for path,screen,accepted in [('game/boats.rs','super::Screen::Inventory',''),('game/mount_menu.rs','Screen::Inventory',''),('game/minecarts.rs','super::Screen::Inventory',''),('game/remote_stations.rs','Screen::Inventory','self.recipe_ui.reset_screen(); '),('game/storage.rs','super::Screen::Inventory',''),('game/trading.rs','super::Screen::Inventory','if self.remote.is_some() { self.recipe_ui.reset_screen(); } ')]:
        edit('crates/horizon-client/src/'+path,lambda text,s=screen,a=accepted:guarded(text,once,s,';\n        self.capture(false);',accepted_head=a))
    def game(text):
        text=guarded(text,once,'Screen::Inventory',';\n        self.capture(false);')
        # CreativeModeInventoryScreen.containerTick -> InventoryScreen.
        return once(text,'            if !self.creative {\n                if let Some(c) = self.carried.take() {','            if !self.creative && self.client_gui_closer_accept_screen(Screen::Inventory) {\n                if let Some(c) = self.carried.take() {')
    edit('crates/horizon-client/src/game.rs',game)
    edit('crates/horizon-client/src/game/play_keys.rs',player_inventory)
    def remote(text):
        text=player_inventory(text)
        # Recipe state belongs to the prospective screen, while Horizon stores
        # it on Game. Keep the old screen's ghost/overlay state when cancelled.
        text=once(text,'''        // A new RecipeBookComponent owns fresh ghost slots and an overlay,
        // even when the server replaces a menu without a close packet.
        self.recipe_ui.reset_screen();
        let menu = self''','''        let menu = self''')
        text=once(text,'            self.remote_open_lectern(target.unwrap_or_default());','            self.recipe_ui.reset_screen();\n            self.remote_open_lectern(target.unwrap_or_default());')
        text=once(text,'                // Unsupported menu: tell the server we closed it.','                self.recipe_ui.reset_screen();\n                // Unsupported menu: tell the server we closed it.')
        # Unlike vanilla, Horizon initializes a mod menu's visual surface before
        # its final assignment. Deliver the event immediately before that init,
        # after constructing the server-sent menu/title, and only once.
        start=text.index('        let title = menu_title(&p.title);')
        end=text.index('    /// `ClientPacketListener.handleMountScreenOpen`',start)
        part=text[start:end]
        part=once(part,'        if let Some(entity) = entity {','''        if !self.client_gui_closer_accept_screen(Screen::Inventory) { return; }
        self.recipe_ui.reset_screen();
        if let Some(entity) = entity {''')
        # This also checks crafting (entity=None), before any native screen init.
        return text[:start]+part+text[end:]
    edit('crates/horizon-client/src/game/remote.rs',remote)
    def mode(text):
        return once(text,'            Screen::Creative => {\n','            Screen::Creative => {\n                if !self.client_gui_closer_accept_screen(Screen::Creative) { return; }\n')
    edit('crates/horizon-client/src/game/game_mode.rs',mode)
    def options(text):
        # Preserve evaluation (including take()) of a dynamic return screen;
        # observer, assignment and screen init run only after acceptance.
        start=text.index('                    self.screen = { let client_next_screen = { self.client_inventory_screen_replaced(); if let Some(screen) = self.panel.recipes_settings_return.take() {')
        body_start=text.index('if let Some(screen)',start)
        body_end=text.index(' }; self.client_keymapping_screen(client_next_screen) };',body_start)
        expression=text[body_start:body_end]
        end=text.index('                    self.show_calibration((0.0, 0.0));',body_end)+len('                    self.show_calibration((0.0, 0.0));')
        new='                    let client_gui_closer_next = '+expression+';\n                    if self.client_gui_closer_accept_screen(client_gui_closer_next) {\n                        self.client_inventory_screen_replaced();\n                        self.screen = self.client_keymapping_screen(client_gui_closer_next);\n                        self.dragging = false;\n                        self.show_calibration((0.0, 0.0));\n                    }'
        return text[:start]+new+text[end:]
    edit('crates/horizon-client/src/game/options.rs',options)
    import importlib.util
    spec=importlib.util.spec_from_file_location('gui_closer_view_patch',HERE/'gui-closer-view-patch.py')
    view=importlib.util.module_from_spec(spec);spec.loader.exec_module(view);view.apply(edit,once)
    return {'hook_names':['client_gui_closer_screen'],'helper_names':[]}
