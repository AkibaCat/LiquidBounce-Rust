#!/usr/bin/env python3
"""Check original Twerk arithmetic and compile exact native input/helper bodies."""
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


def block(text, signature):
    start = text.index(signature)
    end = text.index('{', start) + 1
    depth = 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[start:end]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/twerk-01')
    parser.add_argument('--prepare-only', action='store_true')
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True, exist_ok=True)
    changes, originals, fragments = {}, {}, {}

    def pristine(name):
        path = ROOT / 'upstream/Horizon' / name
        originals[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        return path.read_text(encoding='utf-8')

    def edit(name, transform):
        changes[name] = transform(changes.get(name, pristine(name)))

    def once(text, before, after):
        if text.count(before) != 1:
            raise ValueError(f'Expected one anchor: {before[:90]}')
        return text.replace(before, after, 1)

    for name in ['mining', 'sprint', 'twerk']:
        spec = importlib.util.spec_from_file_location(name + '_patch', HERE / (name + '-patch.py'))
        patch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(patch)
        fragments[name] = patch.apply(edit, once)
    assert fragments['twerk'] == {'hook_names': [], 'helper_names': ['client_player_input_tick_count']}
    native = changes['crates/horizon-client/src/game/native_hooks.rs']
    remote = changes['crates/horizon-client/src/game/remote.rs']
    motion = changes['crates/horizon-client/src/motion.rs']
    health = pristine('crates/horizon-client/src/health.rs')
    sprint = pristine('crates/horizon-client/src/sprint.rs')
    assert '    pub tick_count: i32,' in health
    clock = block(health, '    pub fn advance_clock(')
    assert 'self.tick_count = self.tick_count.wrapping_add(1);' in clock
    start = remote.index('        let directional_before = [keys.forward, keys.backward, keys.left, keys.right];')
    end = remote.index('        // LocalPlayer.applyInput and aiStep:', start)
    input_body = remote[start:end]
    assert input_body.index('self.native_client_directional_input(') < input_body.index('let input = u8::from(keys.forward)')
    tick = block(remote, '    fn tick_remote_level(')
    assert tick.index('if !self.streaming.is_lit(cx, cz)') < tick.index('self.native_client_directional_input(')
    assert tick.index('self.native_client_directional_input(') < tick.index('.controlled_simulation_tick_with_sprint_window(')
    assert tick.index('self.native_client_directional_input(') < tick.index('player.health.advance_clock();')
    local_tick = block(motion, '    fn local_tick(')
    assert local_tick.index('self.player.health.advance_clock();') < local_tick.index('sprint(self.player, keys)?;')
    paths = {
        'TWERK_PATH': ROOT / 'native/liquidbounce-behaviors/src/twerk.rs',
        'NO_BLOCK_PATH': ROOT / 'native/liquidbounce-behaviors/src/no_block_interact.rs',
        'NO_ENTITY_PATH': ROOT / 'native/liquidbounce-behaviors/src/no_entity_interact.rs',
    }
    blocks = {
        'CLOCK': clock,
        'IDENTITY': block(motion, '    pub fn native_client_queries_allowed('),
        'KEYS': block(sprint, 'pub struct Keys {'),
        'NATIVE': '\n'.join([
            block(native, '    pub(super) fn native_client_player_input_tick_count('),
            block(native, '    #[allow(clippy::too_many_arguments)]\n    pub(super) fn native_client_directional_input('),
        ]),
        'INPUT': input_body,
    }
    source = (HERE / 'packet-tests/twerk-fixture.rs').read_text(encoding='utf-8')
    for name, body in blocks.items():
        source = source.replace('/* ' + name + ' */', body)
    for name, path in paths.items():
        source = source.replace(name, path.as_posix())
    (out / 'lib.rs').write_text(source, encoding='utf-8', newline='\n')
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    oracle = subprocess.run([str(java), str(HERE / 'TwerkReference.java')], capture_output=True, text=True, check=True)
    vectors = json.loads(oracle.stdout)
    (out / 'java-vectors.json').write_text(json.dumps(vectors, indent=2) + '\n', encoding='utf-8')
    api = out / 'api-source'
    api_game = api / 'crates/horizon-client/src/game'
    api_game.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (api / 'crates/horizon-client/src/mod_api.rs').write_text('', encoding='utf-8')
    (api_game / 'native_hooks.rs').write_text("impl Game<'_> {\n" + blocks['NATIVE'] + '\n}\n', encoding='utf-8')
    catalog = {
        'helpers': [{'name': name, 'target': f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in fragments['twerk']['helper_names']],
        'hooks': [{'name': 'client_directional_input', 'target': "horizon_client::game::native_hooks::Game<'_>::native_client_directional_input", 'mod_api': {'contract': 'listen'}}],
    }
    (api / 'native-hooks.json').write_text(json.dumps(catalog, indent=2) + '\n', encoding='utf-8')
    (out / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::fill_hooks(&root).unwrap();horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n', encoding='utf-8')
    manifest = '[package]\nname="liquidbounce-twerk-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n'
    manifest += f'[build-dependencies]\nhorizon-mod-api-gen={{path="{(ROOT / "upstream/Horizon/crates/horizon-mod-api-gen").as_posix()}"}}\n'
    (out / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    if args.prepare_only:
        print(out / 'Cargo.toml')
        return
    files = [HERE / 'twerk-patch.py', HERE / 'mining-patch.py', HERE / 'sprint-patch.py', HERE / 'test-twerk-patch.py', HERE / 'TwerkReference.java', HERE / 'packet-tests/twerk-fixture.rs', *paths.values(), ROOT / 'native/liquidbounce-behaviors/src/no_interact_items.rs']
    liquid_sources = [
        'src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/fun/ModuleTwerk.kt',
        'src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/player/ModuleNoBlockInteract.kt',
        'src/main/kotlin/net/ccbluex/liquidbounce/event/EventManager.kt',
        'src/main/kotlin/net/ccbluex/liquidbounce/event/EventHookRegistry.kt',
        'src/main/java/net/ccbluex/liquidbounce/injection/mixins/minecraft/client/MixinKeyboardInput.java',
    ]
    files.extend(ROOT / 'upstream/LiquidBounce' / name for name in liquid_sources)
    hashes = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in files}
    env = os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR', str(HERE / 'packet-tests/target'))
    commands = []
    for name, argv in [('test', ['cargo', '+stable', 'test', '--manifest-path', str(out / 'Cargo.toml')]), ('clippy', ['cargo', '+stable', 'clippy', '--locked', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings'])]:
        start = time.monotonic()
        run = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        for stream in ['stdout', 'stderr']:
            (out / f'{name}.{stream}.log').write_text(getattr(run, stream), encoding='utf-8')
        commands.append({'argv': argv, 'exitCode': run.returncode, 'durationSeconds': round(time.monotonic() - start, 3)})
        if run.returncode:
            break
    report = {
        'passed': all(command['exitCode'] == 0 for command in commands),
        'scope': 'Production Twerk signed Int arithmetic against 312 actual JVM vectors and both incoming sneak states; exact native local-player helper and identity, damage-clock body and remote input-bit assembly; actual horizon_mod_api_gen full Host/helper/hook generation and native/generated implementation compilation. Twerk priority-10 then production NoBlockInteract composition is a fixture runtime; actor/registry/world execution and full client/game are separate.',
        'phase': 'Directional input precedes imminent local_tick/ordinary entity advance_clock or explicit dead/riding clock; helper reads live signed health.tick_count.wrapping_add(1). Unlit chunks return before input. Sleeping branch belongs to the pinned Horizon source and does not advance this clock.',
        'commands': commands,
        'sourceSha256': hashes,
        'upstreamSha256': originals,
        'extractedBlocksSha256': {name: hashlib.sha256(body.encode()).hexdigest() for name, body in blocks.items()},
        'phaseSourceBlocksSha256': {'tick_remote_level': hashlib.sha256(tick.encode()).hexdigest(), 'local_tick': hashlib.sha256(local_tick.encode()).hexdigest()},
        'productionFragment': fragments['twerk'],
        'oracle': {'source': 'TwerkReference.java', 'vectors': len(vectors), 'runtime': str(java)},
        'fullClientCompilationPassed': False,
        'fullGameIntegrationPassed': False,
    }
    report['sourceUnchangedDuringVerification'] = all(hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest for name, digest in hashes.items())
    report['upstreamUnchanged'] = all(hashlib.sha256((ROOT / 'upstream/Horizon' / name).read_bytes()).hexdigest() == digest for name, digest in originals.items())
    report['passed'] &= report['sourceUnchangedDuringVerification'] and report['upstreamUnchanged']
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': report['passed'], 'report': str(out / 'report.json')}, indent=2))


if __name__ == '__main__':
    main()
