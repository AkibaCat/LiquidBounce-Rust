#!/usr/bin/env python3
"""Read-only native snapshot for ModuleDerp's actual Sprint key guard."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HELPER_NAMES=['client_derp_state']
HOOK_NAMES=[]
HELPERS=(HERE/'derp-host.rs').read_text(encoding='utf-8')
def apply(edit,replace_once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:replace_once(text,anchor,HELPERS+'\n'+anchor))
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
