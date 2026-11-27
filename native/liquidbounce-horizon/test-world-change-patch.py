#!/usr/bin/env python3
"""Generated API + actual client-level replacement and unload consumer slices."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import runpy
import shutil
import subprocess
import time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
UPSTREAM=ROOT/'upstream/Horizon'

def block(text,anchor):
    start=text.index(anchor);end=text.index('{',start)+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args()
    out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
    if out.exists():raise SystemExit('Use a fresh evidence directory')
    out.mkdir(parents=True);changed={};upstream_hash={};records=[]
    def once(text,before,after):
        assert text.count(before)==1,(before,text.count(before));return text.replace(before,after,1)
    def edit(path,action):
        source=UPSTREAM/path;upstream_hash[path]=hashlib.sha256(source.read_bytes()).hexdigest()
        changed[path]=action(changed.get(path,source.read_text(encoding='utf-8')))
    extension=runpy.run_path(str(HERE/'timer-patch.py'));added=extension['apply'](edit,once)
    assert 'client_world_change'in added['hook_names']
    remote=changed['crates/horizon-client/src/game/remote.rs'];game=changed['crates/horizon-client/src/game.rs']
    # Copy the exact consumer fragments; the remainder of those methods needs the whole game.
    enter=block(remote,'    fn enter_remote_dimension(')
    same=next(line for line in enter.splitlines()if 'let same = keep_world &&'in line)
    start=enter.index('            // Minecraft.updateLevelInEngines HEAD')
    end=enter.index('            let mut streaming =',start)
    enter_consumer=enter[start:end]
    leave=block(remote,'    pub(super) fn leave_server(')
    early=block(leave,'        if reason.is_none() && self.integrated')
    start=leave.index('        self.native_client_world_change(None);')
    leave_consumer=leave[start:leave.index('        // A world is leaving either way:',start)]
    finish=block(game,'    fn finish_close_world(')
    finish_consumer=finish[:finish.index('        // The save screen has finished')]
    helper=block(extension['HELPERS'],'    pub(super) fn native_client_world_change(')
    native=(UPSTREAM/'crates/horizon-client/src/game/native_hooks.rs').read_text(encoding='utf-8')
    window=block(native,'    pub(super) fn native_window(')
    fixture=(HERE/'world-change-fixture.rs').read_text(encoding='utf-8')
    for marker,value in {'WORLD_HELPER':helper,'WINDOW_HELPER':window,'SAME_LEVEL':same,'ENTER_CONSUMER':enter_consumer,'LEAVE_EARLY':early,'LEAVE_CONSUMER':leave_consumer,'FINISH_CONSUMER':finish_consumer}.items():fixture=fixture.replace('/* '+marker+' */',value)
    (out/'lib.rs').write_text(fixture,encoding='utf-8')
    shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/timer.rs',out/'timer.rs')
    shutil.copyfile(UPSTREAM/'crates/horizon-client/src/game/execution_role.rs',out/'execution_role.rs')
    for path,text in changed.items():
        target=out/'transformed'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text,encoding='utf-8')
    api=out/'api-source';hooks=api/'crates/horizon-client/src/game/native_hooks.rs';hooks.parent.mkdir(parents=True)
    hooks.write_text("impl Game<'_>{\n"+helper+'\n}\n',encoding='utf-8')
    (hooks.parent.parent/'mod_api.rs').write_text('',encoding='utf-8');(api/'tools/native-hook-fragments').mkdir(parents=True)
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True);render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}\n',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'hooks':[{'name':'client_world_change','target':"horizon_client::game::native_hooks::Game<'_>::native_client_world_change",'mod_api':{'contract':'listen'}}],'helpers':[]}),encoding='utf-8')
    (out/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n',encoding='utf-8')
    (out/'Cargo.toml').write_text('[package]\nname="liquidbounce-world-change-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[build-dependencies]\nhorizon-mod-api-gen={path="'+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n',encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target')
    for name,args in [('test',['test']),('clippy',['clippy','--all-targets'])]:
        argv=['cargo','+stable',*args,'--offline','--manifest-path',str(out/'Cargo.toml')]
        if name=='clippy':argv+=['--','-D','warnings']
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (out/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8');records.append({'argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)})
        if result.returncode:print((result.stdout+result.stderr)[-12000:]);break
    files=[HERE/'timer-patch.py',HERE/'world-change-fixture.rs',Path(__file__),ROOT/'native/liquidbounce-behaviors/src/timer.rs',UPSTREAM/'crates/horizon-client/src/game/execution_role.rs',UPSTREAM/'crates/horizon-client/src/game/native_hooks.rs']
    report={'passed':len(records)==2 and all(record['exitCode']==0 for record in records),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'upstreamSha256':upstream_hash,'transformedSha256':{p:hashlib.sha256(t.encode()).hexdigest()for p,t in changed.items()},'scope':['actual same-level predicate, nonnull/null event consumers, remote close delegation and save defer branch','actual ExecutionRole enum and native_window gate','real generated native API and Timer.world_change effect'],'boundaries':['surrounding Game/remote/audio/level shells; whole lifecycle methods are not compiled','Horizon load/unload callbacks follow the new-level event; source HEAD precedes sound/camera/particle/renderer engine updates','runtime event binding, aggregate full build and in-game tests belong to integration owner'],'fullGameIntegrationPassed':False}
    report['upstreamUnchanged']=all(hashlib.sha256((UPSTREAM/p).read_bytes()).hexdigest()==h for p,h in upstream_hash.items());report['passed']&=report['upstreamUnchanged']
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)

if __name__=='__main__':main()
