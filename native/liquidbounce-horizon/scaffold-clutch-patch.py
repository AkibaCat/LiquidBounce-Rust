#!/usr/bin/env python3
"""ScaffoldClutchTechnique dependency; compose after block-placement-patch."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def apply(edit, once):
    for target, source in [
        ('crates/horizon-client/src/game/scaffold_clutch_rules.rs', ROOT / 'native/liquidbounce-behaviors/src/scaffold_clutch.rs'),
        ('crates/horizon-client/src/game/scaffold_clutch.rs', HERE / 'scaffold-clutch-native.rs'),
    ]:
        edit(target, lambda _, source=source: source.read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs', lambda s: once(s,
        'mod block_items;\n', 'mod block_items;\nmod scaffold_clutch_rules;\nmod scaffold_clutch;\n'))
    host = (HERE / 'scaffold-clutch-host.rs').read_text(encoding='utf8')
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda s: once(s, anchor, host + '\n' + anchor))
    return {'hook_names': [], 'helper_names': ['client_scaffold_clutch']}
