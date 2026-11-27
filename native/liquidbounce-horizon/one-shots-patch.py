#!/usr/bin/env python3
"""Actual NameCollector player list and Kick/AutoLeave native effects."""
from pathlib import Path
HERE = Path(__file__).resolve().parent

def apply(edit, once):
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda source:
         once(source, anchor, (HERE / 'one-shots-host.rs').read_text(encoding='utf8') + '\n' + anchor))
    anchor = 'impl Remote {\n'
    edit('crates/horizon-client/src/game/remote.rs', lambda source: once(source, anchor, anchor + '''    pub(super) fn liquidbounce_online_player_names(&self) -> Vec<String> {
        self.tab_players.values().map(|player| player.name.clone()).collect()
    }
'''))
    return {'hook_names': [], 'helper_names': ['client_name_collector_names', 'client_auto_leave_context', 'client_kick']}
