#!/usr/bin/env python3
"""AntiVoid focused harness; --model-only runs the seven deferred model/config cases."""
from pathlib import Path
import argparse
import ast
import json
import runpy
import subprocess
import tempfile
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]

def static_checks():
    texts={};staged=ROOT/'build/staging/horizon'
    def read(name):
        if name not in texts:
            path=staged/name;texts[name]=path.read_text(encoding='utf8')if path.exists()else''
        return texts[name]
    def edit(name,transform):texts[name]=transform(read(name))
    def once(text,old,new):
        if text.count(old)!=1:raise RuntimeError(f'AntiVoid native anchor count {text.count(old)}: {old[:100]}')
        return text.replace(old,new)
    if 'mod anti_void_native;'not in read('crates/horizon-client/src/game.rs'):
        fragment=runpy.run_path(str(HERE/'anti-void-patch.py'))['apply'](edit,once)
        assert fragment=={'hook_names':['client_anti_void_network'],'helper_names':['client_anti_void_query','client_anti_void_clear','client_anti_void_raise']}
    if 'native_client_anti_void_network(true'not in read('crates/horizon-client/src/game/remote.rs'):
        runpy.run_path(str(HERE/'anti-void-late-patch.py'))['apply'](edit,once)
    remote=read('crates/horizon-client/src/game/remote.rs');start=remote.index('    fn no_slow_vanilla_send_state(');end=remote.index('    fn creative_stacks(',start);network=remote[start:end]
    assert network.count('native_client_anti_void_network(true')==1 and network.count('native_client_anti_void_network(false')==1
    assert network.index('native_client_anti_void_network(true')<network.index('.update(pos,')<network.index('native_client_anti_void_network(false')
    assert 'if self.riding.is_none(){'in network and 'if !riding {'in network
    assert 'fn anti_void_dead(&self)'in remote and 'self.world.borrow().player.health.dead_or_dying()'in remote
    native=read('crates/horizon-client/src/game/anti_void_native.rs');assert '0..10'in native and 'tick < 40 && !ground'in native
    assert 'without_shape_changes'in native and '.with_min(Axis::Y'in native and 'usize::MAX'in native
    assert 'let mut simulation = NativeSimulation::new'in native and native.count('NativeSimulation::new')==1
    assert 'crate::anti_void::shape(p.y,shape)'in read('crates/horizon-world/src/block_collision_scan.rs')
    assert all('::anti_void::active()'in read(p)for p in ['crates/horizon-client-render/src/collisions.rs','crates/horizon-world/src/registry_collision_scan.rs'])
    applied='pub mod anti_void;'in(ROOT/'native/liquidbounce-behaviors/src/lib.rs').read_text(encoding='utf8')
    plan=None if applied else runpy.run_path(str(HERE/'integrate-anti-void.py'))['build_plan']()
    def shared(name):return(ROOT/name).read_text(encoding='utf8')if applied else plan.text[name]
    generator=shared('native/liquidbounce-horizon/make-behavior-patch.py');assert generator.rindex("'anti-void-patch.py'")<generator.rindex("'step-patch.py'):")
    step=shared('native/liquidbounce-horizon/step-patch.py');assert step.count("'anti-void-late-patch.py'")==1
    assert step.index("edit('crates/horizon-client/src/game/remote.rs',remote)")<step.index("'anti-void-late-patch.py'")
    criticals=shared('native/liquidbounce-horizon/engine/src/criticals_runtime.rs');assert 'self.anti_void.queues(outgoing)'in criticals and 'p.anti_void_dead()'in criticals
    loc=runpy.run_path(str(ROOT/'scripts/update-anti-void-localization.py'))
    for locale,index in [('zh_cn',1),('zh_tw',2)]:
        data=loc['update'](json.loads((loc['FOLDER']/f'display_{locale}.json').read_text(encoding='utf8')),index)
        assert all(data['descriptions'][loc['description_key'](path)]for path in loc['TEXTS'])
        assert all(data['labels'][tag]for tag in loc['LABELS'])
    if plan is not None:
        for name,raw in plan.original.items():assert(ROOT/name).read_bytes()==raw,'Shared file changed during check: '+name
    for path in [HERE/'anti-void-patch.py',HERE/'anti-void-late-patch.py',HERE/'integrate-anti-void.py',ROOT/'scripts/update-anti-void-localization.py',Path(__file__)]:ast.parse(path.read_text(encoding='utf8'))
    transaction='already applied; transaction replay skipped'if applied else f'{len(plan.steps)} shared edits/{len(plan.text)} files'
    print(f'AntiVoid static checks; {len(texts)} native files in memory; {transaction}; {len(loc["TEXTS"])} descriptions; no shared/staging writes')

def run_model_tests():
    with tempfile.TemporaryDirectory(prefix='liquidbounce-antivoid-')as temporary:
        temporary=Path(temporary)
        model=(HERE/'anti-void-smoke.rs').read_text(encoding='utf8').replace('../liquidbounce-behaviors/src/',(ROOT/'native/liquidbounce-behaviors/src').as_posix()+'/')
        source='extern crate self as liquidbounce_behaviors;\n'+model+'\nuse serde_json::{Value,Number,Map,json};\nuse std::{fmt,collections::BTreeMap};\n'
        shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
        start=shared.index('#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum JsonFormat');end=shared.index('#[derive(Clone, Debug, PartialEq)]\npub enum SettingChange',start);source+=shared[start:end]
        source+='enum ModuleKind{AntiVoid} impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.antiVoid"}}\n'
        source+='#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>} enum SettingChange{AntiVoid(anti_void_config::AntiVoidSetting)}\n'
        source+=shared[shared.index('struct Tree<'):shared.index('fn float_range(')]
        source+=shared[shared.index("type Nodes<'a>"):shared.index('pub fn parse_settings_text(')]
        source+='#[path="'+(HERE/'engine/src/config/anti_void.rs').as_posix()+'"]mod anti_void_config;\n'
        source+=(HERE/'anti-void-config-smoke.rs').read_text(encoding='utf8')
        fixture=temporary/'anti-void-smoke.rs';fixture.write_text(source,encoding='utf8');exe=temporary/'anti-void-smoke.exe'
        dependency=max((ROOT/'build/staging/horizon/target/debug/deps').glob('libserde_json-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
        subprocess.run(['rustc','--edition=2024','--test',str(fixture),'-L','dependency='+str(dependency.parent),'--extern','serde_json='+str(dependency),'-o',str(exe)],check=True)
        subprocess.run([str(exe),'anti_void_smoke','--nocapture'],check=True)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group();group.add_argument('--static-only',action='store_true');group.add_argument('--model-only',action='store_true')
    args=parser.parse_args()
    if not args.model_only:static_checks()
    if not args.static_only:run_model_tests()
    else:print('Rust execution deferred to combined batch')

if __name__=='__main__':main()
