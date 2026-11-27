#!/usr/bin/env python3
"""Compile exact native/pose/chat bodies through the real generated Host API."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = ROOT / 'upstream/Horizon'


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
    parser.add_argument('--out', default='build/sleep-walker-01')
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True, exist_ok=True)
    changed, original_hashes = {}, {}

    def original(path):
        source = UPSTREAM / path
        original_hashes[path] = hashlib.sha256(source.read_bytes()).hexdigest()
        return source.read_text(encoding='utf-8')

    def edit(path, transform):
        changed[path] = transform(changed.get(path, original(path)))

    def once(text, before, after):
        if text.count(before) != 1:
            raise ValueError(f'Expected one production anchor: {before!r}')
        return text.replace(before, after, 1)

    runpy.run_path(str(HERE / 'mining-patch.py'))['apply'](edit, once)
    helper = runpy.run_path(str(HERE / 'sleep-walker-patch.py'))
    contract = helper['apply'](edit, once)
    assert contract == {'hook_names': [], 'helper_names': ['client_sleep_walker_tick']}
    pose = original('crates/horizon-client/src/pose.rs')
    chat = original('crates/horizon-client/src/game/chat.rs')
    native = changed['crates/horizon-client/src/game/native_hooks.rs']
    body = block(native, '    pub(super) fn native_client_sleep_walker_tick(')
    assert body.index('player.set_pose(') < body.index('self.sleeping = None;') < body.index('self.screen = Screen::Playing;')
    assert 'wake_up(' not in body and 'remote_stop_sleeping(' not in body
    assert 'self.sleep_timer' not in body and '.send(' not in body
    blocks = {
        'POSE_DECLARATIONS': pose[:pose.index('impl PlayerMotion {')],
        'DIMENSIONS_FOR_POSE': block(pose, '    pub fn dimensions_for_pose('),
        'REFRESH_DIMENSIONS': block(pose, '    pub fn refresh_dimensions('),
        'SET_POSE': block(pose, '    pub fn set_pose('),
        'IDENTITY': block(changed['crates/horizon-client/src/motion.rs'], '    pub fn native_client_queries_allowed('),
        'CHAT_DRAFT': block(chat, 'pub struct ChatDraft {'),
        'CHAT_EXIT': '#[derive(Clone, Copy, PartialEq, Eq, Debug)]\n' + block(chat, 'pub(super) enum ChatExit {'),
        'SHOULD_DISCARD_DRAFT': block(chat, 'fn should_discard_draft('),
        'CLOSE_CHAT': block(chat, '    pub(super) fn close_chat('),
        'KEEP_CHAT_DRAFT': block(chat, '    fn keep_chat_draft('),
        'TICK_IN_BED_CHAT': block(chat, '    pub(super) fn tick_in_bed_chat('),
        'NATIVE_WINDOW': block(native, '    pub(super) fn native_window('),
        'NATIVE_HELPER': body,
        'NATIVE_TICK': block(native, '    pub(super) fn native_client_tick_pre('),
    }
    source = (HERE / 'sleep-walker-fixture.rs').read_text(encoding='utf-8')
    for name, value in blocks.items():
        source = source.replace('__' + name + '__', value)
    assert '__NATIVE_' not in source
    (out / 'lib.rs').write_text(source, encoding='utf-8')

    # Verify the actual 26.3 accessor contract, independently of Horizon's caches.
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/javap.exe'
    jar = Path.home() / '.gradle/caches/fabric-loom/26.3/minecraft-client.jar'
    inspected = {}
    for name in ['net.minecraft.world.entity.LivingEntity', 'net.minecraft.client.gui.Gui']:
        result = subprocess.run([str(java), '-classpath', str(jar), '-p', '-c', name],
                                capture_output=True, text=True, encoding='utf-8', check=True)
        inspected[name] = result.stdout
        (out / (name.rsplit('.', 1)[1] + '.javap.txt')).write_text(result.stdout, encoding='utf-8')
    living = inspected['net.minecraft.world.entity.LivingEntity']
    sleeping = living[living.index('  public boolean isSleeping();'):]
    sleeping = sleeping[:sleeping.index('\n  public ', 4)]
    clear = living[living.index('  public void clearSleepingPos();'):]
    clear = clear[:clear.index('\n  public ', 4)]
    assert 'getSleepingPos' in sleeping and 'Optional.isPresent' in sleeping
    assert 'SLEEPING_POS_ID' in clear and 'Optional.empty' in clear
    gui = inspected['net.minecraft.client.gui.Gui']
    gui = gui[gui.index('  public void setScreen('):]
    gui = gui[:gui.index('\n  public ', 4)]
    assert 'Screen.removed' in gui and 'Screen.clearFocus' in gui and '.onClose' not in gui

    api = out / 'api-source'
    game = api / 'crates/horizon-client/src/game'
    game.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (game.parent / 'mod_api.rs').write_text('', encoding='utf-8')
    (game / 'native_hooks.rs').write_text("impl Game<'_>{\n" + body + '\n' + blocks['NATIVE_TICK'] + '\n}\n', encoding='utf-8')
    render = api / 'crates/horizon-client-render/src/gfx/native_render/generated.rs'
    render.parent.mkdir(parents=True, exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}', encoding='utf-8')
    catalog = {'helpers': [{'name': name, 'target': "horizon_client::game::native_hooks::Game<'_>::native_" + name}
                           for name in contract['helper_names']], 'hooks': [{'name': 'client_tick_pre', 'target': "horizon_client::game::native_hooks::Game<'_>::native_client_tick_pre", 'mod_api': {'contract': 'listen'}}]}
    (api / 'native-hooks.json').write_text(json.dumps(catalog, indent=2) + '\n', encoding='utf-8')
    (out / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n', encoding='utf-8')
    (out / 'Cargo.toml').write_text('''[package]
name="sleep-walker-tests"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
horizon-core={path="''' + (UPSTREAM / 'crates/horizon-core').as_posix() + '''"}
[build-dependencies]
horizon-mod-api-gen={path="''' + (UPSTREAM / 'crates/horizon-mod-api-gen').as_posix() + '"}\n', encoding='utf-8')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(ROOT / 'build/no-pose-hook-tests/target')
    commands = []
    for name, command in [
        ('test', ['cargo', '+stable', 'test', '--offline', '--manifest-path', str(out / 'Cargo.toml')]),
        ('clippy', ['cargo', '+stable', 'clippy', '--offline', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings']),
    ]:
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True,
                                encoding='utf-8', errors='replace')
        (out / (name + '.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        print(result.stdout + result.stderr)
        commands.append({'argv': command, 'exitCode': result.returncode})
        if result.returncode:
            break
    files = [HERE / 'sleep-walker-host.rs', HERE / 'sleep-walker-patch.py', HERE / 'sleep-walker-fixture.rs', Path(__file__)]
    upstream = ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/exploit/ModuleSleepWalker.kt'
    files.append(upstream)
    report = {
        'passed': all(command['exitCode'] == 0 for command in commands),
        'commands': commands,
        'sourceSha256': {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in files},
        'originalHorizonSha256': original_hashes,
        'extractedBodiesSha256': {key: hashlib.sha256(value.encode()).hexdigest() for key, value in blocks.items()},
        'officialJarSha1': hashlib.sha1(jar.read_bytes()).hexdigest(),
        'official26_3BytecodeChecks': ['isSleeping reads SleepingPos presence independently of pose', 'clearSleepingPos stores Optional.empty', 'Gui.setScreen calls clearFocus/removed rather than onClose'],
        'scope': 'Production native helper and real generated Host implementation; actual pose/dimension and interrupted-chat cleanup bodies in storage shells. Position/velocity/timer, typed Chinese draft, pose-vs-position guard, repeated sleeps, and server/guest/replay/missing-player exclusions execute in Rust.',
        'boundaries': ['attribute and mouse capture shells', 'no full game actor/module registry execution', 'no server-authority bypass assertion', 'no in-game sleep screenshot'],
    }
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
