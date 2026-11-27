#!/usr/bin/env python3
"""Actual model-parts options access; broadcasting stays on vanilla tick path."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HELPER_NAMES=['client_skin_derp_parts','client_skin_derp_set_parts']
def apply(edit,once):
    helper=(HERE/'skin-derp-host.rs').read_text(encoding='utf-8')
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,
        '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
        helper+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    return {'hook_names':[],'helper_names':HELPER_NAMES}
