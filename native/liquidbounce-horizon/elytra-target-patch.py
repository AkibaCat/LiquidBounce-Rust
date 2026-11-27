from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/elytra_target.rs',lambda _:(HERE/'elytra-target-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_bow;','mod auto_bow;\nmod elytra_target;'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'elytra-target-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 edit('crates/horizon-client/src/game/remote.rs',lambda s:s if '    pub(super) fn next_sequence(&mut self) -> i32 {'in s else once(s,'    fn next_sequence(&mut self) -> i32 {','    pub(super) fn next_sequence(&mut self) -> i32 {'))
 return {'hook_names':[],'helper_names':['client_elytra_target_query']}
