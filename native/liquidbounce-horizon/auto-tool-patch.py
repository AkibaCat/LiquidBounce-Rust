#!/usr/bin/env python3
"""AutoTool's exact continueDestroyBlock HEAD plus inventory actor seams.

Compose after attack, slots, MultiActions; no upstream checkout is modified.
"""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HOOK_NAMES=['client_block_breaking_progress','client_auto_tool_frame','client_inventory_observation','client_silent_hotbar_render']
HELPER_NAMES=['client_auto_tool_snapshot','client_auto_tool_block_registry','client_auto_tool_menu_provider','client_auto_tool_time','client_auto_tool_swap','client_auto_tool_close','client_auto_tool_move_cursor']
HOOKS=(HERE/'auto-tool-host.rs').read_text(encoding='utf-8').replace('__MENU_PROVIDER_MATCH__',(HERE/'auto-tool-menu-providers.rs').read_text(encoding='utf-8'))

def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    def native(text):
        text=once(text,anchor,HOOKS+'\n'+anchor)
        text=once(text,'    pub(super) fn native_frame(&mut self, elapsed: Duration) {\n',
            '    pub(super) fn native_frame(&mut self, elapsed: Duration) {\n        self.native_client_auto_tool_frame();\n')
        return once(text,'    pub(super) fn native_client_tick_post(&mut self) {\n',
            '    pub(super) fn native_client_tick_post(&mut self) {\n        self.client_inventory_previous_rotation.set(self.player.as_ref().map(|p|[p.yaw,p.pitch]));\n')
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    def game(text):
        text=once(text,'    inventory: Vec<Option<ItemStack>>,\n    selected: usize,\n','    inventory: Vec<Option<ItemStack>>,\n    selected: usize,\n    client_inventory_previous_rotation: std::cell::Cell<Option<[f32;2]>>,\n')
        return once(text,'            selected: 0,\n','            selected: 0,\n            client_inventory_previous_rotation: std::cell::Cell::new(None),\n')
    edit('crates/horizon-client/src/game.rs',game)
    def first(text):
        text=once(text,'    fn hit(&mut self) -> Hit;\n','    fn hit(&mut self) -> Hit;\n    fn client_block_breaking_progress(&self, _pos:[i32;3]) {}\n')
        return once(text,'    pub fn continue_destroy(&mut self, w: &mut impl HandWorld, pos: [i32; 3], face: u8) -> bool {\n',
            '    pub fn continue_destroy(&mut self, w: &mut impl HandWorld, pos: [i32; 3], face: u8) -> bool {\n        w.client_block_breaking_progress(pos);\n')
    edit('crates/horizon-client/src/first_person.rs',first)
    def hand(text):
        text=once(text,"impl HandWorld for GameHands<'_, '_> {\n", "impl HandWorld for GameHands<'_, '_> {\n    fn client_block_breaking_progress(&self,pos:[i32;3]) {self.game.native_client_block_breaking_progress(pos);}\n")
        text=once(text,'        let next = [(self.client_hotbar_slot(), main), (OFFHAND, self.offhand.clone())];\n',
            '''        let mut visible_slot=self.client_hotbar_slot() as i32;
        let speed=self.main_hand_attack_speed();
        let mut client_swap_scale=self.hand_strength.swap_scale(1.0,speed);
        self.native_client_silent_hotbar_render(&mut visible_slot,&mut client_swap_scale);
        let visible_slot=usize::try_from(visible_slot).ok().filter(|slot|*slot<9).unwrap_or(self.client_hotbar_slot());
        let visible_main=self.inventory[visible_slot].clone();
        let next = [(visible_slot, visible_main), (OFFHAND, self.offhand.clone())];
''')
        text=once(text,'        let speed = self.main_hand_attack_speed();\n        let replace = self.heights.tick_with(\n','        let replace = self.heights.tick_with(\n')
        return once(text,'            self.hand_strength.swap_scale(1.0, speed),\n','            client_swap_scale,\n')
    edit('crates/horizon-client/src/game/hand.rs',hand)
    def remote(text):
        # native_hooks is a sibling of remote, so the inventory snapshot needs
        # the same game-module visibility as Remote itself.
        text=once(text,'    is_destroying: bool,\n','    pub(super) is_destroying: bool,\n')
        # The HEAD event must precede ensureHasSentCarriedItem, destroy delay,
        # adventure/spectator rejection and every mining-speed/item read.
        text=once(text,'                // MultiPlayerGameMode.continueDestroyBlock starts with ensureHasSentCarriedItem.\n',
            '                self.native_client_block_breaking_progress(hit.pos);\n                // MultiPlayerGameMode.continueDestroyBlock starts with ensureHasSentCarriedItem.\n')
        text=once(text,'        self.native_client_packet(&packet);\n',
            '''        self.native_client_packet(&packet);
        if self.client_auto_tool_player() {
            use serde_json::json;
            let payload=match &packet {
                C::ContainerSetContent(p) if p.container==0 => Some(json!({"kind":"content","items":p.items.iter().enumerate().map(|(slot,item)|json!({"slot":slot,"stack":self.client_inventory_stack_record(self.to_stack(item).as_ref())})).collect::<Vec<_>>()})),
                C::ContainerSetSlot(p) if p.container==0 => Some(json!({"kind":"slot","slot":p.slot,"stack":self.client_inventory_stack_record(self.to_stack(&p.item).as_ref())})),
                C::OpenScreen(_) => Some(json!({"kind":"open"})),
                C::ContainerClose(_) => Some(json!({"kind":"close"})),
                _=>None,
            };
            if let Some(payload)=payload {self.native_client_inventory_observation(&payload.to_string());}
        }
''')
        return text
    edit('crates/horizon-client/src/game/remote.rs',remote)
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
