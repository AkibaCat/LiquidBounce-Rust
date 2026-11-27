#!/usr/bin/env python3
"""Exact wet-expression and push-argument seams for ModuleTridentBoost."""
from pathlib import Path
HERE = Path(__file__).resolve().parent
HOOK_NAMES = ['client_trident_water', 'client_trident_impulse']
HELPER_NAMES = []
HELPERS = (HERE / 'trident-boost-host.rs').read_text(encoding='utf-8')

def apply(edit, once):
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: once(text, anchor, HELPERS+'\n'+anchor))
    def trident(text):
        for needle in ['(riptide && !self.in_water_or_rain())',
                       '(!self.in_water_or_rain() || self.riding.is_some())']:
            text = once(text, needle, needle.replace('self.in_water_or_rain()', 'self.native_trident_water()'))
        text = once(text, '            let d = view_vector(self.yaw, self.pitch);\n',
            '            let d = view_vector(self.yaw, self.pitch);\n'
            '            let impulse = self.native_trident_impulse(d.map(|component| component * f64::from(strength)));\n')
        for index, component in enumerate('xyz'):
            needle = f'p.velocity.{component} + d[{index}] * f64::from(strength)'
            text = once(text, needle, f'p.velocity.{component} + impulse[{index}]')
        return text
    edit('crates/horizon-client/src/game/trident.rs', trident)
    return {'hook_names': HOOK_NAMES, 'helper_names': HELPER_NAMES}
