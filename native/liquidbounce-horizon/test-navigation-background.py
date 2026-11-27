#!/usr/bin/env python3
"""Unchanged Agent BackgroundSearch/Replanner with a controlled real Executor."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import struct
import subprocess
import time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'
def bits(value):return struct.unpack('<Q',struct.pack('<d',value))[0]
def vec(x,y=0.0,z=0.0):return ','.join(str(bits(v))for v in [float(x),float(y),float(z)])
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args()
 out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*p):commands.append(' '.join(map(str,p)))
 def request(key=1,program='S1,S0,F7',region='-2,-2,-2,2000,2,2'):add('request',key,region,program)
 def cycle(count=1):
  for _ in range(count):add('run');add('poll')
 def req(goal=4,start=0,radius=1,length=64,stage=32,variant='v',program='S1,F7',y=0):add('req',vec(start),vec(goal,y),bits(float(radius)),bits(float(length)),bits(float(stage)),variant,program)
 # A slice stays pending; same key ignores changed region/factory; null previews retain the last one.
 add('new','bg',8);request();add('poll');add('poll');request(program='X',region='0,0,0,0,0,0');cycle(3);request(program='A');add('poll');add('reset');add('run');add('poll')
 # Canceling the dependent exceptionally future does not cancel the queued supplyAsync.
 request(1,'X');add('poll');add('reset');request(2,'A');add('poll');cycle(2);add('level');add('poll')
 # Changed cached state starts over only at the twentieth poll; equal or unobserved state does not.
 add('new','bg',8);request(program='S1');add('poll');cycle(3);add('state','0,0,0',1);add('change','0,0,0');cycle(15);cycle(2);add('change','0,0,0');add('change','100,0,0');cycle(20)
 # No level preserves pending changes; returning level applies them. A completed job also restarts.
 add('new','bg',8);request(program='F9');add('poll');cycle();add('state','0,0,0',1);add('change','0,0,0');add('world','false');[add('poll')for _ in range(20)];add('world','true');add('poll');cycle();add('world','false');request(3);add('poll')
 # Exactly 1024 unrelated changes are processed; the 1025th overflows, clears them and forces restart.
 for count in [1024,1025]:
  add('new','bg',8);request(program='F1');add('poll');cycle();[add('change','100,0,0')for _ in range(count)];[add('poll')for _ in range(19)];cycle();add('reset')
 # Out of region changes and no-job changes are ignored. Worker exceptions become NO_PATH.
 add('new','bg',8);add('change','0,0,0');request(program='X',region='0,0,0,0,0,0');add('change','1,0,0');add('poll');cycle();add('level');add('poll')
 # Found/AlreadyThere/Failed are delivered once by Replanner; request history and retry time remain separate.
 for program in ['S1,F7','A','ENO_PATH','X']:
  add('new','rp',8);req(program=program);add('poll');cycle(2);add('query',vec(4));add('query',vec(5.499999));add('query',vec(5.5));add('end','4,0,0');add('poll');add('tick',5);add('reset');add('tick',14);req(program='A');add('poll');cycle();add('forget');add('query',vec(4));add('level');add('poll')
 # Staging: exact half range, unloaded destination, height bounds, goal reached vs stage endpoint.
 for goal,y,known in [(32,0,True),(31.999999,0,True),(4,0,False),(4,400,True),(100,0,True)]:
  add('new','rp',3);add('known',str(known).lower());req(goal=goal,y=y,program='F5');add('poll');cycle();add('end','4,0,0');add('end',f'{int(goal)},{y},0');add('query',vec(goal,y));req(goal=goal,y=y,program='X');req(goal=goal,y=y,variant='other',program='A');add('poll');cycle();add('level')
 # Same-key requests still refresh lastRequest, while no-level requests touch no replanner state.
 add('new','rp',8);req(program='S1');add('poll');add('tick',7);req(program='X');add('world','false');req(goal=100);add('reset');add('world','true');add('tick',5);req(program='ENO_PATH');add('poll');cycle();add('seed',2147483647,2147483644,0);add('tick',1);add('reset');add('seed',2147483647,2147483644,0);req(program='ENO_PATH');add('poll');cycle();add('tick',1)
 # Java data-class key semantics distinguish signed zero and canonicalize NaN radius values.
 for radius in [0.0,-0.0,float('nan'),float('nan')]:req(radius=radius,program='A');add('poll');cycle()
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/route'/n for n in ['BackgroundSearch.kt','Replanner.kt']]
 for p in original:shutil.copyfile(p,out/p.name)
 shutil.copyfile(HERE/'navigation-source-fixture/Background.kt',out/'Fixture.kt')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar'
 cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),next(kotlin.glob('slf4j-api-*.jar')),next((ROOT/'build/resource-runtime/libraries').rglob('fastutil-8.5.18.jar')),binary]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-11000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');shutil.copyfile(HERE/'navigation-source-fixture/background.rs',out/'navigation/background_oracle.rs')
 with(out/'navigation/replanner.rs').open('a',encoding='utf-8')as file:file.write('\n#[cfg(test)]#[path="background_oracle.rs"]mod background_oracle;\n')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-background-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,Path(__file__),HERE/'navigation-source-fixture/Background.kt',HERE/'navigation-source-fixture/background.rs',*sorted((ROOT/'native/liquidbounce-behaviors/src/navigation').glob('*.rs'))]
 report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'scope':'unchanged BackgroundSearch/Replanner; controlled Executor with cancellation, cache changes, 20-poll threshold, 1024 change overflow, preview, failure/retry/staging and wrapping ticks','boundaries':['published Terrain is used for state-only invalidation; block physics and native snapshots are not exercised','deterministic queue verifies scheduled worker lifecycle; real Rust thread integration remains separate','outer AutoWalk and movement/body controllers remain unintegrated'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
