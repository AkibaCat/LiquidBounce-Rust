"""Native physical input, Safe prediction gate and next-loop silent replay."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/inventory_move_model.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/inventory_move.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/inventory_move.rs',lambda _:(HERE/'inventory-move-bridge.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod packet_bridge;','pub mod packet_bridge;\npub mod inventory_move;\npub mod inventory_move_model;'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'inventory-move-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 def keys(t):
  t=once(t,'        if self.screen == Screen::Playing && self.replay.playing() {','        if matches!(source,Source::Keyboard) {self.client_inventory_move_keyboard(key,pressed);}\n        if self.screen == Screen::Playing && self.replay.playing() {')
  return once(t,'    pub(super) fn input_player_replaced(&mut self) {','    pub(super) fn input_player_replaced(&mut self) {\n        horizon_client::inventory_move::clear();')
 edit('crates/horizon-client/src/game/play_input.rs',keys)
 edit('crates/horizon-client/src/game/window_handler.rs',lambda t:once(t,'    pub(super) fn idle(&mut self, event_loop: &dyn EventLoopCtl) {','    pub(super) fn idle(&mut self, event_loop: &dyn EventLoopCtl) {\n        self.client_inventory_move_next_loop();'))
 edit('crates/horizon-client/src/game/menu_clicks.rs',lambda t:once(t,'        // BundleMouseActions.onSlotClicked: a Shift click or a swap clears the bundle\'s selection.','        let container=self.remote_menu().0;\n        if self.client_inventory_move_click(container){return true;}\n        // BundleMouseActions.onSlotClicked: a Shift click or a swap clears the bundle\'s selection.'))
 edit('crates/horizon-client/src/game/remote.rs',lambda t:once(t,'    pub(super) fn remote_menu_ops(&mut self, ops: &[(Option<usize>, i32, Input)]) -> bool {','    pub(super) fn remote_menu_ops(&mut self, ops: &[(Option<usize>, i32, Input)]) -> bool {\n        let container=self.remote_menu().0;\n        if self.client_inventory_move_click(container){return true;}'))
 return {'hook_names':[],'helper_names':['client_inventory_move_context','client_inventory_move_keys','client_inventory_move_expired']}
