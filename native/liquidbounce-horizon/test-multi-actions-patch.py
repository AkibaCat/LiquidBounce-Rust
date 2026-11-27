#!/usr/bin/env python3
"""Compile actual transformed click/input methods plus generated native hooks."""
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
UPSTREAM = ROOT / 'upstream/Horizon'
OUT = ROOT / 'build/multi-actions-tests'


def main():
    block = runpy.run_path(str(HERE / 'test-attack-patch.py'))['block']
    changed = {}
    def once(text, before, after):
        if text.count(before) != 1:
            raise ValueError(f'MultiActions expected one anchor ({text.count(before)}): {before[:100]!r}')
        return text.replace(before, after, 1)
    def edit(path, transform):
        source = UPSTREAM / path
        text = changed.get(path, source.read_text(encoding='utf-8') if source.exists() else '')
        if path.endswith('native_hooks.rs') and '/// Integer Options.fov' not in text:
            text = text.replace('    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
                '    /// Integer Options.fov value before Camera.calculateFov / getNearPlane.\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
        changed[path] = transform(text)
    for name in ['attack-patch.py', 'fast-place-patch.py', 'no-block-interact-patch.py']:
        runpy.run_path(str(HERE / name))['apply'](edit, once)
    helper = runpy.run_path(str(HERE / 'multi-actions-patch.py'))
    contract = helper['apply'](edit, once)
    OUT.mkdir(parents=True, exist_ok=True)
    cache = Path(os.environ['USERPROFILE']) / '.gradle/caches/modules-2/files-2.1/com.google.code.gson'
    jars = sorted(cache.rglob('*.jar'))
    java = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    oracle = subprocess.run([str(java), '--class-path', os.pathsep.join(map(str, jars)),
        str(HERE / 'oracles/MultiActionsFacts.java'), str(OUT / 'actions-oracle.json')], capture_output=True, text=True)
    (OUT / 'java-oracle.log').write_text(oracle.stdout + oracle.stderr, encoding='utf-8')
    if oracle.returncode: raise SystemExit(oracle.stdout + oracle.stderr)
    first = changed['crates/horizon-client/src/first_person.rs']
    declarations = []
    for declaration in ['pub enum Hand', 'impl Hand', 'pub struct SwingAnimation', 'impl SwingAnimation',
                        'pub enum Hit', 'pub enum UseResult', 'impl UseResult', 'pub trait HandWorld',
                        'pub struct Destroying', 'impl Default for Destroying', 'impl Destroying', 'pub struct Clicks']:
        snippet = block(first, declaration)
        if declaration.startswith('pub enum') or declaration == 'pub struct SwingAnimation':
            snippet = '#[derive(Clone,Copy,Debug,PartialEq)]\n' + snippet
        elif declaration == 'pub struct Destroying': snippet = '#[derive(Clone,Debug,PartialEq)]\n' + snippet
        elif declaration == 'pub struct Clicks': snippet = '#[derive(Default)]\n' + snippet
        declarations.append(snippet)
    declarations.append(block(first, 'impl Clicks'))
    hand = changed['crates/horizon-client/src/game/hand.rs']
    remote = changed['crates/horizon-client/src/game/remote.rs']
    play_input = changed['crates/horizon-client/src/game/play_input.rs']
    game = changed['crates/horizon-client/src/game.rs']
    frame = game[game.index('        let client_keybind_was_using ='):game.index('        self.tick_bundle_hover();')]
    frame += 'horizon_client::spans::time("tick_local_hand", || self.tick_local_hand(client_keybind_was_using))?;Ok(())'
    remote_input = block(remote, 'impl horizon_client::interaction_input::InteractionWorld for RemoteInputWorld')
    replacements = {
        'FIRST_PERSON': '\n'.join(declarations),
        'NATIVE': helper['HOOKS'],
        'LOCAL_TICK': block(hand, 'pub(super) fn tick_local_hand('),
        'LOCAL_USE': block(hand, 'pub(super) fn use_with_hand('),
        'PICK_ITEM': block(play_input, 'pub(super) fn client_pick_item('),
        'PLAY_MOUSE': block(play_input, 'pub(super) fn play_mouse('),
        'REMOTE_MOUSE': block(remote, 'pub(super) fn remote_mouse('),
        'REMOTE_USE': block(remote, 'fn remote_use('),
        'REMOTE_CONTINUE': block(remote, 'fn remote_continue_attack('),
        'REMOTE_USE_WORLD': block(remote, 'impl horizon_client::interaction_input::TargetUseWorld for RemoteUseWorld'),
        'REMOTE_INPUT_METHODS': '\n'.join(block(remote_input, anchor) for anchor in [
            'fn client_attack_while_using(', 'fn pick_hit_result(', 'fn release_hit_result(',
            'fn using_item(', 'fn release_using(', 'fn start_attack(', 'fn start_use(', 'fn use_delay(', 'fn continue_attack(']),
        'HAND_QUERY': block(block(hand, "impl HandWorld for GameHands<'_, '_>"), 'fn client_multi_actions('),
        'FRAME': frame,
        'POLICY_PATH': (ROOT / 'native/liquidbounce-behaviors/src/multi_actions.rs').as_posix(),
    }
    source = (HERE / 'multi-actions-fixture.rs').read_text(encoding='utf-8')
    for name, value in replacements.items(): source = source.replace('__' + name + '__', value)
    source = source.replace('pub(super)', 'pub(crate)').replace("RemoteUseWorld<'_, '_>", "RemoteUseWorld<'_>")
    (OUT / 'lib.rs').write_text(source, encoding='utf-8')
    (OUT / 'interaction_input.rs').write_text(changed['crates/horizon-client-input/src/interaction_input.rs'], encoding='utf-8')
    config = (HERE / 'engine/src/config.rs').read_text(encoding='utf-8')
    config_source = 'use serde_json::{Value,Map,json};use std::collections::BTreeMap;\n'
    config_source += '#[derive(Clone,Copy,PartialEq,Eq)]enum JsonFormat{Interop,File}\nenum ModuleKind{MultiActions}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.multiActions"}}\n'
    for anchor in ['pub struct CommonSettings', 'pub struct ConfigError', 'impl ConfigError', "struct Tree<'a>", "impl Tree<'_>", 'fn scalar_string(', "fn nodes<'a>("]:
        if anchor == 'pub struct ConfigError': config_source += '#[derive(Debug)]\n'
        config_source += block(config, anchor) + '\n'
    config_source += 'type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n'
    config_source += 'enum SettingChange{MultiActions(multi::MultiActionsSetting)}#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}\n'
    config_source += '#[path="' + (HERE / 'engine/src/config/multi_actions.rs').as_posix() + '"]mod multi;\n'
    config_source += '''#[cfg(test)]mod tests {
use super::*;use crate::multi_actions::{ACTIONS,MultiActions};
#[test]fn actual_config_serializer_preserves_declaration_order_and_multichoice_metadata(){
let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({"key":"key.keyboard.unknown","modifiers":[]})};
for format in [JsonFormat::Interop,JsonFormat::File]{let tree=multi::serialize_multi_actions(&MultiActions::default(),&common,format,&|key|key.to_owned()).unwrap();
let values=tree["value"].as_array().unwrap();let names:Vec<_>=values.iter().map(|v|v["name"].as_str().unwrap()).collect();
assert_eq!(names,if format==JsonFormat::Interop{vec!["Bind","Hidden","Actions"]}else{vec!["Enabled","Bind","Hidden","Actions"]});
let actions=values.last().unwrap();assert_eq!(actions["value"],json!(ACTIONS));
if format==JsonFormat::Interop{assert_eq!(actions["valueType"],"MULTI_CHOOSE");assert_eq!(actions["choices"],json!(ACTIONS));assert_eq!(actions["canBeNone"],true);assert_eq!(actions["isOrderSensitive"],false);}else{assert!(actions.get("choices").is_none());}}}
#[test]fn actual_config_child_applies_jvm_oracle_values_with_partial_failure_and_null(){
let facts:Value=serde_json::from_str(include_str!("actions-oracle.json")).unwrap();
for row in facts["rows"].as_array().unwrap(){let tree=json!({"name":"MultiActions","value":[{"name":"Actions","value":row["input"]}]});let root=nodes(&tree,"MultiActions","MultiActions").unwrap();let mut patch=SettingsPatch::default();multi::parse_multi_actions(&mut patch,&root);assert!(patch.issues.is_empty());assert_eq!(patch.changes.len(),1);let mut state=MultiActions::default();for change in patch.changes{let SettingChange::MultiActions(change)=change;let result=change.apply(&mut state);assert_eq!(result.is_err(),!row["error"].is_null());}assert_eq!(state.actions as u64,row["mask"].as_u64().unwrap());}
let tree=json!({"name":"MultiActions","value":[{"name":"Actions"}]});let mut patch=SettingsPatch::default();multi::parse_multi_actions(&mut patch,&nodes(&tree,"MultiActions","MultiActions").unwrap());assert!(patch.changes.is_empty());assert_eq!(patch.issues.len(),1);}
}\n'''
    (OUT / 'config-fixture.rs').write_text(config_source, encoding='utf-8')
    assert 'pick_clicks: u32,' in game and 'pick_clicks: 0,' in game
    assert 'self.use_with_hand()?;' not in game[game.index('fn tick_world('):game.index('fn title_input_active(')]
    # Source guards: every direct consumer uses the same module's static queries.
    liquid = ROOT / 'upstream/LiquidBounce/src/main'
    mixin = (liquid / 'java/net/ccbluex/liquidbounce/injection/mixins/minecraft/client/MixinMinecraft.java').read_text(encoding='utf-8')
    for name in ['mayBreakWhileUsing', 'mayPlaceWhileBreaking', 'mayAttackWhileUsing']:
        assert f'ModuleMultiActions.{name}()' in mixin
    kotlin = (liquid / 'kotlin/net/ccbluex/liquidbounce/features/module/modules/exploit/ModuleMultiActions.kt').read_text(encoding='utf-8')
    assert kotlin.count('this.enabled && Actions.') == 3 and 'aliases = listOf("MultiTask")' in kotlin
    api = OUT / 'api-source'; native = api / 'crates/horizon-client/src/game'
    native.mkdir(parents=True, exist_ok=True)
    (api / 'tools/native-hook-fragments').mkdir(parents=True, exist_ok=True)
    (api / 'crates/horizon-client/src/mod_api.rs').write_text('', encoding='utf-8')
    (native / 'native_hooks.rs').write_text("impl Game<'_> {\n" + helper['HOOKS'] + '\n}', encoding='utf-8')
    catalog = {'hooks': [{'name': name, 'target': f"horizon_client::game::native_hooks::Game<'_>::native_{name}",
                           'mod_api': {'contract': 'listen'}} for name in helper['HOOK_NAMES']], 'helpers': []}
    (api / 'native-hooks.json').write_text(json.dumps(catalog), encoding='utf-8')
    (OUT / 'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate_test_hooks(&root,&out).unwrap();}', encoding='utf-8')
    manifest = '[package]\nname="liquidbounce-multi-actions-tests"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n'
    manifest += '[build-dependencies]\nhorizon-mod-api-gen={path="' + (UPSTREAM / 'crates/horizon-mod-api-gen').as_posix() + '"}\n'
    (OUT / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    env = os.environ.copy(); env['CARGO_TARGET_DIR'] = str(ROOT / 'build/no-entity-interact-tests/target')
    result = subprocess.run(['cargo', '+stable', 'test', '--manifest-path', str(OUT / 'Cargo.toml')], cwd=ROOT, env=env)
    evidence = {'exitCode': result.returncode, 'sourceSha256': hashlib.sha256(source.encode()).hexdigest(),
                'fragment': contract, 'productionMethods': list(replacements), 'transformedPaths': list(changed),
                'composedExtensions': ['attack', 'FastPlace', 'NoBlockInteract', 'MultiActions'],
                'oracle': 'actual Gson+EnumSet+String.CASE_INSENSITIVE_ORDER following source MultiChoiceListValue loop',
                'scope': 'actual_generated_native_hook;actual_first_person_Clicks_and_Destroying;actual_InteractionInput;actual_local_tick_mouse_pick_use;actual_remote_mouse_use_continue_and_input_wrappers;world_side_effects_stubbed;full_client_differential_not_run'}
    (OUT / 'result.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    raise SystemExit(result.returncode)


if __name__ == '__main__': main()
