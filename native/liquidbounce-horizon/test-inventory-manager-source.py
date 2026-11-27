#!/usr/bin/env python3
"""Unchanged InventoryManager/InventoryPlan/InventoryAction coroutine oracle."""
from pathlib import Path
import argparse, hashlib, json, os, runpy, shutil, subprocess, time
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent

def commands():
    rows = [
        {'cmd':'constraints','owner':1,'start':[0,0],'click':[0,0],'close':[0,0],'multiplier':0},
        {'cmd':'set','slot':'p9','item':'stone','count':64},
        {'cmd':'plan','id':1,'chains':[[{'kind':'click','slot':'p9','input':4,'button':1}]]},
        {'cmd':'tick'}, {'cmd':'tick'},
        {'cmd':'constraints','owner':1,'start':[3,7],'click':[11,17],'close':[13,19],'multiplier':0.2},
        {'cmd':'set','slot':'p10','item':'apple','count':8},
        {'cmd':'plan','id':2,'chains':[[{'kind':'click','slot':'p10','input':4,'button':1}]]},
        {'cmd':'tick'}, {'cmd':'advance','ms':5}, {'cmd':'advance','ms':10}, {'cmd':'advance','ms':100},
        {'cmd':'screen','kind':'container','id':7,'slots':27},
        {'cmd':'constraints','owner':1,'start':[0,0],'click':[0,0],'close':[0,0],'miss':[100,100],'multiplier':0},
        {'cmd':'set','screen':True,'slot':'c9','item':'quartz','count':4},
        {'cmd':'plan','id':3,'chains':[[{'kind':'click','screen':True,'slot':'c9','input':1,'button':0}]]},
        {'cmd':'tick'}, {'cmd':'advance','ms':20},
        {'cmd':'packet','kind':'slot','id':7,'slot':9,'item':'quartz','count':4},
        {'cmd':'packet','kind':'content','id':7}, {'cmd':'advance','ms':400},
        {'cmd':'world'}, {'cmd':'screen','kind':'none'},
        {'cmd':'constraints','owner':1,'start':[0,0],'click':[0,0],'close':[0,0],'miss':[0,0],'multiplier':0},
        {'cmd':'plan','id':4,'chains':[[{'kind':'creative','slot':'p11','item':'dirt','count':64},{'kind':'click','slot':'p11','input':4,'button':1}]]},
        {'cmd':'tick'}, {'cmd':'advance','ms':400},
    ]
    def zero(owner=1, **extra):
        return {'cmd':'constraints','owner':owner,'start':[0,0],'click':[0,0],'close':[0,0],'miss':[0,0],'multiplier':0,**extra}
    def plan(identity, actions, **extra):
        return {'cmd':'plan','id':identity,'chains':[actions],**extra}
    def throw(slot='p9', **extra):
        return {'kind':'click','slot':slot,'input':4,'button':1,**extra}
    def creative(item='stone'):
        return {'kind':'creative','slot':'p12','item':item,'count':64}
    # Same expected menu, priorities and FIFO: CreativeAdd does not mutate it.
    rows += [{'cmd':'world'},{'cmd':'screen','kind':'none'},zero(),
        plan(10,[creative('normal-a')],priority=1),plan(11,[creative('high')],priority=2),
        plan(12,[creative('low')],priority=0),plan(13,[creative('normal-b')],priority=1),
        {'cmd':'tick'},{'cmd':'advance','ms':500}]
    # A pending plan checks live owner and guards; active atomic chains keep
    # sending their actions before the cancelled continuation is resumed.
    rows += [{'cmd':'world'},zero(start=[20,20],click=[10,10]),
        {'cmd':'set','slot':'p9','item':'iron','count':5},
        plan(20,[throw(),creative('tail')]),{'cmd':'tick'},
        {'cmd':'owner','id':1,'running':False},{'cmd':'advance','ms':100},
        {'cmd':'owner','id':1,'running':True},zero(),
        plan(21,[creative()]),{'cmd':'owner','id':1,'guard':False},{'cmd':'tick'},
        {'cmd':'owner','id':1,'guard':True},{'cmd':'advance','ms':500}]
    # External inventory and carried changes invalidate an expected menu even
    # when the actual planned slot is unchanged.
    rows += [{'cmd':'world'},plan(30,[creative()]),
        {'cmd':'set','slot':'p15','item':'external','count':1},{'cmd':'tick'},
        plan(31,[creative()]),{'cmd':'carried','item':'external','count':1},{'cmd':'tick'},
        {'cmd':'carried','count':0},{'cmd':'advance','ms':500}]
    # Every ordinary requirement is live while pending, including jumping and
    # the exact rotation/using/breaking/combat flags.
    for bit, state, fails, passes in [(1,'moving',True,False),(1,'jumping',True,False),
            (2,'rotation',False,True),(4,'using',True,False),(8,'breaking',True,False),(16,'combat',True,False)]:
        identity=40+len(rows)
        rows += [{'cmd':'world'},{'cmd':'screen','kind':'inventory'},zero(requires=bit),
            {'cmd':'set','slot':'p9','item':'requirement','count':3},
            {'cmd':'state',state:fails},plan(identity,[throw()]),{'cmd':'tick'},
            {'cmd':'state',state:passes},{'cmd':'tick'},{'cmd':'advance','ms':500}]
    # One plan can synchronously queue its next chain before click delay.
    rows += [{'cmd':'world'},{'cmd':'screen','kind':'none'},zero(click=[7,7],requires=0),
        {'cmd':'plan','id':60,'chains':[[creative('chain-one')],[creative('chain-two')],[creative('chain-three')]]},
        {'cmd':'tick'},{'cmd':'advance','ms':6},{'cmd':'advance','ms':1},{'cmd':'advance','ms':7},{'cmd':'advance','ms':500}]
    # Container-open waits block player inventory clicks until the real
    # response timeout, and are cleared by an accepted server open.
    rows += [{'cmd':'world'},zero(),{'cmd':'set','slot':'p9','item':'wait','count':1},
        {'cmd':'packet','kind':'open-pending'},plan(70,[throw()]),{'cmd':'tick'},
        {'cmd':'advance','ms':399},{'cmd':'tick'},{'cmd':'advance','ms':1},{'cmd':'tick'},
        {'cmd':'packet','kind':'out-close','id':0},{'cmd':'packet','kind':'out-close','id':0},
        {'cmd':'packet','kind':'open-pending'},{'cmd':'packet','kind':'opened'},{'cmd':'advance','ms':500}]
    # Real stdlib monotonic time and actual coroutine DefaultDelay now share
    # the deterministic clock. Nearest-first includes unresolved/outside ids,
    # stable ties, duplicates and screen-specific player slot mapping.
    for kind in ['none','inventory','container','creative','other']:
        rows += [{'cmd':'world'},{'cmd':'screen','kind':kind,'id':17,'slots':27}]
        for cursor in [None,[0,0],[80.5,74.25],[-0.0,-0.0],[1e10,-1e10],[3.4e38,3.4e38]]:
            rows += [{'cmd':'cursor','value':cursor},{'cmd':'nearestFirst','slots':[
                'p0','p8','p9','p35','p36','p39','p40','c0','c26','c27','c62','c999','c-1','p0']}]
    # Visible travel uses genuine source cubic easing and Float/Double cursor
    # conversions at every 5ms sample, with live interruption by mouse capture,
    # silent-screen hiding and screen replacement.
    for index,interrupt in enumerate([None,'grabbed','hidden','screen']):
        rows += [{'cmd':'world'},{'cmd':'screen','kind':'inventory'},zero(start=[37,37],click=[23,23],multiplier=0.3),
            {'cmd':'visiblecursor','enabled':True,'grabbed':False,'hidden':False,
                'x':117.125,'y':83.625,'width':1920,'guiWidth':683,'left':11,'top':19},
            {'cmd':'set','slot':'p9','item':'cursor-test','count':3},plan(200+index,[throw()]),{'cmd':'tick'},
            {'cmd':'advance','ms':5},{'cmd':'advance','ms':11}]
        if interrupt in ['grabbed','hidden']: rows += [{'cmd':'visiblecursor',interrupt:True}]
        elif interrupt=='screen': rows += [{'cmd':'screen','kind':'other'}]
        rows += [{'cmd':'advance','ms':300},{'cmd':'visiblecursor','enabled':False,'grabbed':False,'hidden':False}]
    # Container misses find the nearest empty owned slot; only PICKUP/SWAP/
    # QUICK_MOVE miss paths qualify, and failed misses still consume the draws.
    for index,action in enumerate([0,1,2,4]):
        rows += [{'cmd':'world'},{'cmd':'screen','kind':'container','id':23,'slots':27},
            zero(start=[3,7],click=[11,17],close=[5,9],miss=[100,100],multiplier=0.2),
            {'cmd':'set','slot':'c9','screen':True,'item':'miss-target','count':8},
            plan(210+index,[{'kind':'click','screen':True,'slot':'c9','input':action,'button':0}]),
            {'cmd':'tick'},{'cmd':'advance','ms':500}]
    # Response settings are read at each newly started server wait. An active
    # wait keeps its original duration, while the next use/open samples anew.
    rows += [{'cmd':'world'},{'cmd':'screen','kind':'none'},zero(),{'cmd':'settings','responseTime':73,'lockAfterRejections':2},
        {'cmd':'packet','kind':'open-pending'},{'cmd':'advance','ms':72},{'cmd':'settings','responseTime':111},
        {'cmd':'advance','ms':1},plan(220,[{'kind':'use','slot':40}]),{'cmd':'tick'},
        {'cmd':'advance','ms':110},{'cmd':'advance','ms':1},plan(221,[{'kind':'use','slot':0}]),
        {'cmd':'tick'},{'cmd':'advance','ms':111}]
    # Repeat a true restored throw with the unchanged source slot: first
    # restoration rearms its wait, timeout records a rejection then pauses,
    # and the configured second rejection locks that item until WorldChange.
    for index in range(3):
        rows += [{'cmd':'screen','kind':'none'},zero(),{'cmd':'set','slot':'p9','item':'server-locked','count':9},
            plan(230+index,[throw()]),{'cmd':'tick'},
            {'cmd':'packet','kind':'slot','id':0,'slot':9,'item':'server-locked','count':9},
            {'cmd':'advance','ms':110},{'cmd':'advance','ms':1},{'cmd':'advance','ms':111}]
    rows += [{'cmd':'world'},{'cmd':'settings','responseTime':400,'lockAfterRejections':3},
        {'cmd':'screen','kind':'none'},zero(),{'cmd':'advance','ms':500}]
    # Creative game-mode helper has its own gate: filled slots are sent only
    # in creative, while drops additionally require nonempty stacks and permit
    # the Creative screen and unrelated screens but reject handled containers.
    for creative_mode in [True,False]:
        for kind in ['none','inventory','container','creative','other']:
            for count in [0,1]:
                identity=300+len(rows)
                rows += [{'cmd':'world'},{'cmd':'screen','kind':kind,'id':31,'slots':27},
                    {'cmd':'state','creative':creative_mode},zero(),
                    plan(identity,[{'kind':'creative','item':'test-item','count':count},
                        {'kind':'creative','slot':'p40','item':'test-item','count':count}]),
                    {'cmd':'tick'},{'cmd':'advance','ms':500}]
    rows += [{'cmd':'world'},{'cmd':'state','creative':True},{'cmd':'screen','kind':'none'},zero()]
    return rows

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True)
    fixture=out/'fixture';shutil.copytree(HERE/'inventory-manager-source-fixture',fixture,dirs_exist_ok=True)
    origins={}
    for package,name in [('utils/inventory','InventoryManager'),('utils/inventory','InventoryPlan'),('utils/inventory','InventoryAction'),('utils/inventory','InventoryExtensions'),('event','Guards'),('event','EventListenerScope')]:
        original=ROOT/f'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/{package}/{name}.kt'
        shutil.copyfile(original,fixture/f'{name}.kt');origins[name]=hashlib.sha256(original.read_bytes()).hexdigest()
    rows=commands();(out/'commands.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    libraries=[next(kotlin.glob(pattern)) for pattern in ['kotlin-stdlib-*.jar','kotlinx-coroutines-core-jvm-*.jar','gson-*.jar','slf4j-api-*.jar']]
    libraries+=[next((ROOT/'build/resource-runtime/libraries').rglob('joml-*.jar'))]
    cp=os.pathsep.join(map(str,libraries));operations=[]
    def run(name,argv):
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:
            result=subprocess.run(argv,cwd=ROOT,stdout=stdout,stderr=stderr)
        operations.append({'stage':name,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)})
        return result.returncode==0
    okay=run('kotlin-compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-jvm-target','25','-Xcompanion-blocks-and-extensions','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(out/'classes')]+[str(p) for p in sorted(fixture.glob('*.kt'))])
    source_clock = None
    if okay:
        source_clock = runpy.run_path(str(fixture/'patch_monotonic.py'))['patch'](libraries[0], out/'classes')
        (out/'source-clock.json').write_text(json.dumps(source_clock,indent=2)+'\n',encoding='utf-8')
        service=out/'classes/META-INF/services/kotlinx.coroutines.internal.MainDispatcherFactory';service.parent.mkdir(parents=True,exist_ok=True);service.write_text('fixture.MainFactory\n')
        okay=run('kotlin-source',[str(java),'--add-opens','java.base/java.lang=ALL-UNNAMED','-cp',str(out/'classes')+os.pathsep+cp,'fixture.MainKt',str(out/'commands.json'),str(out/'source-vectors.json')])
    report={'passed':okay,'commands':operations,'rows':len(rows),'sourceSha256':origins,'sourceClock':source_clock,'scope':'Byte-for-byte original InventoryManager, InventoryPlan, InventoryAction, InventoryExtensions, Guards and EventListenerScope. Real Kotlin JVM Random.Default and kotlinx.coroutines run on an explicit deterministic render/delay scheduler. The test-only MonotonicTimeSource.class changes exactly the shared owner of two System.nanoTime invocations to fixture.Clock.nanoTime, retaining every original method bytecode and all Kotlin Duration/time math. Minecraft menu/native helpers and Constraints remain fixture stubs. Production Rust oracle comparison is separate; this report alone does not claim runtime/native validation.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':okay,'report':str(out/'report.json')}))
    raise SystemExit(0 if okay else 1)

if __name__=='__main__':main()
