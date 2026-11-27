#!/usr/bin/env python3
"""Replay production input tick/Clicks methods and unchanged real binding/click queue sources."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True)
    generator=HERE/'make-behavior-patch.py';ns={'__file__':str(generator),'__name__':'auto_break_consumption'}
    exec(compile(generator.read_text(encoding='utf8').split('destination = HERE /',1)[0],str(generator),'exec'),ns)
    marker='fn native_client_auto_break_key('
    if marker not in ns['changes']['crates/horizon-client/src/game/native_hooks.rs'][1]:
        spec=importlib.util.spec_from_file_location('auto_break_patch',HERE/'auto-break-patch.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        module.apply(ns['edit'],ns['replace_once'])
    snapshots={}
    def write(name,text):
        (out/name).write_text(text,encoding='utf8');snapshots[name]=hashlib.sha256(text.encode()).hexdigest()
    first=ns['changes']['crates/horizon-client/src/first_person.rs'][1]
    # Extract whole production declarations and click implementation, retaining every business method.
    hand=first[first.index('/// `InteractionHand`.'):first.index('/// `LivingEntity.SwingDescription`.')]
    clicks=first[first.index('/// `HitResult` at the crosshair'):first.index('\n#[cfg(test)]\nmod tests')]
    write('first_person.rs',hand+clicks)
    write('first_person-full.rs',first)
    write('interaction_input.rs',ns['changes']['crates/horizon-client-input/src/interaction_input.rs'][1])
    write('keybinds.rs',(ROOT/'upstream/Horizon/crates/horizon-client-input/src/keybinds.rs').read_text(encoding='utf8'))
    write('keymapping_clicks.rs',(HERE/'keymapping-clicks.rs').read_text(encoding='utf8'))
    write('auto_break.rs',(ROOT/'native/liquidbounce-behaviors/src/auto_break.rs').read_text(encoding='utf8'))
    write('consumption.rs',(HERE/'auto-break-consumption-fixture.rs').read_text(encoding='utf8'))
    write('lib.rs','pub mod first_person;\npub mod interaction_input;\npub mod keybinds;\npub mod keymapping_clicks;\npub mod auto_break;\n#[cfg(test)] mod consumption;\n')
    (out/'Cargo.toml').write_text('[package]\nname="auto-break-consumption-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nwinit="0.30.13"\nhorizon-client-core={path="'+str(ROOT/'upstream/Horizon/crates/horizon-client-core').replace('\\','/')+'"}\nhorizon-items={path="'+str(ROOT/'upstream/Horizon/crates/horizon-items').replace('\\','/')+'"}\n',encoding='utf8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
    commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(run,stream),encoding='utf8')
        commands.append({'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'scope':'Complete transformed production InteractionInput source, complete Hand/HandWorld/Hit/UseResult/Destroying/Clicks source declarations and methods extracted from transformed first_person.rs without business method changes, unchanged production KeyBinds and ClickQueues. Real next-tick consumption, original held=true first click, screen/instant short-circuit, mouse grab ordering, item-use draining, unknown key, disable/rebind queued identity, and local mem-taken miss/destroying values. Full native Game helper/canonical ABI and actual game remain integration checks.','snapshotSha256':snapshots,'commands':commands}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
