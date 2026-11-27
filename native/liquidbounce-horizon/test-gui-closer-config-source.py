#!/usr/bin/env python3
"""Original ListValue, regex TypeAdapter, ValueGroupSerializer and exclusions.

Value/group/input/MC classes are explicit minimal providers. The production
Rust configuration and behavior snapshot remain unchanged except registration.
"""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=(ROOT/a.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    # Reuse only the production snapshot preparation before its command phase.
    source=(HERE/'test-gui-closer-config.py').read_text(encoding='utf8').split('    env=os.environ',1)[0]+'\nif __name__=="__main__":main()\n'
    exec(compile(source,str(HERE/'test-gui-closer-config.py'),'exec'),{'__file__':str(HERE/'test-gui-closer-config.py'),'__name__':'__main__'})
    fixture=out/'fixture';shutil.copytree(HERE/'gui-closer-config-source-fixture',fixture,dirs_exist_ok=True)
    original=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/config'
    originals=['types/list/ListValue.kt','gson/adapter/SimpleStringTypeAdapter.kt','gson/serializer/ValueGroupSerializer.kt','gson/stategies/ExcludeStrategy.kt','gson/stategies/ProtocolExcludeStrategy.kt']
    unchanged={}
    for name in originals:
        src=original/name;dst=fixture/src.name;shutil.copyfile(src,dst);unchanged[name]=hashlib.sha256(dst.read_bytes()).hexdigest();assert src.read_bytes()==dst.read_bytes()
    inputs=[[],None,'^Vote$','',True,False,0,1,1.5,{},[None],['a',None,'a'],['ok','['],['[','ok'],['ok',True],['ok',{}],[['x']],['(?i)vote',r'\Q[' ],r'(?<x>a)\k<x>',r'\p{L}+',r'\p{Unknown}',r'(?x) a { 1 , 2 }','😀']
    (out/'inputs.json').write_text(json.dumps(inputs,ensure_ascii=True),encoding='utf8')
    (out/'source_oracle.rs').write_text((fixture/'oracle.rs').read_text(encoding='utf8'),encoding='utf8')
    lib=out/'lib.rs';lib.write_text(lib.read_text(encoding='utf8')+'\n#[cfg(test)] mod source_oracle;\n',encoding='utf8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';javac=java.with_name('javac.exe');gradle=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';stdlib=next(gradle.glob('kotlin-stdlib-*.jar'));gson=next(gradle.glob('gson-*.jar'));classes=out/'classes';classes.mkdir(exist_ok=True)
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/better-inventory-target');commands=[]
    invocations=[('javac',[str(javac),'-d',str(classes)]+[str(f) for f in fixture.glob('*.java')]),('kotlin',[str(java),'-cp',str(gradle/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',os.pathsep.join(map(str,[stdlib,gson,classes])),'-d',str(classes)]+[str(f) for f in fixture.glob('*.kt')]),('oracle',[str(java),'-cp',os.pathsep.join(map(str,[classes,stdlib,gson])),'MainKt',str(out/'inputs.json')]),('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]
    for name,argv in invocations:
        start=time.monotonic();run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True)
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_bytes(getattr(run,stream))
        commands.append({'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)})
        if run.returncode:break
        if name=='oracle':(out/'expected.json').write_bytes(run.stdout)
    report={'passed':all(c['exitCode']==0 for c in commands),'vectors':len(inputs),'scope':'Byte-identical original ListValue, SimpleStringTypeAdapter, ValueGroupSerializer and exclusions. Explicit minimal Value/group/input/MC providers. Actual Gson/JDK25, whole production Rust config/behavior snapshot, File/Interop/public inclusion, ordered/nullable/duplicate filters, strict Regex JSON input and atomic failure. No game claim.','originalSha256':unchanged,'commands':commands,'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*') if p.is_file() and p.suffix in ['.rs','.kt','.java','.toml','.bin','.json'] and p.name!='report.json'}}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
