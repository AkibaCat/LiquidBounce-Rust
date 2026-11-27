#!/usr/bin/env python3
"""Source-derived JVM score vectors and actual Rust AutoWeapon behavior."""
from pathlib import Path
import argparse, hashlib, json, os, subprocess, time
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--out',default='build/auto-weapon-01')
    args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists(): raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    vectors=subprocess.run([str(java),str(HERE/'AutoWeaponReference.java')],capture_output=True,text=True,check=True).stdout
    (out/'java-vectors.json').write_text(vectors,encoding='utf-8')
    source=ROOT/'native/liquidbounce-behaviors/src/auto_weapon.rs'
    (out/'auto_weapon.rs').write_bytes(source.read_bytes())
    (out/'lib.rs').write_text('pub mod auto_weapon;\n',encoding='utf-8')
    (out/'Cargo.toml').write_text('[package]\nname="auto-weapon-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n',encoding='utf-8')
    env=os.environ.copy();env.setdefault('CARGO_TARGET_DIR',str(HERE/'packet-tests/target'))
    commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(run,stream),encoding='utf-8')
        commands.append({'argv':argv,'exitCode':run.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(command['exitCode']==0 for command in commands),'scope':'Actual production AutoWeapon source; 384 source-derived JVM arithmetic vectors, Rust facet-order/priority/lifecycle/settings tests. Native item/attack glue is outside this fixture.','commands':commands,'sourceSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'javaOracleSha256':hashlib.sha256((HERE/'AutoWeaponReference.java').read_bytes()).hexdigest()}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
