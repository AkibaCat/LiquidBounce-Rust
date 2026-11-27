#!/usr/bin/env python3
"""Scaffold original tick native operations; compose after placement-use-patch."""
from pathlib import Path

HERE = Path(__file__).resolve().parent


def apply(edit, once):
    edit('crates/horizon-client/src/game/scaffold_action.rs', lambda _: (HERE / 'scaffold-action-native.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs', lambda s: once(s, 'mod block_items;\n', 'mod block_items;\nmod scaffold_action;\n'))
    host = (HERE / 'scaffold-action-host.rs').read_text(encoding='utf8')
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda s: once(s, anchor, host + '\n' + anchor))
    edit('crates/horizon-world/src/level/place.rs', lambda s: s + '\n' + (HERE / 'scaffold-placement-state.rs').read_text(encoding='utf8'))
    return {'hook_names': ['client_scaffold_success'], 'helper_names': ['client_scaffold_action']}
