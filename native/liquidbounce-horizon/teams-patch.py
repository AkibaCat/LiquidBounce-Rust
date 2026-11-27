#!/usr/bin/env python3
"""Teams reads genuine scoreboard, formatted names and dyed equipment."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'teams-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'"within_border":within_border,', '"within_border":within_border,"teams":self.client_teams_facts(entity),'))
    return {'hook_names':[],'helper_names':[]}
