#!/usr/bin/env python3
"""Unchanged ItemScroller/Chronometer, explicit input/menu/DSL fixture, actual JVM random/time."""
from pathlib import Path
import argparse,hashlib,itertools,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True);fixture=out/'fixture';shutil.copytree(HERE/'item-scroller-source-fixture',fixture,dirs_exist_ok=True)
    upstream=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
    sources=[upstream/'features/module/modules/misc/ModuleItemScroller.kt',upstream/'utils/client/Chronometer.kt']
    for source in sources:shutil.copyfile(source,fixture/source.name)
    inputs=[]
    for running,left,right,mouse in itertools.product([False,True],repeat=4):
        for first,last,elapsed in [(2,3,0),(2,3,10000),(0,0,0),(20,20,10000)]:inputs.append({'running':running,'leftShift':left,'rightShift':right,'leftMouse':mouse,'from':first,'to':last,'elapsed':elapsed})
    for first,last,elapsed in [(1,1,0),(1,1,10000),(0,20,-10000),(0,20,10000),(-2,-1,0),(-2,-1,-10000),(2147483647,2147483647,10000),(-2147483648,-2147483648,0)]:inputs.append({'running':True,'leftShift':True,'rightShift':False,'leftMouse':True,'from':first,'to':last,'elapsed':elapsed})
    assert len(inputs)==72
    (out/'inputs.json').write_text(json.dumps(inputs),encoding='utf8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';libs=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';stdlib=next(libs.glob('kotlin-stdlib-*.jar'));gson=next(libs.glob('gson-*.jar'));classes=out/'classes';classes.mkdir(exist_ok=True)
    commands=[]
    def run(name,argv,env=None):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(result,stream),encoding='utf8')
        commands.append({'argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)})
        return result
    result=run('compile',[str(java),'-cp',str(libs/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',str(stdlib)+';'+str(gson),'-d',str(classes),*map(str,sorted(fixture.glob('*.kt')))])
    if result.returncode==0:result=run('source',[str(java),'--add-opens','java.base/java.lang=ALL-UNNAMED','-cp',str(classes)+';'+str(stdlib)+';'+str(gson),'MainKt',str(out/'inputs.json')])
    if result.returncode==0:
        oracle=json.loads(result.stdout);(out/'source-oracle.json').write_text(json.dumps(oracle),encoding='utf8')
        for source in [ROOT/'native/liquidbounce-behaviors/src/item_scroller.rs',ROOT/'native/liquidbounce-behaviors/src/random.rs',fixture/'oracle.rs']:shutil.copyfile(source,out/source.name)
        (out/'lib.rs').write_text('pub mod item_scroller;\npub mod random;\n#[cfg(test)]mod oracle;\n',encoding='utf8')
        (out/'Cargo.toml').write_text('[package]\nname="item-scroller-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n',encoding='utf8')
        env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
        result=run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')],env)
        if result.returncode==0:run('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    report={'passed':all(c['exitCode']==0 for c in commands),'vectors':len(inputs),'scope':'Byte-for-byte original ModuleItemScroller.kt and complete Chronometer.kt. Minimal input/menu/config DSL/addon fixtures. Actual JDK25 wall clock and ThreadLocalRandom seeded per vector; exact post-query random draw verifies guard and sampling order. QuickMove callback parameters and real reset time bounds. Rust exact elapsed boundary/overflow and no query reset. No game or native consumption claim.','sourceSha256':{source.name:hashlib.sha256(source.read_bytes()).hexdigest() for source in sources},'unchangedSources':all(source.read_bytes()==(fixture/source.name).read_bytes() for source in sources),'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.glob('*.rs')},'commands':commands}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
