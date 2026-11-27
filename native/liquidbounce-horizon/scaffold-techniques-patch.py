#!/usr/bin/env python3
"""Source Normal/Expand techniques; compose after block-placement-patch."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def apply(edit, once):
    for target, source in [
        ('scaffold_aim_rules.rs', ROOT / 'native/liquidbounce-behaviors/src/scaffold_aim.rs'),
        ('scaffold_height_rules.rs', ROOT / 'native/liquidbounce-behaviors/src/scaffold_lifecycle.rs'),
        ('scaffold_technique_rules.rs', ROOT / 'native/liquidbounce-behaviors/src/scaffold_techniques.rs'),
        ('scaffold_techniques.rs', HERE / 'scaffold-techniques-native.rs'),
    ]:
        edit('crates/horizon-client/src/game/' + target, lambda _, source=source: source.read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs', lambda s: once(s, 'mod block_items;\n',
        'mod block_items;\nmod scaffold_aim_rules;\nmod scaffold_height_rules;\nmod scaffold_technique_rules;\nmod scaffold_techniques;\n'))
    host = (HERE / 'scaffold-techniques-host.rs').read_text(encoding='utf8')
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda s: once(s, anchor, host + '\n' + anchor))
    return {'hook_names': [], 'helper_names': ['client_scaffold_technique']}
