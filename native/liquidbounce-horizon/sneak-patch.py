from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/sneak.rs',lambda _:(HERE/'sneak-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod movement_guard;\n','mod movement_guard;\nmod sneak;\n'))
 def hooks(s):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,anchor,'    pub(super) fn native_client_sneak_context(&self)->Option<[bool;2]>{self.sneak_context()}\n    pub(super) fn native_client_sneak_magma(&mut self,raw:[bool;7])->bool{self.sneak_magma(raw)}\n'+anchor)
  return once(s,'        super::movement_guard::clear();\n','        super::movement_guard::clear();\n        super::sneak::clear();\n')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 return {'hook_names':[],'helper_names':['client_sneak_context','client_sneak_magma']}
