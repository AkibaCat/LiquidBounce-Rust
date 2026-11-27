#!/usr/bin/env python3
"""Independent full cumulative Horizon typecheck, including synchronous HighJump."""
import argparse,hashlib,json,os,runpy,shutil,subprocess,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);args=p.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    stage=out/'horizon';out.mkdir(parents=True,exist_ok=True);generator=HERE/'make-behavior-patch.py';source=generator.read_text(encoding='utf-8')
    ns={'__file__':str(generator),'__name__':'high_jump_full_native'};exec(compile(source.split('\noutput = []\n',1)[0],str(generator),'exec'),ns)
    additions=[]
    for extension,member,kind in [('timer','client_timer_scale','names'),('terrain-speed','client_terrain_speed','names'),('high-jump','client_high_jump_state','helper_names')]:
        if member not in ns[kind]:additions.append(runpy.run_path(str(HERE/(extension+'-patch.py')))['apply'](ns['edit'],ns['replace_once']))
    changes=ns['changes'];fragment='tools/native-hook-fragments/liquidbounce_behavior.json';data=json.loads(changes[fragment][1])
    for addition in additions:
        data['hooks'].extend(dict(name=n,target=f"horizon_client::game::native_hooks::Game<'_>::native_{n}",side='client',cancellable=False,mod_api={'contract':'listen'})for n in addition['hook_names'])
        data['helpers'].extend(dict(name=n,target=f"horizon_client::game::native_hooks::Game<'_>::native_{n}")for n in addition['helper_names'])
    changes[fragment]=('',json.dumps(data,indent=2)+'\n')
    shutil.copytree(ROOT/'upstream/Horizon',stage,dirs_exist_ok=True,ignore=shutil.ignore_patterns('target','.git','node_modules','.local'))
    for path,(_,content)in changes.items():dest=stage/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(content,encoding='utf-8',newline='\n')
    gen=out/'generator';gen.mkdir(exist_ok=True)
    (gen/'Cargo.toml').write_text('[package]\nname="high-jump-full-api-gen"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="generate"\npath="main.rs"\n[dependencies]\nhorizon-mod-api-gen={path="../horizon/crates/horizon-mod-api-gen"}\n',encoding='utf-8')
    (gen/'main.rs').write_text('fn main(){horizon_mod_api_gen::fill_hooks(&std::path::PathBuf::from(std::env::args_os().nth(1).unwrap())).unwrap();}\n',encoding='utf-8')
    snapshots={path:hashlib.sha256(content.encode()).hexdigest()for path,(_,content)in changes.items()};(out/'source-snapshot.json').write_text(json.dumps(snapshots,indent=2)+'\n',encoding='utf-8')
    env=os.environ.copy();commands=[]
    for name,argv,cwd,target in [('generate',['cargo','+stable','run','--offline','--manifest-path',str(gen/'Cargo.toml'),'--',str(stage)],ROOT,'build/no-entity-interact-tests/target'),('check',['cargo','+stable','check','--offline','-p','horizon-client','--lib'],stage,'build/auto-weapon-native-target')]:
        env['CARGO_TARGET_DIR']=str(ROOT/target);started=time.monotonic()
        with(out/(name+'.stdout.log')).open('w',encoding='utf-8')as stdout,(out/(name+'.stderr.log')).open('w',encoding='utf-8')as stderr:r=subprocess.run(argv,cwd=cwd,env=env,stdout=stdout,stderr=stderr)
        commands.append(dict(name=name,argv=argv,exitCode=r.returncode,seconds=round(time.monotonic()-started,3)))
        if r.returncode:break
    report=dict(passed=len(commands)==2 and all(c['exitCode']==0 for c in commands),commands=commands,snapshotSha256=snapshots,scope='Full pinned Horizon cumulative transforms plus TerrainSpeed and HighJump; actual fill_hooks and horizon-client --lib cargo check. Runtime registration and full client differential remain separate.')
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(passed=report['passed'],report=str(out/'report.json'))));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
