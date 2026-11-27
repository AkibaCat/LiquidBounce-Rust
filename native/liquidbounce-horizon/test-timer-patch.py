#!/usr/bin/env python3
"""Unchanged Kotlin Timer/RequestHandler oracle and patched native clock consumers."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import runpy
import shutil
import struct
import subprocess
import time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
UPSTREAM=ROOT/'upstream/Horizon'
def block(text,anchor):
    start=text.index(anchor);end=text.index('{',start)+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]
def bits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='build/timer-01');args=parser.parse_args()
    out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
    if out.exists():raise SystemExit('Choose a fresh evidence directory')
    out.mkdir(parents=True);records=[]
    def run(name,argv,env=None):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (out/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
        records.append({'argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)})
        if result.returncode:print((result.stdout+result.stderr)[-15000:])
        return result
    source=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/client'
    originals=[source/'Timer.kt',source/'RequestHandler.kt',source/'BalanceReset.kt',ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/world/ModuleTimer.kt',ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/event/CoroutineTicker.kt']
    for path in originals:shutil.copyfile(path,out/path.name)
    for path in (HERE/'timer-source-fixture').glob('*.kt'):shutil.copyfile(path,out/path.name)
    commands=[]
    def add(*items):commands.extend(items)
    for speed in [-2.0,-0.0,0.0,0.1,0.5,1.0,1.3,2.0,20.0,float('inf'),float('-inf'),float('nan')]:
        add('world',f'request 1 20 1 {bits(speed)}',f'scale {bits(2.75)}','frame 5 0','playerTick false','playerTick true','tick 1',f'scale {bits(0.5)}','tick 1',f'scale {bits(1)}')
    # Equal priorities expose PBQ offer/remove/bulkRemove behavior, not FIFO.
    for index in range(35):add(f'request {index+100} 20 {index%7} {bits(float(index))}')
    for index in range(35):add(f'provider {index+100} false')
    add('world',f'request 1 20 10 {bits(0.5)}',f'scale {bits(2)}','hold 9 3','frame 2 0','frame 2 0','frame 2 1','playerTick false','burst 5 -10 20','frame 1 0','playerTick false','frame 1 1','playerTick true','frame 1 2','world','within 5 -5','within 16777217 16777216')
    rng=random.Random(2603)
    for index in range(4000):
        operation=rng.randrange(13);provider=rng.randrange(1,22)
        if operation==0:add(f'request {provider} {rng.choice([-2147483648,-20,0,20,100,2147483647])} {rng.choice([-2147483648,-3,0,1,2,7,2147483647])} {bits(rng.choice([0.1,0.6,1.3,2.0,-1.0,0.0]))}')
        elif operation==1:add(f'provider {provider} {rng.choice(["true","false"])}')
        elif operation==2:add(f'tick {rng.randrange(4)}')
        elif operation==3:add(f'scale {bits(rng.uniform(0,8))}')
        elif operation==4:add(f'frame {rng.randrange(11)} 0')
        elif operation==5:add(f'playerTick {rng.choice(["true","false"])}')
        elif operation==6:add(f'hold {provider} {rng.choice([-1,0,1,2,6])}')
        elif operation==7:add(f'burst {rng.randrange(-2,8)} {rng.randrange(-20,1)} {rng.randrange(0,21)}')
        elif operation==8:add(f'within {rng.randrange(-20,1)} {rng.randrange(0,21)}')
        elif operation==9:add(f'balanced {provider} 20 1 {bits(rng.choice([0.6,1.0,1.3,2.0]))} {rng.randrange(-20,1)} {rng.randrange(0,21)}')
        elif operation==10:add(f'same {rng.choice(["true","false"])}')
        elif operation==11:add('world')
        else:add('reset')
    for provider in [*range(1,22),*range(100,135)]:add(f'provider {provider} false')
    add('same true','player true','tick 20','world','mode Classic','module true','context true false none','tick 1',f'scale {bits(1)}','playerTick false')
    add(f'set Classic Speed {bits(-3)}','tick 1',f'scale {bits(2)}','module false',f'scale {bits(1)}','module true','mode Pulse')
    add('set Pulse NormalSpeedTicks 3','set Pulse BoostSpeedTicks 2','set Pulse OnMove true','context false false none','tick 1',f'scale {bits(1)}','context true false none')
    for index in range(25):add('tick 1',f'scale {bits(1)}','playerTick false',f'context {str(index%4!=0).lower()} false none')
    add('set Pulse NormalSpeedTicks 0','set Pulse BoostSpeedTicks -2')
    for _ in range(10):add('tick 1',f'scale {bits(1)}')
    add('mode Boost','module true','world','context true false none','tick 1',f'scale {bits(1)}','context false false none','tick 1',f'scale {bits(1)}','playerTick false')
    for index in range(80):
        add(f'context {str(index%3==0).lower()} {str(index%11==0).lower()} {rng.choice(["none","inventory","container","other"])}','tick 1',f'scale {bits(1)}','frame 1 0','playerTick false')
        if index%7==0:add('flag')
    add('set Boost Balance 5 -5','tick 1','set Boost Balance -10 20','set Boost ResetBalanceOnFlag false','playerTick true','flag','module false','flag')
    (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
    kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    jars=[next(kotlin.glob(pattern))for pattern in ['kotlin-stdlib-*.jar','kotlinx-coroutines-core-jvm-*.jar','slf4j-api-*.jar']]
    jars.append(next((ROOT/'build/resource-runtime/libraries').rglob('fastutil-8.5.18.jar')))
    classpath=os.pathsep.join(map(str,jars));java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    result=run('kotlin-compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',classpath,'-d',str(out/'classes'),*[str(p)for p in sorted(out.glob('*.kt'))]])
    if result.returncode==0:result=run('kotlin-oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),classpath]),'fixture.MainKt'])
    changed={};upstream_hash={}
    def edit(path,action):
        source=UPSTREAM/path
        if source.exists():upstream_hash[path]=hashlib.sha256(source.read_bytes()).hexdigest()
        changed[path]=action(changed.get(path,source.read_text(encoding='utf-8')if source.exists()else''))
    def once(text,before,after):
        assert text.count(before)==1,(before,text.count(before));return text.replace(before,after,1)
    extension=runpy.run_path(str(HERE/'timer-patch.py'));added=extension['apply'](edit,once)
    for name in ['packet','hand-derp']:
        extra=runpy.run_path(str(HERE/(name+'-patch.py')))['apply'](edit,once)
        if name=='packet':changed.update(extra)
    for name,path in [('timer.rs',ROOT/'native/liquidbounce-behaviors/src/timer.rs'),('oracle.rs',HERE/'timer-source-fixture/oracle.rs')]:shutil.copyfile(path,out/name)
    (out/'clock.rs').write_text(changed['crates/horizon-client/src/clock.rs'],encoding='utf-8')
    shutil.copyfile(UPSTREAM/'crates/horizon-client-render/src/java_text.rs',out/'java_text.rs')
    for name,path in {'timer_runtime.rs':HERE/'engine/src/timer_runtime.rs','combat_window.rs':ROOT/'native/liquidbounce-behaviors/src/combat_window.rs','no_swing.rs':ROOT/'native/liquidbounce-behaviors/src/no_swing.rs','packets.rs':HERE/'engine/src/packets.rs'}.items():shutil.copyfile(path,out/name)
    ticker=(HERE/'engine/src/coroutine_ticker.rs').read_text(encoding='utf-8')
    if '    Timer,'not in ticker:ticker=ticker.replace('    InventoryManager,','    InventoryManager,\n    Timer,')
    (out/'coroutine_ticker.rs').write_text(ticker,encoding='utf-8');(out/'packet_bridge.rs').write_text(changed['crates/horizon-client/src/packet_bridge.rs'],encoding='utf-8')
    fixture=(HERE/'timer-fixture.rs').read_text(encoding='utf-8')
    game=changed['crates/horizon-client/src/game.rs'];start=game.index('                    let timer_frame = ');end=game.index('                    self.extras_frame(elapsed);',start)
    remote=changed['crates/horizon-client/src/game/remote.rs']
    player_body=block(remote,'    fn tick_remote_level(')
    a=player_body.index('        let client_player_cancelled = ');b=player_body.index('        // ClientLevel.tick: the border',a)
    cooldown=player_body[a:b]
    a=player_body.index('        if !hold {\n            self.tick_client_ambient();');b=player_body.index('        // Gui.tick:',a)
    effects=player_body[a:b]
    a=player_body.index('        if client_player_cancelled {');b=player_body.index('        let stick = self.input_move();',a)
    cancellation=player_body[a:b]
    # These are the production consumers, not a reimplementation of cancellation.
    fixture=fixture.replace('/* PLAYER_CONSUMERS */',cooldown+effects+cancellation)
    fixture=fixture.replace('/* SLEEP_TIMER */',block((UPSTREAM/'crates/horizon-client/src/game.rs').read_text(encoding='utf-8'),'fn tick_sleep_timer('))
    fixture=fixture.replace('/* FRAME_LOOP */',game[start:end]).replace('/* HELPERS */',extension['HELPERS']).replace('/* SNAPSHOT */',extension['SNAPSHOT_HELPER'])
    (out/'lib.rs').write_text(fixture,encoding='utf-8')
    config=(HERE/'engine/src/config.rs').read_text(encoding='utf-8')
    wrapper='#![allow(dead_code)]\nuse serde_json::{Value,Map,Number,json};use std::collections::BTreeMap;\n'
    for declaration in ['pub enum JsonFormat','pub struct CommonSettings','pub struct ConfigError']:
        wrapper+='#[derive(Clone,Copy,Debug,PartialEq,Eq)]\n'if declaration.endswith('JsonFormat')else'#[derive(Clone,Debug,PartialEq)]\n'
        wrapper+=block(config,declaration)+'\n'
    wrapper+='enum ModuleKind {Timer}impl ModuleKind {fn key(self)->&\'static str {"liquidbounce.module.timer"}}\n'
    for declaration in ['impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn float_json(','fn scalar_string(',"fn nodes<'a>(","fn value_of<'a>(",'fn parse_bool(','fn parse_float(','fn parse_int_pair(','fn parse_int(','fn decimal_int(',"fn pair_members<'a>(", '\nfn leaf(']:wrapper+=block(config,declaration)+'\n'
    wrapper+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;#[derive(PartialEq,Debug)]enum SettingChange{Timer(child::TimerSetting),Enabled(bool)}#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}\n'
    wrapper+='#[path="config_child.rs"]mod child;\nfn parse_settings(_:ModuleKind,value:&Value)->Result<SettingsPatch,ConfigError>{let root=nodes(value,"Timer","Timer")?;let mut patch=SettingsPatch::default();child::parse_timer(&mut patch,&root);Ok(patch)}\n'
    (out/'config.rs').write_text(wrapper,encoding='utf-8');shutil.copyfile(HERE/'engine/src/config/timer.rs',out/'config_child.rs')
    api=out/'api-source';hooks=api/'crates/horizon-client/src/game/native_hooks.rs';hooks.parent.mkdir(parents=True)
    hooks.write_text("impl Game<'_>{\n"+extension['HELPERS']+extension['SNAPSHOT_HELPER']+'}\n',encoding='utf-8')
    (hooks.parent.parent/'mod_api.rs').write_text('',encoding='utf-8');(api/'tools/native-hook-fragments').mkdir(parents=True)
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True);render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}\n',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'hooks':[{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}",'mod_api':{'contract':'listen'}}for name in added['hook_names']],'helpers':[{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}"}for name in added['helper_names']]}),encoding='utf-8')
    (out/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n',encoding='utf-8')
    manifest='[features]\nwebview=[]\n[package]\nname="liquidbounce-timer-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nlog="0.4"\nserde_json={version="1",features=["arbitrary_precision"]}\nhorizon-protocol={path="'+(UPSTREAM/'crates/horizon-protocol').as_posix()+'"}\n[build-dependencies]\nhorizon-mod-api-gen={path="'+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n'
    (out/'Cargo.toml').write_text(manifest,encoding='utf-8')
    if result.returncode==0:
        env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target')
        result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],env)
        if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    files=[*originals,HERE/'timer-patch.py',HERE/'timer-fixture.rs',Path(__file__),ROOT/'native/liquidbounce-behaviors/src/timer.rs',HERE/'engine/src/timer_runtime.rs',HERE/'engine/src/config/timer.rs',HERE/'engine/src/config.rs',HERE/'engine/src/coroutine_ticker.rs',HERE/'engine/src/packets.rs',HERE/'packet-patch.py',HERE/'hand-derp-patch.py',ROOT/'native/liquidbounce-behaviors/src/combat_window.rs',ROOT/'native/liquidbounce-behaviors/src/no_swing.rs',UPSTREAM/'crates/horizon-client-render/src/java_text.rs',*sorted((HERE/'timer-source-fixture').glob('*'))]
    report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'commands':records,'cases':len(commands),'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files if p.is_file()},'upstreamSha256':upstream_hash,'transformedSha256':{p:hashlib.sha256(t.encode()).hexdigest()for p,t in changed.items()},'scope':['unchanged five Kotlin source files and actual Kotlin coroutine/queue arithmetic','whole patched Horizon clock and actual game frame loop through generated native API','production TimerService, packet pipeline, configuration child and copied parent codecs','extracted production remote player cancellation consumers and chunk-ready predicate'],'boundaries':['minimal host, world and module lifecycle shells','remote tests execute extracted consumers, not the whole tick_remote_level method','aggregate patch order, full game compilation and in-game smoke remain integration-owner evidence'],'fullGameIntegrationPassed':False}
    report['upstreamUnchanged']=all(hashlib.sha256((UPSTREAM/p).read_bytes()).hexdigest()==h for p,h in upstream_hash.items());report['passed']&=report['upstreamUnchanged']
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
