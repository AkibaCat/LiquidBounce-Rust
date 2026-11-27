#!/usr/bin/env python3
"""Compile unmodified ModuleAutoBreak against real official 26.3 KeyMapping/HitResult classes."""
from pathlib import Path
import argparse, hashlib, itertools, json, os, shutil, subprocess, time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True);fixture=out/'fixture';shutil.copytree(HERE/'auto-break-source-fixture',fixture,dirs_exist_ok=True)
    source=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/player/ModuleAutoBreak.kt'
    shutil.copyfile(source,fixture/source.name)
    commands=[dict(zip(['running','attack','missTime','hit','state','destroying','original'],values)) for values in itertools.product([False,True],[False,True],[-2147483648,-1,0,1,2147483647],['null','miss','entity','block'],['missing','air','solid'],[False,True],[False,True])]
    (out/'commands.json').write_text(json.dumps(commands),encoding='utf8')
    java_root=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    official=Path('C:/Users/AkibaCat/.gradle/caches/fabric-loom/26.3/minecraft-client.jar')
    deps=[p for p in Path('C:/Users/AkibaCat/.gradle/caches/modules-2/files-2.1').rglob('*.jar') if not p.name.endswith(('-sources.jar','-javadoc.jar'))]
    stdlib=next(kotlin.glob('kotlin-stdlib-*.jar'))
    cp=os.pathsep.join(map(str,[official,stdlib,*deps]));classes=out/'classes';classes.mkdir(exist_ok=True)
    commands_run=[]
    def run(name,argv,env=None):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(result,stream),encoding='utf8')
        commands_run.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        return result
    def args_file(name,values):
        path=out/(name+'.args');path.write_text('\n'.join('"'+str(value).replace('\\','/').replace('"','\\"')+'"' for value in values),encoding='utf8');return '@'+str(path)
    result=run('javac',[str(java_root/'javac.exe'),args_file('javac',['-cp',cp,'-d',classes,*sorted(fixture.glob('*.java'))])])
    if result.returncode==0:
        result=run('compile',[str(java_root/'java.exe'),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler',args_file('kotlin',['-no-stdlib','-no-reflect','-classpath',str(classes)+os.pathsep+cp,'-d',classes,*sorted(fixture.glob('*.kt'))])])
    rows=0;snapshots={}
    click_rows=0
    if result.returncode==0:
        result=run('clicks',[str(java_root/'java.exe'),args_file('clicks',['-cp',str(classes)+os.pathsep+cp,'fixture.KeyMappingClickOracle'])])
        if result.returncode==0:
            clicks=json.loads(result.stdout);click_rows=len(clicks);(out/'click-vectors.json').write_text(json.dumps(clicks),encoding='utf8')
    if result.returncode==0:
        result=run('source',[str(java_root/'java.exe'),args_file('run',['-cp',str(classes)+os.pathsep+cp,'MainKt',out/'commands.json'])])
    if result.returncode==0:
        vectors=json.loads(result.stdout);rows=len(vectors);(out/'source-vectors.json').write_text(json.dumps(vectors),encoding='utf8')
        for name,path in [('auto_break',ROOT/'native/liquidbounce-behaviors/src/auto_break.rs'),('keymapping_clicks',HERE/'keymapping-clicks.rs')]:
            shutil.copyfile(path,out/(name+'.rs'));snapshots[str(path.relative_to(ROOT))]=hashlib.sha256(path.read_bytes()).hexdigest()
        shutil.copyfile(fixture/'oracle.rs',out/'oracle.rs')
        (out/'lib.rs').write_text('pub mod auto_break;\npub mod keymapping_clicks;\n#[cfg(test)] mod oracle;\n')
        (out/'Cargo.toml').write_text('[package]\nname="auto-break-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n')
        env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
        result=run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')],env)
        if result.returncode==0:run('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    report={'passed':all(c['exitCode']==0 for c in commands_run),'scope':'Byte-for-byte unchanged ModuleAutoBreak.kt with minimal event/module/environment fixture, real unmodified official MC26.3 KeyMapping, HitResult, BlockHitResult, BlockPos and Direction. Java subclass only exposes the protected key field already widened by LiquidBounce. Differential behavior and actual same-key click counts/held state, official queued-click/rebinding/release lifecycle. Unknown-key exclusion is independently copied from pinned MixinKeyMapping, not present in official vanilla classes. No actual game/GPU or canonical registration claim.','vectors':rows,'clickLifecycleVectors':click_rows,'unchangedSource':source.read_bytes()==(fixture/source.name).read_bytes(),'upstreamModuleSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'officialJarSha256':hashlib.sha256(official.read_bytes()).hexdigest(),'snapshotSha256':snapshots,'commands':commands_run}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'vectors':rows,'report':str(out/'report.json')}))
if __name__=='__main__':main()
