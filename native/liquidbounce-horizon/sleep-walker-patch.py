#!/usr/bin/env python3
"""ModuleSleepWalker's complete local player mutation, called from GameTick."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
HOOK_NAMES = []
HELPER_NAMES = ['client_sleep_walker_tick']
HELPERS = (HERE / 'sleep-walker-host.rs').read_text(encoding='utf-8')


def apply(edit, replace_once):
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text:
         replace_once(text, anchor, HELPERS + '\n' + anchor))
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES}
