#!/usr/bin/env python3
"""Read-only InventoryTracker engine/config/locale integration preparation."""
from pathlib import Path
import argparse, ast, json, runpy, shutil, subprocess, tempfile
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native', action='store_true', help='Also compose the complete current generator plus the native owned patch')
    args = parser.parse_args()
    names = ['integrate-inventory-tracker.py', 'update-inventory-tracker-locales.py']
    for name in names:
        ast.parse((HERE / name).read_text(encoding='utf8'))
    plan = runpy.run_path(str(HERE / names[0]))['prepare']()
    files = {Path(path).name: source for path, source in plan.changed.items() if path.suffix == '.rs'}
    for path in [ROOT / 'native/liquidbounce-behaviors/src/inventory_tracker.rs',
                 HERE / 'engine/src/inventory_tracker_runtime.rs',
                 HERE / 'engine/src/config/inventory_tracker.rs']:
        files['owned-' + path.name] = path.read_text(encoding='utf8')
    if args.native:
        source = (HERE / 'make-behavior-patch.py').read_text(encoding='utf8')
        source = source[:source.index('\noutput = []')]
        if "'inventory-tracker-patch.py'" not in source:
            source = source.replace("'logoff-spot-patch.py',", "'logoff-spot-patch.py', 'inventory-tracker-patch.py',")
        context = {'__file__': str(HERE / 'make-behavior-patch.py'), '__name__': 'inventory_tracker_static'}
        exec(compile(source, context['__file__'], 'exec'), context)
        assert context['names'].count('client_inventory_tracker') == 1
        assert context['helper_names'].count('client_inventory_tracker_query') == 1
        for name, (_, text) in context['changes'].items():
            if 'inventory_tracker' in name and name.endswith('.rs'):
                files['native-' + Path(name).name] = text
    with tempfile.TemporaryDirectory(prefix='liquidbounce-inventory-tracker-static-') as directory:
        temporary = Path(directory)
        paths = []
        for name, source in files.items():
            path = temporary / name
            path.write_text(source, encoding='utf8')
            paths.append(str(path))
        result = subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths], capture_output=True, text=True)
        if result.returncode:
            raise RuntimeError(result.stderr)
        locale_folder = temporary / 'locales'
        locale_folder.mkdir()
        for locale in ['zh_cn', 'zh_tw']:
            shutil.copyfile(HERE.parent / f'liquidbounce-interop/resources/localization/display_{locale}.json', locale_folder / f'display_{locale}.json')
        update = runpy.run_path(str(HERE / names[1]))['update']
        update(locale_folder)
        first = {path.name: path.read_bytes() for path in locale_folder.iterdir()}
        update(locale_folder)
        assert first == {path.name: path.read_bytes() for path in locale_folder.iterdir()}
        for raw in first.values():
            data = json.loads(raw)
            assert data['moduleNames']['InventoryTracker']
            assert data['labels']['SavePlayers']
            assert data['descriptions']['liquidbounce.module.inventoryTracker.savePlayers.description']
    runtime = files['owned-inventory_tracker_runtime.rs']
    event = runtime[runtime.index('pub(crate) fn inventory_tracker_event'):runtime.index('pub(crate) fn inventory_tracker_flush')]
    assert 'query(host' not in event and 'client_inventory_tracker_query' not in event
    assert 'crate::anti_bot_runtime::query' in event and 'state.is_bot' in event
    shared = '\n'.join(plan.changed.values())
    assert 'InventoryTracker => ("InventoryTracker", "World", "inventoryTracker")' in shared
    assert '"invsee"=>return self.inventory_tracker_command(host,r)' in shared
    print(f'InventoryTracker static checks passed: {len(plan.changed)} transactional shared files, {len(files)} Rust sources parsed in temporary files, Chinese updater idempotent; no compilation/shared/staging writes')


if __name__ == '__main__':
    main()
