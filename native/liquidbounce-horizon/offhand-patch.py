"""Actual player hazards, FallingPlayer, Smart packets and existing native clicks."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'offhand-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod crosshair_falling;','mod crosshair_falling;\nmod offhand_falling;\nmod offhand_native;'))
    for name in ['offhand-falling','offhand-native']:
        edit('crates/horizon-client/src/game/'+name.replace('-','_')+'.rs',lambda _,name=name:(HERE/(name+'.rs')).read_text(encoding='utf8'))
    return {'hook_names':[],'helper_names':['client_offhand_context','client_offhand_smart']}
