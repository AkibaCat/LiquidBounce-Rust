#!/usr/bin/env python3
"""One focused ProjectilePuncher smoke; integration and localization stay in memory."""
from pathlib import Path
import ast
import json
import re
import runpy
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def main():
    texts = {}
    staged = ROOT / 'build/staging/horizon'

    def edit(name, transform):
        if name not in texts:
            path = staged / name
            texts[name] = path.read_text(encoding='utf8') if path.exists() else ''
        texts[name] = transform(texts[name])

    def once(text, old, new):
        if text.count(old) != 1:
            raise RuntimeError(f'ProjectilePuncher native anchor count {text.count(old)}: {old[:100]}')
        return text.replace(old, new)

    fragment = runpy.run_path(str(HERE / 'projectile-puncher-patch.py'))['apply'](edit, once)
    assert len(fragment['helper_names']) == 3 and not fragment['hook_names']
    hooks = texts['crates/horizon-client/src/game/native_hooks.rs']
    for name in fragment['helper_names']:
        assert hooks.count('fn native_' + name + '(') == 1, name
    hand = texts['crates/horizon-client/src/game/hand.rs']
    assert 'fn swing_hand(&mut self, hand: Hand, animation: SwingAnimation) {\n        if self.client_projectile_puncher_hide_implicit{return;}' in hand
    native = texts['crates/horizon-client/src/game/projectile_puncher.rs']
    assert 'RayBlockMode::Outline' in native and 'client_middle_click::visible' in native
    assert 'client_reach_pick' in native and 'client_weapon_before_attack' in native
    assert 'self.send_action(8)' in texts['crates/horizon-client/src/game/remote.rs']
    assert 'projectile_puncher_math' in texts['crates/horizon-client/src/client_lib.rs']
    plan = runpy.run_path(str(HERE / 'integrate-projectile-puncher.py'))['build_plan']()
    runtime = plan.text['native/liquidbounce-horizon/engine/src/runtime.rs']
    assert runtime.index('self.projectile_puncher_first_tick();') < runtime.index('self.projectile_puncher_prepare(host);') < runtime.index('let projectile_previous_target=') < runtime.index('self.projectile_puncher_tick(host);')
    assert 'PROCESSOR_BASE).filter' in runtime and 'previous_rotation_target()' in runtime
    click = plan.text['native/liquidbounce-behaviors/src/crosshair_clicker.rs']
    assert 'host.click_success(id);' in click
    cross = plan.text['native/liquidbounce-horizon/engine/src/crosshair_clicker_runtime.rs']
    assert 'projectile_puncher_other_click(id)' in cross and 'projectile_puncher_player_press' in cross
    model = (HERE / 'engine/src/projectile_puncher_runtime.rs').read_text(encoding='utf8')
    assert 'p.clicker.record_success(None)' in model and 'pressed_by(&leader, now)' in model
    provider = re.search(r'ROTATION_PROVIDER: u64 = (\d+)', model).group(1)
    for path in (HERE / 'engine/src').glob('*runtime.rs'):
        if path.name != 'projectile_puncher_runtime.rs':
            assert not re.search(r'ROTATION_PROVIDER\s*:\s*u64\s*=\s*' + provider + r'\b', path.read_text(encoding='utf8')), path
    loc = runpy.run_path(str(ROOT / 'scripts/update-projectile-puncher-localization.py'))
    for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
        data = loc['update'](json.loads((loc['FOLDER'] / f'display_{locale}.json').read_text(encoding='utf8')), index)
        assert all(data['descriptions'][loc['description_key'](p)] for p in loc['TEXTS'])
        assert data['descriptions']['liquidbounce.module.projectilePuncher.rotations.angleSmooth.acceleration.yawAcceleration.description']
    with tempfile.TemporaryDirectory(prefix='liquidbounce-projectile-') as temporary:
        temporary = Path(temporary)
        source = (HERE / 'projectile-puncher-smoke.rs').read_text(encoding='utf8')
        source = source.replace('../liquidbounce-behaviors/src/', (ROOT / 'native/liquidbounce-behaviors/src').as_posix() + '/')
        fixture = temporary / 'projectile-smoke.rs'
        fixture.write_text(source, encoding='utf8')
        exe = temporary / 'projectile-smoke.exe'
        subprocess.run(['rustc', '--edition=2024', '--test', str(fixture), '-o', str(exe)], check=True)
        subprocess.run([str(exe), 'projectile_smoke', '--nocapture'], check=True)
    for path in [HERE / 'projectile-puncher-patch.py', HERE / 'integrate-projectile-puncher.py', ROOT / 'scripts/update-projectile-puncher-localization.py', Path(__file__)]:
        ast.parse(path.read_text(encoding='utf8'))
    print(f'ProjectilePuncher checked: 7 focused model tests, {len(texts)} native files in memory, {len(plan.steps)} shared changes/{len(plan.text)} files, {len(loc["TEXTS"])} descriptions plus shared rotation tree; no shared/staging writes')


if __name__ == '__main__':
    main()
