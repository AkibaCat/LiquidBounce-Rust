#!/usr/bin/env python3
"""Unchanged Agent Handling measurements and exact live Player getter order."""
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
def fbits(value):return struct.unpack('<I',struct.pack('<f',value))[0]
def raw(value):return ','.join(str(bits(float(v)))for v in value)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);commands=[];records=[]
 def add(*parts):commands.append(' '.join(map(str,parts)))
 def base(control='MOMENTUM',speed=1.1,vertical=.375,flying=True):add('base',control,bits(speed),bits(vertical),str(flying).lower())
 def player(tick,position,yaw=0,sprint=False,horizontal=False,vertical=False,delta=(.2,.3,.4)):
  add('player',tick,raw(position),raw(delta),fbits(yaw),str(sprint).lower(),str(horizontal).lower(),str(vertical).lower())
 def record(tick,position,mask=0,**kwargs):player(tick,position,**kwargs);add('record','-'if mask is None else mask)
 # Uncaptured defaults, fallback movement and original NaN/reset and min-speed rules.
 add('new');add('rest')
 for control in ['DIRECT','MOMENTUM']:
  for speed,vertical in [(1.1,.375),(0.,0.),(-1.,-2.),(-0.,-0.),(float('nan'),.2),(.2,float('nan')),(float('inf'),float('-inf'))]:
   base(control,speed,vertical);add('rest');base(control,speed,vertical,False)
 # All seven key bits; turn projection, diagonals, cancellation and vertical held-key gating.
 for yaw in [0.,90.,-45.,33.25,180.,45.,135.,270.,-179.999]:
  for mask in range(128):
   add('new');base('MOMENTUM'if mask&1 else'DIRECT');record(10,(0,0,0),mask,yaw=yaw,sprint=bool(mask&1));record(11,(.75,.25,.5),mask,yaw=yaw,sprint=bool(mask&1));record(12,(1.2,.65,1.1),mask,yaw=yaw,sprint=bool(mask&1));add('rest')
 # Collision gates, missing/changed keys, duplicate/skipped ticks and wrapping Int ticks.
 for collision in ['none','horizontal','vertical','both']:
  add('new');base();record(0,(0,0,0),68);record(1,(.5,.25,.5),68)
  for mask in [68,64,66,70,None,68,68,68]:
   tick=2+commands.count('rest')%3
   record(tick,(1,.5,1),mask,horizontal=collision in ['horizontal','both'],vertical=collision in ['vertical','both']);add('rest')
 for ticks in [[1,2,2,3,5,6,7],[2147483646,2147483647,-2147483648,-2147483647]]:
  add('new');base()
  for i,tick in enumerate(ticks):record(tick,(i*.3,i*.2,i*.4),68);add('rest')
 # Exact movement and drag thresholds, teleport rejection, stopped and reversed motion.
 for before,after in [((.02,.02,0),(.01,.01,0)),((.0200000001,.0200000001,0),(.01,.01,0)),((1,1,0),(-1,-1,0)),((1,1,0),(2,2,0)),((1,1,0),(.95,.95,0)),((1,1,0),(.950000001,.950000001,0)),((1,1,0),(0,0,0)),((1,1,0),(9.9,0,0)),((1,1,0),(10,0,0)),((1,1,0),(10.1,0,0)),((0,0,0),(0,0,0))]:
  for mask in [0,64,68,66,70]:
   add('new');base();record(0,(0,0,0),mask);record(1,before,mask);record(2,tuple(a+b for a,b in zip(before,after)),mask);add('rest')
 # Changes of base values clear measurements but preserve recorded movement/keys.
 add('new');base();record(0,(0,0,0),68);record(1,(.7,.2,.4),68);record(2,(1.1,.4,.8),68)
 for control,speed,vertical in [('MOMENTUM',1.1,.375),('MOMENTUM',1.2,.375),('DIRECT',1.2,.375),('DIRECT',1.2,.4),('DIRECT',float('nan'),.4),('DIRECT',float('nan'),.4)]:base(control,speed,vertical);add('rest')
 # Repeated samples blend, first drag measurement discards prior thrust, and sprint shares fallback.
 add('new');base();record(0,(0,0,0),64);record(1,(.3,.2,0),64)
 for tick,position,mask,sprint in [(2,(.6,.4,0),64,False),(3,(.9,.6,0),65,True),(4,(1.1,.7,0),0,False),(5,(1.2,.75,0),68,False),(6,(1.4,.8,0),68,False),(7,(1.7,.9,0),69,True),(8,(1.85,.95,0),0,False)]:record(tick,position,mask,sprint=sprint);add('rest')
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'navigation/flight/Handling.kt',AGENT/'pathfinder/flight/FlyAbilities.kt']
 for p in original:shutil.copyfile(p,out/p.name)
 shutil.copyfile(HERE/'navigation-source-fixture/FlightHandling.kt',out/'Fixture.kt')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-14000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')}
 shutil.copyfile(HERE/'navigation-source-fixture/flight_handling.rs',out/'navigation/flight_handling_oracle.rs')
 with(out/'navigation/flight_handling.rs').open('a',encoding='utf-8')as file:file.write('\n#[cfg(test)]#[path="flight_handling_oracle.rs"]mod flight_handling_oracle;\n')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-flight-handling-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,ROOT/"upstream/Horizon/crates/horizon-core/src/jmath.rs",ROOT/"upstream/Horizon/LICENSE",Path(__file__),HERE/'navigation-source-fixture/FlightHandling.kt',HERE/'navigation-source-fixture/flight_handling.rs'];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged Handling and FlyAbilities source, every input mask, directional projection, blend/reset/NaN/thresholds/collision/tick wrapping and exact live Player getter order','boundaries':['live native Player remains an explicit adapter seam','not the flight controller or full AutoWalk'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
