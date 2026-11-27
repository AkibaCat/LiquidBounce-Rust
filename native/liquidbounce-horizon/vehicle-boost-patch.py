#!/usr/bin/env python3
"""Actual local GameTick snapshot and velocity assignment for VehicleBoost."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
HOOK_NAMES=[]
HELPER_NAMES=['client_vehicle_boost_state','client_vehicle_boost_velocity','client_java_sin_cos']
HELPERS=(HERE/'vehicle-boost-host.rs').read_text(encoding='utf-8')
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,anchor,HELPERS+'\n'+anchor))
    return {'hook_names':HOOK_NAMES,'helper_names':HELPER_NAMES}
