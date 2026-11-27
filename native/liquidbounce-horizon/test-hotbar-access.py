#!/usr/bin/env python3
"""Compile exact transformed main-hand access/mutation bodies with real stacks."""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def function(text, signature):
    start = text.index(signature)
    end = text.index('{', start) + 1
    depth = 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[start:end]


def between(text, start, end):
    return text[text.index(start):text.index(end, text.index(start))]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/hotbar-access-01')
    parser.add_argument('--prepare-only', action='store_true')
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite existing verification evidence')
    changes, originals = {}, {}

    def edit(name, transform):
        path = ROOT / 'upstream/Horizon' / name
        originals[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        changes[name] = transform(changes.get(name, path.read_text(encoding='utf-8')))

    def once(text, before, after):
        if text.count(before) != 1:
            raise ValueError(f'Expected one anchor: {before[:100]}')
        return text.replace(before, after, 1)

    spec = importlib.util.spec_from_file_location('slot_patch', HERE / 'slot-patch.py')
    patch = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(patch)
    # net.rs slot instrumentation composes after the packet transport hook.
    packet_spec = importlib.util.spec_from_file_location('packet_patch', HERE / 'packet-patch.py')
    packet = importlib.util.module_from_spec(packet_spec)
    packet_spec.loader.exec_module(packet)
    packet.apply(edit, once)
    patch.apply(edit, once)
    game = 'crates/horizon-client/src/game/'
    text = lambda name: changes.get(game + name, (ROOT / 'upstream/Horizon' / game / name).read_text(encoding='utf-8'))
    source = (HERE / 'packet-tests/hotbar-access-fixture.rs').read_text(encoding='utf-8')
    helpers = '\n'.join(function(text('native_hooks.rs'), signature) for signature in [
        '    pub(super) fn native_client_slot_owner(',
        '    pub(super) fn native_client_real_selected_slot(',
        '    pub(super) fn client_hotbar_slot(',
    ])
    held = '\n'.join(function(text('held_items.rs'), signature) for signature in [
        '    pub(super) fn held(', '    pub(super) fn held_mut(', '    pub(super) fn held_slot(',
    ])
    held += '\n' + function(text('using_item.rs'), '    pub(super) fn consume_held(')
    item_get = function(text('commands/items.rs'), '    pub(in crate::game) fn slot_get(')
    item_set = function(text('commands/items.rs'), '    pub(in crate::game) fn slot_set(')
    get_arm = between(item_get, '            Slot::MainHand =>', '            Slot::Armor(')
    set_arm = function(item_set, '            Slot::MainHand =>') + ',\n'
    main_get = '    fn get_main(&mut self, s: Slot) -> Option<ItemStack> {\n        let selected = self.client_hotbar_slot();\n        match s {\n' + get_arm + '            Slot::Inv(i) => self.inventory.get(i).cloned().flatten(),\n        }\n    }\n'
    main_set = '    fn set_main(&mut self, s: Slot, stack: Option<ItemStack>) -> bool {\n        let selected = self.client_hotbar_slot();\n        match s {\n' + set_arm + '            Slot::Inv(_) => false,\n        }\n    }\n'
    native_server = (ROOT / 'upstream/Horizon' / game / 'native_server.rs').read_text(encoding='utf-8')
    take_functions = '\n'.join(function(native_server, signature) for signature in [
        'fn plain(', 'fn order(', "fn slots<'a>(", 'fn take_from(',
    ])
    mutations = function(text('item_entities.rs'), '    pub(super) fn drop_held(')
    mutations += '\n' + function(text('nautili.rs'), '    fn use_nautilus_food(')
    mutations += '\n' + function(text('native_server.rs'), '    pub(super) fn native_held(')
    mutations += '\n' + function(text('native_server.rs'), '    pub(super) fn native_take_items(')
    mutations += '\n' + function(text('mounts.rs'), '    pub(super) fn use_steering_stick(')
    totem = between(text('totem.rs'), '        let main = self.held()', '        self.award_stat(')
    swap = between(text('play_keys.rs'), '                let selected = self.client_hotbar_slot();', '                self.using = None;')
    shelf = between(text('shelf.rs'), '            let selected = self.client_hotbar_slot();', '            self.container_changed(key);')
    saddle = between(text('nautili.rs'), '            let selected = self.client_hotbar_slot();\n            let m', '                let sound = nautilus::saddle_sound(m);')
    equip = function(text('native_living_refs.rs'), 'fn player_slot(')
    armor = function(text('native_living_refs.rs'), 'fn armor_index(')
    equip_call = between(text('native_living_refs.rs'), '            Target::Player => player_slot(', '            Target::Guest(').strip().removeprefix('Target::Player => ').removesuffix(',')
    enchant = between(text('station_control.rs'), '                let selected = self.client_hotbar_slot();', '                self.apply_equipment();')
    blocks = {
        'HELPERS': helpers,
        'HELD': held,
        'MAIN_ACCESS': main_get + main_set,
        'MUTATIONS': mutations,
        'TOTEM': totem,
        'SWAP': swap,
        'SHELF': shelf,
        'SADDLE': saddle + '\n            }\n',
        'EQUIPMENT': armor + '\n' + equip,
        'EQUIPMENT_CALL': equip_call,
        'ENCHANT': enchant,
        'TAKE_FUNCTIONS': take_functions,
        'STICK_BREAK': function(text('mounts.rs'), 'fn stick_on_break('),
    }
    for name, body in blocks.items():
        source = source.replace('/* ' + name + ' */', body)
    source = source.replace('BEHAVIORS_PATH', (ROOT / 'native/liquidbounce-behaviors/src/lib.rs').as_posix())
    source = source.replace('SLOT_BRIDGE_PATH', (HERE / 'slot-bridge.rs').as_posix())
    source = source.replace('SLOTS_PATH', (HERE / 'engine/src/slots.rs').as_posix())
    (out / 'src').mkdir(exist_ok=True)
    (out / 'src/lib.rs').write_text(source, encoding='utf-8', newline='\n')
    dependencies = {
        'horizon-items': ROOT / 'upstream/Horizon/crates/horizon-items',
        'horizon-world': ROOT / 'upstream/Horizon/crates/horizon-world',
        'horizon-protocol': ROOT / 'upstream/Horizon/crates/horizon-protocol',
        'liquidbounce-behaviors': ROOT / 'native/liquidbounce-behaviors',
    }
    manifest = '[package]\nname="liquidbounce-hotbar-access-fixture"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
    manifest += ''.join(f'{name}={{path="{path.as_posix()}"}}\n' for name, path in dependencies.items())
    (out / 'Cargo.toml').write_text(manifest, encoding='utf-8', newline='\n')
    source_paths = [HERE / 'slot-patch.py', HERE / 'slot-bridge.rs', HERE / 'engine/src/slots.rs', HERE / 'packet-tests/hotbar-access-fixture.rs', Path(__file__)]
    hashes = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in source_paths}
    if args.prepare_only:
        print(out / 'Cargo.toml')
        return
    commands = []
    env = os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR', str(ROOT / 'build/hotbar-access-dev/target'))
    for name, argv in [
        ('test', ['cargo', '+stable', 'test', '--manifest-path', str(out / 'Cargo.toml')]),
        ('clippy', ['cargo', '+stable', 'clippy', '--locked', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings']),
    ]:
        start = time.monotonic()
        process = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (out / f'{name}.stdout.log').write_text(process.stdout, encoding='utf-8')
        (out / f'{name}.stderr.log').write_text(process.stderr, encoding='utf-8')
        commands.append({'argv': argv, 'exitCode': process.returncode, 'durationSeconds': round(time.monotonic() - start, 3), 'stdout': str(out / f'{name}.stdout.log'), 'stderr': str(out / f'{name}.stderr.log')})
        if process.returncode:
            break
    retained = []
    for path in sorted((ROOT / 'upstream/Horizon/crates/horizon-client/src').rglob('*.rs')):
        relative = path.relative_to(ROOT / 'upstream/Horizon').as_posix()
        if '/game/' not in relative and not relative.endswith('/game.rs'):
            continue
        for line, content in enumerate(changes.get(relative, path.read_text(encoding='utf-8')).splitlines(), 1):
            if re.search(r'\bself\.selected\b', content):
                retained.append({'path':relative,'line':line,'text':content.strip()})
    report = {
        'passed': all(command['exitCode'] == 0 for command in commands),
        'scope': 'Exact transformed SilentHotbar identity/getter/mutable access bodies, command main-hand branch, drop/swap/totem/shelf/nautilus/steering/native API; real Horizon ItemStack and take_held_drop',
        'commands': commands,
        'sourceSha256': hashes,
        'extractedBlocksSha256': {name: hashlib.sha256(body.encode()).hexdigest() for name, body in blocks.items()},
        'retainedRawSelectedAccesses': retained,
        'retainedRawAccessPolicy': 'Input/UI/debug and persistence selected field; Inventory.add placement order; guest state swap and guest equipment; server-only fake inventory; studio replay reset; upstream test fixture setup. No gameplay main-hand getter/setter remains outside the effective-slot helper.',
        'upstreamSourceSha256': originals,
        'sourceUnchangedDuringVerification': all(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest for path, digest in hashes.items()),
        'upstreamUnchanged': all(hashlib.sha256((ROOT / 'upstream/Horizon' / path).read_bytes()).hexdigest() == digest for path, digest in originals.items()),
        'fullClientCompilationPassed': False,
        'fullGameIntegrationPassed': False,
    }
    report['passed'] &= report['sourceUnchangedDuringVerification'] and report['upstreamUnchanged']
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': report['passed'], 'report': str(out / 'report.json')}, indent=2))


if __name__ == '__main__':
    main()
