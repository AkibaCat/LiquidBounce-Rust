#!/usr/bin/env python3
"""AutoQueue typed accepted chat, exact snapshots and Main delay wakeups."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    def game(s):
        s=once(s,'    client_inventory_wake_at: std::cell::Cell<Option<i64>>,\n','    client_inventory_wake_at: std::cell::Cell<Option<i64>>,\n    client_auto_queue_wake_at: std::cell::Cell<Option<i64>>,\n')
        return once(s,'            client_inventory_wake_at: std::cell::Cell::new(None),\n','            client_inventory_wake_at: std::cell::Cell::new(None),\n            client_auto_queue_wake_at: std::cell::Cell::new(None),\n')
    edit('crates/horizon-client/src/game.rs',game)
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'auto-queue-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    def remote(s):
        s=once(s,'    fn remote_shift_down(&self) -> bool {','    pub(super) fn remote_shift_down(&self) -> bool {')
        anchor='        if !error {self.native_client_auto_account_message(0,auto_account_original.as_deref().unwrap_or(&line.plain()));}\n'
        s=once(s,anchor,anchor+'        if !error {self.native_client_auto_queue_message(if auto_account_original.is_some(){1}else{0},auto_account_original.as_deref().unwrap_or(&line.plain()));}\n')
        anchor='                self.native_client_auto_account_message(0,&remote_text(&p.content).plain());\n'
        return once(s,anchor,anchor+'                self.native_client_auto_queue_message(2,&remote_text(&p.content).plain());\n')
    edit('crates/horizon-client/src/game/remote.rs',remote)
    def idle(s):
        s=once(s,'        self.client_inventory_poll_wake();\n','        self.client_inventory_poll_wake();\n        self.client_auto_queue_poll_wake();\n')
        return once(s,'        event_loop.set_control_flow(self.client_inventory_control_flow(flow));\n','        event_loop.set_control_flow(self.client_auto_queue_control_flow(self.client_inventory_control_flow(flow)));\n')
    edit('crates/horizon-client/src/game/window_handler.rs',idle)
    edit('crates/horizon-client/src/game/play_input.rs',lambda s:once(s,'        self.client_inventory_wake_at.set(None);\n','        self.client_inventory_wake_at.set(None);\n        self.client_auto_queue_wake_at.set(None);\n'))
    # A sibling private helper is shared only within Game; expose its exact
    # ItemStack.getHoverName implementation, including potion/book components.
    edit('crates/horizon-client/src/game/stations.rs',lambda s:once(s,'    fn stack_hover_name(&self, stack: &ItemStack) -> String {','    pub(super) fn stack_hover_name(&self, stack: &ItemStack) -> String {'))
    return {'hook_names':['client_auto_queue_message','client_auto_queue_wake'],'helper_names':['client_auto_queue_snapshot','client_auto_queue_schedule','client_auto_queue_send','client_auto_queue_raw_use','client_auto_queue_use','client_auto_queue_interact']}
