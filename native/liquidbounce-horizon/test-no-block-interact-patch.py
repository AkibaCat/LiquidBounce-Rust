#!/usr/bin/env python3
"""Compile actual local/remote use invocation paths, native predicates and hooks."""
from pathlib import Path
import json
import os
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = ROOT / 'build/no-block-interact-tests'


def main():
    helper = runpy.run_path(str(HERE / 'no-block-interact-patch.py'))
    method = runpy.run_path(str(HERE / 'test-movement-patch.py'))['method']
    changed = {}
    def replace_once(text, old, new):
        if text.count(old) != 1: raise ValueError(old[:100])
        return text.replace(old, new, 1)
    def edit(path, transform):
        changed[path] = transform(changed.get(path, (ROOT / 'upstream/Horizon' / path).read_text(encoding='utf-8')))
    fragment = helper['apply'](edit, replace_once)
    def take(path, anchor): return method(changed.get(path, (ROOT / 'upstream/Horizon' / path).read_text(encoding='utf-8')), anchor).replace('pub(super)', 'pub(crate)')
    remote = 'crates/horizon-client/src/game/remote.rs'
    replacements = {
        'NATIVE': helper['HOOKS'], 'RULES': helper['RULES'],
        'USE_ITEM_AS_PLAYER': take('crates/horizon-client/src/game/item_actions.rs', 'pub(super) fn use_item_as_player('),
        'USE_WITH_HAND': take('crates/horizon-client/src/game/hand.rs', 'pub(super) fn use_with_hand('),
        'REMOTE_USE': take(remote, 'pub(super) fn remote_use('),
        'REMOTE_USE_BLOCK': take(remote, 'fn remote_use_block('),
        'REMOTE_USE_HANDS': take(remote, 'fn remote_use_hands('),
        'REMOTE_USE_WORLD': take(remote, 'impl horizon_client::interaction_input::TargetUseWorld for RemoteUseWorld'),
        'TAG_JSON': take('crates/horizon-client/src/item_render.rs', 'pub fn tag_json('),
        'CUSTOM_JSON': take('crates/horizon-client/src/item_render.rs', 'fn custom_model_data_json('),
        'STACK_VIEW': take('crates/horizon-client/src/item_render.rs', 'pub fn stack_view('),
        'COMPONENTS_VIEW': take('crates/horizon-assets/src/item_model.rs', "impl<'a> ComponentsView<'a>"),
    }
    source = (HERE / 'no-block-interact-fixture.rs').read_text(encoding='utf-8')
    for key, value in replacements.items(): source = source.replace('__' + key + '__', value)
    source = source.replace('__RULES_PATH__', (ROOT / 'native/liquidbounce-behaviors/src/no_block_interact.rs').as_posix()).replace('__ENTITY_RULES_PATH__', (ROOT / 'native/liquidbounce-behaviors/src/no_entity_interact.rs').as_posix()).replace('__RESOURCE_ROOT__', (ROOT / 'build/resources/26.3').as_posix())
    source = source.replace("RemoteUseWorld<'_, '_>", "RemoteUseWorld<'_>")
    source = source.replace('pub(super)', 'pub(crate)')
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / 'src').mkdir(exist_ok=True)
    (OUT / 'src/lib.rs').write_text(source, encoding='utf-8')
    (OUT / 'oracle.json').write_bytes((ROOT / 'build/no-interact-oracle/facts.json').read_bytes())
    api = OUT / 'api-source'; native = api / 'crates/horizon-client/src/game'; native.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (api / 'crates/horizon-client/src/mod_api.rs').write_text('', encoding='utf-8')
    (native / 'native_hooks.rs').write_text("impl Game<'_> {\n" + helper['HOOKS'] + '\n}\n', encoding='utf-8')
    catalog = {'helpers': [{'name': name, 'target': f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in helper['HELPER_NAMES']], 'hooks': [{'name': name, 'target': f"horizon_client::game::native_hooks::Game<'_>::native_{name}", 'mod_api': {'contract': 'listen'}} for name in helper['HOOK_NAMES']]}
    (api / 'native-hooks.json').write_text(json.dumps(catalog), encoding='utf-8')
    (OUT / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate_test_hooks(&root,&out).unwrap();}', encoding='utf-8')
    manifest = '[package]\nname="liquidbounce-no-block-interact-tests"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nserde_json="1"\n'
    for crate in ['horizon-core', 'horizon-world', 'horizon-items', 'horizon-client-input']:
        manifest += f'{crate}={{path="{(ROOT / "upstream/Horizon/crates" / crate).as_posix()}"}}\n'
    manifest += f'[build-dependencies]\nhorizon-mod-api-gen={{path="{(ROOT / "upstream/Horizon/crates/horizon-mod-api-gen").as_posix()}"}}\n'
    (OUT / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    env = os.environ.copy(); env['CARGO_TARGET_DIR'] = str(ROOT / 'build/no-entity-interact-tests/target')
    result = subprocess.run(['cargo', '+stable', 'test', '--manifest-path', str(OUT / 'Cargo.toml')], cwd=ROOT, env=env)
    evidence = {'exitCode': result.returncode, 'productionMethods': list(replacements), 'pristinePaths': list(changed), 'fragment': fragment,
                'scope': 'actual_local_startUseItem_loop_and_remote_main_offhand_cancellation;actual_native_predicates;actual_ItemStack_component_conversion_and_ComponentsView;actual_Items_BlockFacts_BlockRegistry;full_client_differential_not_run'}
    (OUT / 'result.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    raise SystemExit(result.returncode)


if __name__ == '__main__': main()
