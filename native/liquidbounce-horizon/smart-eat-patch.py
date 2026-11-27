#!/usr/bin/env python3
"""SmartEat snapshot/key/render seams and actual Item.use RETURN wrappers.

This fragment does not infer PASS from Horizon's consumed-action bool.
"""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
HOOK_NAMES=['client_smart_eat_wait','client_use_key_pressed','client_item_interacted']
HELPER_NAMES=['client_smart_eat_snapshot','client_smart_eat_draw']
HOOKS=(HERE/'smart-eat-host.rs').read_text(encoding='utf-8').replace('__ANIMATION_OVERRIDES__',(HERE/'smart-eat-animation-overrides.rs').read_text(encoding='utf-8'))
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    def native(text):
        text=once(text,anchor,HOOKS+'\n'+anchor)
        text=once(text,'    pub(super) fn native_client_tick_pre(&mut self) {\n',
            '    pub(super) fn native_client_tick_pre(&mut self) {\n        self.native_client_smart_eat_wait();\n')
        text=once(text,'        let down = match key {\n            Some(k) => self.movement_key_held(k),\n            None => self.mouse_reading().1[usize::from(name == "key.use")],\n        };\n',
            '        let mut down = match key {\n            Some(k) => self.movement_key_held(k),\n            None => self.mouse_reading().1[usize::from(name == "key.use")],\n        };\n        if name == "key.use" {self.native_client_use_key_pressed(&mut down);}\n')
        return text
    edit('crates/horizon-client/src/game/native_hooks.rs',native)
    # MultiActions moves Minecraft.handleKeybinds' held-use query into hand.rs.
    # Compose after that transform while keeping the standalone seam testable.
    multi_actions=False
    def held_use(text):
        nonlocal multi_actions
        anchor='        if self.right\n            && self.use_delay == 0\n'
        if anchor in text:
            return once(text,anchor,'        if self.client_use_key_down()\n            && self.use_delay == 0\n')
        if text.count('self.tick_local_hand(client_keybind_was_using)')!=1:
            raise ValueError('SmartEat expected original or MultiActions held-use query')
        multi_actions=True
        return text
    edit('crates/horizon-client/src/game.rs',held_use)
    if multi_actions:
        edit('crates/horizon-client/src/game/hand.rs',lambda text:once(text,
            '                if self.right && self.use_delay == 0 && !was_using && self.using.is_none() && !self.dialog_open() {\n',
            '                if self.client_use_key_down() && self.use_delay == 0 && !was_using && self.using.is_none() && !self.dialog_open() {\n'))
    edit('crates/horizon-client/src/game/using_item.rs',lambda text:once(text,'        if !self.right || hand_changed || self.screen != Screen::Playing {\n','        if !self.client_use_key_down() || hand_changed || self.screen != Screen::Playing {\n'))
    def remote(text):
        text=once(text,'        if self.mod_use_tick(slot, ticks, !self.right || !in_hand) {\n','        if self.mod_use_tick(slot, ticks, !self.client_use_key_down() || !in_hand) {\n')
        return once(text,'        let (attack, use_item) = (self.left, self.right);\n','        let (attack, use_item) = (self.left, self.client_use_key_down());\n')
    edit('crates/horizon-client/src/game/remote.rs',remote)
    edit('crates/horizon-client/src/game/frame_geometry.rs',lambda text:once(text,'        let legacy_block = self.pvp_legacy_look()\n            && self.right\n','        let legacy_block = self.pvp_legacy_look()\n            && self.client_use_key_down()\n'))
    result=runpy.run_path(str(HERE/'smart-eat-return-patch.py'))['apply'](edit,once)
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES,**result}
