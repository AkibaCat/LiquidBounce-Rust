#!/usr/bin/env python3
"""Original Scaffold BlockItemSelection/AutoBlock dependency, without registration."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def apply(edit, once):
    for target, source in [
        ('crates/horizon-client/src/game/scaffold_block_rules.rs', ROOT / 'native/liquidbounce-behaviors/src/scaffold_block_selection.rs'),
        ('crates/horizon-client/src/game/scaffold_selection.rs', HERE / 'scaffold-selection-native.rs'),
    ]:
        edit(target, lambda _, source=source: source.read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs', lambda s: once(s,
        'mod block_items;\n', 'mod block_items;\nmod scaffold_block_rules;\nmod scaffold_selection;\n'))
    host = (HERE / 'scaffold-selection-host.rs').read_text(encoding='utf8')
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda s: once(s, anchor, host + '\n' + anchor))
    return {'hook_names': [], 'helper_names': ['client_scaffold_selection']}
