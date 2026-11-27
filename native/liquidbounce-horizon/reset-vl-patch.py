#!/usr/bin/env python3
"""ResetVL's actual local-player ground read and full setDeltaMovement effect."""
from pathlib import Path
HERE = Path(__file__).resolve().parent


def apply(edit, once):
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',
         lambda source: once(source, anchor, (HERE / 'reset-vl-host.rs').read_text(encoding='utf8') + '\n' + anchor))
    return {'hook_names': [], 'helper_names': ['client_reset_vl_ground', 'client_reset_vl_velocity']}
