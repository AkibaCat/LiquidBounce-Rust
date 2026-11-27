#!/usr/bin/env python3
"""Prove the shared replay preserves the frozen Crosshair loop; no Rust execution."""
from pathlib import Path
import ast
import re
import runpy
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def main():
    originals = {name: (HERE / name).read_bytes() for name in [
        'crosshair-falling.rs', 'falling-forecast-native.rs',
        'falling-forecast-crosshair.rs', 'falling-forecast-patch.py',
    ]}
    original = originals['crosshair-falling.rs'].decode('utf8').replace('\r\n', '\n')
    expected = original.replace(
        '//! Genuine cloned local PlayerMotion, fixed transformed keys, no jumping.',
        '//! Shared full FallingPlayer replay: real loaded chunks, source hooks and timed effects.',
    )
    expected, changed = re.subn(
        r'pub\(super\) fn collision\([^\n]+\{',
        '''pub(super) fn visit(
    g: &mut Game<'_>,
    ticks: i32,
    add_y: f64,
    mut after_movement: impl FnMut(&horizon_client::motion::PlayerMotion, i32) -> bool,
) -> std::result::Result<(), String> {''', expected, count=1,
    )
    assert changed == 1
    expected = expected.replace('    let result = (|| {', '    let result: std::result::Result<(), String> = (|| {')
    expected = expected.replace(
        'if motion.player.ground.on_ground {\n                return Ok(Some(tick));',
        'if after_movement(motion.player, tick) {\n                return Ok(());',
    ).replace('        Ok(None)\n', '        Ok(())\n')
    # Mechanical fixed-key extraction: the original wrapper still constructs
    # exactly the same FallingPlayer keys; Scaffold supplies its own edge keys.
    expected = expected.replace('    mut after_movement: impl FnMut', '    after_movement: impl FnMut', 1)
    expected = expected.replace('    let mut source = p.clone();\n    let mut body = p.clone();\n', '', 1)
    expected = expected.replace('    body.auto_jump_enabled = false;', '''    visit_with_keys(g, ticks, add_y, keys, after_movement)
}
/// The same real SimulatedPlayer replay with the caller's fixed keys. Edge
/// prediction supplies jump=false and sneak=false, unlike FallingPlayer.
pub(super) fn visit_with_keys(
    g: &mut Game<'_>,
    ticks: i32,
    add_y: f64,
    keys: Keys,
    mut after_movement: impl FnMut(&horizon_client::motion::PlayerMotion, i32) -> bool,
) -> std::result::Result<(), String> {
    if !g.native_client_slot_owner() {
        return Err("no local player for player simulation".into());
    }
    let p = g.player.as_ref().ok_or("no local player")?;
    let mut source = p.clone();
    let mut body = p.clone();
    body.auto_jump_enabled = false;''', 1)

    texts = {}
    staged = ROOT / 'build/staging/horizon'
    generated = {}

    def edit(name, transform):
        if name not in texts:
            path = staged / name
            generated[name] = path.read_bytes() if path.exists() else None
            texts[name] = generated[name].decode('utf8').replace('\r\n', '\n') if generated[name] else ''
        texts[name] = transform(texts[name])

    def once(text, old, new):
        assert text.count(old) == 1, (old, text.count(old))
        return text.replace(old, new)

    for path in [Path(__file__), HERE / 'falling-forecast-patch.py']:
        ast.parse(path.read_text(encoding='utf8'))
    fragment = runpy.run_path(str(HERE / 'falling-forecast-patch.py'))['apply'](edit, once)
    assert fragment == {'hook_names': [], 'helper_names': []}
    assert texts['crates/horizon-client/src/game.rs'].count('mod falling_forecast;') == 1
    wrapper = texts['crates/horizon-client/src/game/crosshair_falling.rs']
    assert 'super::falling_forecast::visit(game, ticks, add_y' in wrapper
    assert 'landing = Some(tick);' in wrapper and 'if player.ground.on_ground {' in wrapper
    with tempfile.TemporaryDirectory(prefix='liquidbounce-falling-static-') as temporary:
        temporary = Path(temporary)
        paths = []
        for name, value in [
            ('expected.rs', expected),
            ('actual.rs', texts['crates/horizon-client/src/game/falling_forecast.rs']),
            ('wrapper.rs', wrapper),
        ]:
            path = temporary / name
            path.write_text(value, encoding='utf8')
            paths.append(str(path))
        subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths], check=True)
        assert (temporary / 'expected.rs').read_text(encoding='utf8') == (temporary / 'actual.rs').read_text(encoding='utf8'), 'full replay drifted beyond visitor extraction'
    for name, raw in originals.items():
        assert (HERE / name).read_bytes() == raw, name
    for name, raw in generated.items():
        path = staged / name
        assert (path.read_bytes() if path.exists() else None) == raw, name
    print('Shared FallingPlayer static checks passed: exact frozen loop extraction, three native files in memory, temporary rustfmt; no production/shared/staging writes or Rust execution')


if __name__ == '__main__':
    main()
