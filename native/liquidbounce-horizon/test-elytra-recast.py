#!/usr/bin/env python3
"""Original MC stack oracle, policy flow, generated Host and real dry-tail/send bodies.

Only writes this module's isolated build directory; no full client differential.
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
OUT = ROOT / 'build/elytra-recast-tests'


def compose():
    generator = HERE / 'make-behavior-patch.py'
    source = generator.read_text(encoding='utf-8')
    boundary = '\noutput = []\n'
    assert source.count(boundary) == 1
    scope = {'__file__': str(generator), '__name__': 'elytra_recast_composition_audit'}
    exec(compile(source.split(boundary, 1)[0], str(generator), 'exec'), scope)
    patch = runpy.run_path(str(HERE / 'elytra-recast-patch.py'))
    if 'client_elytra_recast_running' not in scope['names']:
        contract = patch['apply'](scope['edit'], scope['replace_once'])
    else:
        contract = {'alreadyInProduction': True}
    work = OUT / 'composition'
    work.mkdir(parents=True, exist_ok=True)
    paths = []
    for name, (_, text) in scope['changes'].items():
        if not name.endswith('.rs'):
            continue
        destination = work / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(text, encoding='utf-8')
        paths.append(str(destination))
    syntax = []
    for offset in range(0, len(paths), 48):
        result = subprocess.run(['rustfmt', '+stable', '--edition', '2024', '--config', 'skip_children=true', '--emit', 'files', *paths[offset:offset + 48]],
            cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
        syntax.append(result.stdout + result.stderr)
        assert result.returncode == 0, result.stdout + result.stderr
    (OUT / 'composition-syntax.log').write_text(''.join(syntax), encoding='utf-8')
    (OUT / 'composition.json').write_text(json.dumps({'contract': contract, 'rustSources': len(paths),
        'generatorSha256': hashlib.sha256(source.encode()).hexdigest(),
        'scope': 'all_production_transforms_then_ElytraRecast;syntax_only;not_full_native_typecheck'}, indent=2) + '\n', encoding='utf-8')
    return scope['changes'], patch, contract


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    if not (OUT / 'oracle.json').is_file():
        raise SystemExit('Run collect-elytra-recast-facts.py first')
    facts=json.loads((OUT / 'oracle.json').read_text(encoding='utf-8'))
    if not facts.get('originalModuleUnchanged') or len(facts.get('direct',[]))!=2048:
        raise SystemExit('Run collect-elytra-recast-original.py after stack collection')
    block = runpy.run_path(str(HERE / 'test-attack-patch.py'))['block']
    changes, patch, contract = compose()
    motion = changes['crates/horizon-client/src/motion.rs'][1]
    native = changes['crates/horizon-client/src/game/native_hooks.rs'][1]
    network = changes['crates/horizon-client/src/net.rs'][1]
    render = (UPSTREAM / 'crates/horizon-client/src/item_render.rs').read_text(encoding='utf-8')
    native_methods = '\n'.join(block(native, '    pub(super) fn native_' + name + '(')
        for name in ['client_elytra_recast_running', 'client_recasts_elytra'])
    replacements = {
        'STACK_VIEW': '\n'.join(block(render, anchor) for anchor in ['pub fn tag_json(', 'fn custom_model_data_json(', 'pub fn stack_view(']),
        'POLICY': ROOT / 'native/liquidbounce-behaviors/src/elytra_recast.rs',
        'CHEST': HERE / 'elytra-recast-chest.rs',
        'MOTION_ADAPTER': HERE / 'elytra-recast-motion.rs',
        'TESTS': OUT / 'tests.rs',
        'SESSION': '\n'.join(block(network, signature) for signature in ['    pub fn send(&self,', '    pub fn send_silently(&self,', '    fn send_with_path(&self,']),
        'IDENTITY': block(motion, '    pub fn native_client_queries_allowed('),
        'TICK_INPUT': block(motion, 'enum TickInput {'),
        'DRY_TICK': block(motion, '    fn dry_tick_with_ai_tail('),
        'NATIVE': native_methods,
        'CAPTURE': patch['CAPTURE'],
        'ITEMS': ROOT / 'build/resources/26.3/items.json',
    }
    fixture = (HERE / 'elytra-recast-native-fixture.rs').read_text(encoding='utf-8')
    for name, value in replacements.items():
        fixture = fixture.replace('__' + name + '__', value.as_posix() if isinstance(value, Path) else value)
    fixture += '\n#[cfg(test)]#[path="config-fixture.rs"]mod config;\n'
    (OUT / 'lib.rs').write_text('#![cfg(test)]\n#![allow(dead_code)]\nextern crate self as horizon_client;\n' + fixture, encoding='utf-8')
    (OUT / 'tests.rs').write_text((HERE / 'elytra-recast-tests.rs').read_text(encoding='utf-8'), encoding='utf-8')
    (OUT / 'packet-bridge.rs').write_text(changes['crates/horizon-client/src/packet_bridge.rs'][1], encoding='utf-8')
    shared = (HERE / 'engine/src/config.rs').read_text(encoding='utf-8')
    config = '''use serde_json::{Value,Map,json};
#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat {Interop,File}
enum ModuleKind {ElytraRecast} impl ModuleKind {fn key(self)->&'static str {"liquidbounce.module.elytraRecast"}}
'''
    for anchor in ['pub struct CommonSettings', 'pub struct ConfigError', "struct Tree<'a>", "impl Tree<'_>"]:
        if anchor == 'pub struct ConfigError':
            config += '#[allow(dead_code)]\n#[derive(Debug)]\n'
        if anchor == "impl Tree<'_>":
            config += '#[allow(dead_code)]\n'
        config += block(shared, anchor) + '\n'
    config += '#[path="' + (HERE / 'engine/src/config/elytra_recast.rs').as_posix() + '"]mod elytra;\n'
    config += (HERE / 'elytra-recast-config-tests.rs').read_text(encoding='utf-8')
    (OUT / 'config-fixture.rs').write_text(config, encoding='utf-8')
    api = OUT / 'api-source'
    game = api / 'crates/horizon-client/src/game'
    game.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (game.parent / 'mod_api.rs').write_text('', encoding='utf-8')
    (game / 'native_hooks.rs').write_text("impl Game<'_>{\n" + native_methods + '\n}', encoding='utf-8')
    render_api = api / 'crates/horizon-client-render/src/gfx/native_render/generated.rs'
    render_api.parent.mkdir(parents=True, exist_ok=True)
    render_api.write_text('use super::*;pub trait RenderHooks:Send+Sync{}', encoding='utf-8')
    catalog = {'helpers': [{'name': 'client_recasts_elytra', 'target': "horizon_client::game::native_hooks::Game<'_>::native_client_recasts_elytra"}],
        'hooks': [{'name': 'client_elytra_recast_running', 'target': "horizon_client::game::native_hooks::Game<'_>::native_client_elytra_recast_running", 'mod_api': {'contract': 'listen'}}]}
    (api / 'native-hooks.json').write_text(json.dumps(catalog), encoding='utf-8')
    (OUT / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}', encoding='utf-8')
    manifest = '[package]\nname="liquidbounce-elytra-recast-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\nlog="0.4"\n'
    for crate in ['horizon-assets', 'horizon-core', 'horizon-world', 'horizon-items', 'horizon-protocol']:
        manifest += crate + '={path="' + (UPSTREAM / 'crates' / crate).as_posix() + '"}\n'
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
    files = [HERE / name for name in ['elytra-recast-chest.rs', 'elytra-recast-motion.rs', 'elytra-recast-host.rs', 'elytra-recast-patch.py',
        'elytra-recast-tests.rs', 'elytra-recast-config-tests.rs', 'elytra-recast-native-fixture.rs', 'engine/src/config/elytra_recast.rs']]
    files += [ROOT / 'native/liquidbounce-behaviors/src/elytra_recast.rs', Path(__file__),
        HERE/'collect-elytra-recast-facts.py',HERE/'collect-elytra-recast-original.py',HERE/'oracles/ElytraRecastFacts.java']
    report = {'passed': len(commands) == 2 and all(c['exitCode'] == 0 for c in commands), 'commands': commands, 'contract': contract,
        'sourceSha256': {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in files},
        'oracleSha256':hashlib.sha256((OUT/'oracle.json').read_bytes()).hexdigest(),
        'originalModuleUnchanged':facts['originalModuleUnchanged'],
        'extractedBodiesSha256': {name: hashlib.sha256(value.encode()).hexdigest() for name, value in replacements.items() if isinstance(value, str)},
        'scope': 'actual_MC_ItemStack_362_cases;unchanged_Kotlin_exact_original_mixin_8192_tail_and_2048_direct_cases;actual_shared_config;actual_generated_Host;actual_transformed_dry_tick;isolated_remote_module_minimal_imports;actual_Session_Pipe_packet_codec;complete_composition_syntax',
        'boundaries': ['native_context_storage_and_outer_controlled_tick_shells', 'slot_bridge_stub_not_used_by_PlayerCommand', 'no full registry/actor execution', 'full original/native client differential not_run']}
    (OUT / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
