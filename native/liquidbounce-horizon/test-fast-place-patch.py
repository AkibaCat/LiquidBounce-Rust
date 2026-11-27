#!/usr/bin/env python3
"""Compile production FastPlace seams and compare real Items::load with the game oracle."""
import hashlib
import json
from pathlib import Path
import re
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
UPSTREAM = ROOT / 'upstream/Horizon'
OUT = ROOT / 'build/fast-place-hook-tests'


def main():
    helper = runpy.run_path(str(HERE / 'fast-place-patch.py'))
    slot = runpy.run_path(str(HERE / 'slot-patch.py'))
    block = runpy.run_path(str(HERE / 'test-camera-control-patch.py'))['block']
    changed = {}

    def edit(path, transform):
        src = UPSTREAM / path
        changed[path] = transform(changed.get(path, src.read_text(encoding='utf-8') if src.exists() else ''))

    def once(text, before, after):
        if text.count(before) != 1:
            raise ValueError(f'Expected one anchor: {before!r}')
        return text.replace(before, after, 1)

    contract = helper['apply'](edit, once)
    game = changed['crates/horizon-client/src/game.rs']
    hand = changed['crates/horizon-client/src/game/hand.rs']
    remote = changed['crates/horizon-client/src/game/remote.rs']
    guests = changed['crates/horizon-client/src/game/lan_actions.rs']
    assert re.search(r'\buse_delay: i32,', game)
    assert len(re.findall(r'\buse_delay: i32,', guests)) == 2
    assert 'self.use_with_hand()?;\n            self.use_delay = 4;' not in game
    assert 'used = self.use_with_hand();\n                self.use_delay = 4;' not in hand
    assert 'self.use_delay = 4;' not in remote
    # The pinned tracker never writes its keyboard map, anywhere in the source.
    kotlin = ROOT / 'upstream/LiquidBounce/src/main'
    tracker = (kotlin / 'kotlin/net/ccbluex/liquidbounce/utils/input/InputTracker.kt').read_text(encoding='utf-8')
    assert tracker.count('keyLastPressed') == 3
    assert 'keyLastPressed[' not in tracker.replace('val lastPressed = keyLastPressed[keyCode]', '')
    assert 'mouseLastPressed[event.button] = System.currentTimeMillis()' in tracker
    mouse_mixin = (kotlin / 'java/net/ccbluex/liquidbounce/injection/mixins/minecraft/client/MixinMouseHandler.java').read_text(encoding='utf-8')
    assert 'target = "Lnet/minecraft/client/gui/Gui;overlay()Lnet/minecraft/client/gui/screens/Overlay;", shift = At.Shift.BEFORE' in mouse_mixin
    inventory_mixin = (kotlin / 'java/net/ccbluex/liquidbounce/injection/mixins/minecraft/entity/MixinInventory.java').read_text(encoding='utf-8')
    assert '"getSelectedItem"' in inventory_mixin and 'SilentHotbar.INSTANCE.getServersideSlot()' in inventory_mixin

    OUT.mkdir(parents=True, exist_ok=True)
    source = (HERE / 'fast-place-fixture.rs').read_text(encoding='utf-8')
    native_source = (UPSTREAM / 'crates/horizon-client/src/game/native_hooks.rs').read_text(encoding='utf-8')
    held = (UPSTREAM / 'crates/horizon-client/src/game/held_items.rs').read_text(encoding='utf-8')
    shield = (UPSTREAM / 'crates/horizon-client/src/game/shield.rs').read_text(encoding='utf-8')
    window = (UPSTREAM / 'crates/horizon-client/src/game/window_handler.rs').read_text(encoding='utf-8')
    window_fragment = window[window.index('        let mouse = if !self.pad_synthetic_input()'):window.index('\n    /// The loop ended:')]
    window_fragment = window_fragment.rsplit('\n    }', 1)[0]
    runtime = (HERE / 'engine/src/runtime.rs').read_text(encoding='utf-8')
    methods = {
        '__NATIVE_METHODS__': helper['HELPERS'] + block(native_source, 'pub(super) fn native_mouse_button_pre('),
        '__SLOT_METHODS__': '\n'.join(block(slot['HELPERS'], signature) for signature in ['pub(super) fn native_client_slot_owner(', 'pub(super) fn client_hotbar_slot(']),
        '__STACK_METHOD__': block(shield, 'pub(super) fn stack_in('),
        '__HELD_METHODS__': block(held, 'pub(super) fn held(') + '\n' + block(held, 'pub(super) fn held_slot(').replace('Hand::Main => self.selected,', 'Hand::Main => self.client_hotbar_slot(),'),
        '__LOCAL_METHOD__': block(hand, 'pub(super) fn use_with_hand('),
        '__REMOTE_METHOD__': block(remote, 'fn remote_use('),
        '__REMOTE_DELAY__': block(remote, 'fn use_delay(&self) -> u32'),
        '__WINDOW_FRAGMENT__': window_fragment,
        '__RUNTIME_METHODS__': block(runtime, 'fn client_use_cooldown(') + '\n' + block(runtime, 'fn mouse_button_pre('),
    }
    for name, value in methods.items():
        source = source.replace(name, value)
    source = source.replace('__RESOURCE_ROOT__', json.dumps((ROOT / 'build/resources/26.3').as_posix()))
    source = source.replace('__ORACLE_PATH__', json.dumps((ROOT / 'build/fastplace-oracle/item-classes.json').as_posix()))
    (OUT / 'lib.rs').write_text(source, encoding='utf-8')
    (OUT / 'projectile_items.rs').write_text(changed['crates/horizon-client/src/fast_place_items.rs'], encoding='utf-8')
    (OUT / 'interaction_input.rs').write_text((UPSTREAM / 'crates/horizon-client-input/src/interaction_input.rs').read_text(encoding='utf-8'), encoding='utf-8')
    api = OUT / 'api-source'
    api_game = api / 'crates/horizon-client/src/game'
    api_game.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (api / 'crates/horizon-client/src/mod_api.rs').write_text('', encoding='utf-8')
    (api_game / 'native_hooks.rs').write_text("impl Game<'_>{" + methods['__NATIVE_METHODS__'] + '}\n', encoding='utf-8')
    render = api / 'crates/horizon-client-render/src/gfx/native_render/generated.rs'
    render.parent.mkdir(parents=True, exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}', encoding='utf-8')
    hooks = [{'name': name, 'target': "horizon_client::game::native_hooks::Game<'_>::native_" + name,
              'mod_api': {'contract': 'listen'}} for name in ['client_use_cooldown', 'mouse_button_pre']]
    (api / 'native-hooks.json').write_text(json.dumps({'hooks': hooks, 'helpers': []}, indent=2) + '\n', encoding='utf-8')
    (OUT / 'build.rs').write_text('''fn main(){
        let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");
        let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();
    }''', encoding='utf-8')
    dependencies = {'liquidbounce-behaviors': ROOT / 'native/liquidbounce-behaviors',
                    'horizon-items': UPSTREAM / 'crates/horizon-items',
                    'horizon-world': UPSTREAM / 'crates/horizon-world'}
    manifest = '''[package]
name="fast-place-hook-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[dependencies]
serde_json="1"
'''
    manifest += ''.join(f'{name}={{path="{path.as_posix()}"}}\n' for name, path in dependencies.items())
    manifest += '[build-dependencies]\nhorizon-mod-api-gen={path="' + (UPSTREAM / 'crates/horizon-mod-api-gen').as_posix() + '"}\n'
    (OUT / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    commands = [(['cargo', '+stable', 'test', '--offline', '--manifest-path', str(OUT / 'Cargo.toml'), '--', '--test-threads=1'], 'test.log'),
                (['cargo', '+stable', 'clippy', '--offline', '--manifest-path', str(OUT / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings'], 'clippy.log')]
    exitcode = 0
    statuses = {}
    for command, name in commands:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        (OUT / name).write_text(result.stdout + result.stderr, encoding='utf-8')
        print(result.stdout + result.stderr)
        statuses[name] = 'passed' if result.returncode == 0 else 'failed'
        exitcode = exitcode or result.returncode
        if result.returncode:
            break
    report = {'hookContract': contract,
              'sourceFiles': {p: hashlib.sha256(t.encode()).hexdigest() for p, t in sorted(changed.items())},
              'testedProductionBodies': ['native_client_use_cooldown', 'native_begin_use_cooldown', 'use_with_hand', 'remote_use',
                                        'stack_in', 'client_hotbar_slot', 'native_client_slot_owner', 'mouse pre dispatch window fragment',
                                        'runtime client_use_cooldown', 'runtime mouse_button_pre', 'Items::load', 'BlockFacts::load',
                                        'BlockRegistry::from_facts_json_with_flags', 'InteractionInput::tick', 'use_on_target'],
              'oracle': {'source': 'FastPlaceFacts.java executed against pinned official Minecraft 26.3 JAR',
                         'file': 'build/fastplace-oracle/item-classes.json', 'items': 1658, 'projectiles': 14},
              'apiGenerator': 'real horizon_mod_api_gen::generate(Target::Client); actual event/dispatch/Host implementation compiled',
              'boundaries': ['game inventory/player/world shell', 'actual item/world/input crates loaded from prepared user-owned 26.3 resources',
                             'deterministic wall clock', 'window event shell', 'interaction result shell'],
              'commands': statuses, 'fullGameIntegrationPassed': False}
    (OUT / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    raise SystemExit(exitcode)


if __name__ == '__main__':
    main()
