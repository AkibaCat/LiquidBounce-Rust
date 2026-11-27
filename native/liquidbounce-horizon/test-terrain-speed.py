#!/usr/bin/env python3
"""Execute original Kotlin vectors, exact shared configuration helpers and owned policy."""
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
OUT=ROOT/'build/terrain-speed-tests'
UPSTREAM=ROOT/'upstream/Horizon'

def compose():
    generator=HERE/'make-behavior-patch.py';source=generator.read_text(encoding='utf-8')
    scope={'__file__':str(generator),'__name__':'terrain_speed_composition_audit'}
    exec(compile(source.split('\noutput = []\n',1)[0],str(generator),'exec'),scope)
    if 'client_timer_scale' not in scope['names']:
        runpy.run_path(str(HERE/'timer-patch.py'))['apply'](scope['edit'],scope['replace_once'])
    patch=runpy.run_path(str(HERE/'terrain-speed-patch.py'))
    contract=patch['apply'](scope['edit'],scope['replace_once']) if 'client_terrain_speed' not in scope['names'] else {'alreadyInProduction':True}
    # All eight raw Block.getFriction sites, including the global queries from
    # non-player entities. Entity identity is required to preserve its tick turn.
    coverage={
        'crates/horizon-client/src/motion.rs':1,
        'crates/horizon-client/src/entities.rs':3,
        'crates/horizon-client/src/mobs/living.rs':1,
        'crates/horizon-client/src/mobs/leash.rs':1,
        'crates/horizon-client/src/game/boats.rs':1,
        'crates/horizon-client/src/game/boat_leads.rs':1,
    }
    for name,count in coverage.items():
        assert scope['changes'][name][1].count('terrain_speed_bridge::friction_for(')==count,name
    remote=scope['changes']['crates/horizon-client/src/game/remote.rs'][1]
    start=remote.index('    fn tick_remote_with_motion_modules(')
    assert remote.index('self.native_terrain_finish_player();',start)<remote.index('self.tick_own_rockets();',start)
    player_start=remote.index('    fn native_terrain_tick_player(')
    assert remote.index('self.native_client_player_tick_post();',player_start)<remote.index('Ok(Some(terrain_input))',player_start)
    native=scope['changes']['crates/horizon-client/src/game/native_hooks.rs'][1]
    player_tick=native.index('    pub(super) fn client_timer_player_tick(')
    assert native.index('self.native_terrain_player_tick();',player_tick)<native.index('self.native_client_player_tick_final(cancelled);',player_tick)
    assert 'boat_ground_friction(b.id,bb)' in scope['changes']['crates/horizon-client/src/game/boats.rs'][1]
    piston=scope['changes']['crates/horizon-client/src/game/piston_push.rs'][1]
    start=piston.index('// This optimized piston path bypasses MotionWorld::move_self.')
    assert piston.index('self.terrain_local_move_head(movement)',start)<piston.index('self.limit_piston(target, movement, accumulators)',start)
    paths=[]
    for name,(_,text) in scope['changes'].items():
        if name.endswith('.rs'):
            p=OUT/'composition'/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text,encoding='utf-8');paths.append(str(p))
    syntax=[]
    for offset in range(0,len(paths),48):
        result=subprocess.run(['rustfmt','+stable','--edition','2024','--config','skip_children=true','--emit','files',*paths[offset:offset+48]],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace')
        syntax.append(result.stdout+result.stderr)
        if result.returncode:raise SystemExit(result.stdout+result.stderr)
    (OUT/'composition-syntax.log').write_text(''.join(syntax),encoding='utf-8')
    (OUT/'composition.json').write_text(json.dumps({'contract':contract,'rustSources':len(paths),'globalFrictionSites':coverage,'generatorSha256':hashlib.sha256(source.encode()).hexdigest(),'scope':'all production transforms plus TerrainSpeed;syntax only;full client differential not_run'},indent=2)+'\n',encoding='utf-8')
    return scope['changes']

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    if not (OUT/'oracle.json').is_file():raise SystemExit('Run collect-terrain-speed-facts.py first')
    block=runpy.run_path(str(HERE/'test-attack-patch.py'))['block']
    changes=compose()
    source='''#![cfg(test)]
#![allow(dead_code)]
extern crate self as liquidbounce_behaviors;
extern crate self as horizon_client;
'''
    for module in ['terrain_speed','strafe']:
        source+=f'#[path="{(ROOT/"native/liquidbounce-behaviors/src"/(module+".rs")).as_posix()}"]pub mod {module};\n'
    source+='#[path="tests.rs"]mod tests;\n#[path="config-fixture.rs"]mod config;\n#[path="scheduler-tests.rs"]mod scheduler_tests;\n'
    source+='#[path="ticker-fixture.rs"]mod ticker_fixture;\n'
    source+='#[path="push-fixture.rs"]mod push_fixture;\n'
    source+=block((UPSTREAM/'crates/horizon-client/src/game/boat_passengers.rs').read_text(encoding='utf-8'),'pub(super) fn push_impulse(').replace('pub(super) fn','fn',1)+'\n'
    motion=changes['crates/horizon-client/src/motion.rs'][1]
    native=changes['crates/horizon-client/src/game/native_hooks.rs'][1]
    piston=changes['crates/horizon-client/src/game/piston_push.rs'][1]
    host=(HERE/'terrain-speed-host.rs').read_text(encoding='utf-8')
    native_input=block(native,'    pub(super) fn native_movement_input(')
    replace={
        'BRIDGE':(HERE/'terrain-speed-bridge.rs').as_posix(),
        'ORDER':(HERE/'terrain-speed-order.rs').as_posix(),
        'ADAPTER':(HERE/'terrain-speed-motion.rs').as_posix(),
        'HOST':host+'\n'+native_input+'\n'+block(native,'    pub(super) fn client_timer_player_tick('),
        'STRAFE_MODIFIER':block(motion,'    fn client_strafe_movement('),
        'MOVE_SELF':block(motion,'    pub fn move_self('),
        'SHULKER_PUSH':block(motion,'    pub fn shulker_box_push('),
        'PLAYER_FRICTION':block(motion,'    fn block_friction('),
        'PISTON_HEAD':block(piston,'    fn terrain_local_move_head('),
    }
    fixture=(HERE/'terrain-speed-native-fixture.rs').read_text(encoding='utf-8')
    for name,value in replace.items():fixture=fixture.replace('__'+name+'__',value)
    source+=fixture
    (OUT/'lib.rs').write_text(source,encoding='utf-8')
    (OUT/'tests.rs').write_text((HERE/'terrain-speed-tests.rs').read_text(encoding='utf-8'),encoding='utf-8')
    (OUT/'scheduler-tests.rs').write_text((HERE/'terrain-speed-scheduler-tests.rs').read_text(encoding='utf-8'),encoding='utf-8')
    ticker_fixture=(HERE/'terrain-speed-ticker-fixture.rs').read_text(encoding='utf-8')
    ticker_fixture=ticker_fixture.replace('__RESET__',block((HERE/'terrain-speed-scheduler.rs').read_text(encoding='utf-8'),'    fn native_terrain_reset_passenger_velocity('))
    ticker_fixture=ticker_fixture.replace('__TICKER__',(HERE/'terrain-speed-ticker.rs').read_text(encoding='utf-8').replace('native_terrain_tick_player(Vec3::ZERO)?','native_terrain_tick_player(Vec3::ZERO,false)?'))
    (OUT/'ticker-fixture.rs').write_text(ticker_fixture,encoding='utf-8')
    push_fixture=(HERE/'terrain-speed-push-fixture.rs').read_text(encoding='utf-8')
    push_body=block((HERE/'terrain-speed-scheduler.rs').read_text(encoding='utf-8'),'    fn native_terrain_entity_push(').replace('use horizon_client::mobs','use super::mobs').replace('horizon_client::scoreboard','super::scoreboard')
    push_fixture=push_fixture.replace('__PUSH__',push_body).replace('__BOAT_PUSHES__',block((UPSTREAM/'crates/horizon-client/src/game/boat_passengers.rs').read_text(encoding='utf-8'),'pub(super) fn boat_pushes('))
    (OUT/'push-fixture.rs').write_text(push_fixture,encoding='utf-8')
    shared=(HERE/'engine/src/config.rs').read_text(encoding='utf-8')
    config='''use std::collections::BTreeMap;
use serde_json::{Map,Value,Number,json};
#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat{Interop,File}
enum ModuleKind{TerrainSpeed}impl ModuleKind{fn key(self)->&'static str{"liquidbounce.module.terrainSpeed"}}
'''
    for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_bool(','fn parse_float(','\nfn leaf(']:
        if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
        config+=block(shared,anchor)+'\n'
    config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Debug,PartialEq)]enum SettingChange{TerrainSpeed(terrain::TerrainSpeedSetting)}
#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}
'''
    config+=f'#[path="{(HERE/"engine/src/config/terrain_speed.rs").as_posix()}"]mod terrain;\n'
    config+=(HERE/'terrain-speed-config-tests.rs').read_text(encoding='utf-8')
    (OUT/'config-fixture.rs').write_text(config,encoding='utf-8')
    formatted=subprocess.run(['rustfmt','+stable','--edition','2024','--config','skip_children=true',str(OUT/'lib.rs'),str(OUT/'config-fixture.rs'),str(OUT/'tests.rs')],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace')
    if formatted.returncode:raise SystemExit(formatted.stdout+formatted.stderr)
    api=OUT/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (game.parent/'mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text("impl Game<'_>{\n"+block(native,'    pub(super) fn native_client_terrain_speed(')+'\n'+native_input+'\n}',encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True,exist_ok=True);render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}',encoding='utf-8')
    catalog={'helpers':[],'hooks':[{'name':name,'target':"horizon_client::game::native_hooks::Game<'_>::native_"+name,'mod_api':{'contract':'listen'}} for name in ['client_terrain_speed','movement_input']]}
    (api/'native-hooks.json').write_text(json.dumps(catalog),encoding='utf-8')
    (OUT/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}',encoding='utf-8')
    manifest='[package]\nname="liquidbounce-terrain-speed-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\nlog="0.4"\n'
    for name in ['horizon-core','horizon-world']:manifest+=name+'={path="'+(UPSTREAM/'crates'/name).as_posix()+'"}\n'
    manifest+='[build-dependencies]\nhorizon-mod-api-gen={path="'+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n'
    (OUT/'Cargo.toml').write_text(manifest,encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-entity-interact-tests/target')
    commands=[]
    for label,argv in [('test',['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (OUT/(label+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout+result.stderr,end='')
        commands.append({'argv':argv,'exitCode':result.returncode})
        if result.returncode:break
    files=[Path(__file__),HERE/'terrain-speed-tests.rs',HERE/'terrain-speed-config-tests.rs',HERE/'engine/src/config/terrain_speed.rs',ROOT/'native/liquidbounce-behaviors/src/terrain_speed.rs',*[HERE/name for name in ['terrain-speed-native-fixture.rs','terrain-speed-ticker-fixture.rs','terrain-speed-push-fixture.rs','terrain-speed-motion.rs','terrain-speed-host.rs','terrain-speed-bridge.rs','terrain-speed-patch.py','terrain-speed-order.rs','terrain-speed-order-patch.py','terrain-speed-passenger-patch.py','terrain-speed-scheduler.rs','terrain-speed-scheduler-tests.rs','terrain-speed-ticker.rs','collect-terrain-speed-scheduler.py','test-terrain-speed-native.py','oracles/terrain-speed/SchedulerOracle.java']]]
    report={'passed':len(commands)==2 and all(c['exitCode']==0 for c in commands),'commands':commands,'oracleSha256':hashlib.sha256((OUT/'oracle.json').read_bytes()).hexdigest(),'schedulerSha256':hashlib.sha256((OUT/'scheduler/oracle.json').read_bytes()).hexdigest(),'pushSha256':hashlib.sha256((OUT/'scheduler/pushes.json').read_bytes()).hexdigest(),'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files},'scope':'7040 unchanged original Kotlin events;305 actual ClientLevel scheduler cases;144 actual Entity.push cases;exact shared config helpers;full native client differential not_run'}
    (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    raise SystemExit(0 if report['passed'] else 1)

if __name__=='__main__':main()
