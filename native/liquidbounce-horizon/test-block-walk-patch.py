#!/usr/bin/env python3
"""Execute real MC scans and compile complete production BlockWalk consumers."""
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
    parser.add_argument('--out', default='build/block-walk-01')
    parser.add_argument('--oracle-only', action='store_true')
    parser.add_argument('--with-airjump', action='store_true')
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True, exist_ok=True)
    cache = Path(os.environ['USERPROFILE']) / '.gradle/caches'
    jars = [cache / 'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar']
    for group in ['com.mojang', 'com.google.guava', 'com.google.code.gson', 'it.unimi.dsi', 'org.slf4j', 'org.joml', 'io.netty', 'org.apache.logging.log4j', 'org.apache.commons', 'org.lwjgl']:
        jars.extend((cache / 'modules-2/files-2.1' / group).rglob('*.jar'))
    jars.extend((ROOT / 'build/resource-runtime/libraries').rglob('*.jar'))
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    result = subprocess.run([str(java), '-Duser.language=en', '-Duser.country=US', '-Dfile.encoding=UTF-8',
        '--class-path', os.pathsep.join(map(str, jars)), str(HERE / 'oracles/BlockWalkFacts.java'), str(out / 'mc-oracle.json')],
        capture_output=True, text=True, encoding='utf-8', errors='replace')
    (out / 'java-oracle.log').write_text(result.stdout + result.stderr, encoding='utf-8')
    if result.returncode:
        raise SystemExit((result.stdout + result.stderr)[-12000:])
    oracle = json.loads((out / 'mc-oracle.json').read_text(encoding='utf-8'))
    print(json.dumps({key: len(value) for key, value in oracle.items()}))
    if args.oracle_only:
        print(json.dumps(oracle['imports'], indent=2))
        return
    changed, originals = {}, {}

    def read(path):
        source = UPSTREAM / path
        if not source.exists():
            return ''
        originals[path] = hashlib.sha256(source.read_bytes()).hexdigest()
        return source.read_text(encoding='utf-8')

    def edit(path, action):
        changed[path] = action(changed.get(path, read(path)))

    def once(text, before, after):
        assert text.count(before) == 1, (before, text.count(before))
        return text.replace(before, after, 1)

    if args.with_airjump:
        runpy.run_path(str(HERE / 'airjump-push-patch.py'))['apply'](edit, once)
    runpy.run_path(str(HERE / 'block-walk-patch.py'))['apply'](edit, once)
    collisions = changed['crates/horizon-client-render/src/collisions.rs']
    native = changed['crates/horizon-client/src/game/native_hooks.rs']
    native_names = ['native_client_block_walk', 'sync_native_block_walk', 'native_client_tick_pre']
    native_impl = "impl Game<'_> {\n" + '\n'.join(block(native, '    pub(super) fn ' + name + '(') for name in native_names) + '\n}\n'
    assert native_impl.index('mod_api::dispatch_client_tick_pre(') < native_impl.index('        self.sync_native_block_walk();')
    source = (HERE / 'block-walk-fixture.rs').read_text(encoding='utf-8')
    replacements = {
        'FAST_EMPTY': block(collisions, '    pub fn cells_without_collision('),
        'SCENE_IMPL': block(collisions, "impl<'w> BlockCollisionWorld for SceneCollisions<'w, '_> {"),
        'MOVEMENT_IMPL': block(collisions, "impl EntityMovementWorld for SceneCollisions<'_, '_> {"),
        'WRAPPER_IMPL': block(collisions, "impl<W: BlockCollisionWorld> BlockCollisionWorld for EntityCollisions<'_, W> {"),
        'SUPPORT': block(changed['crates/horizon-client/src/mobs/living.rs'], '    pub fn supporting_block('),
        'NATIVE': native_impl,
    }
    for key, value in replacements.items():
        source = source.replace('/* ' + key + ' */', value)
    assert not re.search(r'/\* [A-Z_]+ \*/', source)
    assert 'crate::block_walk::shape(&state.name, shape)' in changed['crates/horizon-world/src/registry_collision_scan.rs']
    assert 'fn changes_collision_shapes(&self)' in changed['crates/horizon-world/src/registry_collision_scan.rs']
    (out / 'lib.rs').write_text(source, encoding='utf-8')
    (out / 'block_walk.rs').write_text(changed['crates/horizon-world/src/block_walk.rs'], encoding='utf-8')
    (out / 'block_collision_scan.rs').write_text(changed['crates/horizon-world/src/block_collision_scan.rs'], encoding='utf-8')
    for name in ['world_movement', 'ground_support']:
        (out / (name + '.rs')).write_text(read('crates/horizon-world/src/' + name + '.rs'), encoding='utf-8')
    (out / 'behavior.rs').write_text((ROOT / 'native/liquidbounce-behaviors/src/block_walk.rs').read_text(encoding='utf-8'), encoding='utf-8')
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
        'target': "horizon_client::game::native_hooks::Game<'_>::native_" + name, 'mod_api': {'contract': 'listen'}} for name in ['client_tick_pre', 'client_block_walk']]}, indent=2) + '\n', encoding='utf-8')
    (out / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n', encoding='utf-8')
    manifest = '''[package]
name="liquidbounce-block-walk-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[workspace]
[features]
combined-airjump=[]
[dependencies]
serde_json="1"
horizon-core={path="''' + (UPSTREAM / 'crates/horizon-core').as_posix() + '''"}
world-kernel={package="horizon-world",path="''' + (UPSTREAM / 'crates/horizon-world').as_posix() + '''"}
[build-dependencies]
horizon-mod-api-gen={path="''' + (UPSTREAM / 'crates/horizon-mod-api-gen').as_posix() + '"}\n'
    (out / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    env = os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR', str(ROOT / 'build/no-pose-hook-tests/target'))
    commands = []
    flags = ['--features', 'combined-airjump'] if args.with_airjump else []
    for name, argv in [('test', ['cargo', '+stable', 'test', '--offline', '--manifest-path', str(out / 'Cargo.toml'), *flags, '--', '--test-threads=1']),
        ('clippy', ['cargo', '+stable', 'clippy', '--offline', '--manifest-path', str(out / 'Cargo.toml'), *flags, '--all-targets', '--', '-D', 'warnings'])]:
        start = time.monotonic()
        result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (out / (name + '.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        print((result.stdout + result.stderr)[-14000:])
        commands.append({'argv': argv, 'exitCode': result.returncode, 'durationSeconds': round(time.monotonic() - start, 3)})
        if result.returncode:
            break
    files = [HERE / name for name in ['block-walk-patch.py', 'block-walk-policy.rs', 'block-walk-fixture.rs', 'test-block-walk-patch.py', 'oracles/BlockWalkFacts.java', 'engine/src/config/block_walk.rs']]
    files.append(ROOT / 'native/liquidbounce-behaviors/src/block_walk.rs')
    report = {'passed': all(row['exitCode'] == 0 for row in commands), 'commands': commands,
        'sourceSha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files},
        'upstreamSha256': originals, 'transformedSha256': {p: hashlib.sha256(t.encode()).hexdigest() for p, t in changed.items()},
        'productionBodies': list(replacements) + ['entire BlockCollisionScan', 'entire world_movement and ground_support'],
        'apiGenerator': 'real horizon_mod_api_gen Target::Client Host/events/dispatch',
        'javaOracle': {'registryEntries': len(oracle['registry']), 'states': len(oracle['states']), 'scanCases': len(oracle['scans']), 'importCases': len(oracle['imports']),
            'combinedAirJumpCases': len(oracle['combinedScans']) if args.with_airjump else 0,
            'scope': 'actual MC26.3 registry, block shape metadata and BlockCollisions; source-derived return modifier and Gson ListValue adapter, not Kotlin/mixin execution'},
        'boundaries': ['resident chunk/palette/binding shell supplies actual MC shape and metadata facts', 'Game/window shell', 'parent engine registry/config/lifecycle integration'], 'fullGameIntegrationPassed': False}
    report['upstreamUnchanged'] = all(hashlib.sha256((UPSTREAM / path).read_bytes()).hexdigest() == digest for path, digest in originals.items())
    report['passed'] &= report['upstreamUnchanged']
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': report['passed'], 'report': str(out / 'report.json')}, indent=2))
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
