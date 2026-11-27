#!/usr/bin/env python3
"""Exact jump consumer, generated native event and actual MC registry/AABB oracle."""
import argparse
import hashlib
import json
import os
from pathlib import Path
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
    parser.add_argument('--out', default='build/block-bounce-01')
    out = (ROOT / parser.parse_args().out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True, exist_ok=True)
    changed, originals = {}, {}

    def edit(name, fn):
        path = UPSTREAM / name
        if path.exists():
            originals[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        changed[name] = fn(changed.get(name, path.read_text(encoding='utf-8') if path.exists() else ''))

    def once(text, before, after):
        if text.count(before) != 1:
            raise ValueError(f'Expected one production anchor: {before!r}')
        return text.replace(before, after, 1)

    for name in ['movement', 'mining', 'block-bounce']:
        runpy.run_path(str(HERE / (name + '-patch.py')))['apply'](edit, once)
    helper = runpy.run_path(str(HERE / 'block-bounce-patch.py'))
    native = changed['crates/horizon-client/src/game/native_hooks.rs']
    native_impl = "impl Game<'_> {\n" + block(native, '    pub(super) fn native_client_block_bounce(') + '\n}\n'
    motion = changed['crates/horizon-client/src/motion.rs']
    living = changed['crates/horizon-world/src/living_motion.rs']
    remote = changed['crates/horizon-client/src/game/remote.rs']
    assert living.index('world.modify_jump_power(power)?') < living.index('if power <=')
    assert remote.index('self.native_client_block_bounce(') < remote.index('player.client_block_bounce_motion =')
    assert remote.index('player.client_block_bounce_motion =') < remote.index('let was_gliding =')
    assert 'client_block_bounce_motion: None' in changed['crates/horizon-client/src/motion/body.rs']
    (out / 'living_motion.rs').write_text(living, encoding='utf-8')
    (out / 'block_bounce.rs').write_text(changed['crates/horizon-client/src/block_bounce.rs'], encoding='utf-8')
    source = (HERE / 'block-bounce-fixture.rs').read_text(encoding='utf-8')
    source = source.replace('/* IDENTITY */', block(motion, '    pub fn native_client_queries_allowed('))
    source = source.replace('/* MODIFIER */', helper['MODIFIER']).replace('/* NATIVE */', native_impl)

    cache = Path(os.environ['USERPROFILE']) / '.gradle/caches'
    jars = [cache / 'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar']
    for group in ['com.mojang', 'com.google.guava', 'com.google.code.gson', 'it.unimi.dsi', 'org.slf4j', 'org.joml', 'io.netty', 'org.apache.logging.log4j', 'org.apache.commons', 'org.lwjgl']:
        jars.extend((cache / 'modules-2/files-2.1' / group).rglob('*.jar'))
    jars.extend((ROOT / 'build/resource-runtime/libraries').rglob('*.jar'))
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    oracle_run = subprocess.run([str(java), '-Duser.language=en', '-Duser.country=US', '-Dfile.encoding=UTF-8',
        '--class-path', os.pathsep.join(map(str, jars)), str(HERE / 'oracles/BlockBounceFacts.java'), str(out / 'mc-oracle.json')],
        capture_output=True, text=True, encoding='utf-8', errors='replace')
    (out / 'java-oracle.log').write_text(oracle_run.stdout + oracle_run.stderr, encoding='utf-8')
    if oracle_run.returncode:
        raise SystemExit(oracle_run.stdout + oracle_run.stderr)
    oracle = json.loads((out / 'mc-oracle.json').read_text(encoding='utf-8'))
    assert sum(row['bouncy'] for row in oracle['blocks']) == 18
    assertions = ''
    for row in oracle['blocks']:
        negation = '' if row['bouncy'] else '!'
        assertions += f'assert!({negation}block_bounce::is_bouncy_block({json.dumps(row["id"])}));\n'
    for index, row in enumerate(oracle['boxes']):
        c = row['bounds']
        floats = lambda values: '[' + ','.join(repr(v) + 'f64' for v in values) + ']'
        assertions += f'''{{let mut visited=Vec::new();
        assert!(!block_bounce::standing_on_bouncy_block({floats(c[:3])},{floats(c[3:])},|p|{{visited.push(p);false}}));
        assert_eq!(visited,vec!{json.dumps(row['visited'])},"actual AABB row {index}");}}\n'''
    for index, row in enumerate(oracle['jumps']):
        boost = 'None' if row['boost'] < 0 else f'Some({row["boost"]})'
        assertions += f'''{{let mut player=PlayerMotion{{strength:f64::from({row['attribute']}f32),block_factor:{row['block']}f32,boost:{boost},client_block_bounce_motion:Some({row['motion']}f32),..Default::default()}};
        let assets=assets();let mut world=MotionWorld{{player:&mut player,assets:&assets,blocks:HashMap::from([(BlockPos::new(0,63,0),1)])}};
        let power=living_motion::jump_power(&mut world,1.).unwrap();assert_eq!(power.to_bits(),{row['powerBits']},"Float power {index}");
        let adjusted=living_motion::LivingMotionWorld::modify_jump_power(&mut world,power).unwrap();assert_eq!(adjusted.to_bits(),{row['adjustedBits']},"Float motion {index}");}}\n'''
    source = source.replace('/* ORACLE */', assertions)
    (out / 'lib.rs').write_text(source, encoding='utf-8')
    api = out / 'api-source'
    game = api / 'crates/horizon-client/src/game'
    game.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (game.parent / 'mod_api.rs').write_text('', encoding='utf-8')
    (game / 'native_hooks.rs').write_text(native_impl, encoding='utf-8')
    render = api / 'crates/horizon-client-render/src/gfx/native_render/generated.rs'
    render.parent.mkdir(parents=True, exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}\n', encoding='utf-8')
    (api / 'native-hooks.json').write_text(json.dumps({'helpers': [], 'hooks': [{'name': 'client_block_bounce',
        'target': "horizon_client::game::native_hooks::Game<'_>::native_client_block_bounce", 'mod_api': {'contract': 'listen'}}]}, indent=2) + '\n', encoding='utf-8')
    (out / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n', encoding='utf-8')
    (out / 'Cargo.toml').write_text('''[package]
name="liquidbounce-block-bounce-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[workspace]
[dependencies]
horizon-core={path="''' + (UPSTREAM / 'crates/horizon-core').as_posix() + '''"}
horizon-world={path="''' + (UPSTREAM / 'crates/horizon-world').as_posix() + '''"}
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
        print((result.stdout + result.stderr)[-16000:])
        commands.append({'argv': argv, 'exitCode': result.returncode, 'durationSeconds': round(time.monotonic() - start, 3)})
        if result.returncode:
            break
    files = [HERE / name for name in ['block-bounce-patch.py', 'block-bounce-fixture.rs', 'test-block-bounce-patch.py', 'oracles/BlockBounceFacts.java']]
    files.append(ROOT / 'native/liquidbounce-behaviors/src/block_bounce.rs')
    report = {'passed': all(row['exitCode'] == 0 for row in commands), 'commands': commands,
        'sourceSha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files},
        'upstreamSha256': originals, 'transformedSha256': {p: hashlib.sha256(t.encode()).hexdigest() for p, t in changed.items()},
        'productionBodies': ['complete living_motion module including unmodified world default and upstream tests', 'MotionWorld::modify_jump_power', 'PlayerMotion::native_client_queries_allowed', 'native_client_block_bounce', 'same-source complete BlockBounce behavior'],
        'apiGenerator': 'horizon_mod_api_gen::generate(Target::Client), generated event/dispatch/Host compiled',
        'javaOracle': {'blockRows': len(oracle['blocks']), 'aabbRows': len(oracle['boxes']), 'jumpRows': len(oracle['jumps']),
            'scope': 'actual MC26.3 registry class membership/AABB plus source-derived Java Float jump formula, not original Kotlin module execution'},
        'boundaries': ['Game/window/player identity and world block lookup shells', 'engine registry/lifecycle/config is parent integration'],
        'fullGameIntegrationPassed': False}
    report['upstreamUnchanged'] = all(hashlib.sha256((UPSTREAM / p).read_bytes()).hexdigest() == digest for p, digest in originals.items())
    report['passed'] &= report['upstreamUnchanged']
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': report['passed'], 'report': str(out / 'report.json')}, indent=2))
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
