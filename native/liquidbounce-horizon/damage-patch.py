#!/usr/bin/env python3
"""Damage ordinary position sends and live vertical-velocity assignment."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda source:once(source,anchor,(HERE/'damage-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    return {'hook_names':[],'helper_names':['client_damage_context','client_damage_send','client_damage_velocity']}
