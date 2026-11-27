#!/usr/bin/env python3
"""Unchanged BetterInventory + list state/layout against production Rust."""
from pathlib import Path
import argparse,copy,hashlib,json,os,shutil,struct,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def bits(value):return struct.unpack('<i',struct.pack('<f',value))[0]
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True);shutil.copytree(HERE/'better-inventory-source-fixture',out/'fixture',dirs_exist_ok=True)
    kotlin_root=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
    sources=['features/module/modules/render/ModuleBetterInventory.kt','render/gui/ItemStackListRenderState.kt','render/gui/ItemStackListLayout.kt']
    for relative in sources:shutil.copy2(kotlin_root/relative,out/'fixture'/Path(relative).name)
    state={'highlight':True,'highlightMode':'Border','highlightColor':-12158977,'text':True,'cooldownMode':'PERCENTAGE','cooldownScale':bits(1),'cooldownColor':-1,'container':True,'skipEmpty':False,'containerScale':bits(1),'relative':True,'offsetX':bits(150),'offsetY':bits(0),'move':True}
    base={'state':state,'enabled':True,'inGame':True,'player':True,'progress':bits(.5),'entry':[20,10],'last':36,'slot':36,'x':8,'y':142,'items':[True,False,True],'empty':False,'component':True,'carriedEmpty':True,'hover':True,'left':120,'top':50,'mouseX':150,'mouseY':90}
    commands=[]
    def add(changes=None,settings=None):
        row=copy.deepcopy(base)
        if changes:row.update(changes)
        if settings:row['state'].update(settings)
        commands.append(row)
    add()
    for key in ['enabled','inGame','player','empty','component','carriedEmpty','hover']:add({key:False});add({key:True})
    for key in ['highlight','text','container','move']:add(settings={key:False})
    for highlight in ['Border','Texture']:
        for last,slot in [(0,0),(36,36),(-1,-1),(1,2),(2147483647,2147483647)]:
            for x,y in [(8,142),(-3,-4),(2147483647,-2147483648)]:add({'last':last,'slot':slot,'x':x,'y':y}, {'highlightMode':highlight,'highlightColor':0x12345678})
    for mode in ['PERCENTAGE','DURATION_TICKS','DURATION_SECONDS']:
        for ticks in [-2147483648,-100,-25,-15,-7,-5,-1,0,1,2,3,5,7,9,10,15,19,20,21,25,30,100,2147483647]:
            for progress in [-1,0,.001,.05,.125,.9999,1,2,float('inf'),float('nan')]:
                add({'entry':[ticks,0],'progress':bits(progress)}, {'cooldownMode':mode})
        for scale in [.25,.75,1,4,-1,0,float('inf'),float('nan')]:add(settings={'cooldownMode':mode,'cooldownScale':bits(scale)})
        add({'entry':None},{'cooldownMode':mode})
        add({'entry':[-2147483648,2147483647]},{'cooldownMode':mode})
    for size in [0,1,8,9,10,18,27,54,256]:
        for skip in [False,True]:
            for relative in [False,True]:
                for scale in [.25,1,4,0,-1,float('inf'),float('nan')]:
                    add({'items':[i%3==0 for i in range(size)]},{'skipEmpty':skip,'relative':relative,'containerScale':bits(scale)})
    for x,y,mx,my in [(0,0,0,0),(-50,-25,-10,20),(2147483647,-2147483648,2147483647,-2147483648)]:
        for ox,oy in [(150,0),(-4096,4096),(3.402823466e38,-3.402823466e38),(float('nan'),float('inf'))]:
            add({'left':x,'top':y,'mouseX':mx,'mouseY':my},{'offsetX':bits(ox),'offsetY':bits(oy)})
    (out/'commands.json').write_text(json.dumps(commands),encoding='utf-8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';libs=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';stdlib=next(libs.glob('kotlin-stdlib-*.jar'));gson=next(libs.glob('gson-*.jar'));records=[]
    def run(name,argv,env=None):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(result,stream),encoding='utf-8')
        records.append({'argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)});return result
    result=run('compile',[str(java),'-cp',str(libs/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-classpath',os.pathsep.join([str(stdlib),str(gson)]),'-d',str(out/'classes')]+[str(path) for path in sorted((out/'fixture').glob('*.kt'))])
    if result.returncode==0:
        result=run('source',[str(java),'-cp',os.pathsep.join([str(out/'classes'),str(stdlib),str(gson)]),'MainKt',str(out/'commands.json')])
        if result.returncode==0:
            json.loads(result.stdout);(out/'source-vectors.json').write_text(result.stdout,encoding='utf-8')
            shutil.copy2(ROOT/'native/liquidbounce-behaviors/src/better_inventory.rs',out/'better_inventory.rs');shutil.copy2(HERE/'better-inventory-source-fixture/oracle.rs',out/'oracle.rs')
            shutil.copy2(ROOT/'native/liquidbounce-behaviors/src/better_inventory_locale.rs',out/'better_inventory_locale.rs')
            shutil.copy2(HERE/'better-inventory-locale-oracle.json',out/'locale-oracle.json')
            (out/'lib.rs').write_text('pub mod better_inventory;\n#[cfg(test)]mod oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="better-inventory-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n',encoding='utf-8')
            env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/better-inventory-target')
            result=run('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')],env)
            if result.returncode==0:run('clippy',['cargo','+stable','clippy','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    report={'passed':all(r['exitCode']==0 for r in records),'commands':records,'rows':len(commands),'unchangedSources':{relative:hashlib.sha256((kotlin_root/relative).read_bytes()).hexdigest() for relative in sources},'scope':'Byte-for-byte production BetterInventory and ItemStackListRenderState/Layout with rendering/service stubs. Covers every behavior setting, true source operations/order, all cooldown modes including raw float bits/tick overflow/Java formatter ties, all list boundaries and filtering, scale validation and screen outer gate. Actual native renderer and menu/component extraction remain separate checks.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'rows':len(commands),'report':str(out/'report.json')}))
if __name__=='__main__':main()
