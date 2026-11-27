"""Extinguish uses the shared real placement context and doPlacement service."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/client_extinguish.rs',lambda _:(HERE/'extinguish-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod placement_use;','mod placement_use;\nmod client_extinguish;'))
 a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,a,(HERE/'extinguish-host.rs').read_text(encoding='utf8')+'\n'+a))
 return {'hook_names':['client_extinguish_success'],'helper_names':['client_extinguish_query']}
