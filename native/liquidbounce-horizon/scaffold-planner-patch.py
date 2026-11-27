#!/usr/bin/env python3
"""Scaffold original support/history planner; shared owner composes before Step."""
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def apply(edit, once):
    for target, source in [
        ('crates/horizon-client/src/game/scaffold_planner_rules.rs', ROOT / 'native/liquidbounce-behaviors/src/scaffold_movement_planner.rs'),
        ('crates/horizon-client/src/game/scaffold_planner.rs', HERE / 'scaffold-planner-native.rs'),
    ]:
        edit(target, lambda _, source=source: source.read_text(encoding='utf8'))

    def game(text):
        text = once(text, 'mod block_items;\n', 'mod block_items;\nmod scaffold_planner_rules;\nmod scaffold_planner;\n')
        text = once(text, '    keys: horizon_client::held::HeldKeys,\n', '    keys: horizon_client::held::HeldKeys,\n    scaffold_planner_state: scaffold_planner_rules::Planner,\n')
        return once(text, '            keys: Default::default(),\n', '            keys: Default::default(),\n            scaffold_planner_state: Default::default(),\n')

    edit('crates/horizon-client/src/game.rs', game)
    host = (HERE / 'scaffold-planner-host.rs').read_text(encoding='utf8')
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: once(text, anchor, host + '\n' + anchor))
    return {'hook_names': [], 'helper_names': ['client_scaffold_planner']}
