#!/usr/bin/env python3
"""Actual local rocket boost argument and shared managed-look boundaries."""
from pathlib import Path
HERE = Path(__file__).resolve().parent

HOOKS = '''    /// The actual client's attached rocket, before Vec3.add boost arguments.
    pub(super) fn native_client_firework_boost(
        &self, look: &mut [f64; 3], movement: [f64; 3],
        multiplier: &mut [f64; 3], apply: &mut bool,
    ) {
        if !self.native_window() || self.lan_acting_uuid.is_some()
            || !self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed()) { return; }
        mod_api::dispatch_client_firework_boost(self, mod_api::mods(), look, movement, multiplier, apply);
    }

'''

def apply(edit, once):
    anchor = '    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs', lambda text: once(text, anchor, HOOKS + anchor))
    def fireworks(text):
        start = text.index('    pub(super) fn follow_own_player(')
        end = text.index('    /// The trail', start)
        before = text[start:end]
        after = once(before, '''        let Some(p) = self.player.as_mut() else {
''', '''        let mut look_components = [look.x, look.y, look.z];
        let movement = self.player.as_ref().map(|p| [p.velocity.x, p.velocity.y, p.velocity.z]);
        let mut multiplier = [0.1, 1.5, 0.5];
        let mut apply = false;
        if self.player.as_ref().is_some_and(|p| p.fall_flying) {
            self.native_client_firework_boost(&mut look_components, movement.unwrap(), &mut multiplier, &mut apply);
        }
        let look = Vec3::new(look_components[0], look_components[1], look_components[2]);
        let Some(p) = self.player.as_mut() else {
''')
        original = '''            p.velocity = Vec3::new(
                m.x + look.x * 0.1 + (look.x * 1.5 - m.x) * 0.5,
                m.y + look.y * 0.1 + (look.y * 1.5 - m.y) * 0.5,
                m.z + look.z * 0.1 + (look.z * 1.5 - m.z) * 0.5,
            );
'''
        replacement = '''            p.velocity = if apply {
                let [acceleration, target, convergence] = multiplier;
                Vec3::new(
                    m.x + (look.x * acceleration + (look.x * target - m.x) * convergence),
                    m.y + (look.y * acceleration + (look.y * target - m.y) * convergence),
                    m.z + (look.z * acceleration + (look.z * target - m.z) * convergence),
                )
            } else {
                Vec3::new(
                    m.x + (look.x * 0.1 + (look.x * 1.5 - m.x) * 0.5),
                    m.y + (look.y * 0.1 + (look.y * 1.5 - m.y) * 0.5),
                    m.z + (look.z * 0.1 + (look.z * 1.5 - m.z) * 0.5),
                )
            };
'''
        after = once(after, original, replacement)
        return once(text, before, after)
    edit('crates/horizon-client/src/game/fireworks.rs', fireworks)
    return {'hook_names': ['client_firework_boost'], 'helper_names': []}
