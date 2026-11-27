#!/usr/bin/env python3
"""Unchanged Agent Reach and flight route generation over an observable world seam."""
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
def fbits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def raw(values):return ','.join(str(bits(float(v)))for v in values)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*p):commands.append(' '.join(map(str,p)))
 for id,shape in [(0,'E'),(1,'F'),(2,raw([0,0,0,1,.5,1])),(3,raw([0,.5,0,1,1,1]))]:add('shapeDef',id,shape)
 for id,name,shape,fluid in [(0,'air',0,0),(1,'stone',1,0),(2,'void_air',0,0),(7,'fixture_slab',2,0),(8,'fixture_roof',3,0),(36,'water',0,1),(37,'lava',0,2)]:add('stateDef',id,name,0,fluid,-1,-1,-1,0,shape,shape,-1,-1)
 def new(floor=-1,width=struct.unpack('<f',struct.pack('<f',.6))[0],height=struct.unpack('<f',struct.pack('<f',1.8))[0],maximum=24):add('new','false',bits(width),bits(height),-8,maximum+9,bits(-20),bits(20),'false',floor,1)
 def setat(x,y,z,id):add('set',f'{x},{y},{z}',id)
 new()
 for speed in [.05,.1,0,-0.0,-.05,float('nan'),float('inf')]:
  for mayfly,flying in [(False,False),(False,True),(True,False),(True,True)]:add('creative',fbits(speed),str(mayfly).lower(),str(flying).lower())
 # Rays deliberately return known patterns; exact sampled endpoints, clip kind,
 # default fluid/entity and early return are observable. Ray traversal is a seam.
 target='0,0,0'
 for mode in range(5):
  for block in ['-',target]:
   for box,eyes,reach in [([0,0,0,1,1,1],[.5,.5,.5],0),([0,0,0,1,1,1],[-1,.5,.5],1),([0,0,0,1,1,1],[-1,.5,.5],2),([0,0,0,1,1,1],[1,1,1],2),([0,0,0,1,1,1],[.5,2,.5],2),([0,0,0,1,1,1],[.5,2,.5],float('nan')),([.5,.5,.5,.5,.5,.5],[.5,.5,.5],0),([-2,0,-1,-1,2,0],[0,1,0],3),([0,0,0,1,.01,1],[-1,.1,.5],2)]:
    add('ray',mode,target,17);add('sees',raw(box),bits(reach),raw(eyes),block)
 for support in [True,False]:
  for mode,block in [(0,'-'),(1,'0,0,0'),(2,'0,0,0'),(3,'0,0,0'),(4,'-')]:
   new();setat(0,0,0,1);setat(1,0,0,7);add('ray',mode,target,23)
   add('reach',raw([0,0,0,1,1,1]),bits(1.5),raw([-.5,0,.5]),block,'-',str(support).lower())
 for reach in [-1,0,.5,1.0,float('nan')]:
  new(floor=-8);add('ray',0,target,0);add('reach',raw([0,0,0,1,1,1]),bits(reach),raw([0,0,0]),'-',bits(.4),'false')
 # Flight neighbor ordering, corner blocking, water factor and ground penalty.
 def fly(control='M',start=(.5,0,.5),kind='B',goal='3,0,0',radius=0,length=32,visited=2000,slice=16,nanos=9223372036854775807,hspeed=.7,vspeed=.375):add('fly',control,bits(hspeed),bits(vspeed),'true',raw(start),kind,goal,bits(radius),bits(length),visited,slice,nanos)
 for control in ['M','D']:
  for obstacle in [None,(1,0,0),(0,1,0),(1,1,1)]:
   new()
   if obstacle:setat(*obstacle,1)
   setat(-1,0,0,36);setat(0,1,-1,36);fly(control=control)
   for node in ['0,0,0','0,1,0','0,0,0','1,0,0']:add('moves',node)
 for hspeed,vspeed in [(0,0),(-.1,.2),(.2,-.1),(float('nan'),.2),(float('inf'),.1)]:
  new();fly(hspeed=hspeed,vspeed=vspeed);add('moves','0,0,0')
 # Straight line samples, 0.5 endpoint margin threshold and FMA interpolation.
 for obstacle in [None,(1,0,0),(1,1,0),(1,2,0)]:
  new()
  if obstacle:setat(*obstacle,1)
  for end in [[.5,0,.5],[.75,0,.5],[1,0,.5],[1.25,0,.5],[1.5,0,.5],[2.5,0,.5],[2.5,1,1.5],[.5,2,.5]]:add('line',raw([.5,0,.5]),raw(end))
 def advance(n=8):
  add('preview')
  for _ in range(n):add('advance')
 # Normal direct success / already there / blocked feet that fit above.
 new();fly();advance(2)
 new();fly(goal='0,0,0');advance(2)
 new();setat(0,0,0,1);fly(goal='0,1,0');advance(2)
 # Small wall: A* route, forced climb fallback and timeout on the begin slice.
 for visited,slice,nanos,roof in [(2000,5,9223372036854775807,False),(1,1,9223372036854775807,False),(20,2,9223372036854775807,True),(2000,5,0,False),(0,1,9223372036854775807,False)]:
  new()
  for x in [1,2]:
   for y in [0,1,2]:
    for z in [-1,0,1]:setat(x,y,z,1)
  if roof:
   for x in range(-2,5):
    for z in range(-2,3):setat(x,3,z,1)
  fly(visited=visited,slice=slice,nanos=nanos);advance(25)
 # No climb exists: enclosed start; no-goal-end heuristic fallback.
 new();setat(0,0,0,1);setat(0,1,0,1);fly();advance(3)
 new();fly(kind='G',goal='3,0,0');add('h','0,0,0');advance(8)
 new();fly(kind='N',goal=raw([2.9,2,.3]),radius=.7,start=(.5,2,.5));add('h','0,2,0');advance(3)
 new();fly(goal='3,0,0;0,0,3');add('h','0,0,0');add('h','1,1,1');advance(3)
 new();fly(length=3);advance(8)
 new();fly(hspeed=-.1);setat(1,0,0,1);advance(3)
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/name for name in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt','Reach.kt']]+[AGENT/'pathfinder/flight'/name for name in ['FlyAbilities.kt','Moves.kt','RouteSearch.kt']]+[AGENT/'pathfinder/route'/name for name in ['AStar.kt','AStarSearch.kt','Limits.kt','Outcome.kt','Failure.kt','Search.kt']]
 for i,file in enumerate(original):shutil.copyfile(file,out/f'{i:02}_{file.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8');extra=(HERE/'navigation-source-fixture/AirReach.kt').read_text(encoding='utf-8');imports,code=extra.split('// fixture code\n',1)
 base=base.replace('package fixture','package fixture\n'+imports).replace('private class World:LevelBlocks{','private class World:LevelBlocks{\n override fun clip(context:ClipContext)=rayClip(context)').replace('fun main(){',code+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)')
 (out/'Fixture.kt').write_text(base,encoding='utf-8')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8');extra_rs=(HERE/'navigation-source-fixture/air_reach.rs').read_text(encoding='utf-8');fixture=fixture.replace('#[test]',extra_rs+'\n#[test]',1).replace('let mut results = Vec::new();','let mut fly_search:Option<AirSearch>=None;let mut fly_moves:Option<AirMoves>=None;let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut fly_search,&mut fly_moves,expected),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_reach_and_full_flight_route_search')
 (out/'air_reach_oracle.rs').write_text(fixture,encoding='utf-8')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-16000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').glob('*.rs')}
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod air_reach_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-air-reach-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Terrain.kt','terrain.rs','AirReach.kt','air_reach.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged Terrain/Reach/flight/AStar sources; ray sample endpoints/order/limits, support/air reach positions and stable ties, flight abilities/26 neighbors/costs, full sliced routes and previews, climb fallback, simplification and budget failures','boundaries':['world/registry are controlled adapter seams; ray clip returns observable deterministic answers and does not validate native ray traversal','JVM computeNanos is measured and fed to the Rust clock seam','flight route generation only; flight input/controller/native body are not yet integrated'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
