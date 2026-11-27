#!/usr/bin/env python3
"""Prepare a focused production KillAura config fixture; compile only with --run.

This keeps KillAura's four original tests and all tested config bodies intact.
Only registration declarations, the actor/Host boundary and sampling are supplied
locally. It does not compile Horizon, the engine, or the remaining module tests.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BODY = runpy.run_path(str(HERE / 'test-reset-vl.py'))['body']


def prepare(out):
    out.mkdir(parents=True, exist_ok=True)
    shared = (HERE / 'engine/src/config.rs').read_text(encoding='utf8')
    config = '''use std::{collections::BTreeMap,fmt};
use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{KillAura,Aimbot,AutoWalk,NoRotateSet}
impl ModuleKind{
 fn name(self)->&'static str{match self{Self::KillAura=>"KillAura",Self::Aimbot=>"Aimbot",Self::AutoWalk=>"AutoWalk",Self::NoRotateSet=>"NoRotateSet"}}
 fn key(self)->&'static str{match self{Self::KillAura=>"liquidbounce.module.killAura",Self::Aimbot=>"liquidbounce.module.aimbot",Self::AutoWalk=>"liquidbounce.module.autoWalk",Self::NoRotateSet=>"liquidbounce.module.noRotateSet"}}
}
'''
    declarations = [
        ('pub enum JsonFormat', '#[derive(Clone,Copy,Debug,PartialEq,Eq)]'),
        ('pub struct CommonSettings', '#[derive(Clone,Debug,PartialEq)]'),
        ('pub struct ConfigError', '#[derive(Clone,Debug,PartialEq,Eq)]'),
        ('impl ConfigError', ''),
        ('impl fmt::Display for ConfigError', ''),
        ("struct Tree<'a>", ''),
        ("impl Tree<'_>", ''),
        ('fn scalar_string(', ''),
        ("fn nodes<'a>(", ''),
        ('fn float_json(', ''),
        ('fn float_range(', ''),
        ("fn value_of<'a>(", ''),
        ('fn parse_float(', ''),
        ("fn pair_members<'a>(", ''),
        ('fn parse_float_pair(', ''),
        ('fn parse_int_pair(', ''),
        ('fn parse_int(', ''),
        ('fn decimal_int(', ''),
        ('fn parse_bool(', ''),
        ('\nfn leaf(', ''),
        ('pub struct SettingsPatch', '#[derive(Clone,Debug,PartialEq)]'),
    ]
    for anchor, attributes in declarations:
        extracted = BODY(shared, anchor)
        if anchor == '\nfn leaf(':
            assert '&self' not in extracted and 'patch: &mut SettingsPatch' in extracted
        config += attributes + '\n' + extracted + '\n'
    config += '''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{
 Enabled(bool),Bind(Value),Hidden(bool),KillAura(KillAuraSetting),Aimbot(aimbot::AimbotSetting),AutoWalk(AutoWalkSetting),NoRotateSet(NoRotateSetSetting)
}
'''
    # Actual common-leaf parsing; only the module registration dispatch is narrow.
    entry = BODY(shared, 'pub fn parse_settings(')
    common = entry[entry.index('    let root ='):entry.index('    match module {')]
    common = common.replace('nodes(json, group_name, module.name())?', 'nodes(json, module.name(), module.name())?')
    config += 'pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n' + common
    config += '''match module{
 ModuleKind::KillAura=>kill_aura::parse_kill_aura(&mut patch,&root),
 ModuleKind::Aimbot=>aimbot::parse_aimbot(&mut patch,&root),
 ModuleKind::AutoWalk=>auto_walk::parse_auto_walk(&mut patch,&root),
 ModuleKind::NoRotateSet=>no_rotate_set::parse_no_rotate_set(&mut patch,&root),
}Ok(patch)}
'''
    sources = [HERE / 'engine/src/config.rs']
    for name in ['no_rotate_set', 'auto_walk', 'combat_rotations', 'target_renderer', 'aimbot', 'kill_aura']:
        path = HERE / f'engine/src/config/{name}.rs'
        source = path.read_text(encoding='utf8')
        if name != 'kill_aura':
            source = source.split('\n#[cfg(test)]', 1)[0]
        (out / f'{name}-config.rs').write_text(source, encoding='utf8')
        config += f'#[path="{name}-config.rs"]mod {name};\n'
        sources.append(path)
    config += 'pub use no_rotate_set::{NoRotateSetSetting,serialize_no_rotate_set};\npub use auto_walk::{AutoWalkSetting,serialize_auto_walk};\npub use kill_aura::{KillAuraSetting,serialize_kill_aura};\n'
    (out / 'config.rs').write_text(config, encoding='utf8')

    walk = (HERE / 'engine/src/auto_walk_runtime.rs').read_text(encoding='utf8')
    aura = (HERE / 'engine/src/kill_aura_runtime.rs').read_text(encoding='utf8')
    aimbot = (HERE / 'engine/src/aimbot_runtime.rs').read_text(encoding='utf8')
    runtime = '''use liquidbounce_behaviors::{no_rotate_set::RotationsSettings,rotations::DefaultRotationTarget};
'''
    for anchor in ['pub struct RenderSettings', 'impl Default for RenderSettings', 'pub struct AutoWalkSettings', 'impl Default for AutoWalkSettings']:
        runtime += ('#[derive(Clone,Debug,PartialEq)]\n' if anchor.startswith('pub struct') else '') + BODY(walk, anchor) + '\n'
    (out / 'auto-walk-runtime.rs').write_text(runtime, encoding='utf8')
    runtime = '''use liquidbounce_behaviors::{aimbot::Aimbot,point_tracker::PointTracker,clicking::JavaRandom,target_renderer::{TargetRendererSettings,TargetRendererRuntime}};
use crate::auto_rod_runtime::NativeSamples;
'''
    for anchor in ['pub struct AimbotRuntime', 'impl Default for AimbotRuntime']:
        runtime += BODY(aimbot, anchor) + '\n'
    (out / 'aimbot-runtime.rs').write_text(runtime, encoding='utf8')
    runtime = '''use liquidbounce_behaviors::{kill_aura::{self as model,AutoBlock},clicking::{Clicker,JavaRandom},combat_rotations,point_tracker::{PointTracker,Samples},target_renderer::{TargetRendererSettings,TargetRendererRuntime}};
use serde_json::Value;
use std::sync::{Arc,Mutex,atomic::{AtomicBool,AtomicI32}};
use crate::auto_rod_runtime::NativeSamples;
#[derive(Default)]
'''
    for anchor in ['pub struct KillAuraBlocking', 'pub struct KillAuraRuntime', 'impl Default for KillAuraRuntime']:
        runtime += BODY(aura, anchor) + '\n'
    runtime += 'impl KillAuraRuntime{' + BODY(aura, 'pub fn new(blocking:') + '}\n'
    (out / 'kill-aura-runtime.rs').write_text(runtime, encoding='utf8')
    # No sample is asserted by the four config tests. These deterministic
    # providers avoid linking native math or constructing an actual Host.
    lib = '''#![allow(dead_code,unused_imports)]
mod auto_rod_runtime{
 use liquidbounce_behaviors::{clicking::JavaRandom,point_tracker::Samples};
 pub struct NativeSamples<'a>(pub &'a mut JavaRandom);
 impl Samples for NativeSamples<'_>{
  fn float(&mut self,r:[f32;2])->f32{r[0]}
  fn int(&mut self,r:[i32;2])->i32{r[0]}
  fn gaussian(&mut self)->f64{0.}
  fn chance(&mut self,p:f32)->bool{p>=100.}
 }
}
#[path="auto-walk-runtime.rs"]mod auto_walk_runtime;
#[path="aimbot-runtime.rs"]mod aimbot_runtime;
#[path="kill-aura-runtime.rs"]mod kill_aura_runtime;
mod config;
'''
    (out / 'lib.rs').write_text(lib, encoding='utf8')
    behavior = ROOT / 'native/liquidbounce-behaviors'
    (out / 'Cargo.toml').write_text(f'''[package]
name="kill-aura-config-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={{version="1",features=["arbitrary_precision"]}}
liquidbounce-behaviors={{path="{behavior.as_posix()}"}}
''', encoding='utf8')
    sources.extend(HERE / f'engine/src/{name}_runtime.rs' for name in ['auto_walk', 'aimbot', 'kill_aura'])
    original = (HERE / 'engine/src/config/kill_aura.rs').read_text(encoding='utf8')
    assert original.count('#[test]') == 4
    assert (out / 'kill_aura-config.rs').read_text(encoding='utf8') == original
    report = {
        'status': 'prepared-no-compiler-invoked',
        'tests': 4,
        'scope': 'Actual production KillAura config and unchanged four tests; actual shared Tree, scalar/range parsers, common-leaf entry, settings containers/default/new, actual shared AimPoint/Rotations/AutoWalk/TargetRendering config and real behavior dependency. Registration declarations narrowed; Host/actor omitted; runtime sample provider deterministic. No engine/game/source differential claim.',
        'sources': {path.relative_to(ROOT).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest() for path in sources},
    }
    (out / 'preparation.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True)
    parser.add_argument('--run', action='store_true')
    parser.add_argument('--filter', default='kill_aura_config_tests')
    args = parser.parse_args()
    out = ROOT / args.out
    prepare(out)
    if not args.run:
        print(f'KillAura focused config fixture prepared: {out}; no compiler invoked')
        return
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(HERE / 'packet-tests/target')
    command = ['cargo', '+stable', 'test', '--offline', '--manifest-path', str(out / 'Cargo.toml'), args.filter, '--', '--test-threads=1']
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf8', errors='replace')
    (out / 'test.log').write_text(result.stdout + result.stderr, encoding='utf8')
    (out / 'report.json').write_text(json.dumps({'passed': result.returncode == 0, 'exitCode': result.returncode, 'filter': args.filter, 'scope': 'Focused production config; declaration/runtime boundary only; no game test.'}, indent=2) + '\n', encoding='utf8')
    print(result.stdout + result.stderr)
    raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
