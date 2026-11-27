#!/usr/bin/env python3
"""Scaffold's scoped actual hit.inside override; preserve all ordinary callers."""
from pathlib import Path

HERE = Path(__file__).resolve().parent


def apply(edit, once):
    edit('crates/horizon-client/src/game/scaffold_click_inside.rs',
         lambda _: (HERE / 'scaffold-click-inside-native.rs').read_text(encoding='utf8'))

    def game(text):
        text = once(text, 'mod block_items;\n', 'mod block_items;\nmod scaffold_click_inside;\n')
        text = once(text, '    guest_use: Option<GuestUse>,',
                    '    guest_use: Option<GuestUse>,\n    client_scaffold_click_inside: Option<bool>,')
        return once(text, '            guest_use: None,',
                    '            guest_use: None,\n            client_scaffold_click_inside: None,')

    edit('crates/horizon-client/src/game.rs', game)
    edit('crates/horizon-client/src/game/use_hand.rs', lambda s: once(s,
        '            inside: false,\n',
        '            inside: self.client_scaffold_click_inside.unwrap_or(false),\n'))

    def remote(text):
        old = 'inside: hit.face == [0; 3],'
        assert text.count(old) == 2, text.count(old)
        return text.replace(old, 'inside: self.client_scaffold_click_inside.unwrap_or(hit.face == [0; 3]),')

    edit('crates/horizon-client/src/game/remote.rs', remote)
    return {'hook_names': [], 'helper_names': []}
