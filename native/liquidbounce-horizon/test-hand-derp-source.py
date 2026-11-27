#!/usr/bin/env python3
"""Unmodified ModuleHandDerp.kt with real Kotlin suspension against Rust."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='build/hand-derp-source-01');args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True);shutil.copytree(HERE/'hand-derp-source-fixture',out/'fixture',dirs_exist_ok=True)
    source=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/fun/ModuleHandDerp.kt';shutil.copyfile(source,out/'fixture/ModuleHandDerp.kt')
    commands='''actual 0
enabled true
tick
delay 20
tick
tick
delay 0
mode Delay
tick
mode Swing
punch false
punch true
silent true
metadata 999 17 0
metadata -1 1
actual 1
metadata 999 17 0
metadata 19 1 17
noSwing true
punch false
noSwing false
enabled false
enabled true
punch false
inGame false
punch false
mode Delay
delay -2147483648
tick
enabled false
inGame true
tick
enabled true
tick
tick
delay 0
tick
tick
actual 0
enabled false
enabled true
tick
mode Swing
punch false
enabled false
actual 1
enabled true
punch false
enabled false
'''
    for delay in [-5,-1,0,1,2,3,20,2147483647]:
        commands+=f'mode Swing\nmode Delay\ndelay {delay}\nenabled true\n'+'tick\n'*24+'enabled false\n'
    (out/'commands.txt').write_text(commands,encoding='utf-8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';stdlib=next(kotlin.glob('kotlin-stdlib-*.jar'));records=[]
    def run(name,argv,env=None):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(result,stream),encoding='utf-8')
        records.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)});return result
    result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',str(stdlib),'-d',str(out/'classes')]+[str(path) for path in sorted((out/'fixture').glob('*.kt'))])
    rows=0
    if result.returncode==0:
        for initial in [0,1]:
            result=run(f'kotlin-{initial}',[str(java),'-cp',os.pathsep.join([str(out/'classes'),str(stdlib)]),'MainKt',str(out/'commands.txt'),str(initial)])
            if result.returncode:break
            data=json.loads(result.stdout);rows+=len(data['rows']);(out/f'source-vectors-{initial}.json').write_text(json.dumps(data),encoding='utf-8')
    if result.returncode==0:
        shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/hand_derp.rs',out/'hand_derp.rs');shutil.copyfile(HERE/'hand-derp-source-fixture/oracle.rs',out/'oracle.rs')
        (out/'lib.rs').write_text('pub mod hand_derp;\n#[cfg(test)]mod oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="hand-derp-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n',encoding='utf-8')
        env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');result=run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')],env)
        if result.returncode==0:run('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    report={'passed':all(record['exitCode']==0 for record in records),'commands':records,'rows':rows,'unchangedSource':source.read_bytes()==(out/'fixture/ModuleHandDerp.kt').read_bytes(),'upstreamModuleSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'behaviorSha256':hashlib.sha256((ROOT/'native/liquidbounce-behaviors/src/hand_derp.rs').read_bytes()).hexdigest(),'scope':'Byte-for-byte pinned ModuleHandDerp.kt with minimal Minecraft/event/settings stubs and real Kotlin continuations, replayed against actual Rust behavior. Both initial hands, all modes, live/captured/raw delay, same/changed mode, enabled/world guards, NoSwing-before-event, cancelled Punch, Silent unusual metadata predicate, disable and real-option changes. Packet encoding/full game hosting tested separately.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'rows':rows,'report':str(out/'report.json')}))
if __name__=='__main__':main()
