#!/usr/bin/env python3
"""NoRotateSet source/JVM/native phase verification, without touching upstream."""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import os
import subprocess
import time

ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent

def function(text,signature):
    first=text.index(signature)
    end=text.index('{',first)+1
    depth=1
    while depth:
        depth+=(text[end]=='{')-(text[end]=='}')
        end+=1
    return text[first:end]

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--out',default='build/no-rotate-01')
    parser.add_argument('--prepare-only',action='store_true')
    args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    out.mkdir(parents=True,exist_ok=True)
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite existing evidence')
    changes,originals,extras={},{},{}
    def edit(name,transform):
        path=ROOT/'upstream/Horizon'/name
        originals[name]=hashlib.sha256(path.read_bytes()).hexdigest()
        changes[name]=transform(changes.get(name,path.read_text(encoding='utf-8')))
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'Expected one anchor: {before[:90]}')
        return text.replace(before,after,1)
    for name in ['mining','packet','pitch','no-rotate']:
        spec=importlib.util.spec_from_file_location(name.replace('-','_')+'_patch',HERE/(name+'-patch.py'))
        patch=importlib.util.module_from_spec(spec)
        spec.loader.exec_module(patch)
        extras.update(patch.apply(edit,once) or {})
    fragment=json.loads(extras['tools/native-hook-fragments/liquidbounce_no_rotate.json'])
    assert isinstance(fragment['hooks'],list) and isinstance(fragment['helpers'],list)
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib/kotlin-stdlib-2.4.21.jar'
    oracle=subprocess.run([str(java),'--class-path',str(kotlin),str(HERE/'NoRotateReference.java')],capture_output=True,text=True,check=True)
    vectors=json.loads(oracle.stdout)
    (out/'java-vectors.json').write_text(json.dumps(vectors,indent=2)+'\n',encoding='utf-8')
    game='crates/horizon-client/src/game/'
    native=changes[game+'native_hooks.rs']
    remote=changes[game+'remote.rs']
    network=changes['crates/horizon-client/src/net.rs']
    blocks={
        'HELPERS':'\n'.join(function(native,signature) for signature in ['    pub(super) fn native_client_no_rotate_capture(', '    pub(super) fn native_no_rotate_previous(', '    pub(super) fn native_client_server_rotation(', '    pub(super) fn native_no_rotate_apply(', '    pub(super) fn native_client_player_last_rotation(', '    pub(super) fn native_client_mouse_rotation(', '    pub(super) fn native_client_change_look(', '    pub(super) fn native_change_look_now(']),
        'REMOTE':'\n'.join([function(remote,'    fn remote_teleport('),function(remote,'    pub(super) fn send(&self,'),function(changes[game+'remote_abilities.rs'],'    pub(super) fn remote_rotation('),function((ROOT/'upstream/Horizon'/game/'perspective.rs').read_text(encoding='utf-8'),'    pub(super) fn snap_player_model_pitch(')]),
        'SESSION':'\n'.join(function(network,signature) for signature in ['    pub fn send(&self,','    pub fn send_silently(&self,','    fn send_with_path(&self,']),
        'POSITION_PITCH':function(remote,'fn position_packet_pitch('),
        'IDENTITY':function(changes['crates/horizon-client/src/motion.rs'],'    pub fn native_client_queries_allowed('),
    }
    assert [hook['name'] for hook in fragment['hooks']]==['client_no_rotate_capture','client_server_rotation','client_mouse_rotation','client_change_look']
    pointer=changes[game+'pointer_capture.rs']
    assert pointer.index('self.native_client_mouse_rotation(xo * zoom, yo * zoom);') < pointer.index('self.turn_player(xo * zoom, yo * zoom);')
    assert 'self.native_client_mouse_rotation' not in function(pointer,'    pub(super) fn look_turn(')
    frame=changes['crates/horizon-client/src/game.rs']
    assert frame.count('self.native_change_look_now(camera_alpha as f32);')==1
    assert frame.index('self.native_change_look_now(camera_alpha as f32);') < frame.index('let vanilla = self.camera_view(camera_alpha as f32);')
    source=(HERE/'packet-tests/no-rotate-fixture.rs').read_text(encoding='utf-8')
    for name,body in blocks.items():source=source.replace('/* '+name+' */',body)
    paths={'ROTATIONS_PATH':ROOT/'native/liquidbounce-behaviors/src/rotations.rs','RANDOM_PATH':ROOT/'native/liquidbounce-behaviors/src/random.rs','BEHAVIOR_PATH':ROOT/'native/liquidbounce-behaviors/src/no_rotate_set.rs','PACKET_BRIDGE_PATH':HERE/'packet-bridge.rs','PLAYER_SYNC_PATH':ROOT/'upstream/Horizon/crates/horizon-client/src/player_sync.rs'}
    for name in sorted(paths,key=len,reverse=True):source=source.replace(name,paths[name].as_posix())
    (out/'lib.rs').write_text(source,encoding='utf-8',newline='\n')
    # Use the actual API signature parser and compile production dispatches
    # against its output, rather than accepting handwritten dispatcher shapes.
    api=out/'api-source'
    api_game=api/'crates/horizon-client/src/game'
    api_game.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('',encoding='utf-8')
    (api_game/'native_hooks.rs').write_text("impl Game<'_> {\n"+blocks['HELPERS']+'\n}\n',encoding='utf-8')
    (api/'native-hooks.json').write_text(json.dumps({'hooks':fragment['hooks'],'helpers':[]}),encoding='utf-8')
    (out/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate_test_hooks(&root,&out).unwrap();}\n',encoding='utf-8')
    manifest='[package]\nname="liquidbounce-no-rotate-tests"\nversion="0.0.0"\nedition="2024"\n[lib]\npath="lib.rs"\n[workspace]\n[dependencies]\nserde_json="1"\n'
    for name,path in [('horizon-core',ROOT/'upstream/Horizon/crates/horizon-core'),('horizon-protocol',ROOT/'upstream/Horizon/crates/horizon-protocol')]:manifest+=f'{name}={{path="{path.as_posix()}"}}\n'
    manifest+=f'[build-dependencies]\nhorizon-mod-api-gen={{path="{(ROOT/"upstream/Horizon/crates/horizon-mod-api-gen").as_posix()}"}}\n'
    (out/'Cargo.toml').write_text(manifest,encoding='utf-8',newline='\n')
    if args.prepare_only:print(out/'Cargo.toml');return
    files=[HERE/'no-rotate-patch.py',HERE/'test-no-rotate-patch.py',HERE/'NoRotateReference.java',HERE/'packet-tests/no-rotate-fixture.rs',*paths.values()]
    hashes={str(path.relative_to(ROOT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in files}
    env=os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR',str(HERE/'packet-tests/target'))
    commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic()
        run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(run,stream),encoding='utf-8')
        commands.append({'argv':argv,'exitCode':run.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(command['exitCode']==0 for command in commands),'scope':'Production NoRotateSet decisions and three smoothers against actual JDK/Kotlin arithmetic; exact native correction capture/post/old-angle writes, ChangeLook and mouse event dispatch compiled against actual horizon_mod_api_gen output; actual remote teleport/rotation bodies, Horizon player_sync and framed Pipe transport. Actor execution and peripheral world services are explicit stubs.','commands':commands,'sourceSha256':hashes,'upstreamSha256':originals,'extractedBlocksSha256':{name:hashlib.sha256(body.encode()).hexdigest() for name,body in blocks.items()},'productionExtrasSha256':{name:hashlib.sha256((value if isinstance(value,str) else json.dumps(value,sort_keys=True)).encode()).hexdigest() for name,value in extras.items()},'oracle':{'source':'NoRotateReference.java, actual JDK RandomGenerator.nextFloat(origin,bound) and Kotlin primitive/ClosedFloatingPointRange coerceIn','kotlinSha256':hashlib.sha256(kotlin.read_bytes()).hexdigest()},'fullClientCompilationPassed':False,'fullGameIntegrationPassed':False}
    report['sourceUnchangedDuringVerification']=all(hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest for name,digest in hashes.items())
    report['upstreamUnchanged']=all(hashlib.sha256((ROOT/'upstream/Horizon'/name).read_bytes()).hexdigest()==digest for name,digest in originals.items())
    report['passed']&=report['sourceUnchangedDuringVerification'] and report['upstreamUnchanged']
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')},indent=2))

if __name__=='__main__':main()
