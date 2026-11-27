#!/usr/bin/env python3
"""Real saved-world and multiplayer operations for the original LiquidBounce menus.

Consumed by make-behavior-patch.py with the same apply(edit, once) contract as
the other exact-source extensions. It never edits upstream or staging itself.
"""
from pathlib import Path

HERE = Path(__file__).resolve().parent


def apply(edit, once):
    def game(text):
        text = once(text, "mod saves_control;\n", "mod saves_control;\nmod liquidbounce_menu;\n")
        text = once(text, "    pending_entry: Option<PendingWorldEntry>,\n",
                    "    pending_entry: Option<PendingWorldEntry>,\n    liquidbounce_menu: liquidbounce_menu::MenuState,\n")
        return once(text, "            pending_entry: None,\n",
                    "            pending_entry: None,\n            liquidbounce_menu: Default::default(),\n")

    edit("crates/horizon-client/src/game.rs", game)
    edit("crates/horizon-client/src/game/liquidbounce_menu.rs",
         lambda _: (HERE / "menu_backend_game.rs").read_text(encoding="utf-8"))
    edit("crates/horizon-client/src/title.rs",
         lambda text: text + "\n" + (HERE / "menu_backend_title.rs").read_text(encoding="utf-8"))
    edit("crates/horizon-client/src/multiplayer.rs",
         lambda text: text + "\n" + (HERE / "menu_backend_multiplayer.rs").read_text(encoding="utf-8"))
    helpers = """    /// Real menu state, without publishing a callback or taking a mod lock.
    pub(super) fn native_client_menu_screen(&self) -> Option<String> {
        self.liquidbounce_menu_screen()
    }
    /// Must be invoked by the mod only after releasing its browser/actor locks.
    pub(super) fn native_client_menu_request(&mut self, action: &str, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        self.liquidbounce_menu_request(action, payload)
    }

"""
    anchor = "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n"
    edit("crates/horizon-client/src/game/native_hooks.rs", lambda text: once(text, anchor, helpers + anchor))
    return {"hook_names": [], "helper_names": ["client_menu_screen", "client_menu_request"]}
