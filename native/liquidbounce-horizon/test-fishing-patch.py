#!/usr/bin/env python3
"""AutoFish production state/listener and exact committed native/transport seams."""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import os
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


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/auto-fish-01')
    parser.add_argument('--prepare-only', action='store_true')
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    if (out / 'report.json').exists(): raise SystemExit('Refusing to overwrite existing evidence')
    changes, originals, production_extras = {}, {}, {}
    def edit(name, transform):
        path = ROOT / 'upstream/Horizon' / name
        originals[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        changes[name] = transform(changes.get(name, path.read_text(encoding='utf-8')))
    def once(text, before, after):
        if text.count(before) != 1: raise ValueError(f'Expected one anchor: {before[:100]}')
        return text.replace(before, after, 1)
    for name in ['mining', 'packet', 'slot', 'fishing']:
        spec = importlib.util.spec_from_file_location(name + '_patch', HERE / (name + '-patch.py'))
        patch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(patch)
        production_extras.update(patch.apply(edit, once) or {})
    fragment = json.loads(production_extras['tools/native-hook-fragments/liquidbounce_fishing.json'])
    if not isinstance(fragment['hooks'], list) or not isinstance(fragment['helpers'], list):
        raise ValueError('Production native hook fragment requires hooks and helpers arrays')
    if {helper['name'] for helper in fragment['helpers']} != {
        'client_fishing_rod_hand', 'client_fishing_hook', 'client_use_item', 'client_sound_event_names'
    }: raise ValueError('Production fishing Host helpers do not match the tested native seams')
    game = 'crates/horizon-client/src/game/'
    native, remote, fishing = [changes[game + name] for name in ['native_hooks.rs', 'remote.rs', 'fishing.rs']]
    network = changes['crates/horizon-client/src/net.rs']
    source = (HERE / 'packet-tests/fishing-fixture.rs').read_text(encoding='utf-8')
    blocks = {
        'HELPERS': '\n'.join(function(native, signature) for signature in ['    pub(super) fn native_client_fishing_owner(', '    pub(super) fn native_client_sound_event_names(', '    pub(super) fn native_client_fishing_rod_hand(', '    pub(super) fn native_client_fishing_hook(', '    pub(super) fn native_publish_fishing_state(', '    pub(super) fn native_client_use_item(', '    pub(super) fn native_with_item_rotation<']),
        'HOTBAR': '\n'.join(function(native, signature) for signature in ['    pub(super) fn native_client_slot_owner(', '    pub(super) fn client_hotbar_slot(']),
        'HELD': '\n'.join(function(changes[game + 'held_items.rs'], signature) for signature in ['    pub(super) fn held(', '    pub(super) fn held_slot(', '    pub(super) fn with_hand<']),
        'REMOTE_USE': '\n'.join(function(remote, signature) for signature in ['    pub(super) fn remote_use_item_rotated(', '    fn remote_predict_item(', '    pub(super) fn send(&self,', '    fn remote_send_carried(', '    fn next_sequence(']),
        'REMOTE_FACTS': '\n'.join(function(remote, signature) for signature in ['fn hand_id(', 'fn use_ends_click(', 'fn remote_use_swings(', 'pub(super) fn item_has_use(']),
        'SESSION': '\n'.join(function(network, signature) for signature in ['    pub fn send(&self,', '    pub fn send_silently(&self,', '    fn send_with_path(&self,']),
        'HOOK_METHODS': '\n'.join(function(fishing, signature) for signature in ['    pub(super) fn spawn_remote_hook(', '    pub(super) fn spawn_remote_hook_committed(', '    pub(super) fn tick_remote_hooks(', '    pub(super) fn tick_remote_hooks_committed(', '    pub(super) fn remove_remote_hook(', '    pub(super) fn remove_remote_hook_committed(', '    pub(super) fn clear_remote_hooks(', '    pub(super) fn clear_remote_hooks_committed(', '    pub(super) fn tick_fishing(']),
        'IDENTITY': function(changes['crates/horizon-client/src/motion.rs'], '    pub fn native_client_queries_allowed('),
    }
    for name, body in blocks.items(): source = source.replace('/* ' + name + ' */', body)
    paths = {'PACKET_BRIDGE_PATH': HERE / 'packet-bridge.rs', 'FISH_BRIDGE_PATH': HERE / 'fish-bridge.rs', 'SLOT_BRIDGE_PATH': HERE / 'slot-bridge.rs', 'PACKETS_PATH': HERE / 'engine/src/packets.rs', 'SLOTS_PATH': HERE / 'engine/src/slots.rs', 'FISHING_PACKETS_PATH': HERE / 'engine/src/fishing_packets.rs', 'BEHAVIOR_TESTS_PATH': HERE / 'packet-tests/fishing-behavior-tests.rs', 'SOUNDS_PATH': ROOT/'native/liquidbounce-behaviors/src/auto_fish_sounds.rs'}
    for name in sorted(paths, key=len, reverse=True): source = source.replace(name, paths[name].as_posix())
    (out / 'src').mkdir(exist_ok=True)
    (out / 'src/lib.rs').write_text(source, encoding='utf-8', newline='\n')
    manifest = '[package]\nname="liquidbounce-fishing-fixture"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
    for name, path in [('horizon-protocol', ROOT/'upstream/Horizon/crates/horizon-protocol'), ('horizon-items', ROOT/'upstream/Horizon/crates/horizon-items'), ('horizon-world', ROOT/'upstream/Horizon/crates/horizon-world'), ('liquidbounce-behaviors', ROOT/'native/liquidbounce-behaviors')]: manifest += f'{name}={{path="{path.as_posix()}"}}\n'
    manifest += 'serde_json="1"\n'
    (out/'Cargo.toml').write_text(manifest, encoding='utf-8', newline='\n')
    if args.prepare_only: print(out/'Cargo.toml'); return
    files = [HERE/'fishing-patch.py', HERE/'test-fishing-patch.py', HERE/'fish-bridge.rs', HERE/'engine/src/fishing_packets.rs', HERE/'packet-tests/fishing-fixture.rs', HERE/'packet-tests/fishing-behavior-tests.rs', ROOT/'native/liquidbounce-behaviors/src/auto_fish.rs', ROOT/'native/liquidbounce-behaviors/src/auto_fish_sounds.rs']
    hashes = {str(path.relative_to(ROOT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in files}
    env = os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR', str(HERE/'packet-tests/target'))
    commands = []
    for name, argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]), ('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start = time.monotonic()
        result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        for stream in ['stdout','stderr']: (out/f'{name}.{stream}.log').write_text(getattr(result,stream), encoding='utf-8')
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3),'stdout':str(out/f'{name}.stdout.log'),'stderr':str(out/f'{name}.stderr.log')})
        if result.returncode: break
    report = {'passed':all(command['exitCode']==0 for command in commands), 'scope':'Production AutoFish state/listener/snapshot, exact native ownership/query/use/rotation, remote item prediction and committed remote-hook bodies, real Horizon stacks/cooldowns and framed Pipe codec; terrain/local fishing physics/item resources stubbed', 'commands':commands, 'sourceSha256':hashes, 'extractedBlocksSha256':{name:hashlib.sha256(body.encode()).hexdigest() for name,body in blocks.items()}, 'upstreamSourceSha256':originals, 'sourceUnchangedDuringVerification':all(hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest for name,digest in hashes.items()), 'upstreamUnchanged':all(hashlib.sha256((ROOT/'upstream/Horizon'/name).read_bytes()).hexdigest()==digest for name,digest in originals.items()), 'fullClientCompilationPassed':False, 'fullGameIntegrationPassed':False}
    report['productionExtrasSha256'] = {name:hashlib.sha256((source if isinstance(source,str) else json.dumps(source,sort_keys=True)).encode()).hexdigest() for name,source in production_extras.items()}
    report['productionNativeHookFragmentSchemaPassed'] = True
    report['passed'] &= report['sourceUnchangedDuringVerification'] and report['upstreamUnchanged']
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n', encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}, indent=2))


if __name__ == '__main__': main()
