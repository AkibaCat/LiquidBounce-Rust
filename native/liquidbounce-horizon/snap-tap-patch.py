#!/usr/bin/env python3
"""Expose actual rebound-key matching; existing raw input/movement seams suffice."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HOOK_NAMES=[]
HELPER_NAMES=['client_snap_tap_key_mask']
HOOKS=(HERE/'snap-tap-host.rs').read_text(encoding='utf-8')

def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,anchor,HOOKS+'\n'+anchor))
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
