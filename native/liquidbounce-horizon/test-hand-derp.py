#!/usr/bin/env python3
"""Production HandDerp pure state, original packet listener and real framed Pipe."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent

def snapshot_changes():
    generator=HERE/'make-behavior-patch.py'
    ns={'__file__':str(generator),'__name__':'hand_derp_snapshot'}
    exec(compile(generator.read_text(encoding='utf-8').split('destination = HERE /',1)[0],str(generator),'exec'),ns)
    path='crates/horizon-client/src/game/native_hooks.rs'
    if 'fn native_client_hand_derp_hand(' not in ns['changes'][path][1]:
        spec=importlib.util.spec_from_file_location('hand_derp_patch',HERE/'hand-derp-patch.py');patch=importlib.util.module_from_spec(spec);spec.loader.exec_module(patch)
        added=patch.apply(ns['edit'],ns['replace_once'])
        fragment='tools/native-hook-fragments/liquidbounce_behavior.json'
        data=json.loads(ns['changes'][fragment][1])
        data['helpers'].extend({'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in added['helper_names'])
        ns['changes'][fragment]=('',json.dumps(data,indent=2)+'\n')
    return ns['changes']

def function(text,signature):
    start=text.index(signature);end=text.index('{',start)+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]

def behaviors(out):
    shutil.copytree(ROOT/'native/liquidbounce-behaviors',out/'behaviors',dirs_exist_ok=True,ignore=shutil.ignore_patterns('target'))
    lib=out/'behaviors/src/lib.rs';text=lib.read_text(encoding='utf-8')
    if 'pub mod hand_derp;' not in text:lib.write_text(text+'\npub mod hand_derp;\n',encoding='utf-8')

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='build/hand-derp-01');parser.add_argument('--suite',choices=['packets','config','native'],default='packets');parser.add_argument('--prepare-only',action='store_true');args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True);commands=[];changes={}
    if args.suite=='native':
        changes=snapshot_changes();stage=out/'horizon';shutil.copytree(ROOT/'upstream/Horizon',stage,dirs_exist_ok=True,ignore=shutil.ignore_patterns('target','.git','node_modules'))
        for path,(_,text) in changes.items():
            destination=stage/path;destination.parent.mkdir(parents=True,exist_ok=True);destination.write_text(text,encoding='utf-8',newline='\n')
        gen=out/'generator';gen.mkdir(exist_ok=True)
        (gen/'Cargo.toml').write_text('[package]\nname="hand-derp-api-gen"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="generate"\npath="main.rs"\n[dependencies]\nhorizon-mod-api-gen={path="../horizon/crates/horizon-mod-api-gen"}\n',encoding='utf-8')
        (gen/'main.rs').write_text('fn main(){horizon_mod_api_gen::fill_hooks(&std::path::PathBuf::from(std::env::args_os().nth(1).unwrap())).unwrap();}\n',encoding='utf-8')
        target=ROOT/'build/auto-weapon-native-target'
        commands=[('generate',['cargo','+stable','run','--manifest-path',str(gen/'Cargo.toml'),'--',str(stage)],ROOT),('check',['cargo','+stable','check','-p','horizon-client','--lib'],stage)]
    else:
        behaviors(out)
        if args.suite=='packets':
            changes=snapshot_changes()
            for name,path in [('packet_bridge','crates/horizon-client/src/packet_bridge.rs'),('hand_derp_bridge','crates/horizon-client/src/hand_derp_bridge.rs')]:
                (out/f'{name}.rs').write_text(changes[path][1],encoding='utf-8')
            for name,path in [('packets','engine/src/packets.rs'),('hand_derp_packets','engine/src/hand_derp_packets.rs'),('slot_bridge','slot-bridge.rs')]:shutil.copyfile(HERE/path,out/f'{name}.rs')
            network=changes['crates/horizon-client/src/net.rs'][1]
            methods='\n'.join(function(network,signature) for signature in ['    pub fn send(&self,','    pub fn send_silently(&self,','    fn send_with_path(&self,'])
            (out/'transport.rs').write_text((HERE/'hand-derp-packet-fixture.rs').read_text(encoding='utf-8').replace('/* SESSION_METHODS */',methods),encoding='utf-8')
            source='extern crate self as horizon_client;\npub mod mod_api { pub use horizon_protocol::packets::play::{Clientbound as ClientboundPlayPacket,Serverbound as ServerboundPlayPacket}; }\npub mod packet_bridge;\npub mod hand_derp_bridge;\npub mod packets;\npub mod hand_derp_packets;\npub mod slot_bridge;\n#[cfg(test)] mod transport;\n'
            shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/hand_derp.rs',out/'hand_derp.rs')
            source+='pub mod hand_derp;\n'
            (out/'lib.rs').write_text(source,encoding='utf-8')
            deps=f'liquidbounce-behaviors={{path="behaviors"}}\nhorizon-protocol={{path="{(ROOT/"upstream/Horizon/crates/horizon-protocol").as_posix()}"}}\nserde_json="1"\n'
            testargs=['--','--test-threads=1']
        else:
            config=(HERE/'engine/src/config.rs').read_text(encoding='utf-8')
            if 'mod hand_derp;' not in config:
                def once(before,after):
                    nonlocal config
                    if config.count(before)!=1:raise ValueError((before,config.count(before)))
                    config=config.replace(before,after,1)
                once('mod derp;','mod derp;\n#[path="config/hand_derp.rs"]\nmod hand_derp;\npub use hand_derp::{HandDerpSetting,serialize_hand_derp};')
                once('    Derp => ("Derp", "Fun", "derp"),','    Derp => ("Derp", "Fun", "derp"),\n    HandDerp => ("HandDerp", "Fun", "handDerp"),')
                if config.count('    Derp(DerpSetting),')!=2:raise ValueError('SettingChange/RuntimeRequest anchors')
                config=config.replace('    Derp(DerpSetting),','    Derp(DerpSetting),\n    HandDerp(HandDerpSetting),')
                once('        ModuleKind::Derp => derp::parse_derp(&mut patch, &root),','        ModuleKind::Derp => derp::parse_derp(&mut patch, &root),\n        ModuleKind::HandDerp => hand_derp::parse_hand_derp(&mut patch, &root),')
                once('        SettingChange::Derp(value) => {','        SettingChange::HandDerp(value) => { return ApplyResult::Runtime(RuntimeRequest::HandDerp(*value)); }\n        SettingChange::Derp(value) => {')
            (out/'lib.rs').write_text(config,encoding='utf-8');shutil.copytree(HERE/'engine/src/config',out/'config',dirs_exist_ok=True)
            deps='liquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n';testargs=[]
        (out/'Cargo.toml').write_text(f'[package]\nname="hand-derp-{args.suite}-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\n'+deps,encoding='utf-8')
        target=HERE/('packet-tests/target' if args.suite=='packets' else 'config-tests/target')
        commands=[('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]+testargs,ROOT),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],ROOT)]
    if args.prepare_only:print(out);return
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(target);recorded=[]
    for name,argv,cwd in commands:
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:result=subprocess.run(argv,cwd=cwd,env=env,stdout=stdout,stderr=stderr)
        recorded.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if result.returncode:break
    snapshot={str(path.relative_to(out)):hashlib.sha256(path.read_bytes()).hexdigest() for path in out.rglob('*.rs') if 'target' not in path.parts}
    report={'passed':all(command['exitCode']==0 for command in recorded),'suite':args.suite,'commands':recorded,'snapshotSha256':snapshot,'scope':{'packets':'Actual production HandDerp pure state, original listener, options bridge, cumulative scoped PacketEvent sender, exact patched Session send bodies and real framed Pipe codec. No full game simulation.','config':'Whole production configuration and behaviors with explicit HandDerp registration-only shim when absent. Complete modes/inactive settings, raw imports, lifecycle and RuntimeRequest forwarding.','native':'Independent complete pinned Horizon source, cumulative exact-source patches including HandDerp; real fill_hooks API generation and complete horizon-client --lib check. Shared staging untouched; game execution separate.'}[args.suite]}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
