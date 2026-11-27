#!/usr/bin/env python3
"""Byte-identical ResetVL, complete suspend machinery, shared Timer/queue oracle."""
from pathlib import Path
import argparse, hashlib, itertools, json, os, shutil, struct, subprocess, time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]

def f(value): return str(struct.unpack('>I',struct.pack('>f',value))[0])
def d(value): return str(struct.unpack('>Q',struct.pack('>d',value))[0])
def tick(ground=True,nested=False): return {'op':'tick','ground':ground,'nested':nested}
def enable(value=True): return {'op':'enabled','value':value}

def inputs():
    rows=[]
    for times,speed in itertools.product([1,2,10,50,0,-1,51],[.1,2,10,-0.0,-2.5,12.5]):
        rows.append([{'op':'times','value':times},{'op':'timer','bits':f(speed)},enable(),*[tick(nested=i%7==0) for i in range(55)]])
    for pattern in [[False]*12,[True,False]*8,[False,True,False,False]*5,[True]*12]:
        rows.append([{'op':'times','value':3},enable(),*[tick(ground,nested=i%3==0) for i,ground in enumerate(pattern)]])
    for when,mutation in itertools.product([0,1,2,3,9],['disable','guard','world','times','timer']):
        prefix=[enable(),*[tick() for _ in range(when)]]
        middle={'disable':[enable(False),enable()], 'guard':[{'op':'guard','value':False},tick(),{'op':'guard','value':True}],
                'world':[{'op':'world'}], 'times':[{'op':'times','value':when}], 'timer':[{'op':'timer','bits':f(4)}]}[mutation]
        rows.append(prefix+middle+[tick(i%2==0) for i in range(15)])
    for count,times,ground in itertools.product([0,1,9,10,11,2147483647],[0,1,10,-2147483648],[False,True]):
        rows.append([{'op':'count','value':count},{'op':'times','value':times},enable(),tick(ground),tick(False),tick(False),tick(True)])
    # Callback registration before/after the module's waiter exercises FIFO guards/settings.
    for action in [enable(False),{'op':'guard','value':False},{'op':'times','value':1},{'op':'count','value':9}]:
        for before in [False,True]:
            callback={'op':'callback','action':action}
            rows.append([enable(),*([callback] if before else []),tick(),*([callback] if not before else []),tick(),{'op':'guard','value':True},enable(),tick(),tick()])
    for priority,reset,same_thread in itertools.product([0,20,60],[0,1,3],[False,True]):
        rows.append([{'op':'times','value':2},{'op':'other','enabled':True,'bits':f(3),'priority':priority,'reset':reset},enable(),tick(),{'op':'sameThread','value':same_thread},enable(False),tick(False),{'op':'sameThread','value':True},tick(False),enable(),tick(),tick()])
    rows.append([enable(),{'op':'ground','value':True},{'op':'emit'},{'op':'emit'},{'op':'emit'},tick(),{'op':'velocity','bits':[d(2),d(3),d(4)]},tick()])
    # Config imports can contain raw nonfinite/denormal floats; exports reject
    # nonfinite values, but the original runtime passes them to Timer unchanged.
    for bits in [0x00000001,0x007fffff,0x00800000,0x7f7fffff,0x7f800000,0xff800000,0x7fc01234,0xffc01234]:
        rows.append([{'op':'times','value':2},{'op':'timer','bits':str(bits)},enable(),tick(False),tick(),tick(),tick()])
    return rows

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=ROOT/args.out
    if (out/'report.json').exists(): raise SystemExit('Refusing to overwrite evidence')
    original=out/'original';classes=original/'classes';classes.mkdir(parents=True,exist_ok=True)
    base=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
    names=['features/module/modules/exploit/ModuleResetVL.kt','event/CoroutineTicker.kt','event/SuspendHandlers.kt',
           'event/SuspendHandlerBehavior.kt','event/EventListenerScope.kt','event/Guards.kt',
           'utils/client/Timer.kt','utils/client/RequestHandler.kt','utils/kotlin/Priority.kt']
    sources=[base/name for name in names]
    for path in sources: shutil.copyfile(path,original/path.name)
    for path in (HERE/'reset-vl-source-fixture').glob('*.kt'): shutil.copyfile(path,original/path.name)
    service=classes/'META-INF/services/kotlinx.coroutines.internal.MainDispatcherFactory';service.parent.mkdir(parents=True,exist_ok=True)
    service.write_text('fixture.ImmediateMainFactory\n',encoding='utf8')
    (out/'inputs.json').write_text(json.dumps(inputs()),encoding='utf8')
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    mc=cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    gradle=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';jdk=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    jars=[mc,*gradle.glob('kotlin-stdlib-*.jar'),*gradle.glob('kotlinx-coroutines-core-jvm-*.jar')]
    for group in ['it.unimi.dsi','com.google.code.gson','org.slf4j']: jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    cp=os.pathsep.join(map(str,[classes,*jars]));commands=[]
    for name,argv in [('compile',[str(jdk/'java.exe'),'-cp',str(gradle/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-jvm-target','25','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(classes),*map(str,original.glob('*.kt'))]),
                      ('oracle',[str(jdk/'java.exe'),'-cp',cp,'fixture.Fixture',str(out/'inputs.json'),str(out/'oracle.json')])]:
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace')
        (out/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf8');commands.append({'name':name,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)})
        if result.returncode: break
    report={'passed':all(command['exitCode']==0 for command in commands),'scenarios':len(inputs()),'commands':commands,
            'scope':'Byte-identical original ResetVL, complete CoroutineTicker/SuspendHandlers/SuspendHandlerBehavior/EventListenerScope/Guards and Timer/RequestHandler/Priority. Actual Kotlin coroutine runtime and JDK25. Explicit settings/module/player/events, immediate Main render dispatcher and JDK AtomicReference-backed atomicfu API providers. No original complete-client or game claim.',
            'unchangedOriginalKotlin':all(path.read_bytes()==(original/path.name).read_bytes() for path in sources),
            'sources':{path.relative_to(ROOT).as_posix():hashlib.sha256(path.read_bytes()).hexdigest() for path in sources},
            'fixtureSources':{path.name:hashlib.sha256(path.read_bytes()).hexdigest() for path in (HERE/'reset-vl-source-fixture').glob('*.kt')}}
    if (out/'oracle.json').is_file(): report['oracleSha256']=hashlib.sha256((out/'oracle.json').read_bytes()).hexdigest()
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
    raise SystemExit(0 if report['passed'] else 1)

if __name__=='__main__':main()
