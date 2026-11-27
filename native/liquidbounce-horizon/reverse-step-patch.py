#!/usr/bin/env python3
"""ReverseStep real jump observation, final keys and native game-tick replay."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 for target,source in [
  ('crates/horizon-client/src/reverse_step.rs',HERE.parent/'liquidbounce-behaviors/src/reverse_step.rs'),
  ('crates/horizon-client/src/reverse_step_bridge.rs',HERE/'reverse-step-bridge.rs'),
  ('crates/horizon-client/src/game/reverse_step.rs',HERE/'reverse-step-native.rs')]:
  edit(target,lambda _,p=source:p.read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod high_jump_bridge;\n','pub mod high_jump_bridge;\npub mod reverse_step;\npub mod reverse_step_bridge;\n'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod edge_assistance;\n','mod edge_assistance;\nmod reverse_step;\n'))
 edit('crates/horizon-client/src/motion.rs',lambda s:once(s,'            crate::high_jump_bridge::jump(&mut power);','            crate::reverse_step_bridge::jump();\n            crate::high_jump_bridge::jump(&mut power);'))
 def hooks(s):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,anchor,'    pub(super) fn native_client_reverse_step_tick(&mut self,value:&str){self.reverse_step_tick(value);}\n'+anchor)
  anchor='        [*forward,*backward,*left,*right,*jump,*shift,*sprint]=self.movement_guard_input([*forward,*backward,*left,*right,*jump,*shift,*sprint]);\n'
  s=once(s,anchor,anchor+'        horizon_client::reverse_step_bridge::set_keys([*forward,*backward,*left,*right,*jump,*shift,*sprint]);\n')
  return once(s,'        super::edge_assistance::clear();\n','        super::edge_assistance::clear();\n        super::reverse_step::clear();\n')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 return {'hook_names':[],'helper_names':['client_reverse_step_tick']}
