#!/usr/bin/env python3
"""Phase-aware real movement replay and per-tick hypothetical sneak probe.

Apply after FallingForecast, ElytraRecast, and Scaffold's prediction fragment.
Existing consumers keep their full-tick API; only an explicit continues_tick
request omits the first base tick, for callers already inside native aiStep.
"""
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
method = runpy.run_path(str(HERE / 'elytra-recast-patch.py'))['method']


def apply(edit, once):
    def motion(text):
        signature = '''    fn local_tick_with_ai_tail(
        &mut self, keys: crate::sprint::Keys,
        sprint: impl FnOnce(&mut PlayerMotion, crate::sprint::Keys) -> Result<()>,
        ai_tail: &mut dyn FnMut(&mut Self) -> Result<()>,
    ) -> Result<()> {'''
        original = method(text, signature)
        extended = once(original, signature, '''    fn local_tick_from_phase(
        &mut self, keys: crate::sprint::Keys,
        sprint: impl FnOnce(&mut PlayerMotion, crate::sprint::Keys) -> Result<()>,
        ai_tail: &mut dyn FnMut(&mut Self) -> Result<()>,
        continues_tick: bool,
    ) -> Result<()> {''')
        extended = once(extended, '''        self.player.health.advance_clock();
        self.fluid_base_tick()?;
        self.update_local_crouching()?;''', '''        if !continues_tick {
            self.player.health.advance_clock();
            self.fluid_base_tick()?;
            self.update_local_crouching()?;
        }''')
        wrapper = signature + '''
        self.local_tick_from_phase(keys, sprint, ai_tail, false)
    }
'''
        text = once(text, original, wrapper + extended)
        signature = 'pub fn liquidbounce_forecast_tick(&mut self, keys: crate::sprint::Keys) -> Result<()> {'
        original = method(text, signature)
        extended = once(original, signature, '''pub fn liquidbounce_forecast_tick_from_phase(&mut self,
    keys: crate::sprint::Keys, continues_tick: bool) -> Result<()> {''')
        extended = once(extended, 'let result = self.local_tick(keys, |p, keys| {',
                        'let result = self.local_tick_from_phase(keys, |p, keys| {')
        extended = once(extended, '    });\n    self.player.replaying = false;',
                        '    }, &mut |_| Ok(()), continues_tick);\n    self.player.replaying = false;')
        wrapper = signature + '''
    self.liquidbounce_forecast_tick_from_phase(keys, false)
}
'''
        return once(text, original, wrapper + extended)

    edit('crates/horizon-client/src/motion.rs', motion)

    def falling(text):
        signature = '''pub(super) fn visit_with_keys(
    g: &mut Game<'_>,
    ticks: i32,
    add_y: f64,
    keys: Keys,
    mut after_movement: impl FnMut(&horizon_client::motion::PlayerMotion, i32) -> bool,
) -> std::result::Result<(), String> {'''
        extended = once(text, signature, '''pub(super) fn visit_with_keys_probe(
    g: &mut Game<'_>,
    ticks: i32,
    add_y: f64,
    keys: Keys,
    continues_tick: bool,
    mut after_movement: impl FnMut(&horizon_client::motion::PlayerMotion, i32, bool) -> bool,
) -> std::result::Result<(), String> {''')
        extended = once(extended, 'let (step, _) = horizon_client::navigation_physics::with_prediction(',
                        'let (step, backed_off_edge) = horizon_client::navigation_physics::with_prediction(')
        extended = once(extended, '|| motion.liquidbounce_forecast_tick(keys),',
                        '|| motion.liquidbounce_forecast_tick_from_phase(keys, continues_tick && tick == 0),')
        extended = once(extended, 'if after_movement(motion.player, tick) {',
                        'if after_movement(motion.player, tick, backed_off_edge) {')
        wrapper = signature + '''
    visit_with_keys_probe(g, ticks, add_y, keys, false,
        |body, tick, _| after_movement(body, tick))
}
'''
        insertion = extended.index('pub(super) fn visit_with_keys_probe(')
        return extended[:insertion] + wrapper + extended[insertion:]

    edit('crates/horizon-client/src/game/falling_forecast.rs', falling)
    return {'hook_names': [], 'helper_names': []}
