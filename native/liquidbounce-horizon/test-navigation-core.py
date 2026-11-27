#!/usr/bin/env python3
"""Execute unchanged Minecraft Agent core algorithms against their Rust ports."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import random
import shutil
import struct
import subprocess
import time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'

def bits(value):return struct.unpack('<Q',struct.pack('<d',value))[0]
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args()
    out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
    if out.exists():raise SystemExit('Use a fresh evidence directory')
    out.mkdir(parents=True);records=[]
    def run(name,argv,env=None):
        start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (out/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8');records.append({'argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3)})
        if result.returncode:print((result.stdout+result.stderr)[-10000:])
        return result
    original=[AGENT/p for p in ['pathfinder/route/AStar.kt','navigation/Keystrokes.kt','world/Input.kt']]
    for path in original:shutil.copyfile(path,out/path.name)
    shutil.copyfile(HERE/'navigation-source-fixture/Core.kt',out/'Core.kt')
    commands=[];rng=random.Random(2603)
    for mask in range(128):commands.append(f'key {mask} true 0 0')
    for i in range(10000):
        if i%79==0:commands.append('resetKeys')
        commands.append(f'key {rng.randrange(128)} {str(rng.randrange(7)==0).lower()} {rng.choice([-2147483648,-1,0,1,2,5,1000,2147483647])} {rng.choice([-1,0,1,2,5,1000,2147483647])}')
    def graph(count,start,goal,maximum,heuristics,edges,slices):
        commands.append(f'graph {count} {start} {goal} {bits(maximum)} '+','.join(str(bits(h))for h in heuristics)+' '+(';'.join(f'{a},{b},{bits(cost)}'for a,b,cost in edges)or'-'))
        commands.extend(f'advance {cap}'for cap in slices)
    # Equal estimates intentionally distinguish Java heap order from stable/FIFO heaps.
    graph(40,0,39,float('inf'),[0.0]*40,[(0,n,1.0)for n in range(1,39)]+[(n,39,1.0)for n in range(1,39)],[1]*45)
    for i in range(400):
        count=rng.randrange(2,45);edges=[]
        for _ in range(rng.randrange(count,count*6)):
            edges.append((rng.randrange(count),rng.randrange(count),rng.choice([0.0,-0.0,0.1,0.5,1.0,2.0,10.0,float('inf')])))
        if i%31==0:edges.insert(0,(0,1,float('nan')))
        if i%37==0:edges.insert(0,(0,1,-1.0))
        heuristics=[rng.choice([0.0,-0.0,0.1,1.0,2.0,100.0,float('inf'),float('-inf'),float('nan')])for _ in range(count)]
        graph(count,0,count-1,rng.choice([0.0,-0.0,1.0,10.0,float('inf'),float('nan')]),heuristics,edges,[-1,0,1,2,3,1,20,100,100])
    (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
    kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    jars=[next(kotlin.glob('kotlin-stdlib-*.jar')),next((ROOT/'build/resource-runtime/libraries').rglob('fastutil-8.5.18.jar'))]
    classpath=os.pathsep.join(map(str,jars))
    result=run('kotlin-compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',classpath,'-d',str(out/'classes'),*[str(path)for path in out.glob('*.kt')]])
    if result.returncode==0:result=run('kotlin-oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),classpath]),'fixture.CoreKt'])
    shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation')
    shutil.copyfile(HERE/'navigation-source-fixture/core.rs',out/'core.rs')
    (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod core;\n',encoding='utf-8')
    (out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-core-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
    if result.returncode==0:
        env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target')
        result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
        if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
    files=[*original,Path(__file__),*sorted((HERE/'navigation-source-fixture').glob('*')),*sorted((ROOT/'native/liquidbounce-behaviors/src/navigation').glob('*.rs'))]
    report={'passed':len(records)==4 and all(item['exitCode']==0 for item in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'scope':'AStar and key timing source algorithms only','fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)

if __name__=='__main__':main()
