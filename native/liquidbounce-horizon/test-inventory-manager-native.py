#!/usr/bin/env python3
"""Independent full Horizon snapshot check of the cumulative native patch."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);parser.add_argument('--prepare-only',action='store_true');args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True);stage=out/'horizon'
    # Execute the cumulative generator only up to its output-writing phase.
    # This reads current production transformations without touching shared patches.
    generator=HERE/'make-behavior-patch.py';code=generator.read_text(encoding='utf-8').split('destination = HERE /',1)[0]
    ns={'__file__':str(generator),'__name__':'derp_native_snapshot'};exec(compile(code,str(generator),'exec'),ns)
    additions=[]
    for name in ['item-identity','auto-weapon','anti-afk','derp','inventory-manager']:
        # The parent may already have composed either extension while this
        # independent verifier was running. Never apply it twice.
        marker_path,marker={ 'item-identity':('crates/horizon-items/src/items.rs','pub struct ItemIdentity'), 'auto-weapon':('crates/horizon-client/src/game/native_hooks.rs','fn native_client_weapon_context('), 'anti-afk':('crates/horizon-client/src/game/native_hooks.rs','fn native_client_anti_afk_state('), 'derp':('crates/horizon-client/src/game/native_hooks.rs','fn native_client_derp_state('), 'inventory-manager':('crates/horizon-client/src/game/native_hooks.rs','fn native_client_inventory_snapshot(')}[name]
        if marker in ns['changes'].get(marker_path,('',''))[1]:continue
        spec=importlib.util.spec_from_file_location(name.replace('-','_'),HERE/(name+'-patch.py'));module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        additions.append(module.apply(ns['edit'],ns['replace_once']))
    changes=ns['changes'];fragment='tools/native-hook-fragments/liquidbounce_behavior.json'
    data=json.loads(changes[fragment][1])
    for added in additions:
        data['hooks'].extend({'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}",'side':'client','cancellable':False,'mod_api':{'contract':'listen'}} for name in added['hook_names'])
        data['helpers'].extend({'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in added['helper_names'])
    changes[fragment]=('',json.dumps(data,indent=2)+'\n')
    shutil.copytree(ROOT/'upstream/Horizon',stage,dirs_exist_ok=True,ignore=shutil.ignore_patterns('target','.git','node_modules'))
    for path,(_,text) in changes.items():
        destination=stage/path;destination.parent.mkdir(parents=True,exist_ok=True);destination.write_text(text,encoding='utf-8',newline='\n')
    gen=out/'generator';gen.mkdir(exist_ok=True)
    (gen/'Cargo.toml').write_text('[package]\nname="derp-full-api-gen"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="generate"\npath="main.rs"\n[dependencies]\nhorizon-mod-api-gen={path="../horizon/crates/horizon-mod-api-gen"}\n',encoding='utf-8')
    (gen/'main.rs').write_text('fn main(){horizon_mod_api_gen::fill_hooks(&std::path::PathBuf::from(std::env::args_os().nth(1).unwrap())).unwrap();}\n',encoding='utf-8')
    snapshot={path:hashlib.sha256(text.encode()).hexdigest() for path,(_,text) in changes.items()}
    (out/'source-snapshot.json').write_text(json.dumps(snapshot,indent=2)+'\n',encoding='utf-8')
    if args.prepare_only:print(stage);return
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/auto-weapon-native-target')
    commands=[]
    for name,argv,cwd in [('generate',['cargo','+stable','run','--manifest-path',str(gen/'Cargo.toml'),'--',str(stage)],ROOT),('check',['cargo','+stable','check','-p','horizon-client','--lib'],stage)]:
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:
            run=subprocess.run(argv,cwd=cwd,env=env,stdout=stdout,stderr=stderr)
        commands.append({'argv':argv,'exitCode':run.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'scope':'Full independent pinned Horizon source workspace, cumulative existing behavior patches and generic inventory native snapshot/operations, real GUI identities, real verified menus, creative helpers, item feature table and between-frame timer wake; real fill_hooks/generate and complete horizon-client library cargo check. Shared staging untouched. Runtime module registration and real game execution are separate.','commands':commands,'snapshotSha256':snapshot}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
