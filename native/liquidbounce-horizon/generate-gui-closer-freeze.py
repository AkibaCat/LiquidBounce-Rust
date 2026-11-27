#!/usr/bin/env python3
"""Record owned GUICloser sources and scoped shared integration at handoff."""
from pathlib import Path
import argparse
import hashlib
import json

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def hashes(paths):
    return {path.relative_to(ROOT).as_posix(): digest(path) for path in sorted(set(paths))}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='native/liquidbounce-horizon/gui-closer-freeze-02.json')
    parser.add_argument('--previous-freeze', default='native/liquidbounce-horizon/gui-closer-freeze.json')
    args = parser.parse_args()
    output = ROOT / args.out
    if output.exists():
        raise SystemExit('Refusing to overwrite a freeze manifest')
    owned = [ROOT / 'native/liquidbounce-behaviors/src/gui_closer.rs',
             HERE / 'engine/src/config/gui_closer.rs', HERE / 'engine/src/gui_closer_runtime.rs',
             HERE / 'gui-closer-host.rs', HERE / 'gui-closer-patch.py',
             HERE / 'gui-closer-view.rs', HERE / 'gui-closer-view-patch.py']
    vendor = [path for path in (ROOT / 'native/liquidbounce-behaviors/vendor/java_regex').rglob('*') if path.is_file()]
    supporting = [path for path in HERE.glob('*gui-closer*') if path.is_file() and path not in owned and path != output and not path.name.startswith('gui-closer-freeze')]
    supporting += [path for directory in [HERE / 'gui-closer-source-fixture', HERE / 'gui-closer-config-source-fixture']
                   for path in directory.rglob('*') if path.is_file()]
    selected = ['source-16', 'source-consistency-01', 'jdk-data-02', 'config-04', 'config-source-03',
                'packaging-02', 'current-03', 'view-05', 'titles-07', 'consumption-09']
    reports = []
    for name in selected:
        path = ROOT / 'build' / ('gui-closer-' + name) / 'report.json'
        data = json.loads(path.read_text(encoding='utf8'))
        assert data['passed'], name
        reports.append({'path': path.relative_to(ROOT).as_posix(), 'passed': True,
                        'sha256': digest(path), 'scope': data['scope']})
    history = []
    for path in sorted((ROOT / 'build').glob('gui-closer-*/report.json')):
        if path.parent.name.removeprefix('gui-closer-') in selected:
            continue
        data = json.loads(path.read_text(encoding='utf8'))
        history.append({'path': path.relative_to(ROOT).as_posix(), 'passed': data.get('passed'),
                        'sha256': digest(path)})
    locales = {}
    for code in ['zh_cn', 'zh_tw']:
        path = ROOT / f'native/liquidbounce-interop/resources/localization/display_{code}.json'
        data = json.loads(path.read_text(encoding='utf8'))
        subset = {group: {key: value for key, value in entries.items() if key.startswith('liquidbounce.module.guiCloser.')}
                  for group, entries in data.items() if isinstance(entries, dict)}
        subset = {key: value for key, value in subset.items() if value}
        subset['labels'] = {key: data['labels'][key] for key in ['Mode', 'Filter', 'PrintScreenTitle']}
        locales[path.relative_to(ROOT).as_posix()] = {
            'scope': 'GUICloser descriptions/English descriptions and its three setting labels only',
            'entries': subset,
            'canonicalSha256': hashlib.sha256(json.dumps(subset, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()}
    packet_listener = ROOT / 'build/gui-closer-client-packet-listener.txt'
    manifest = {
        'frozenDate': '2026-11-26', 'revision': 2,
        'previousFreeze': {'path': args.previous_freeze, 'sha256': digest(ROOT / args.previous_freeze)},
        'pinnedSources': {'LiquidBounce': 'ae0eb396c07f22383789b28d5912d714c6ca0a86',
                          'Horizon': '9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0',
                          'Horizon-Mixins': '5a64af938ec86b00d53d16a408aabc0038fb3e57'},
        'scope': 'Revision02 adds pub(super) visibility to the transformed cross-sibling client_container_menu_text helper after production76-02 concrete Game E0624. Exact-body context/title/consumption checks rerun. Final retained-view concrete Game/engine compilation, canonical integration and actual game remain root checks. Native05 is historical before retained-view isolation; native09 failed render generation before concrete Game compile.',
        'files': hashes(owned), 'vendorFiles': hashes(vendor), 'supportingFiles': hashes(supporting),
        'sharedIntegrationFilesAtHandoff': hashes([HERE / 'package.py', ROOT / 'scripts/stage-horizon-build.py']),
        'sharedIntegrationScope': 'Records packaging support at handoff, not exclusive ownership of shared files. Shared behavior/config/lib/actor/runtime registration and make-behavior-patch.py remain root-owned.',
        'localeSubset': locales, 'reports': reports, 'preservedHistoricalReports': history,
        'bytecodeEvidence': {'path': packet_listener.relative_to(ROOT).as_posix(), 'sha256': digest(packet_listener),
                             'scope': 'Actual MC26.3 ClientPacketListener.handlePlaceRecipe active container validation then current Screen RecipeUpdateListener'},
        'preservedEvidenceDirectoriesWithoutReport': [path.relative_to(ROOT).as_posix() for path in sorted((ROOT / 'build').glob('gui-closer-*')) if path.is_dir() and not (path / 'report.json').exists()],
        'pendingChecks': ['Root production76 cumulative concrete Game and native engine build.',
                          'Root actual config actor test gui_closer_aliases_atomic_nullable_filter_and_file_interop_roundtrip.',
                          'Root clientGuiCloserScreen TestEngine/runtime scenario and actual game/GPU verification.']}
    output.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
    print(json.dumps({'path': output.relative_to(ROOT).as_posix(), 'sha256': digest(output),
                      'ownedFiles': len(owned), 'vendorFiles': len(vendor), 'supportingFiles': len(supporting)}))


if __name__ == '__main__':
    main()
