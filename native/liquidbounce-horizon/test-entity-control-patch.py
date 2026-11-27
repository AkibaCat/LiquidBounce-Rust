#!/usr/bin/env python3
"""Compile exact EntityControl consumers and real MC constructor-free getter probes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import subprocess
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = ROOT / 'upstream/Horizon'


def block(text, anchor):
    start = text.index(anchor)
    end = text.index('{', start) + 1
    depth = 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[start:end]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/entity-control-01')
    out = (ROOT / parser.parse_args().out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True, exist_ok=True)
    changed, originals = {}, {}

    def read(name):
        path = UPSTREAM / name
        if path.exists():
            originals[name] = hashlib.sha256(path.read_bytes()).hexdigest()
            return path.read_text(encoding='utf-8')
        return ''

    def edit(name, fn):
        changed[name] = fn(changed.get(name, read(name)))

    def once(text, before, after):
        if text.count(before) != 1:
            raise ValueError(f'Expected one production anchor: {before!r}')
        return text.replace(before, after, 1)

    for name in ['mining', 'entity-control']:
        runpy.run_path(str(HERE / (name + '-patch.py')))['apply'](edit, once)
    mounts = changed['crates/horizon-client/src/game/mounts.rs']
    equine = changed['crates/horizon-client/src/mobs/equine.rs']
    mobs = changed['crates/horizon-client/src/mobs.rs']
    nautilus = changed['crates/horizon-client/src/mobs/nautilus.rs']
    remote = changed['crates/horizon-client/src/game/remote_vehicles.rs']
    native = changed['crates/horizon-client/src/game/native_hooks.rs']
    native_names = ['native_client_entity_control', 'sync_native_entity_control', 'native_client_entity_control_jump_strength', 'native_client_riding_jump_scale', 'native_client_tick_pre']
    native_impl = "impl Game<'_> {\n" + '\n'.join(block(native, '    pub(super) fn ' + name + '(') for name in native_names) + '\n}\n'
    assert native_impl.index('mod_api::dispatch_client_tick_pre(') < native_impl.index('        self.sync_native_entity_control();')
    assert 'm.extra.saddled' in block(nautilus, 'fn restriction_radius(')
    assert 'self.extra.saddled.then' in block(mobs, '    pub fn saddle_item(')
    assert 'm.extra.saddled' in read('crates/horizon-client/src/mobs/render.rs')
    assert 'self.send(riding_jump(id, self.native_client_riding_jump_scale(self.mount.jump_scale)))' in remote
    assert 'm.is_saddled()\n                    && !sneaking' in changed['crates/horizon-client/src/game/remote_mob_use.rs']
    source = (HERE / 'entity-control-fixture.rs').read_text(encoding='utf-8')
    replacements = {
        'NATIVE': native_impl,
        'IDENTITY': block(changed['crates/horizon-client/src/motion.rs'], '    pub fn native_client_queries_allowed('),
        'MOB_GETTER': block(mobs, '    pub fn is_saddled('),
        'SADDLE_ITEM': block(mobs, '    pub fn saddle_item('),
        'SET_SADDLE': block(mobs, '    pub fn set_saddle('),
        'NAUTILUS_CONTROLLED': block(nautilus, 'pub fn controlled('),
        'NAUTILUS_ON_JUMP': block(nautilus, 'pub fn on_player_jump('),
        'IS_EQUINE': block(equine, 'pub fn is_equine('),
        'CONTROLLED': block(equine, 'pub fn controlled('),
        'MOVED_BY_RIDER': block(equine, 'pub fn moved_by_rider('),
        'JUMP_PENDING': block(equine, 'pub fn jump_pending_scale('),
        'EQUINE_ON_JUMP': block(equine, 'pub fn on_player_jump('),
        'MOUNT_STATE': '#[derive(Default)]\n' + block(mounts, 'pub(super) struct MountState {'),
        'RESET_JUMP': block(mounts, '    pub(super) fn reset_jump('),
        'MOUNT_INDEX': block(mounts, '    pub(super) fn mount_index('),
        'CHARGE': block(mounts, '    pub(super) fn charge_mount_jump('),
        'JUMPS': block(mounts, '    fn jumps('),
        'JUMPABLE': block(mounts, '    pub(super) fn jumpable_mount('),
        'MOUNT_JUMPING': block(mounts, '    pub(super) fn mount_jumping('),
        'DRAW_JUMP_BAR': block(mounts, '    pub(super) fn draw_jump_bar('),
        'CLIENT_MOVES': block(remote, 'fn client_moves('),
        'RIDING_JUMP': block(remote, 'fn riding_jump('),
        'START_RIDING_JUMP': block(mounts, 'pub(super) fn start_riding_jump('),
    }
    for key, value in replacements.items():
        source = source.replace('/* ' + key + ' */', value)
    cache = Path(os.environ['USERPROFILE']) / '.gradle/caches'
    jars = [cache / 'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar']
    for group in ['com.mojang', 'com.google.guava', 'com.google.code.gson', 'it.unimi.dsi', 'org.slf4j', 'org.joml', 'io.netty', 'org.apache.logging.log4j', 'org.apache.commons', 'org.lwjgl']:
        jars.extend((cache / 'modules-2/files-2.1' / group).rglob('*.jar'))
    jars.extend((ROOT / 'build/resource-runtime/libraries').rglob('*.jar'))
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    oracle_run = subprocess.run([str(java), '-Duser.language=en', '-Duser.country=US', '-Dfile.encoding=UTF-8',
        '--class-path', os.pathsep.join(map(str, jars)), str(HERE / 'oracles/EntityControlFacts.java'), str(out / 'mc-oracle.json')],
        capture_output=True, text=True, encoding='utf-8', errors='replace')
    (out / 'java-oracle.log').write_text(oracle_run.stdout + oracle_run.stderr, encoding='utf-8')
    if oracle_run.returncode:
        raise SystemExit((oracle_run.stdout + oracle_run.stderr)[-12000:])
    oracle = json.loads((out / 'mc-oracle.json').read_text(encoding='utf-8'))
    assertions = ''
    kinds = ['Horse', 'Llama', 'Pig', 'Strider', 'Nautilus']
    b = lambda value: str(value).lower()
    def check_bool(expression, value, message):
        return f'assert!({"" if value else "!"}{expression},"{message}");'
    for index, row in enumerate(oracle['controls']):
        flags = row['mask'] if row['running'] else 0
        rider = 'Some(equine::Rider {steer:' + b(row['steer']) + ',jump:0.})' if row['rider'] else 'None'
        getter = 'crate::mobs::nautilus::controlled(&mob)' if kinds[row['species']] == 'Nautilus' else 'equine::controlled(&mob)'
        assertions += f'''{{set_policy({flags});let mut mob=Mob::new(Species::{kinds[row['species']]});mob.extra.saddled={b(row['physical'])};mob.rider={rider};
        {check_bool('mob.is_saddled()', row['saddled'], f'Mob getter {index}')}{check_bool(getter, row['controlled'], f'actual MC controller {index}')}
        let game=Game{{mobs:vec![mob],riding:Some(1),..Default::default()}};
        {check_bool('game.jumpable_mount()', row['jumpable'], f'actual MC canJump {index}')}}}\n'''
    for index, row in enumerate(oracle['scales']):
        flags = row['mask'] if row['running'] else 0
        assertions += f'''set_policy({flags});assert_eq!(Game::default().native_client_riding_jump_scale({row['original']}f32).to_bits(),{row['bits']},"actual LocalPlayer getter {index}");\n'''
    source = source.replace('/* ORACLE */', assertions)
    assert not re.search(r'/\* [A-Z_]+ \*/', source)
    (out / 'lib.rs').write_text(source, encoding='utf-8')
    (out / 'entity_control.rs').write_text(changed['crates/horizon-client/src/entity_control.rs'], encoding='utf-8')
    (out / 'entity_control_behavior.rs').write_text((ROOT / 'native/liquidbounce-behaviors/src/entity_control.rs').read_text(encoding='utf-8'), encoding='utf-8')
    api = out / 'api-source'
    game = api / 'crates/horizon-client/src/game'
    game.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (game.parent / 'mod_api.rs').write_text('', encoding='utf-8')
    (game / 'native_hooks.rs').write_text(native_impl, encoding='utf-8')
    render = api / 'crates/horizon-client-render/src/gfx/native_render/generated.rs'
    render.parent.mkdir(parents=True, exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}\n', encoding='utf-8')
    (api / 'native-hooks.json').write_text(json.dumps({'helpers': [], 'hooks': [{'name': name,
        'target': "horizon_client::game::native_hooks::Game<'_>::native_" + name, 'mod_api': {'contract': 'listen'}} for name in ['client_tick_pre', 'client_entity_control']]}, indent=2) + '\n', encoding='utf-8')
    (out / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n', encoding='utf-8')
    (out / 'Cargo.toml').write_text('''[package]
name="liquidbounce-entity-control-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[workspace]
[dependencies]
serde_json="1"
[build-dependencies]
horizon-mod-api-gen={path="''' + (UPSTREAM / 'crates/horizon-mod-api-gen').as_posix() + '"}\n', encoding='utf-8')
    env = os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR', str(ROOT / 'build/no-pose-hook-tests/target'))
    commands = []
    for name, argv in [('test', ['cargo', '+stable', 'test', '--offline', '--manifest-path', str(out / 'Cargo.toml'), '--', '--test-threads=1']),
        ('clippy', ['cargo', '+stable', 'clippy', '--offline', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings'])]:
        start = time.monotonic()
        result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (out / (name + '.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        print((result.stdout + result.stderr)[-12000:])
        commands.append({'argv': argv, 'exitCode': result.returncode, 'durationSeconds': round(time.monotonic() - start, 3)})
        if result.returncode:
            break
    files = [HERE / name for name in ['entity-control-patch.py', 'entity-control-policy.rs', 'entity-control-fixture.rs', 'test-entity-control-patch.py', 'oracles/EntityControlFacts.java']]
    files.append(ROOT / 'native/liquidbounce-behaviors/src/entity_control.rs')
    report = {'passed': all(row['exitCode'] == 0 for row in commands), 'commands': commands,
        'sourceSha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files},
        'upstreamSha256': originals, 'transformedSha256': {p: hashlib.sha256(t.encode()).hexdigest() for p, t in changed.items()},
        'productionBodies': list(replacements), 'apiGenerator': 'real horizon_mod_api_gen Target::Client Host/events/dispatch',
        'javaOracle': {'controlRows': len(oracle['controls']), 'scaleRows': len(oracle['scales']),
            'scope': 'actual MC26.3 Mob.isSaddled, horse/llama/pig/strider/nautilus controller/canJump and LocalPlayer getter with constructor-free equipment/passenger probes; source-derived Mixin override, not Kotlin execution'},
        'boundaries': ['Mob storage/species/equipment and Game/window/player shells', 'HUD paint/packet enum shell', 'camel sound/rear delegates', 'engine registry/config/lifecycle parent integration'], 'fullGameIntegrationPassed': False}
    report['upstreamUnchanged'] = all(hashlib.sha256((UPSTREAM / p).read_bytes()).hexdigest() == digest for p, digest in originals.items())
    report['passed'] &= report['upstreamUnchanged']
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': report['passed'], 'report': str(out / 'report.json')}, indent=2))
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
