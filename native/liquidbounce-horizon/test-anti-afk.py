#!/usr/bin/env python3
"""Actual production AntiAFK, rotation settings and JDK25 random vectors."""
from pathlib import Path
import argparse,hashlib,json,os,subprocess,time
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='build/anti-afk-01');args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    vectors=subprocess.run([str(java),'--add-opens','java.base/java.lang=ALL-UNNAMED',str(HERE/'AntiAfkRandomReference.java')],capture_output=True,text=True,check=True).stdout
    (out/'java-random-vectors.json').write_text(vectors,encoding='utf-8')
    snapshot={}
    for name in ['anti_afk','random','rotations','no_rotate_set']:
        source=ROOT/f'native/liquidbounce-behaviors/src/{name}.rs';(out/f'{name}.rs').write_bytes(source.read_bytes());snapshot[name]=hashlib.sha256(source.read_bytes()).hexdigest()
    (out/'lib.rs').write_text('pub mod anti_afk;\npub mod random;\npub mod rotations;\npub mod no_rotate_set;\n',encoding='utf-8')
    (out/'Cargo.toml').write_text('[package]\nname="anti-afk-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n',encoding='utf-8')
    env=os.environ.copy();env.setdefault('CARGO_TARGET_DIR',str(HERE/'packet-tests/target'));commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(run,stream),encoding='utf-8')
        commands.append({'argv':argv,'exitCode':run.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'scope':'Actual production complete AntiAFK state machine, rotation processor settings, ThreadLocalRandom; 256 live JDK25 mixed-sequence oracle vectors and Rust lifecycle/timing tests. Native and config integration separate.','commands':commands,'snapshotSha256':snapshot}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
