#!/usr/bin/env python3
"""Compile SnapTap policy, actual common config, real key bindings and Host bridge.

Only writes its isolated build directory. Full-client differential is not run.
"""
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = ROOT / 'upstream/Horizon'
OUT = ROOT / 'build/snap-tap-tests'


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    block = runpy.run_path(str(HERE / 'test-attack-patch.py'))['block']
    changed = {}

    def once(text, before, after):
        if text.count(before) != 1:
            raise ValueError(f'SnapTap expected one production anchor: {before!r}')
        return text.replace(before, after, 1)

    def edit(path, transform):
        changed[path] = transform(changed.get(path, (UPSTREAM / path).read_text(encoding='utf-8')))

    patch = runpy.run_path(str(HERE / 'snap-tap-patch.py'))
    contract = patch['apply'](edit, once)
    assert contract == {'hook_names': [], 'helper_names': ['client_snap_tap_key_mask']}
    native = changed['crates/horizon-client/src/game/native_hooks.rs']
    native_window = block(native, '    pub(super) fn native_window(')
    helper = block(native, '    pub(super) fn native_client_snap_tap_key_mask(')
    tick = block(native, '    pub(super) fn native_client_tick_pre(')
    assert 'self.screen' not in helper and 'self.player' not in helper
    fixture = (HERE / 'snap-tap-fixture.rs').read_text(encoding='utf-8')
    replacements = {
        'ROTATIONS_PATH': ROOT / 'native/liquidbounce-behaviors/src/rotations.rs',
        'POLICY_PATH': ROOT / 'native/liquidbounce-behaviors/src/snap_tap.rs',
        'CLOCK_PATH': HERE / 'engine/src/snap_tap.rs',
        'TESTS_PATH': HERE / 'snap-tap-tests.rs',
        'NATIVE_WINDOW': native_window,
        'NATIVE_HELPER': helper,
    }
    for name, value in replacements.items():
        fixture = fixture.replace('__' + name + '__', value.as_posix() if isinstance(value, Path) else value)
    (OUT / 'lib.rs').write_text(fixture, encoding='utf-8')

    shared = (HERE / 'engine/src/config.rs').read_text(encoding='utf-8')
    config = '''use serde_json::{Value,Map,json};
#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat {Interop,File}
enum ModuleKind {SnapTap} impl ModuleKind {fn key(self)->&'static str {"liquidbounce.module.snapTap"}}
'''
    for anchor in ['pub struct CommonSettings', 'pub struct ConfigError', "struct Tree<'a>", "impl Tree<'_>"]:
        if anchor == 'pub struct ConfigError':
            config += '#[allow(dead_code)]\n#[derive(Debug)]\n'
        if anchor == "impl Tree<'_>":
            config += '#[allow(dead_code)]\n'
        config += block(shared, anchor) + '\n'
    config += '#[path="' + (HERE / 'engine/src/config/snap_tap.rs').as_posix() + '"]mod snap;\n'
    config += (HERE / 'snap-tap-config-tests.rs').read_text(encoding='utf-8')
    (OUT / 'config-fixture.rs').write_text(config, encoding='utf-8')

    api = OUT / 'api-source'
    game = api / 'crates/horizon-client/src/game'
    game.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (game.parent / 'mod_api.rs').write_text('pub use winit::keyboard::KeyCode;\n', encoding='utf-8')
    (game / 'native_hooks.rs').write_text("impl Game<'_>{\n" + helper + '\n' + tick + '\n}', encoding='utf-8')
    render = api / 'crates/horizon-client-render/src/gfx/native_render/generated.rs'
    render.parent.mkdir(parents=True, exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}', encoding='utf-8')
    catalog = {'hooks': [{'name': 'client_tick_pre', 'target': "horizon_client::game::native_hooks::Game<'_>::native_client_tick_pre", 'mod_api': {'contract': 'listen'}}], 'helpers': [
        {'name': name, 'target': "horizon_client::game::native_hooks::Game<'_>::native_" + name}
        for name in contract['helper_names']]}
    (api / 'native-hooks.json').write_text(json.dumps(catalog, indent=2) + '\n', encoding='utf-8')
    (OUT / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n', encoding='utf-8')
    manifest = '[package]\nname="liquidbounce-snap-tap-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\nwinit="0.30.13"\n'
    manifest += 'horizon-client-input={path="' + (UPSTREAM / 'crates/horizon-client-input').as_posix() + '"}\n'
    manifest += '[build-dependencies]\nhorizon-mod-api-gen={path="' + (UPSTREAM / 'crates/horizon-mod-api-gen').as_posix() + '"}\n'
    (OUT / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(ROOT / 'build/no-entity-interact-tests/target')
    commands = []
    for name, command in [
        ('test', ['cargo', '+stable', 'test', '--offline', '--manifest-path', str(OUT / 'Cargo.toml')]),
        ('clippy', ['cargo', '+stable', 'clippy', '--offline', '--manifest-path', str(OUT / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings']),
    ]:
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (OUT / (name + '.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        print(result.stdout + result.stderr, end='')
        commands.append({'argv': command, 'exitCode': result.returncode})
        if result.returncode:
            break
    files = [Path(value) for value in replacements.values() if isinstance(value, Path)]
    files += [HERE / name for name in ['snap-tap-host.rs', 'snap-tap-patch.py', 'snap-tap-fixture.rs', 'snap-tap-config-tests.rs', 'engine/src/config/snap_tap.rs']]
    files += [Path(__file__), ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/movement/ModuleSnapTap.kt']
    report = {
        'passed': len(commands) == 2 and all(command['exitCode'] == 0 for command in commands),
        'commands': commands,
        'contract': contract,
        'sourceSha256': {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in files},
        'extractedBodiesSha256': {name: hashlib.sha256(value.encode()).hexdigest() for name, value in replacements.items() if isinstance(value, str)},
        'scope': 'Source-derived SnapTap policy; actual shared Tree/common config; real Horizon KeyBinds and winit KeyCode; actual native helper and actual generated Host bridge.',
        'boundaries': ['game options/settings storage shell', 'no full module registry or game actor execution', 'no original/native full-client differential'],
    }
    (OUT / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
