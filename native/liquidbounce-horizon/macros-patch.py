#!/usr/bin/env python3
"""Real KeyEvent consumer, Main delay wake and shared client-command boundary."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    def game(s):
        s=once(s,'    client_auto_queue_wake_at: std::cell::Cell<Option<i64>>,\n','    client_auto_queue_wake_at: std::cell::Cell<Option<i64>>,\n    client_macros_wake_at: std::cell::Cell<Option<i64>>,\n    client_macros_swing_scope: Option<bool>,\n    client_macros_swung: bool,\n')
        return once(s,'            client_auto_queue_wake_at: std::cell::Cell::new(None),\n','            client_auto_queue_wake_at: std::cell::Cell::new(None),\n            client_macros_wake_at: std::cell::Cell::new(None),\n            client_macros_swing_scope: None,\n            client_macros_swung: false,\n')
    edit('crates/horizon-client/src/game.rs',game)
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'macros-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    def idle(s):
        s=once(s,'        self.client_auto_queue_poll_wake();\n','        self.client_auto_queue_poll_wake();\n        self.client_macros_poll_wake();\n')
        return once(s,'        event_loop.set_control_flow(self.client_auto_queue_control_flow(self.client_inventory_control_flow(flow)));\n','        event_loop.set_control_flow(self.client_macros_control_flow(self.client_auto_queue_control_flow(self.client_inventory_control_flow(flow))));\n')
    edit('crates/horizon-client/src/game/window_handler.rs',idle)
    edit('crates/horizon-client/src/game/play_input.rs',lambda s:once(s,'        self.client_auto_queue_wake_at.set(None);\n','        self.client_auto_queue_wake_at.set(None);\n        self.client_macros_wake_at.set(None);\n'))
    def chat(s):
        for name in['auto_disable','auto_account']:
            s=once(s,f'        let mut handled=false;self.native_client_{name}_command(message,&mut handled);if handled{{return;}}\n','')
        return once(s,'    pub(super) fn handle_chat_input(&mut self, message: &str) {\n','    pub(super) fn handle_chat_input(&mut self, message: &str) {\n        let mut handled=false;self.native_client_commands_chat(message,&mut handled);if handled{return;}\n')
    edit('crates/horizon-client/src/game/chat.rs',chat)
    edit('crates/horizon-client/src/game/hand.rs',lambda s:once(s,'    pub(super) fn swing_hand(&mut self, hand: Hand, animation: SwingAnimation) {\n','    pub(super) fn swing_hand(&mut self, hand: Hand, animation: SwingAnimation) {\n        if let Some(hide)=self.client_macros_swing_scope{self.client_macros_swung=true;if hide{return;}}\n'))
    return{'hook_names':['client_macros_wake','client_commands_chat'],'helper_names':['client_macros_snapshot','client_macros_send','client_macros_schedule','client_macros_item','client_commands_say']}
