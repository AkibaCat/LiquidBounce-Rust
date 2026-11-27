#!/usr/bin/env python3
"""Compile exact transformed camera/input bodies, generated API and Java float oracle."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess
import time

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
UPSTREAM=ROOT/'upstream/Horizon'


def block(text,anchor):
    start=text.index(anchor);end=text.index('{',start)+1;depth=1
    while depth:
        depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='build/free-look-01')
    args=parser.parse_args();out=(ROOT/args.out).resolve();out.mkdir(parents=True,exist_ok=True)
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite existing evidence')
    changed={};originals={}
    def edit(name,fn):
        path=UPSTREAM/name
        if path.exists():originals[name]=hashlib.sha256(path.read_bytes()).hexdigest()
        changed[name]=fn(changed.get(name,path.read_text(encoding='utf-8') if path.exists() else ''))
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'Expected one anchor, found {text.count(before)}: {before[:100]!r}')
        return text.replace(before,after,1)
    contracts=[]
    for name in ['mining','packet','pitch','no-rotate','camera-control','camera-clip','free-look']:
        contracts.append(runpy.run_path(str(HERE/(name+'-patch.py')))['apply'](edit,once))
    for name in ['camera','mouse']:
        source=changed['crates/horizon-client-render/src/camera.rs'] if name=='camera' else (UPSTREAM/'crates/horizon-client-input/src/mouse.rs').read_text(encoding='utf-8')
        (out/(name+'.rs')).write_text(source.split('\n#[cfg(test)]',1)[0],encoding='utf-8')
    policy=runpy.run_path(str(HERE/'camera-control-patch.py'))['policy_source']()
    (out/'camera_control.rs').write_text(policy,encoding='utf-8')
    (out/'free_look.rs').write_text((ROOT/'native/liquidbounce-behaviors/src/free_look.rs').read_text(encoding='utf-8'),encoding='utf-8')
    native=changed['crates/horizon-client/src/game/native_hooks.rs']
    methods='\n'.join(block(native,'    pub(super) fn '+name+'(') for name in ['native_client_free_look_camera','native_client_mouse_rotation'])
    native_impl="impl Game<'_> {\n"+methods+'\n}\n'
    pointer=changed['crates/horizon-client/src/game/pointer_capture.rs']
    source=(HERE/'free-look-fixture.rs').read_text(encoding='utf-8')
    source=source.replace('/* NATIVE */',native_impl)
    source=source.replace('/* POINTER */','\n'.join(block(pointer,'    pub(super) fn '+name+'(') for name in ['apply_mouse_look','look_turn']))
    assert 'native_client_mouse_rotation' not in block(pointer,'    pub(super) fn look_turn(')
    assert pointer.index('self.native_client_mouse_rotation(xo * zoom, yo * zoom, &mut canceled)')<pointer.index('if !canceled { self.turn_player')
    perspective=changed['crates/horizon-client/src/game/perspective.rs']
    assert perspective.count('self.native_client_free_look_camera(')==1
    assert perspective.index('self.native_client_free_look_camera(')<perspective.index('let mut rays = VisualRays')
    assert '            free_look_rotation,\n            |from, to|' in perspective
    api=out/'api-source';game=api/'crates/horizon-client/src/game';game.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('',encoding='utf-8')
    (game/'native_hooks.rs').write_text(native_impl,encoding='utf-8')
    render=api/'crates/horizon-client-render/src/gfx/native_render/generated.rs';render.parent.mkdir(parents=True,exist_ok=True)
    render.write_text('use super::*;pub trait RenderHooks:Send+Sync{}\n',encoding='utf-8')
    hooks=['client_free_look_camera','client_mouse_rotation']
    (api/'native-hooks.json').write_text(json.dumps({'helpers':[],'hooks':[{'name':name,
        'target':"horizon_client::game::native_hooks::Game<'_>::native_"+name,
        'mod_api':{'contract':'listen'}} for name in hooks]},indent=2)+'\n',encoding='utf-8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    rows=[]
    for x,y in [(0.,0.),(1.2345678912345,-456.78912345),(16777217.,-16777219.),(0.000000013579,-0.00000002468),(-3.5,1000.)]:
        for boost in [-2.5,0.,0.1,1.,1.7,4.5]:
            for limited in [False,True]:rows.append((x,y,boost,limited))
    java_source='''public class FreeLookOracle {
      public static void row(double x,double y,float boost,boolean limited) {
        float yaw=17.25f,pitch=-21.75f;
        float dx=(float)x*0.15f,dy=(float)y*0.15f;
        yaw+=dx*boost;pitch+=dy*boost;
        if(limited) {if(pitch < -90f)pitch=-90f;else if(pitch > 90f)pitch=90f;}
        System.out.println(Integer.toUnsignedString(Float.floatToRawIntBits(yaw))+","+Integer.toUnsignedString(Float.floatToRawIntBits(pitch)));
      }
      public static void main(String[] args) {
'''
    for x,y,boost,limited in rows:java_source+=f'row({x},{y},{boost}f,{str(limited).lower()});\n'
    java_source+='}}\n';(out/'FreeLookOracle.java').write_text(java_source,encoding='utf-8')
    oracle=subprocess.run([str(java),str(out/'FreeLookOracle.java')],capture_output=True,text=True,check=True)
    bits=[[int(v) for v in row.split(',')] for row in oracle.stdout.splitlines()]
    (out/'java-oracle.json').write_text(json.dumps({'rows':rows,'expectedFloatBits':bits},indent=2)+'\n',encoding='utf-8')
    assertions=''
    for index,((x,y,boost,limited),expected) in enumerate(zip(rows,bits)):
        assertions+=f'''{{let mut module=free_look::FreeLook {{sense_boost:{boost}f32,no_pitch_limit:{str(not limited).lower()},..Default::default()}};
        module.on_enabled([17.25,-21.75]);module.mouse_rotation(true,{x}f64,{y}f64);
        assert_eq!([module.camera_yaw.to_bits(),module.camera_pitch.to_bits()],{expected},"Java oracle row {index}");}}\n'''
    source=source.replace('/* ORACLE */',assertions);(out/'lib.rs').write_text(source,encoding='utf-8')
    (out/'build.rs').write_text('''fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate(&root,&out,horizon_mod_api_gen::Target::Client).unwrap();}\n''',encoding='utf-8')
    (out/'Cargo.toml').write_text('''[package]
name="liquidbounce-free-look-tests"
version="0.0.0"
edition="2024"
[lib]
path="lib.rs"
[workspace]
[dependencies]
glam="0.30"
horizon-core={path="'''+(UPSTREAM/'crates/horizon-core').as_posix()+'''"}
[build-dependencies]
horizon-mod-api-gen={path="'''+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n',encoding='utf-8')
    env=os.environ.copy();env.setdefault('CARGO_TARGET_DIR',str(ROOT/'build/no-pose-hook-tests/target'))
    commands=[]
    for name,argv in [('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1']),
        ('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (out/(name+'.log')).write_text(run.stdout+run.stderr,encoding='utf-8');print(run.stdout+run.stderr)
        commands.append({'argv':argv,'exitCode':run.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    files=[HERE/'free-look-patch.py',HERE/'free-look-fixture.rs',HERE/'test-free-look-patch.py',ROOT/'native/liquidbounce-behaviors/src/free_look.rs']
    report={'passed':all(row['exitCode']==0 for row in commands),'commands':commands,
        'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files},
        'upstreamSha256':originals,'transformedSha256':{p:hashlib.sha256(t.encode()).hexdigest() for p,t in changed.items()},
        'productionBodies':['complete Camera module','complete MouseLook/Rotation input module','Game::apply_mouse_look','Game::look_turn','native_client_mouse_rotation','native_client_free_look_camera'],
        'apiGenerator':'horizon_mod_api_gen::generate(Target::Client), generated events/dispatch/Host compiled',
        'javaOracleRows':len(rows),'oracleScope':'Source-derived Java modern RotationUtil/FreeLook Float arithmetic, not original Kotlin execution',
        'boundaries':['Game/window/player identity shell','tutorial callback and zoom factor shell','engine registry/lifecycle/config are parent integration'],
        'fullGameIntegrationPassed':False}
    report['upstreamUnchanged']=all(hashlib.sha256((UPSTREAM/p).read_bytes()).hexdigest()==digest for p,digest in originals.items())
    report['passed']&=report['upstreamUnchanged']
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')},indent=2))
    raise SystemExit(0 if report['passed'] else 1)


if __name__=='__main__':main()
