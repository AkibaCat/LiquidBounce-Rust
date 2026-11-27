#!/usr/bin/env python3
"""Full unchanged walk Moves/Parkour/RouteSearch with actual jump simulation."""
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
def raw(v):return ','.join(str(bits(float(x)))for x in v)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*p):commands.append(' '.join(map(str,p)))
 for id,shape in [(0,'E'),(1,'F'),(2,raw([0,0,0,1,.5,1])),(3,raw([0,.5,0,1,1,1])),(4,raw([0,0,0,.125,1,1])),(5,raw([.875,0,0,1,1,1])),(6,raw([0,0,0,1,.125,1])),(7,raw([0,0,0,1,1.25,1])),(8,raw([0,0,0,1,.3125,1,0,0,0,.125,1,1,.875,0,0,1,1,1,0,0,0,1,1,.125,0,0,.875,1,1,1])),(9,raw([0,0,0,1,1,.1875]))]:add('shapeDef',id,shape)
 def state(id,name,shape=0,flags=0,fluid=0,friction=.6,speed=1,jump=1,replaced=False,opened=-1,half=-1,facing=-1,opened_shape=None,pair=-1,bounce=0,fall=0):add('stateDef',id,name,flags,fluid,opened,half,facing,fbits(bounce),shape,shape if opened_shape is None else opened_shape,pair,-1,fbits(friction),fbits(speed),fbits(jump),str(replaced).lower(),fbits(fall))
 state(0,'air',replaced=True);state(1,'stone',1);state(2,'void_air',replaced=True);state(7,'fixture_slab',2);state(8,'fixture_roof',3);state(9,'fixture_low_roof',6);state(10,'fixture_tall',7);state(11,'ladder',4,16,facing=2);state(12,'vine',5);state(14,'fixture_bowl',8);state(15,'fixture_falling',1,64);state(16,'slime_block',1,128,bounce=1);state(17,'fixture_replaced',0,replaced=True);state(18,'fixture_hazard',0,32);state(19,'fixture_slow',1,friction=.98,speed=.4);state(20,'honey_block',1,speed=.4,jump=.5,fall=.8);state(36,'water',0,fluid=1);state(37,'lava',0,fluid=2)
 for id,opened,half in [(40,0,0),(41,1,0),(42,0,1),(43,1,1)]:state(id,'oak_door',9,3,opened=opened,half=half,opened_shape=4,pair=id+1 if not opened else id-1)
 def new(floor=-1,floor_state=1,opens=False,legacy=False):add('new',str(opens).lower(),bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-8,33,bits(-16),bits(16),str(legacy).lower(),floor,floor_state)
 def setat(x,y,z,id):add('set',f'{x},{y},{z}',id)
 def ability(can_break=False,place=False,count=64,parkour=False,risky=False,sprint=True,safe=3,water=16,avoid='-',air=300,speed=.1,jump=.42,boost=0,jump_climbs=True,bridge=20,unbreakable='-',health=20,reserved='-'):add('ability',str(can_break).lower(),str(place).lower(),count,str(parkour).lower(),str(risky).lower(),str(sprint).lower(),safe,water,avoid,air,fbits(speed),fbits(jump),fbits(boost),str(jump_climbs).lower(),bits(bridge),unbreakable,fbits(health),reserved)
 def moves(start='0,0,0',target='-'):add('moves',start,target)
 def path(start=(.5,0,.5),kind='B',goal='3,0,0',radius=0,length=32,visited=2000,slice=16,nanos=9223372036854775807):add('path',raw(start),kind,goal,bits(radius),bits(length),visited,slice,nanos)
 def advance(count=10):add('preview');[add('advance')for _ in range(count)]
 # Complete simple branches in actual emission order, with selected Step callbacks.
 for scene in ['flat','half','rise','honey','slow','bowl','crouch','head','lower','twoHigh','wet','hazard']:
  for breaking,placing in [(False,False),(True,True)]:
   new();ability(can_break=breaking,place=placing)
   if scene=='half':setat(1,0,0,7)
   if scene=='rise':setat(1,0,0,1)
   if scene=='honey':setat(0,-1,0,20);setat(1,0,0,1)
   if scene=='slow':setat(0,-1,0,19);setat(1,-1,0,19)
   if scene=='bowl':setat(0,0,0,14)
   if scene=='crouch':setat(0,1,0,8)
   if scene=='head':setat(0,2,0,1);setat(1,0,0,1)
   if scene=='lower':setat(1,-1,0,0);setat(1,-2,0,1)
   if scene=='twoHigh':setat(1,0,0,1);setat(1,1,0,1)
   if scene=='wet':setat(0,0,0,36)
   if scene=='hazard':setat(1,0,0,18)
   moves();add('step','0,0,0','1,0,0');add('step','0,0,0','1,1,0');add('step','0,0,0','0,1,0');moves()
 # World changes are fresh captures; doors are opened by lower-half identity.
 for opens in [False,True]:
  new(opens=opens);setat(1,0,0,40);setat(1,1,0,42);ability(can_break=True);moves();add('step','0,0,0','1,0,0')
 # Floor placement support, two-block stacks, reserved cells and small stock.
 for count,reserved,bridge in [(0,'-',20),(1,'-',20),(2,'-',3),(64,'1,-1,0',20),(64,'-',3)]:
  new(floor=-7);setat(0,-1,0,1);ability(place=True,count=count,reserved=reserved,bridge=bridge);moves();add('step','0,0,0','1,0,0');add('step','0,0,0','1,1,0')
 # Climb and bottom-of-ladder fall; pre-1.14 climb-side requirement.
 for legacy,jump_climbs,grip in [(False,True,False),(True,False,False),(True,False,True)]:
  new(floor=-5,legacy=legacy);setat(0,0,0,12);setat(0,1,0,12)
  if grip:setat(1,0,0,1)
  ability(jump_climbs=jump_climbs);moves();add('step','0,0,0','0,1,0');add('step','0,0,0','0,-4,0')
 for sprint in [False,True]:
  new();setat(0,0,0,36);setat(1,0,0,36);setat(1,1,0,8);ability(sprint=sprint);moves();add('step','0,0,0','1,0,0')
 # Downward mines the floor only with its own lower support; finite callback cost.
 for unbreakable,ticks in [('-',1),('0,-1,0',1),('-',float('inf')),('-',-.5)]:
  new();setat(0,-2,0,1);ability(can_break=True,unbreakable=unbreakable);add('ticks',1,bits(ticks));moves();add('step','0,0,0','0,-1,0')
 add('ticks',1,bits(1))
 # Goal, adjusted starts on partial blocks, and footprint-held bridge starts.
 for start,goal,layout in [((.5,0,.5),'0,0,0','flat'),((.5,0,.5),'3,0,0','flat'),((.5,.5,.5),'3,0,0','slab'),((.5,1,.5),'3,0,0','below'),((1.05,0,.5),'3,0,0','held')]:
  new(floor=-7 if layout=='held'else -1)
  if layout=='slab':setat(0,0,0,7)
  if layout=='held':setat(0,-1,0,1)
  ability(place=layout=='held');path(start=start,goal=goal);advance(8)
 # Mining and detours, avoids, budgets, unloaded/no-path and incomplete bridges.
 for breaking,avoid in [(False,'-'),(True,'-'),(True,'1,0,0')]:
  new()
  for z in [-1,0,1]:setat(1,0,z,1);setat(1,1,z,1)
  ability(can_break=breaking,avoid=avoid);path();advance(12)
 for count in [0,1,2,4]:
  new(floor=-7);setat(0,-1,0,1);ability(place=True,count=count);path(goal='4,0,0',visited=200,slice=20);advance(14)
 for visited,slice,nanos in [(1,1,9223372036854775807),(2000,16,0),(0,1,9223372036854775807),(2000,0,9223372036854775807)]:
  new();ability();path(visited=visited,slice=slice,nanos=nanos);advance(5)
 new();ability();path(kind='N',goal=raw([3.3,0,.2]),radius=.7);advance(8)
 new();ability();path(kind='G',goal='3,0,0');advance(8)
 # Exhausted air allows only upward submerged moves, with real predecessors.
 for air in [0,19,300]:
  new()
  for x in range(-1,5):
   for z in [-1,0,1]:setat(x,0,z,36);setat(x,1,z,36)
  ability(air=air);path(goal='3,0,0',visited=150,slice=10);advance(14)
 # Full Parkour: candidate query/plan interleaving, exact plan and chosen step.
 for risky in [False,True]:
  new(floor=-7);setat(0,-1,0,1);setat(3,-1,0,1);ability(parkour=True,risky=risky);moves(target='3,0,0');add('step','0,0,0','3,0,0');path(goal='3,0,0',visited=50,slice=5);advance(8)
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/n for n in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[p for p in sorted((AGENT/'pathfinder/walk').glob('*.kt'))if p.name not in ['BreakTimes.kt','StraightWalk.kt']]+[AGENT/'pathfinder/route'/n for n in ['AStar.kt','AStarSearch.kt','Limits.kt','Outcome.kt','Failure.kt','Search.kt']]+[AGENT/'simulation'/n for n in ['BlockColliders.kt','BlockEffects.kt','BlockGrid.kt','Collisions.kt','EdgeBackOff.kt','Motion1_8.kt','MotionTraits.kt','PlayerMotion.kt','Support.kt']]
 for i,p in enumerate(original):shutil.copyfile(p,out/f'{i:02}_{p.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8');imports,code=(HERE/'navigation-source-fixture/WalkRoutes.kt').read_text(encoding='utf-8').split('// fixture code\n',1)
 base=base.replace('package fixture','package fixture\n'+imports).replace('BlockKind.FIRE)','BlockKind.FIRE,BlockKind.FALLING,BlockKind.SLIME,BlockKind.FENCE,BlockKind.WALL)').replace('override fun canBeReplaced()=isAir','override fun canBeReplaced()=replaced[id]?:isAir').replace('states[id]=state','states[id]=state;replaced[id]=a[16].toBoolean()').replace('.6f,1f,1f,Float.fromBits','Float.fromBits(a[13].toUInt().toInt()),Float.fromBits(a[14].toUInt().toInt()),Float.fromBits(a[15].toUInt().toInt()),Float.fromBits').replace('0f)','Float.fromBits(a[17].toUInt().toInt()))').replace('fun main(){',code+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)')
 (out/'Fixture.kt').write_text(base,encoding='utf-8')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8');extra=(HERE/'navigation-source-fixture/walk_routes.rs').read_text(encoding='utf-8')
 fixture=fixture.replace('struct Registry {','struct Registry {\n walk_facts:HashMap<String,(f32,f32,f32,bool)>,walk_fall:HashMap<String,f32>,walk_ticks:HashMap<i32,f64>,').replace('let mut r = registry.borrow_mut();','let mut r = registry.borrow_mut();\n r.walk_facts.insert(block.clone(),(f32::from_bits(a[13].parse().unwrap()),f32::from_bits(a[14].parse().unwrap()),f32::from_bits(a[15].parse().unwrap()),a[16].parse().unwrap()));r.walk_fall.insert(block.clone(),f32::from_bits(a[17].parse().unwrap()));').replace('#[test]',extra+'\n#[test]',1).replace('let mut results = Vec::new();','let mut walk_fixture=WalkFixture::default();let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut walk_fixture,expected),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_full_walk_moves_parkour_and_route_search')
 (out/'walk_routes_oracle.rs').write_text(fixture,encoding='utf-8')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-18000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Xmx2g','-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')}
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod walk_routes_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-walk-route-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Terrain.kt','terrain.rs','WalkRoutes.kt','walk_routes.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'complete unchanged walk Moves, Parkour and RouteSearch plus actual jump/simulation/Terrain/search sources; ordered world queries, full offers and Steps, exact jump plans, sliced full/partial routes, stock/air/current-cost constraints and lazy route points','boundaries':['world/registry are controlled observable adapter seams','JVM actual computeNanos elapsed is fed to the Rust clock seam','controllers, live native body and full AutoWalk remain unintegrated'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
