#!/usr/bin/env python3
"""One focused Offhand model smoke plus in-memory integration verification."""
from pathlib import Path
import ast
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
            texts[name] = path.read_text(encoding='utf8').replace('\r\n', '\n') if path.exists() else ''
        texts[name] = transform(texts[name])

    def once(text, old, new):
        if text.count(old) != 1:
            raise RuntimeError(f'Offhand native anchor count {text.count(old)}: {old[:90]}')
        return text.replace(old, new)

    fragment = runpy.run_path(str(HERE / 'offhand-patch.py'))['apply'](edit, once)
    assert fragment == {'hook_names': [], 'helper_names': ['client_offhand_context', 'client_offhand_smart']}
    hooks = texts['crates/horizon-client/src/game/native_hooks.rs']
    assert hooks.count('fn native_client_offhand_context(') == 1
    assert hooks.count('fn native_client_offhand_smart(') == 1
    assert 'action:7' in hooks and 'self.selected=' not in (HERE / 'offhand-host.rs').read_text(encoding='utf8')
    assert 'mod offhand_native;' in texts['crates/horizon-client/src/game.rs']
    plan = runpy.run_path(str(HERE / 'integrate-offhand.py'))['build_plan']()
    runtime = plan.text['native/liquidbounce-horizon/engine/src/runtime.rs']
    inventory = plan.text['native/liquidbounce-horizon/engine/src/inventory_runtime.rs']
    assert 'if action==KeyAction::Press{self.offhand_key(host,code);}' in runtime
    assert inventory.index('self.offhand_plan(host,&snapshot.context);') < inventory.index('self.inventory_cleaner_plan(host,&snapshot.context);')
    assert 'self.offhand_refresh(host);' in inventory and 'b.offhand.outcome(&mut b.inventory,outcome)' in inventory
    config = (HERE / 'engine/src/config/offhand.rs').read_text(encoding='utf8')
    leaf_section = config.split('const LEAVES:')[1].split('fn node(')[0]
    leaves = re.findall(r'\(\s*"([^"]+)"\s*,', leaf_section)
    locales = runpy.run_path(str(ROOT / 'scripts/update-offhand-localization.py'))
    assert len(leaves) == 39, len(leaves)
    assert set(leaves) <= locales['TEXTS'].keys()
    with tempfile.TemporaryDirectory(prefix='liquidbounce-offhand-') as temporary:
        temporary = Path(temporary)
        source = (HERE / 'offhand-smoke.rs').read_text(encoding='utf8').replace('../liquidbounce-behaviors/src/offhand.rs', (ROOT / 'native/liquidbounce-behaviors/src/offhand.rs').as_posix())
        fixture = temporary / 'offhand-smoke.rs'
        fixture.write_text(source, encoding='utf8')
        exe = temporary / 'offhand-smoke.exe'
        subprocess.run(['rustc', '--edition=2024', '--test', str(fixture), '-o', str(exe)], check=True)
        subprocess.run([str(exe), '--nocapture'], check=True)
    for path in [HERE / 'offhand-patch.py', HERE / 'integrate-offhand.py', ROOT / 'scripts/update-offhand-localization.py']:
        ast.parse(path.read_text(encoding='utf8'))
    print(f'Offhand checked: 6 focused model tests, {len(texts)} native files in memory, {len(plan.steps)} shared changes/{len(plan.text)} files, 39 settings plus constraints; no shared/staging writes')


if __name__ == '__main__':
    main()
