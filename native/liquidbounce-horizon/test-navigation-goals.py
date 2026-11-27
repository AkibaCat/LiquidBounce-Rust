#!/usr/bin/env python3
"""Run unchanged destination/region source with the published Agent math/terrain jar."""
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
def bits(v):return struct.unpack('<Q',struct.pack('<d',v))[0]
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args()
 out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];rng=random.Random(260326);commands=[]
 def add(*parts):commands.append(' '.join(map(str,parts)))
 def vec(values):return [bits(v)for v in values]
 ints=[-2147483648,-33554433,-33554432,-4097,-2049,-1,0,1,2047,2048,33554431,33554432,2147483647]
 floats=[-1e308,-2147483649.0,-1024.5,-0.001,-0.0,0.0,0.001,0.5,1.0,63.999,64.0,1e308,float('inf'),float('-inf'),float('nan')]
 for i in range(1300):
  add('block',*[rng.choice(ints)if i%3==0 else rng.randint(-100000,100000)for _ in range(6)])
  a=[rng.choice(floats)if i%5==0 else rng.uniform(-10000,10000)for _ in range(3)];c=a if i%9==0 else [rng.choice(floats)if i%5==0 else rng.uniform(-10000,10000)for _ in range(3)]
  add('vec',*vec(a),*vec(c),bits(rng.choice(floats)if i%5==0 else rng.uniform(-10,10)))
  center=[rng.choice(floats)if i%7==0 else rng.uniform(-80,80)for _ in range(3)];radius=rng.choice(floats)if i%7==0 else rng.uniform(0,10)
  add('near',*vec(center),bits(radius),*[rng.choice(ints)if i%7==0 else rng.randint(-85,85)for _ in range(3)])
  positions=[tuple(rng.choice(ints)if i%11==0 else rng.randint(-200,200)for _ in range(3))for _ in range(rng.randint(1,6))]
  positions+=positions[:rng.randrange(len(positions))]
  add('blocks',';'.join(','.join(map(str,p))for p in positions),*[rng.choice(ints)if i%11==0 else rng.randint(-200,200)for _ in range(3)])
  start=[rng.choice(floats)if i%7==0 else rng.uniform(-300,300)for _ in range(3)];ends=[[rng.choice(floats)if i%7==0 else rng.uniform(-300,300)for _ in range(3)]for _ in range(rng.randrange(4))]
  add('region',*vec(start),bits(rng.choice(floats)if i%7==0 else rng.uniform(0,150)),rng.choice(ints)if i%7==0 else rng.randint(0,30),';'.join(','.join(map(str,vec(end)))for end in ends)or'-')
  center=[rng.uniform(-150,150)for _ in range(3)];start=[rng.uniform(-150,150)for _ in range(3)];query=[rng.randint(-150,150)for _ in range(3)];unknown=[tuple(query[j]+(offset if j==axis else 0)for j in range(3))for axis,offset in [(0,1),(0,-1),(2,1),(2,-1)]if rng.choice([False,True])]
  add('stage',*vec(center),bits(rng.uniform(-1,10)),*vec(start),bits(rng.uniform(-100,100)),*query,';'.join(','.join(map(str,p))for p in unknown)or'-',bits(rng.choice([float('inf'),20.0,100.0])))
 # Exact stage boundary and wrapped vertical subtraction.
 for y in [-2147483648,2147483647,-1,0,1]:add('near',*vec([0.5,float(-y),0.5]),bits(1.0),0,y,0)
 for x in [0,1,2,3,4,5]:add('stage',*vec([5.5,0.0,0.5]),bits(1.0),*vec([0.5,0.0,0.5]),bits(3.0),x,0,0,'-',bits(float('inf')))
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/route/Destination.kt',AGENT/'pathfinder/level/BlockRegion.kt']
 for p in original:shutil.copyfile(p,out/p.name)
 shutil.copyfile(HERE/'navigation-source-fixture/Goals.kt',out/'Goals.kt')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
 binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar'
 jars=[next(kotlin.glob('kotlin-stdlib-*.jar')),next((ROOT/'build/resource-runtime/libraries').rglob('fastutil-8.5.18.jar')),binary]
 cp=os.pathsep.join(map(str,jars))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-10000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.GoalsKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');shutil.copyfile(HERE/'navigation-source-fixture/goals.rs',out/'goals.rs')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod goals;\n',encoding='utf-8')
 (out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-goals-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target')
  result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,Path(__file__),HERE/'navigation-source-fixture/Goals.kt',HERE/'navigation-source-fixture/goals.rs',*sorted((ROOT/'native/liquidbounce-behaviors/src/navigation').glob('*.rs'))]
 report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'scope':'destination/region unchanged source and original published binary math/Terrain.isKnown; block state shell unused outside Terrain companion initialization','fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
