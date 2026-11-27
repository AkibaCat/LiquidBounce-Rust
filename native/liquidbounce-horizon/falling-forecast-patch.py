#!/usr/bin/env python3
"""Shared full FallingPlayer replay, preserving Crosshair's existing landing API."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    edit('crates/horizon-client/src/game/falling_forecast.rs',lambda _:(HERE/'falling-forecast-native.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game/crosshair_falling.rs',lambda _:(HERE/'falling-forecast-crosshair.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod crosshair_falling;','mod crosshair_falling;\nmod falling_forecast;'))
    return {'hook_names':[],'helper_names':[]}
