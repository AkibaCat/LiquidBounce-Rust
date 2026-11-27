#!/usr/bin/env python3
"""Untouched ModuleHighJump and CoroutineTicker with explicit event/storage shells."""
import hashlib,itertools,json,os,shutil,struct,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
OUT=ROOT/'build/high-jump-tests'
def d(v):return str(struct.unpack('>Q',struct.pack('>d',v))[0])
def f(v):return str(struct.unpack('>I',struct.pack('>f',v))[0])
def tick(count,ground=False,fall=1.0,nested=False):return dict(op='tick',count=count,ground=ground,fall=d(fall),nested=nested)
def inputs():
    rows=[]
    for mode,motion in itertools.product(['Vanilla','Vulcan','Vulcan-2.7.5'],[.8,.2,10.,-0.,-2.5,1e-30,12.5]):
        rows.append([dict(op='mode',value=mode),dict(op='motion',bits=f(motion)),dict(op='jump'),*[tick(i,nested=i%7==0)for i in range(105)]])
    for glide,fall in itertools.product([False,True],[-0.,0.,1.,-1.]):
        rows.append([dict(op='mode',value='Vulcan'),dict(op='glide',value=glide),dict(op='jump'),*[tick(i,fall=fall)for i in range(110)],tick(111,ground=True,fall=fall),tick(112,fall=fall)])
    for mutation in ['disable','mode','guard']:
        for when in [0,1,99,100,101]:
            pre=[dict(op='mode',value='Vulcan'),dict(op='glide',value=True),dict(op='jump'),*[tick(i)for i in range(when)]]
            middle=[dict(op='running',value=False),dict(op='running',value=True)] if mutation=='disable' else [dict(op='mode',value='Vanilla'),dict(op='mode',value='Vulcan')]
            if mutation=='guard':middle=[dict(op='running',value=False),tick(200),dict(op='running',value=True)]
            rows.append(pre+middle+[tick(300+i)for i in range(105)])
    for gap in [0,1,10,99]:rows.append([dict(op='mode',value='Vulcan'),dict(op='jump'),*[tick(i)for i in range(gap)],dict(op='jump'),*[tick(gap+i)for i in range(105)]])
    return rows
def main():
    work=OUT/'original';classes=work/'classes';classes.mkdir(parents=True,exist_ok=True)
    base=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
    originals=[base/'features/module/modules/movement/highjump/ModuleHighJump.kt',base/'event/CoroutineTicker.kt']
    for p in originals:shutil.copyfile(p,work/p.name)
    for p in (HERE/'oracles/high-jump').glob('*.kt'):shutil.copyfile(p,work/p.name)
    (OUT/'inputs.json').write_text(json.dumps(inputs()),encoding='utf-8')
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches';mc=cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    gradle=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';jdk=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    jars=[mc,*gradle.glob('kotlin-stdlib-*.jar'),*gradle.glob('kotlinx-coroutines-core-jvm-*.jar')]
    for group in ['it.unimi.dsi','com.google.code.gson','org.slf4j']:jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    cp=os.pathsep.join(map(str,[classes,*jars]));commands=[]
    for name,argv in [('compile',[str(jdk/'java.exe'),'-cp',str(gradle/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(classes),*map(str,work.glob('*.kt'))]),('run',[str(jdk/'java.exe'),'-cp',cp,'fixture.Fixture',str(OUT/'inputs.json'),str(OUT/'oracle.json')])]:
        r=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace');(OUT/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr,end='');commands.append(dict(name=name,exitCode=r.returncode))
        if r.returncode:raise SystemExit(r.returncode)
    report=dict(cases=len(inputs()),commands=commands,unchangedOriginalKotlin=all(p.read_bytes()==(work/p.name).read_bytes()for p in originals),sources={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in originals},oracleSha256=hashlib.sha256((OUT/'oracle.json').read_bytes()).hexdigest(),boundaries=['actual untouched module and CoroutineTicker, actual kotlinx coroutines UNDISPATCHED parallel sequences','event registration, settings, module scope cancellation and player scalar storage are providers','no complete original client execution'])
    (OUT/'oracle-metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report))
if __name__=='__main__':main()
