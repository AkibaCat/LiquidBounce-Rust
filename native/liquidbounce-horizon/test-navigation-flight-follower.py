#!/usr/bin/env python3
"""Unchanged Agent flight Follower/captureFlight/ProgressWatch source comparison."""
from pathlib import Path
import argparse
import hashlib
import json
import math
import os
import shutil
import struct
import subprocess
import time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'
def bits(value):return struct.unpack('<Q',struct.pack('<d',float(value)))[0]
def fbits(value):return struct.unpack('<I',struct.pack('<f',value))[0]
def raw(value):return ','.join(str(bits(v))for v in value)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);commands=[];records=[]
 def add(*p):commands.append(' '.join(map(str,p)))
 def base(control='DIRECT',speed=.6,vertical=.4,flying=True):add('base',control,bits(speed),bits(vertical),str(flying).lower())
 def host(flying=False,can=True,collision=True):add('host',str(flying).lower(),str(can).lower(),str(collision).lower())
 def player(tick=0,position=(0,0,0),delta=(0,0,0),yaw=0,sprint=False,horizontal=False,vertical=False,ground=False,creative=False,mayfly=False,speed=.05):
  add('player',tick,raw(position),raw(delta),fbits(yaw),*[str(x).lower()for x in [sprint,horizontal,vertical,ground,creative,mayfly]],fbits(speed))
 def follow(points,reach=True):add('follow',raw(points[0]),';'.join(map(raw,points)),str(reach).lower())
 def tick(frame=0):add('tick',fbits(frame))
 def new(control='DIRECT',speed=.6,vertical=.4,flying=True):add('new');host();base(control,speed,vertical,flying)
 # Every flight availability branch and getter short circuit, then speed changes.
 for enabled in [False,True]:
  for flying in [False,True]:
   for creative in [False,True]:
    for mayfly in [False,True]:
     add('new');host(flying,enabled);player(creative=creative,mayfly=mayfly);add('capture');player(creative=creative,mayfly=mayfly,speed=.1);add('capture')
 # No route, single point, progress display and interpolated head.
 new();player();tick();add('off');follow([(0,0,0)]);tick();add('clear')
 player(position=(3,2,1),delta=(.1,.2,.3))
 for partial in [-1,0,.5,1,1.5]:add('head',fbits(partial))
 # Takeoff alternates jump and preserves wrapping Int behavior.
 new(flying=False);follow([(0,0,0),(10,0,0)])
 for t in range(8):player(t);tick()
 add('seed',2147483647);tick();tick();add('clear')
 # Full path transitions for direct and coasting flights: straight, sharp, shallow and repeated points.
 for control in ['DIRECT','MOMENTUM']:
  for speed in [.1,.6,2.5]:
   for points in [[(0,0,0),(4,0,0),(8,0,0)],[(0,0,0),(4,0,0),(4,0,4)],[(0,0,0),(4,1,0),(8,1,1)],[(0,0,0),(0,0,0),(4,0,0)]]:
    new(control,speed);follow(points)
    for i,pos in enumerate([(0,0,0),(.2,0,0),(1.9,0,0),(2,0,0),(3.5,0,0),(4,0,0),(4,0,2),(7.5,1,1),(8,1,1)]):player(i*3,pos);tick();add('off')
 # Exact off-path thresholds and repeated zero-length segments.
 for speed in [.1,2]:
  new(speed=speed);follow([(0,0,0),(10,0,0)])
  for z in [0,2.5,2.50000001,4,4.00000001]:player(position=(5,0,z));add('off')
 # Creative landing, toggling flight, falling, touching ground and the retained hasLanded flag.
 for collision in [False,True]:
  for height in [-.01,0,.8,1,1.000001]:
   new('MOMENTUM',.2);host(collision=collision);follow([(.5,height,.5),(.5,0,.5)]);player(0,(.5,height,.5),creative=True);tick();player(1,(.5,height,.5),creative=True);tick();base('MOMENTUM',.2,.4,False);player(2,(.5,height,.5));tick();player(3,(.5,0,.5),ground=True);tick();follow([(.5,0,.5),(5,0,.5)]);tick();add('clear');tick()
 # Vertical steering includes leaving a ledge, a sagging horizontal collision and measured rest motion.
 for control in ['DIRECT','MOMENTUM']:
  for ground in [False,True]:
   for y in [-2,-.8,-.2,0,.2,.8,2]:
    new(control,.6);follow([(0,0,0),(3,y,0),(7,y,0)]);player(0,ground=ground,horizontal=True);tick();player(1,ground=ground,horizontal=True);tick()
 # Consecutive observed motion updates handling while the follower keeps pressing exact keys.
 new('MOMENTUM',1.1);follow([(0,0,0),(10,1,0),(10,1,10)])
 for t,pos in enumerate([(0,0,0),(.1,.1,0),(.3,.2,.1),(.5,.3,.2),(1,.5,.4),(2,.7,.6),(3.5,.8,.8),(6,.9,.9)]):player(t,pos,yaw=33.25,sprint=t%2==0);tick(33.25)
 # Stalling forgives the leg after exactly 60 idle ticks, and three misses give it up.
 new();follow([(0,0,0),(10,0,0)])
 for t in range(65):player(t*3);tick()
 new();follow([(0,0,0),(10,0,0)])
 for i,x in enumerate([9.2,7.5,9.2,7.5,9.2,7.5]):player(i*3,(x,0,0));tick()
 # Direction dead angles and float wrapping, including no direction.
 for direction in [(0,0,0),(0,1,0),(1,0,0),(-1,0,0),(0,0,1),(0,0,-1),(1,0,1),(-1,0,1)]:
  for frame in [-540,-180,-160,-110,-90,-70,-20,-19.999,0,19.999,20,70,90,110,160,180,540]:add('direction',raw(direction),fbits(frame))
 for points in [[(0,0,0),(1,0,0),(2,0,0)],[(0,0,0),(1,0,0),(1,0,1)],[(0,0,0),(1,0,0),(1.7,0,math.sqrt(.51))],[(0,0,0),(0,0,0),(1,0,0)]]:
  for corner in [0,1,2,3]:add('sharp',';'.join(map(raw,points)),corner)
 for point in [(0,0,0),(-1,2,.3),(.5,1,.5),(2,2,2)]:
  for end in [(0,0,0),(.003,0,0),(.0032,0,0),(1,0,0),(1,2,3)]:add('segment',raw(point),raw((0,0,0)),raw(end))
 # The shared progress watch retains strict improvement, reset/excuse and wrapping ticks.
 for minimum in [0,.1,-.1,float('nan')]:
  for stuck in [0,1,3,60]:
   add('watch',bits(minimum),stuck)
   for distance in [float('inf'),float('nan'),10,9.9,9.899999999,9.8,9.8,10,8]:add('stalls',bits(distance))
   add('excuse');add('stalls',bits(8));add('watchseed',2147483647);add('stalls',bits(100));add('reset');add('stalls',bits(0))
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'navigation/flight'/n for n in ['Handling.kt','Follower.kt','FlightCapture.kt']]+[AGENT/'navigation'/n for n in ['ProgressWatch.kt','FollowState.kt']]+[AGENT/'pathfinder/flight/FlyAbilities.kt']+[AGENT/'math'/n for n in ['InputExtensions.kt','VectorExtensions.kt']]
 for i,p in enumerate(original):shutil.copyfile(p,out/f'{i:02}_{p.name}')
 shutil.copyfile(HERE/'navigation-source-fixture/FlightFollower.kt',out/'Fixture.kt');shutil.copyfile(HERE/'navigation-source-fixture/FlightContext.kt',out/'Context.kt')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-15000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')}
 shutil.copyfile(HERE/'navigation-source-fixture/flight_follower.rs',out/'navigation/flight_follower_oracle.rs')
 with(out/'navigation/flight_follower.rs').open('a',encoding='utf-8')as file:file.write('\n#[cfg(test)]#[path="flight_follower_oracle.rs"]mod flight_follower_oracle;\n')
 with(out/'navigation/progress.rs').open('a',encoding='utf-8')as file:file.write('\n#[cfg(test)]pub(crate) fn fixture_seed_idle(w:&mut ProgressWatch,n:i32){w.idle_ticks=n;}\n')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-flight-follower-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['FlightFollower.kt','FlightContext.kt','flight_follower.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'complete unchanged flight Follower/captureFlight/ProgressWatch with exact internal leg state, outputs and ordered Player/Abilities/Body/collision/press calls','boundaries':['NavigationContext is a minimal real Body/options holder for this isolated unit','native Player/level collision remain observable adapter seams','not the flight orchestration or full AutoWalk'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
