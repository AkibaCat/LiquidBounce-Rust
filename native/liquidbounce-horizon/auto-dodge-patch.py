"""Scoped 26.3 AutoDodge body prediction and real arrow/trident state."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/auto_dodge.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/auto_dodge.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/auto_dodge.rs',lambda _:(HERE/'auto-dodge-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/lib.rs',lambda s:once(s,'pub mod navigation;','pub mod navigation;\npub mod auto_dodge;'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_walk_runtime;','mod auto_walk_runtime;\nmod auto_dodge;'))
 edit('crates/horizon-client/src/game/trident.rs',lambda s:once(s,'    returning: u32,','    pub(super) returning: u32,'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    pub(super) fn native_client_auto_walk_config(','''    pub(super) fn native_client_auto_dodge_query(&mut self,value:&str)->String{
        if !self.native_window(){return "null".into();}
        self.auto_dodge_query(value)
    }
    pub(super) fn native_client_auto_walk_config('''))
 return {'hook_names':[],'helper_names':['client_auto_dodge_query']}
