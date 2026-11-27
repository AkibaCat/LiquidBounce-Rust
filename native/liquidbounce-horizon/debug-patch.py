"""Native ModuleDebug collection, overlay, simulation and geometry hooks."""
from pathlib import Path
HERE = Path(__file__).resolve().parent

def apply(edit, once):
    edit("crates/horizon-client/src/game/liquidbounce_debug.rs", lambda _: (HERE / "debug-game.rs").read_text(encoding="utf8"))
    def game(s):
        s = once(s, "mod nametags_snapshot;", "mod nametags_snapshot;\nmod liquidbounce_debug;")
        return once(s, "            self.client_nametags_overlay(&mut batch);", "            self.client_nametags_overlay(&mut batch);\n            self.client_debug_overlay(&mut batch);")
    edit("crates/horizon-client/src/game.rs", game)
    def hooks(s):
        anchor = "    pub(super) fn native_client_nametags("
        s = once(s, anchor, "    pub(super) fn native_client_debug(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_debug(self,mod_api::mods(),input,output);}}\n\n" + anchor)
        s = once(s, "    pub(super) fn native_client_tick_pre(&mut self) {", "    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_debug_tick();")
        s = once(s, "        mod_api::dispatch_screen_mouse_scrolled_pre(\n", "        if self.client_debug_scroll(scroll[1],[self.cursor.0,self.cursor.1]){*canceled=true;}\n        mod_api::dispatch_screen_mouse_scrolled_pre(\n")
        return s
    edit("crates/horizon-client/src/game/native_hooks.rs", hooks)
    edit("crates/horizon-client/src/game/world_primitives.rs", lambda s: once(s, "        self.client_stronghold_geometry(&mut primitives,alpha);", "        self.client_stronghold_geometry(&mut primitives,alpha);\n        self.client_debug_geometry(&mut primitives);"))
    return {"hook_names": ["client_debug"], "helper_names": []}
