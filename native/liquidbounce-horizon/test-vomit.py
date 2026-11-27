#!/usr/bin/env python3
"""Pinned ModuleVomit planning oracle; execution-layer checks are separate."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True);commands=[];fixture=out/'fixture';shutil.copytree(HERE/'vomit-source-fixture',fixture,dirs_exist_ok=True)
    source=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/fun/ModuleVomit.kt';shutil.copyfile(source,fixture/'ModuleVomit.kt')
    rows=[]
    # All combinations include empty/full inventory, container fallback, each
    # storage slot, both creative paths and raw signed delay endpoints.
    masks=[0,(1<<36)-1]+[1<<i for i in range(36)]+[((1<<36)-1)^(1<<i) for i in range(36)]+[0x123456789,0x5a5a5a5a5]
    for creative in ['false','true']:
        for first,last in [(100,200),(0,0),(-2147483648,-1),(200,0),(0,1)]:
            for mask in masks:
                for sync,count,slots in [(-1,-1,'-'),(31,90,'-'),(27,63,'0,5,9,26')]:
                    rows.append(f'{creative} {first} {last} {mask} {sync} {count} {slots} 1153')
    rows+=['true 0 0 0 -1 -1 - 0','true 1 1 0 -1 -1 - 1','true 1 1 0 -1 -1 - 1073741825']*100
    (out/'commands.txt').write_text('\n'.join(rows)+'\n',encoding='utf-8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';minecraft=Path('C:/Users/AkibaCat/.gradle/caches/fabric-loom/26.3/minecraft-client.jar')
    classpath=os.pathsep.join(str(p) for p in [minecraft]+[next(kotlin.glob(name)) for name in ['kotlin-stdlib-*.jar','kotlinx-coroutines-core-jvm-*.jar','guava-*.jar','failureaccess-*.jar','slf4j-api-*.jar']])
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
    def run(name,argv):
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:result=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)});return result.returncode==0
    okay=run('kotlin-compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',classpath,'-d',str(out/'classes')]+[str(p) for p in sorted(fixture.glob('*.kt'))])
    if okay:okay=run('kotlin-source',[str(java),'--add-opens','java.base/java.lang=ALL-UNNAMED','-cp',os.pathsep.join([str(out/'classes'),classpath]),'MainKt',str(out/'commands.txt')])
    if okay:
        data=json.loads((out/'kotlin-source.stdout.log').read_text());(out/'source-vectors.json').write_text(json.dumps(data),encoding='utf-8')
        for name in ['vomit','random']:shutil.copyfile(ROOT/f'native/liquidbounce-behaviors/src/{name}.rs',out/f'{name}.rs')
        shutil.copyfile(fixture/'oracle.rs',out/'oracle.rs');(out/'lib.rs').write_text('pub mod vomit;\npub mod random;\n#[cfg(test)]mod oracle;\n',encoding='utf-8')
        (out/'Cargo.toml').write_text('[package]\nname="vomit-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n',encoding='utf-8')
        okay=run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')])
        if okay:okay=run('clippy',['cargo','+stable','clippy','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])
    report={'passed':okay,'rows':len(rows),'sourceSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'unchanged':source.read_bytes()==(fixture/'ModuleVomit.kt').read_bytes(),'commands':commands,'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*.rs')},'scope':'Unmodified ModuleVomit with real Minecraft 26.3 RandomSource/LegacyRandomSource, real Kotlin Random.Default and coroutines; registry/menu/inventoryPlan stubs record the exact requested chain. Production Rust plan order and both independent RNG states compared per command. This does not validate InventoryManager execution, constraints scheduling or actual game effects.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':okay,'rows':len(rows),'report':str(out/'report.json')}))
if __name__=='__main__':main()
