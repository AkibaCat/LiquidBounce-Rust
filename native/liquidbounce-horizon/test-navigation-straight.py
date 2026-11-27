#!/usr/bin/env python3
"""Unchanged StraightWalk sampling and deadly-side checks over actual Terrain."""
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
def bits(v):return struct.unpack('<Q',struct.pack('<d',v))[0]
def raw(v):return ','.join(str(bits(float(x)))for x in v)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*p):commands.append(' '.join(map(str,p)))
 for id,shape in [(0,'E'),(1,'F'),(2,raw([0,0,0,1,.5,1])),(3,raw([0,.5,0,1,1,1]))]:add('shapeDef',id,shape)
 for id,name,shape,flags,fluid in [(0,'air',0,0,0),(1,'stone',1,0,0),(2,'void_air',0,0,0),(7,'fixture_slab',2,0,0),(8,'fixture_roof',3,0,0),(18,'fixture_fire',0,32,0),(36,'water',0,0,1)]:add('stateDef',id,name,flags,fluid,-1,-1,-1,0,shape,shape,-1,-1)
 def new(floor=-1):add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-16,64,bits(-32),bits(32),'false',floor,1)
 def setat(x,y,z,id):add('set',f'{x},{y},{z}',id)
 def straight(start=(.5,0,.5),end=(4.5,0,.5),stop=.3,safe=3):add('straight',raw(start),raw(end),bits(stop),safe)
 new()
 for end in [(.5,0,.5),(.6,0,.5),(1,0,.5),(1.000000000001,0,.5),(1.5,0,.5),(4.5,0,.5),(4.5,0,4.5),(4.5,1,.5)]:
  for stop in [0,.3,.5,1,-.5,float('nan')]:straight(end=end,stop=stop)
 for layout in ['wall','roof','slabUp','slabDown','oneUp','gap','water','hazard','unknown','deadlySide','waterSide','lowerSide']:
  new()
  if layout=='wall':setat(2,0,0,1)
  if layout=='roof':setat(2,1,0,8)
  if layout=='slabUp':setat(2,0,0,7)
  if layout=='slabDown':setat(2,-1,0,7)
  if layout=='oneUp':setat(2,0,0,1)
  if layout=='gap':setat(2,-1,0,0)
  if layout=='water':setat(2,0,0,36)
  if layout=='hazard':setat(2,0,0,18)
  if layout=='unknown':add('known','2,0,1','false')
  if layout in ['deadlySide','waterSide','lowerSide']:
   setat(2,-1,1,0)
   if layout=='waterSide':setat(2,-2,1,36)
   if layout=='lowerSide':setat(2,-3,1,1)
  for safe in [0,1,3,4]:straight(safe=safe);straight(end=(1.5,0,.5),safe=safe)
 # Initial side checks precede even zero-length and very short paths.
 for side in [(1,0,0),(-1,0,0),(0,0,1),(0,0,-1)]:
  new();add('known',','.join(map(str,side)),'false');straight(end=(.5,0,.5));straight(end=(.6,0,.5),stop=1)
 # Clearance at off-center feet, diagonal samples and a higher starting slab.
 for start in [(.2,0,.2),(.8,0,.8),(-.5,0,-.5),(.5,.5,.5)]:
  new()
  if start[1]:setat(0,0,0,7)
  setat(1,0,1,7);straight(start=start,end=(2.5,0,2.5));straight(start=start,end=(2.5,.5,2.5),stop=.6)
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/n for n in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[AGENT/'pathfinder/walk/StraightWalk.kt']
 for i,p in enumerate(original):shutil.copyfile(p,out/f'{i:02}_{p.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8').replace('package fixture','package fixture\nimport net.ccbluex.agent.pathfinder.walk.isStraightWalkable').replace('else->error(command)','"straight"->result=isStraightWalkable(terrain,v(a[1]),v(a[2]),d(a[3]),a[4].toInt()).toString()\nelse->error(command)')
 (out/'Fixture.kt').write_text(base,encoding='utf-8')
 rust=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8').replace('_ => panic!("{command}"),','"straight" => result=crate::navigation::walk::straight_walk::is_straight_walkable(t,v(a[1]),v(a[2]),d(a[3]),a[4].parse().unwrap()).to_string(),\n_ => panic!("{command}"),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_straight_walk_and_exact_sample_queries')
 (out/'straight_oracle.rs').write_text(rust,encoding='utf-8')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-12000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')}
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod straight_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-straight-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),HERE/'navigation-source-fixture/Terrain.kt',HERE/'navigation-source-fixture/terrain.rs'];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged StraightWalk plus actual Terrain sources; exact sample/side/drop/clearance order, endpoint distance, zero length and step transitions','boundaries':['world/registry remain controlled adapter seams','not the full walk search or controller'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
