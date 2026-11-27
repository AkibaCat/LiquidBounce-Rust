#!/usr/bin/env python3
"""Compile unchanged pinned ModuleDerp.kt and compare every tick with Rust.

The fixture replaces the unavailable Minecraft/event environment with minimal
stubs. Kotlin's real suspend continuations, the unedited module source, and
actual JDK25 ThreadLocalRandom execute. Hidden smoothing has separate tests.
"""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='build/derp-source-01');args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    fixture=HERE/'derp-source-fixture';shutil.copytree(fixture,out/'fixture',dirs_exist_ok=True)
    source=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/fun/ModuleDerp.kt'
    shutil.copyfile(source,out/'fixture/ModuleDerp.kt')
    commands='''yaw Jitter
pitch Static
tick 10 0 false false
intYaw Jitter ForwardTicks 99
intYaw Jitter BackwardTicks 3
tick 20 1 false false
yaw Jitter
tick 30 2 false false
intYaw Jitter BackwardTicks 99
tick 40 3 false false
tick 50 4 false false
intYaw Jitter ForwardTicks 1
tick 60 5 false false
tick 70 6 true false
tick 80 7 false true
tick 90 8 false false
yaw Spin
tick 0 0 false false
intYaw Spin Speed -70
tick 0 0 true false
tick 0 0 false false
intYaw Spin Speed 2147483647
tick 0 0 false false
enabled false
tick 0 0 false false
enabled true
tick 0 0 false false
yaw Jitter
intYaw Jitter ForwardTicks -10
intYaw Jitter BackwardTicks 0
tick 5 0 false false
intYaw Jitter BackwardTicks 1
tick 6 0 false false
intYaw Jitter BackwardTicks -8
tick 7 0 false false
tick 8 0 false false
bool NotDuringSprint false
yaw Spin
intYaw Spin Speed 1000
tick 50 60 true true
yaw Static
floatYaw Static Yaw 500
floatPitch Static Pitch -150
tick 20 80 false false
bool SafePitch false
tick 20 80 false false
yaw Offset
floatYaw Offset Offset 340
pitch Offset
floatPitch Offset Offset 50
tick 20 80 false false
bool SafePitch true
tick 20 80 false false
floatYaw Static Yaw NaN
yaw Static
floatPitch Static Pitch Infinity
pitch Static
tick 20 80 false false
floatYaw Static Yaw 0
floatPitch Static Pitch -90
intYaw Jitter ForwardTicks 2
intYaw Jitter BackwardTicks 2
intYaw Spin Speed 50
'''
    for safe in ['true','false']:
        commands+=f'bool SafePitch {safe}\n'
        for yaw in ['Static','Offset','Random','Jitter','Spin']:
            commands+=f'yaw {yaw}\n'
            for pitch in ['Static','Offset','Random']:
                commands+=f'pitch {pitch}\n'
                for tick in range(4):commands+=f'tick {tick*34.25-50} {tick*17.75-30} false false\n'
    commands+='bool NotDuringSprint true\nyaw Random\npitch Random\ntick 5 10 true false\ntick 5 10 false true\ntick 5 10 false false\ninGame false\ntick 5 10 false false\ninGame true\ntick 5 10 false false\n'
    (out/'commands.txt').write_text(commands,encoding='utf-8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    stdlib=next(kotlin.glob('kotlin-stdlib-*.jar'))
    compiler=[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',str(stdlib),'-d',str(out/'classes')]+[str(p) for p in sorted((out/'fixture').glob('*.kt'))]
    recorded=[]
    def run(name,argv,env=None):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(result,stream),encoding='utf-8')
        recorded.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        return result
    result=run('compile',compiler)
    if result.returncode==0:
        result=run('kotlin',[str(java),'--add-opens','java.base/java.lang=ALL-UNNAMED','-cp',os.pathsep.join([str(out/'classes'),str(stdlib)]),'MainKt',str(out/'commands.txt')])
    rows=0
    if result.returncode==0:
        data=json.loads(result.stdout);rows=len(data['rows']);(out/'source-vectors.json').write_text(json.dumps(data),encoding='utf-8')
        snapshot={}
        for name in ['derp','random','rotations','no_rotate_set']:
            path=ROOT/f'native/liquidbounce-behaviors/src/{name}.rs';shutil.copyfile(path,out/f'{name}.rs');snapshot[name]=hashlib.sha256(path.read_bytes()).hexdigest()
        # The RNG's own production unit test consumes its real JDK oracle.
        rng=run('random',[str(java),'--add-opens','java.base/java.lang=ALL-UNNAMED',str(HERE/'AntiAfkRandomReference.java')])
        (out/'java-random-vectors.json').write_text(rng.stdout,encoding='utf-8')
        shutil.copyfile(fixture/'oracle.rs',out/'oracle.rs')
        (out/'lib.rs').write_text('pub mod derp;\npub mod random;\npub mod rotations;\npub mod no_rotate_set;\n#[cfg(test)] mod oracle;\n',encoding='utf-8')
        (out/'Cargo.toml').write_text('[package]\nname="derp-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n',encoding='utf-8')
        env=os.environ.copy();env.setdefault('CARGO_TARGET_DIR',str(HERE/'packet-tests/target'))
        result=run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')],env)
        if result.returncode==0:run('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    else:snapshot={}
    report={'passed':all(c['exitCode']==0 for c in recorded),'scope':'Byte-for-byte pinned ModuleDerp.kt compiled with minimal Minecraft/event/settings stubs and real Kotlin suspend continuations; actual JDK25 ThreadLocalRandom. Rust production replays identical scripted settings/lifecycle/mode/sprint/tick commands and compares raw float bits and RNG draw count for each tick. Stubs do not validate Minecraft hosting or hidden smoothing.','ticks':rows,'commands':recorded,'upstreamModuleSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'unchangedSource':source.read_bytes()==(out/'fixture/ModuleDerp.kt').read_bytes(),'snapshotSha256':snapshot}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'ticks':rows,'report':str(out/'report.json')}))
if __name__=='__main__':main()
