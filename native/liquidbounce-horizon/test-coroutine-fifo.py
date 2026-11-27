#!/usr/bin/env python3
"""Production FIFO with real cross-module state machines and fresh evidence."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='build/coroutine-fifo-01');args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    snapshot={};commands=[]
    names=['anti_afk','derp','hand_derp','auto_fish','random','rotations','no_rotate_set']
    for name in names:
        source=ROOT/f'native/liquidbounce-behaviors/src/{name}.rs';shutil.copyfile(source,out/f'{name}.rs');snapshot[name]=hashlib.sha256(source.read_bytes()).hexdigest()
    source=HERE/'engine/src/coroutine_ticker.rs';shutil.copyfile(source,out/'coroutine_ticker.rs');snapshot['coroutine_ticker']=hashlib.sha256(source.read_bytes()).hexdigest()
    patch=runpy.run_path(str(HERE/'coroutine-ticker-patch.py'))
    # Verbatim production scope and helper, with only a Game receiver stub.
    scope=patch['SCOPE']+'\npub struct Game;\nimpl Game {\n'+patch['HELPER']+'\n}\n'
    (out/'native_scope.rs').write_text(scope,encoding='utf-8')
    snapshot['native_scope']=hashlib.sha256(scope.encode()).hexdigest()
    shutil.copyfile(HERE/'coroutine-fifo-fixture.rs',out/'fixture.rs')
    (out/'lib.rs').write_text(''.join(f'pub mod {name};\n' for name in names+['coroutine_ticker'])+'#[cfg(test)] mod native_scope;\n#[cfg(test)] mod fixture;\n',encoding='utf-8')
    (out/'Cargo.toml').write_text('[package]\nname="coroutine-fifo-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n',encoding='utf-8')
    env=os.environ.copy();env.setdefault('CARGO_TARGET_DIR',str(HERE/'packet-tests/target'));env['FIFO_RUST_TRACE']=str(out/'rust-trace.json')
    def run(name,argv):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(result,stream),encoding='utf-8')
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        return result
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    rng=run('random',[str(java),'--add-opens','java.base/java.lang=ALL-UNNAMED',str(HERE/'AntiAfkRandomReference.java')])
    (out/'java-random-vectors.json').write_text(rng.stdout,encoding='utf-8')
    result=run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')])
    if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])
    source_files={};source_matches=None
    if result.returncode==0:
        fixture=out/'kotlin';shutil.copytree(HERE/'coroutine-source-fixture',fixture,dirs_exist_ok=True)
        upstream=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
        for name,relative in [('CoroutineTicker','event/CoroutineTicker.kt'),('ModuleDerp','features/module/modules/fun/ModuleDerp.kt'),('ModuleAntiAFK','features/module/modules/player/ModuleAntiAFK.kt')]:
            path=upstream/relative;shutil.copyfile(path,fixture/f'{name}.kt');source_files[name]={'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'unchanged':path.read_bytes()==(fixture/f'{name}.kt').read_bytes()}
        kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
        jars=[next(kotlin.glob(name)) for name in ['kotlin-stdlib-*.jar','kotlinx-coroutines-core-jvm-*.jar','slf4j-api-*.jar']]
        classpath=os.pathsep.join(str(p) for p in jars)
        result=run('kotlin-compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',classpath,'-d',str(out/'classes')]+[str(p) for p in sorted(fixture.glob('*.kt'))])
        if result.returncode==0:
            result=run('kotlin-source',[str(java),'-cp',os.pathsep.join([str(out/'classes'),classpath]),'MainKt'])
            if result.returncode==0:
                source=json.loads(result.stdout);rust=json.loads((out/'rust-trace.json').read_text());source_matches=source==rust
                (out/'source-trace.json').write_text(json.dumps(source),encoding='utf-8')
    report={'passed':all(c['exitCode']==0 for c in commands) and source_matches is True,'scope':'Production global CoroutineTicker queue and complete production AntiAFK/Derp/HandDerp/AutoFish state machines. Unchanged Kotlin CoroutineTicker/ModuleAntiAFK/ModuleDerp execute real kotlinx.coroutines; compare ten actual shared-yaw cross-module ticks. Native whole-tick depth scope/helper compiled verbatim with a Game receiver stub. Other tests cover opposite registration order, later live guard invalidation, pending new waits and cancelled task identity. Minecraft/event/settings stubs do not validate full native hosting.','commands':commands,'snapshotSha256':snapshot,'sourceFiles':source_files,'sourceTraceMatchesRust':source_matches}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
