#!/usr/bin/env python3
"""AntiAFK's actual key, mainhand animation, raw slot and direct angle effects.

The existing client tick and MovementInput hooks supply the event phases;
this extension only adds helpers and never invents a second tick event.
"""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HELPER_NAMES=['client_anti_afk_state','client_anti_afk_forward','client_anti_afk_select_slot','client_anti_afk_rotation','client_anti_afk_swing']
HOOK_NAMES=[]
HELPERS=(HERE/'anti-afk-host.rs').read_text(encoding='utf-8')
def apply(edit,replace_once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:replace_once(text,anchor,HELPERS+'\n'+anchor))
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
