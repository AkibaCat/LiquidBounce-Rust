#!/usr/bin/env python3
"""Production generic InventoryManager deterministic state/constraint fixtures."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);parser.add_argument('--source-oracle',default='build/inventory-manager-source-02');args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    for name in ['inventory_manager','vomit','random']:shutil.copyfile(ROOT/f'native/liquidbounce-behaviors/src/{name}.rs',out/f'{name}.rs')
    shutil.copyfile(HERE/'inventory-manager-tests.rs',out/'tests.rs')
    source_oracle=ROOT/args.source_oracle
    if source_oracle.joinpath('source-vectors.json').is_file():
        shutil.copy(source_oracle/'source-vectors.json',out/'source-vectors.json');shutil.copy(source_oracle/'commands.json',out/'commands.json')
        shutil.copy(HERE/'inventory-manager-source-fixture/oracle.rs',out/'oracle.rs')
        lib='pub mod inventory_manager;\npub mod vomit;\npub mod random;\n#[cfg(test)]mod tests;\n#[cfg(test)]mod oracle;\n'
    else:
        lib='pub mod inventory_manager;\npub mod vomit;\npub mod random;\n#[cfg(test)]mod tests;\n'
    (out/'lib.rs').write_text(lib,encoding='utf-8')
    (out/'Cargo.toml').write_text('[package]\nname="inventory-manager-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dev-dependencies]\nserde_json="1"\n',encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/inventory-manager-target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:result=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if result.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'commands':commands,'sourceOracle':source_oracle.joinpath('source-vectors.json').is_file(),'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*.rs')},'scope':'Actual production generic manager with deterministic native-operation fixture plus an unchanged Kotlin InventoryManager coroutine oracle comparison when source-vectors.json is available. Covers pending priorities/expected inventory guards, full active chains after owner stop, precise milliseconds/live delays, cursor recovery tick wait identities, RNG accounting, real resync timeouts/rejections and raw range errors.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
