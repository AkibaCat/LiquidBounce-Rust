from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 def game(s):
  s=once(s,'    client_macros_wake_at: std::cell::Cell<Option<i64>>,','    client_macros_wake_at: std::cell::Cell<Option<i64>>,\n    client_auto_chat_game_wake_at: std::cell::Cell<Option<i64>>,')
  return once(s,'            client_macros_wake_at: std::cell::Cell::new(None),','            client_macros_wake_at: std::cell::Cell::new(None),\n            client_auto_chat_game_wake_at: std::cell::Cell::new(None),')
 edit('crates/horizon-client/src/game.rs',game)
 a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,a,(HERE/'auto-chat-game-host.rs').read_text(encoding='utf8')+'\n'+a))
 def idle(s):
  s=once(s,'        self.client_macros_poll_wake();','        self.client_macros_poll_wake();\n        self.client_auto_chat_game_poll_wake();')
  a='event_loop.set_control_flow(self.client_macros_control_flow(self.client_auto_queue_control_flow(self.client_inventory_control_flow(flow))));'
  return once(s,a,'event_loop.set_control_flow(self.client_auto_chat_game_control_flow(self.client_macros_control_flow(self.client_auto_queue_control_flow(self.client_inventory_control_flow(flow)))));')
 edit('crates/horizon-client/src/game/window_handler.rs',idle)
 edit('crates/horizon-client/src/game/play_input.rs',lambda s:once(s,'        self.client_macros_wake_at.set(None);','        self.client_macros_wake_at.set(None);\n        self.client_auto_chat_game_wake_at.set(None);'))
 return {'hook_names':['client_auto_chat_game_wake'],'helper_names':['client_auto_chat_game_schedule']}
