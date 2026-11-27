#!/usr/bin/env python3
"""Unchanged sliced A* source; measured JVM nanoseconds supplied to the Rust clock seam."""
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
 out.mkdir(parents=True);records=[];commands=[];rng=random.Random(260350)
 # Real search context before relaxation, duplicate/stale entries, cheaper
 # predecessors and zero-cost cycles; the callback logs every current cost.
 for edges,maximum in [([(0,1,5),(0,1,2),(0,2,1),(2,1,.5),(1,3,10),(2,3,20)],float('inf')), ([(0,1,4),(0,2,1),(2,1,1),(1,2,0)],5), ([(0,1,-0.0),(1,0,0),(1,2,0),(2,3,1)],float('inf'))]:
  commands.append(' '.join(map(str,['graph',4,0,3,bits(maximum),','.join(str(bits(0))for _ in range(4)),';'.join(f'{a},{b},{bits(c)}'for a,b,c in edges),100,1,9223372036854775807,0,0,'false'])))
  commands.extend(['advance']*12)
 for i in range(520):
  count=rng.randrange(2,30);h=[rng.choice([0.0,-0.0,0.25,0.5,1.0,1.5,2.0,100.0,float('inf'),float('nan')])for _ in range(count)]
  edges=[(rng.randrange(count),rng.randrange(count),rng.choice([0.0,-0.0,0.25,0.5,1.0,3.0,float('inf')]))for _ in range(rng.randrange(count,count*5))]
  if i%17==0:edges.insert(0,(0,1,float('nan')))
  if i%19==0:edges.insert(0,(0,1,-1.0))
  commands.append(' '.join(map(str,['graph',count,0,count-1,bits(rng.choice([0.0,1.0,5.0,float('inf')])),','.join(str(bits(v))for v in h),';'.join(f'{a},{b},{bits(c)}'for a,b,c in edges),rng.choice([-2147483648,-1,0,1,2,10,100,2147483647]),rng.choice([-2147483648,-1,0,1,2,10,100,2147483647]),rng.choice([-1,0,150000000,9223372036854775807]),rng.choices([0,1,2,3],[10,1,1,1])[0],rng.choices([0,1,2],[10,1,1])[0],str(rng.choice([False,True])).lower()])))
  if i%6==0:commands.append('seed '+str(rng.choice([9223372036854775807-100,-9223372036854775808,150000000,-1])))
  commands.extend(['advance']*12)
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/route'/name for name in ['AStar.kt','AStarSearch.kt','Limits.kt','Outcome.kt','Failure.kt','Search.kt']]
 for path in original:shutil.copyfile(path,out/('Agent'+path.name if path.name=='Search.kt'else path.name))
 shutil.copyfile(HERE/'navigation-source-fixture/Search.kt',out/'Fixture.kt')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar'
 cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),next((ROOT/'build/resource-runtime/libraries').rglob('fastutil-8.5.18.jar')),binary]))
 def run(name,argv,env=None):
  began=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
  if result.returncode:print((result.stdout+result.stderr)[-11000:])
  return result
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');shutil.copyfile(HERE/'navigation-source-fixture/search.rs',out/'navigation/search_oracle.rs')
 with (out/'navigation/search.rs').open('a',encoding='utf-8')as file:file.write('\n#[cfg(test)]#[path="search_oracle.rs"]mod search_oracle;\n')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;\n',encoding='utf-8')
 (out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-search-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target')
  result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,Path(__file__),HERE/'navigation-source-fixture/Search.kt',HERE/'navigation-source-fixture/search.rs',*sorted((ROOT/'native/liquidbounce-behaviors/src/navigation').rglob('*.rs'))]
 report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'scope':'six unchanged Agent search sources, including exact cost/predecessor context observed inside each neighbor callback before relaxation, duplicate/stale entries, slice budgets, measured JVM CPU elapsed, nanosecond overflow, failure precedence, repeated results and closest progress','boundaries':['clock fixture reads actual private JVM computeNanos with reflection and supplies its elapsed delta to Rust','outer navigator, route generation, terrain simulation and native body remain unintegrated'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
