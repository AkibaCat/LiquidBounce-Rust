#!/usr/bin/env python3
"""Source-derived AutoRespawn and actual actor/transport seam evidence."""
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
    start=text.index(signature)
    end=text.index('{',start)+1
    depth=1
    while depth:
        depth+=(text[end]=='{')-(text[end]=='}')
        end+=1
    return text[start:end]


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--out',default='build/auto-respawn-01')
    parser.add_argument('--prepare-only',action='store_true')
    args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    out.mkdir(parents=True,exist_ok=True)
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite existing verification evidence')
    changes,originals={},{}
    def edit(name,transform):
        path=ROOT/'upstream/Horizon'/name
        originals[name]=hashlib.sha256(path.read_bytes()).hexdigest()
        changes[name]=transform(changes.get(name,path.read_text(encoding='utf-8')))
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'Expected one anchor: {before[:100]}')
        return text.replace(before,after,1)
    for name in ['mining','packet','respawn']:
        spec=importlib.util.spec_from_file_location(name+'_patch',HERE/(name+'-patch.py'))
        patch=importlib.util.module_from_spec(spec)
        spec.loader.exec_module(patch)
        patch.apply(edit,once)
    game='crates/horizon-client/src/game/'
    native=changes[game+'native_hooks.rs']
    remote=changes[game+'remote.rs']
    spawn=changes[game+'player_spawn.rs']
    screens=(ROOT/'upstream/Horizon'/game/'screens.rs').read_text(encoding='utf-8')
    network=changes['crates/horizon-client/src/net.rs']
    source=(HERE/'packet-tests/respawn-fixture.rs').read_text(encoding='utf-8')
    blocks={
        'HELPERS':'\n'.join(function(native,signature) for signature in ['    pub(super) fn native_client_death_screen_opening(','    pub(super) fn native_client_is_death_screen(','    pub(super) fn native_client_respawn(']),
        'RESPAWN':function(screens,'    pub(super) fn respawn_player('),
        'REMOTE_RESPAWN':function(remote,'    pub(super) fn remote_respawn('),
        'GAME_SEND':function(remote,'    pub(super) fn send(&self,'),
        'HEALTH_OPEN':function(remote,'                if p.health <= 0.0 && shown'),
        'KILL_OPEN':function(remote,'                if me && self.screen != Screen::Dead'),
        'SAVED_OPEN':spawn[spawn.index('        self.player = Some(player);'):spawn.index('        self.input_player_replaced();')],
        'SESSION_METHODS':'\n'.join(function(network,signature) for signature in ['    pub fn send(&self,','    pub fn send_silently(&self,','    fn send_with_path(&self,']),
        'IDENTITY_METHOD':function(changes['crates/horizon-client/src/motion.rs'],'    pub fn native_client_queries_allowed('),
    }
    for name,body in blocks.items():source=source.replace('/* '+name+' */',body)
    source=source.replace('PACKET_BRIDGE_PATH',(HERE/'packet-bridge.rs').as_posix()).replace('PACKETS_PATH',(HERE/'engine/src/packets.rs').as_posix())
    (out/'src').mkdir(exist_ok=True)
    (out/'src/lib.rs').write_text(source,encoding='utf-8',newline='\n')
    manifest='[package]\nname="liquidbounce-respawn-fixture"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
    for name,path in [('horizon-protocol',ROOT/'upstream/Horizon/crates/horizon-protocol'),('liquidbounce-behaviors',ROOT/'native/liquidbounce-behaviors')]:manifest+=f'{name}={{path="{path.as_posix()}"}}\n'
    (out/'Cargo.toml').write_text(manifest,encoding='utf-8',newline='\n')
    if args.prepare_only:print(out/'Cargo.toml');return
    paths=[HERE/'respawn-patch.py',HERE/'test-respawn-patch.py',HERE/'packet-tests/respawn-fixture.rs',ROOT/'native/liquidbounce-behaviors/src/auto_respawn.rs']
    hashes={str(path.relative_to(ROOT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    env=os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR',str(HERE/'packet-tests/target'))
    commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic()
        result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(result,stream),encoding='utf-8')
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3),'stdout':str(out/f'{name}.stdout.log'),'stderr':str(out/f'{name}.stderr.log')})
        if result.returncode:break
    originals[game+'screens.rs']=hashlib.sha256((ROOT/'upstream/Horizon'/game/'screens.rs').read_bytes()).hexdigest()
    report={'passed':all(command['exitCode']==0 for command in commands),'scope':'Production AutoRespawn; exact transformed death-screen opening/identity/query/respawn, actual respawn_player/remote_respawn/Game.send/Session.send; real framed Horizon Pipe codec; UI/world plumbing stubbed','commands':commands,'sourceSha256':hashes,'extractedBlocksSha256':{name:hashlib.sha256(body.encode()).hexdigest() for name,body in blocks.items()},'upstreamSourceSha256':originals,'sourceUnchangedDuringVerification':all(hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest for path,digest in hashes.items()),'upstreamUnchanged':all(hashlib.sha256((ROOT/'upstream/Horizon'/path).read_bytes()).hexdigest()==digest for path,digest in originals.items()),'fullClientCompilationPassed':False,'fullGameIntegrationPassed':False}
    report['passed']&=report['sourceUnchangedDuringVerification'] and report['upstreamUnchanged']
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')},indent=2))


if __name__=='__main__':main()
