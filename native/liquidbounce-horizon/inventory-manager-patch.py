#!/usr/bin/env python3
"""General InventoryManager native operations, snapshots and full resyncs."""
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
HELPER_NAMES=['client_inventory_snapshot','client_inventory_click','client_inventory_creative','client_inventory_close','client_inventory_move_cursor','client_inventory_use','client_inventory_schedule']

def observe_screens(text):
    # Wrap the RHS, so assignments in match arms and multiline `if` values
    # keep their expression shape. All nested braces/parentheses are balanced.
    pattern=re.compile(r'\b(?P<owner>(?:self|game|g|guest)(?:\.game)?)\.screen\s*=(?![=>])\s*')
    edits=[]
    for match in pattern.finditer(text):
        start=match.end();index=start;depth=0;quoted=False;escape=False
        while index<len(text):
            c=text[index]
            if quoted:
                if escape:escape=False
                elif c=='\\':escape=True
                elif c=='"':quoted=False
            elif c=='"':quoted=True
            elif c in '([{':depth+=1
            elif c in ')]}':
                if depth==0:break
                depth-=1
            elif c in ';,' and depth==0:break
            index+=1
        if index==len(text):raise ValueError('Unterminated screen assignment')
        edits.append((start,index,'{ '+match['owner']+'.client_inventory_screen_replaced(); '+text[start:index]+' }'))
    for start,end,value in reversed(edits):text=text[:start]+value+text[end:]
    return text

def apply(edit,once):
    helper=(HERE/'inventory-manager-host.rs').read_text(encoding='utf-8')
    def native(text):
        text=observe_screens(text)
        return once(text,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',helper+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    def game(text):
        text=once(text,'    inventory: Vec<Option<ItemStack>>,\n    selected: usize,\n','    inventory: Vec<Option<ItemStack>>,\n    selected: usize,\n    client_inventory_screen_serial: std::cell::Cell<u64>,\n    client_inventory_creative_drop_count: std::cell::Cell<i32>,\n    client_inventory_wake_at: std::cell::Cell<Option<i64>>,\n')
        text=once(text,'            selected: 0,\n','            selected: 0,\n            client_inventory_screen_serial: std::cell::Cell::new(1),\n            client_inventory_creative_drop_count: std::cell::Cell::new(0),\n            client_inventory_wake_at: std::cell::Cell::new(None),\n')
        text=once(text,'mod menu_clicks;\n','mod menu_clicks;\nmod client_vomit_blocks;\n')
        # A legacy in-process single player has no Remote; its own entity
        # still owns a TickThrottler. Default single player uses Remote.
        text=once(text,'        horizon_client::spans::time("tick_hand", || self.tick_hand());\n','        if self.remote.is_none() { self.client_inventory_drop_tick(); }\n        horizon_client::spans::time("tick_hand", || self.tick_hand());\n')
        return observe_screens(text)
    edit('crates/horizon-client/src/game.rs',game)
    edit('crates/horizon-client/src/game/client_vomit_blocks.rs',lambda _: (HERE/'vomit-block-registry.rs').read_text(encoding='utf-8'))
    # The old AutoTool-only callback intentionally ignored non-player menus.
    # Generic manager sees all full resyncs and only container-0 slot resyncs,
    # exactly as the source READ_FINAL_STATE packet listener.
    def remote(text):
        text=once(text,'    fn next_sequence(&mut self) -> i32 {\n','    pub(super) fn next_sequence(&mut self) -> i32 {\n')
        text=once(text,'    fn remote_use_item(&mut self, hand: Hand) -> bool {\n','    pub(super) fn remote_use_item(&mut self, hand: Hand) -> bool {\n')
        text=once(text,'    fn to_network(&self, s: &Option<ItemStack>) -> Option<Stack> {\n','    pub(super) fn to_network(&self, s: &Option<ItemStack>) -> Option<Stack> {\n')
        text=once(text,'        self.remote_handle_input();\n','        self.remote_handle_input();\n        self.client_inventory_drop_tick();\n')
        text=once(text,'C::ContainerSetContent(p) if p.container==0 => Some(json!({"kind":"content","items":',
            'C::ContainerSetContent(p) => Some(json!({"kind":"content","container":p.container,"items":')
        text=once(text,'C::ContainerSetSlot(p) if p.container==0 => Some(json!({"kind":"slot","slot":p.slot,',
            'C::ContainerSetSlot(p) if p.container==0 => Some(json!({"kind":"slot","container":p.container,"slot":p.slot,')
        return text
    edit('crates/horizon-client/src/game/remote.rs',remote)
    def input(text):
        return once(text,'    pub(super) fn input_player_replaced(&mut self) {\n','    pub(super) fn input_player_replaced(&mut self) {\n        self.client_inventory_creative_drop_count.set(0);\n        self.client_inventory_wake_at.set(None);\n')
    edit('crates/horizon-client/src/game/play_input.rs',input)
    def idle(text):
        text=once(text,'    pub(super) fn idle(&mut self, event_loop: &dyn EventLoopCtl) {\n','    pub(super) fn idle(&mut self, event_loop: &dyn EventLoopCtl) {\n        self.client_inventory_poll_wake();\n')
        return once(text,'        event_loop.set_control_flow(event_loop::redraw_flow(\n            &mut self.frame_drawn,\n            self.options.offscreen,\n            Instant::now(),\n        ));\n','        let flow = event_loop::redraw_flow(\n            &mut self.frame_drawn,\n            self.options.offscreen,\n            Instant::now(),\n        );\n        event_loop.set_control_flow(self.client_inventory_control_flow(flow));\n')
    edit('crates/horizon-client/src/game/window_handler.rs',idle)
    # Cover each concrete GUI replacement at its actual assignment, including
    # a same-enum reopen. Temporary snapshot/click screen switches above are
    # implementation details and deliberately do not replace the GUI object.
    for path in sorted((ROOT/'upstream/Horizon/crates/horizon-client/src/game').rglob('*.rs')):
        # native_hooks was observed before injecting temporary host switches;
        # native_machines restores a borrowed fake-player context, not a GUI.
        if path.name in ('native_hooks.rs','native_machines.rs'):continue
        original=path.read_text(encoding='utf-8')
        if not re.search(r'\b(?:self|game|g|guest)(?:\.game)?\.screen\s*=(?![=>])',original):continue
        relative=path.relative_to(ROOT/'upstream/Horizon').as_posix()
        edit(relative,observe_screens)
    return {'hook_names':['client_inventory_wake'],'helper_names':HELPER_NAMES}
