#!/usr/bin/env python3
"""MiddleClickAction live key, inventory and player-only collider ray facts."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'middle-click-action-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 edit('crates/horizon-client/src/game.rs',lambda t:once(t,'mod client_ghost_hand;\n','mod client_ghost_hand;\nmod client_middle_click;\n'))
 edit('crates/horizon-client/src/game/client_middle_click.rs',lambda _:(HERE/'middle-click-action-ray.rs').read_text(encoding='utf8'))
 return {'hook_names':[],'helper_names':['client_middle_click_context','client_middle_click_pearl_slot']}
