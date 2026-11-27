#!/usr/bin/env python3
"""Wire the complete source-derived ModuleSpeed model into Horizon phases."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'speed-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod aimbot;','mod aimbot;\nmod speed;'))
 edit('crates/horizon-client/src/game/speed.rs',lambda _:(HERE/'speed-game.rs').read_text(encoding='utf8'))
 return {'hook_names':[],'helper_names':['client_speed_facts','client_speed_apply']}
