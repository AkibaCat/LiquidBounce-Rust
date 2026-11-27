#!/usr/bin/env python3
"""Original-source ray differential plus actual config and complete composition."""
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = ROOT / 'upstream/Horizon'
OUT = ROOT / 'build/ghost-hand-tests'


def compose():
    generator = HERE / 'make-behavior-patch.py'
    source = generator.read_text(encoding='utf-8')
    boundary = '\noutput = []\n'
    assert source.count(boundary) == 1
    scope = {'__file__': str(generator), '__name__': 'ghost_hand_composition_audit'}
    exec(compile(source.split(boundary, 1)[0], str(generator), 'exec'), scope)
    patch = runpy.run_path(str(HERE / 'ghost-hand-patch.py'))
    if 'client_ghost_hand' not in scope['names']:
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
    result = subprocess.run(['rustfmt', '+stable', '--edition', '2024', '--config', 'skip_children=true', '--emit', 'files', *paths],
        cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
    (OUT / 'composition-syntax.log').write_text(result.stdout + result.stderr, encoding='utf-8')
    assert result.returncode == 0, result.stdout + result.stderr
    (OUT / 'composition.json').write_text(json.dumps({'contract': contract, 'rustSources': len(paths),
        'generatorSha256': hashlib.sha256(source.encode()).hexdigest(),
        'scope': 'all_production_transforms_then_GhostHand;syntax_only;not_full_native_typecheck'}, indent=2) + '\n', encoding='utf-8')
    return scope['changes']


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    if not (OUT / 'oracle.json').is_file():
        raise SystemExit('Run collect-ghost-hand-facts.py first')
    block = runpy.run_path(str(HERE / 'test-attack-patch.py'))['block']
    changes = compose()
    config = '''use serde_json::{Value,Map,json};use std::collections::BTreeMap;
#[derive(Clone,Copy,PartialEq,Eq)]enum JsonFormat {Interop,File}
enum ModuleKind{GhostHand}impl ModuleKind{fn key(self)->&'static str{"liquidbounce.module.ghostHand"}}
'''
    shared = (HERE / 'engine/src/config.rs').read_text(encoding='utf-8')
    for anchor in ['pub struct CommonSettings', 'pub struct ConfigError', 'impl ConfigError', "struct Tree<'a>", "impl Tree<'_>",
                   'fn scalar_string(', "fn nodes<'a>(", "fn value_of<'a>(", '\nfn leaf(']:
        if anchor == 'pub struct ConfigError': config += '#[derive(Debug)]\n#[allow(dead_code)]\n'
        if anchor == "impl Tree<'_>": config += '#[allow(dead_code)]\n'
        config += block(shared, anchor) + '\n'
    config += '''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Debug,PartialEq)]enum SettingChange{GhostHand(ghost::GhostHandSetting)}
#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}
'''
    config += '#[path="' + (HERE / 'engine/src/config/ghost_hand.rs').as_posix() + '"]mod ghost;\n'
    config += (HERE / 'ghost-hand-config-tests.rs').read_text(encoding='utf-8')
    (OUT / 'config-fixture.rs').write_text(config, encoding='utf-8')
    source = '#![cfg(test)]\nextern crate self as liquidbounce_behaviors;\n'
    for name, path in [('ghost_hand', ROOT / 'native/liquidbounce-behaviors/src/ghost_hand.rs'),
                       ('native_ray', HERE / 'ghost-hand-ray.rs')]:
        source += '#[path="' + path.as_posix() + '"]pub mod ' + name + ';\n'
    source += '#[path="config-fixture.rs"]mod config;\n#[path="tests.rs"]mod tests;\n'
    source += 'extern crate self as horizon_client;\n#[path="native-fixture.rs"]mod native;\npub use native::mod_api;\n'
    (OUT / 'lib.rs').write_text(source, encoding='utf-8')
    (OUT / 'tests.rs').write_text((HERE / 'ghost-hand-tests.rs').read_text(encoding='utf-8'), encoding='utf-8')
    patch = runpy.run_path(str(HERE / 'ghost-hand-patch.py'))
    native = (HERE / 'ghost-hand-native-fixture.rs').read_text(encoding='utf-8').replace('__NATIVE__', patch['HOOKS'])
    (OUT / 'native-fixture.rs').write_text(native, encoding='utf-8')
    (OUT / 'native-game-ray.rs').write_text(changes['crates/horizon-client/src/game/client_ghost_hand.rs'][1], encoding='utf-8')
    api = OUT / 'api-source'
    game = api / 'crates/horizon-client/src/game'
    game.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (game.parent / 'mod_api.rs').write_text('', encoding='utf-8')
    (game / 'native_hooks.rs').write_text("impl Game<'_>{\n" + patch['HOOKS'] + '\n}', encoding='utf-8')
    render = api / 'crates/horizon-client-render/src/gfx/native_render/generated.rs'
    render.parent.mkdir(parents=True, exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}', encoding='utf-8')
    catalog = {'helpers': [], 'hooks': [{'name': 'client_ghost_hand', 'target': "horizon_client::game::native_hooks::Game<'_>::native_client_ghost_hand", 'mod_api': {'contract': 'listen'}}]}
    (api / 'native-hooks.json').write_text(json.dumps(catalog), encoding='utf-8')
    (OUT / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}', encoding='utf-8')
    manifest = '[package]\nname="liquidbounce-ghost-hand-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n'
    for crate in ['horizon-core', 'horizon-world']:
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
        if result.returncode: break
    files = [HERE / name for name in ['ghost-hand-ray.rs', 'ghost-hand-game.rs', 'ghost-hand-host.rs', 'ghost-hand-patch.py',
        'ghost-hand-tests.rs', 'ghost-hand-config-tests.rs', 'ghost-hand-native-fixture.rs', 'engine/src/config/ghost_hand.rs']]
    files += [ROOT / 'native/liquidbounce-behaviors/src/ghost_hand.rs', Path(__file__)]
    report = {'passed': len(commands) == 2 and all(c['exitCode'] == 0 for c in commands), 'commands': commands,
        'sourceSha256': {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in files},
        'scope': 'actual_source_TweakedMethods_java_differential_4608_cases;actual_shared_config;complete_composition_syntax',
        'boundaries': ['native_context_storage_shells', 'no full registry/actor execution', 'full original/native client differential not_run']}
    (OUT / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
