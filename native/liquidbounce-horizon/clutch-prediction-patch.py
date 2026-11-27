#!/usr/bin/env python3
"""Clutch's complete real-outline/full-FallingPlayer native query.

This dependency alone does not register Clutch. A complete consumer must
provide real Scaffold.enabled/blockCount and implement Scaffold/reset.
"""
from pathlib import Path

HERE = Path(__file__).resolve().parent


def apply(edit, once):
    edit('crates/horizon-client/src/game/clutch_prediction.rs',
         lambda _: (HERE / 'clutch-prediction-native.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs', lambda s: once(s,
         'mod falling_forecast;\n', 'mod falling_forecast;\nmod clutch_prediction;\n'))
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    host = (HERE / 'clutch-prediction-host.rs').read_text(encoding='utf8')
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda s: once(s, anchor, host + '\n' + anchor))
    return {'hook_names': [], 'helper_names': ['client_clutch_query']}
