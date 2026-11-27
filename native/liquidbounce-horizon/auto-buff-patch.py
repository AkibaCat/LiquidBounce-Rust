from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
 edit('crates/horizon-client/src/game/auto_buff.rs',lambda _:(HERE/'auto-buff-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_walk_runtime;','mod auto_walk_runtime;\nmod auto_buff;'))
 edit('crates/horizon-client/src/game/potions.rs',lambda s:once(s,"pub(super) struct PotionCloud {\n    pub(super) settings: CloudSettings,\n    effects: Vec<(&'static str, i32, i32)>,","pub(super) struct PotionCloud {\n    pub(super) settings: CloudSettings,\n    pub(super) effects: Vec<(&'static str, i32, i32)>,"))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'auto-buff-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 return {'hook_names':[],'helper_names':['client_auto_buff_query','client_auto_buff_packet','client_auto_buff_drop']}
