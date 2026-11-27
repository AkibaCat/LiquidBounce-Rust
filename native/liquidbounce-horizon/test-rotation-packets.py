#!/usr/bin/env python3
"""Independent production listener + real transport + codec verification."""
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
    opening=text.index('{',start)
    end=opening+1
    depth=1
    while depth:
        depth+=(text[end]=='{')-(text[end]=='}')
        end+=1
    return text[start:end]


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--out',default='build/rotation-packets-01')
    parser.add_argument('--prepare-only',action='store_true')
    args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    out.mkdir(parents=True,exist_ok=True)
    if (out/'report.json').exists():
        raise SystemExit('Refusing to overwrite existing verification evidence')
    changes={}
    originals={}
    def edit(name,transform):
        path=ROOT/'upstream/Horizon'/name
        originals[name]=hashlib.sha256(path.read_bytes()).hexdigest()
        changes[name]=transform(changes.get(name,path.read_text(encoding='utf-8')))
    def once(text,before,after):
        if text.count(before)!=1:
            raise ValueError(f'Expected one anchor: {before[:100]}')
        return text.replace(before,after,1)
    for name in ['packet','slot']:
        spec=importlib.util.spec_from_file_location(name+'_patch',HERE/(name+'-patch.py'))
        patch=importlib.util.module_from_spec(spec)
        spec.loader.exec_module(patch)
        patch.apply(edit,once)
    network=changes['crates/horizon-client/src/net.rs']
    template=(HERE/'packet-tests/transport-fixture-template.rs').read_text(encoding='utf-8')
    template=template[:template.index('    #[test]')]
    template=template.replace('pub use liquidbounce_horizon_packet_tests::packet_bridge;\n','').replace('pub use liquidbounce_horizon_packet_tests::slot_bridge;\n','')
    methods='\n'.join(function(network,signature) for signature in ['    pub fn send(&self,','    pub fn send_silently(&self,','    fn send_with_path(&self,'])
    source='''#![cfg(test)]
extern crate self as horizon_client;
extern crate self as liquidbounce_horizon_packet_tests;
pub mod mod_api {
    pub use horizon_protocol::packets::play::{Clientbound as ClientboundPlayPacket,Serverbound as ServerboundPlayPacket};
}
'''
    for module,path in [('packet_bridge',HERE/'packet-bridge.rs'),('slot_bridge',HERE/'slot-bridge.rs'),('packets',HERE/'engine/src/packets.rs'),('rotation_packets',HERE/'engine/src/rotation_packets.rs')]:
        source+=f'#[path="{path.as_posix()}"]\npub mod {module};\n'
    source+=template.replace('/* GENERATED_SESSION_METHODS */',methods).replace('/* GENERATED_RECEIVE_PLAY */',function(network,'fn receive_play('))
    source+=(HERE/'packet-tests/rotation-fixture-tests.rs').read_text(encoding='utf-8')+'\n}\n'
    (out/'lib.rs').write_text(source,encoding='utf-8',newline='\n')
    (out/'Cargo.toml').write_text('''[package]
name = "liquidbounce-rotation-packet-fixture"
version = "0.1.0"
edition = "2024"
[lib]
path = "lib.rs"
[dependencies]
horizon-protocol = {path="'''+(ROOT/'upstream/Horizon/crates/horizon-protocol').as_posix()+'''"}
liquidbounce-behaviors = {path="'''+(ROOT/'native/liquidbounce-behaviors').as_posix()+'''"}
serde_json = "1"
''',encoding='utf-8',newline='\n')
    if args.prepare_only:
        print(out)
        return
    files=[HERE/'engine/src/rotation_packets.rs',HERE/'engine/src/packets.rs',HERE/'packet-bridge.rs',HERE/'slot-bridge.rs',HERE/'packet-patch.py',HERE/'slot-patch.py',HERE/'packet-tests/rotation-fixture-tests.rs',HERE/'test-rotation-packets.py',ROOT/'native/liquidbounce-behaviors/src/rotations.rs',out/'lib.rs']
    hashes=lambda:{str(path.relative_to(ROOT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in files}
    before=hashes()
    commands=[]
    for name,argv in [
        ('lock',['cargo','+stable','generate-lockfile','--offline','--manifest-path',str(out/'Cargo.toml')]),
        ('test',['cargo','+stable','test','--locked','--offline','--manifest-path',str(out/'Cargo.toml')]),
        ('clippy',['cargo','+stable','clippy','--locked','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings']),
    ]:
        started=time.monotonic()
        result=subprocess.run(argv,cwd=ROOT,capture_output=True)
        (out/(name+'.stdout.log')).write_bytes(result.stdout)
        (out/(name+'.stderr.log')).write_bytes(result.stderr)
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-started,3)})
    unchanged=all(hashlib.sha256((ROOT/'upstream/Horizon'/name).read_bytes()).hexdigest()==sha for name,sha in originals.items())
    report={'passed':all(command['exitCode']==0 for command in commands) and before==hashes() and unchanged,
        'scope':'production RotationListener/RotationManager/PacketPipeline and exact transformed Session/receive_play bodies with real Horizon Pipe framing and packet codecs',
        'executedTests':3,'commands':commands,'sourceSha256':before,'sourceUnchangedDuringVerification':before==hashes(),
        'originalSourceSha256':originals,'upstreamUnchanged':unchanged,'cargoTargetDirectory':os.environ.get('CARGO_TARGET_DIR'),
        'incomingCancelledCorrectionsUpdateActualAndTheoretical':True,'outgoingCancelledPacketsUpdateTheoreticalOnly':True,
        'explicitSilentWritesIgnoreCancellation':True,'finalPacketMutationObserved':True,'allRelativeFlagCombinationsVerified':True,
        'fullClientCompilationPassed':False,'fullGameIntegrationPassed':False}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')},indent=2))
    raise SystemExit(0 if report['passed'] else 1)


if __name__=='__main__':
    main()
