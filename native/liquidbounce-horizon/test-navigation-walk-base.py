#!/usr/bin/env python3
"""Compile unchanged Agent walk foundations and compare observable world calls."""
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
 for id,shape in [(0,'E'),(1,'F'),(2,raw([0,0,0,1,.5,1])),(3,raw([0,.5,0,1,1,1])),(4,raw([0,0,0,.125,1,1])),(5,raw([.875,0,0,1,1,1])),(6,raw([0,0,0,1,.125,1])),(7,raw([0,0,0,1,1.25,1])),(8,raw([.8,0,0,1,1.25,1])),(9,raw([0,0,0,1,.5,1,0,0,0,.125,1,1]))]:add('shapeDef',id,shape)
 def state(id,name,shape=0,flags=0,fluid=0,friction=.6,speed=1,jump=1,replaced=False):add('stateDef',id,name,flags,fluid,-1,-1,-1,0,shape,shape,-1,-1,fbits(friction),fbits(speed),fbits(jump),str(replaced).lower())
 state(0,'air',replaced=True);state(1,'stone',1);state(2,'void_air',replaced=True);state(7,'fixture_slab',2);state(8,'fixture_roof',3);state(9,'fixture_lower_roof',6);state(10,'fixture_tall',7);state(11,'ladder',4,16);state(12,'vine',5);state(13,'fixture_partial',8);state(14,'fixture_bowl',9);state(15,'fixture_falling',1,64);state(16,'slime_block',1,128);state(17,'fixture_replaced',0,replaced=True);state(18,'fixture_hazard',0,32);state(19,'fixture_slow',1,friction=.98,speed=.4);state(20,'fixture_nan',1,friction=float('nan'),speed=.5);state(21,'fixture_fast',1,speed=1.4);state(36,'water',0,fluid=1);state(37,'lava',0,fluid=2)
 def new(floor=-1,floor_state=1):add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-16,64,bits(-32),bits(32),'false',floor,floor_state)
 def walk(sprint=True,place=True,count=64,can_break=True,safe=3,water=16,health=20,air=300,reserved='-',avoid='-',unbreakable='-'):add('walk',str(can_break).lower(),str(place).lower(),count,str(sprint).lower(),safe,water,fbits(health),air,reserved,avoid,unbreakable)
 def setat(x,y,z,id):add('set',f'{x},{y},{z}',id)
 # Original defaults, withMotion value replacement and shared immutable captures.
 new();walk(reserved='2,0,0',avoid='1,0,0',unbreakable='3,0,0')
 for speed,jump,boost,legacy in [(.1,.42,0,False),(.3,.6,.2,True),(0,0,0,False),(-.1,-.2,0,False),(float('nan'),float('nan'),0,False)]:add('abilities',fbits(speed),fbits(jump),fbits(boost),str(legacy).lower())
 # Upright/swimming/crouching/climbing, board room center, partial floor footprint.
 for sprint in [False,True]:
  for state_id,ceiling in [(0,0),(1,0),(7,0),(8,0),(10,0),(13,0),(14,0),(11,0),(12,0),(11,8),(0,8),(0,1),(36,0),(36,8),(36,1),(37,0),(18,0)]:
   new();walk(sprint=sprint);setat(0,0,0,state_id)
   if ceiling:setat(0,1,0,ceiling)
   add('node','0,0,0');add('node','0,1,0');add('node','0,0,0')
 # Head-water submergence, low swimming pose and reserved/empty inventory placement.
 for upper in [0,1,36]:
  new();walk();setat(0,0,0,36);setat(0,1,0,upper);add('node','0,0,0')
 for place,count,reserved in [(False,64,'-'),(True,0,'-'),(True,-1,'-'),(True,64,'0,0,0'),(True,64,'-')]:
  new();walk(place=place,count=count,reserved=reserved);add('node','0,0,0');setat(1,0,0,17);add('node','1,0,0')
 # Exact break-floor sampling offset, and float costs/air integer division.
 new();walk();setat(0,0,0,19)
 for h in [-.000001,0,.499999,.5,.500001,.500002,1.5,1.500001,float('nan')]:add('floorBlock','0,0,0',bits(h))
 for id in [1,16,19,20,21]:
  for blocks,air in [(-1,-3),(0,0),(1,1),(2,19),(3,20),(256,299),(257,300),(2147483647,2147483647)]:add('cost',id,blocks,air)
 # Survival leaves strictly more than half health; water and ladder landing precedence.
 for health in [0,10,10.1,11,20,float('nan')]:
  for floor_id in [1,36,11,0]:
   new(floor=0,floor_state=floor_id);walk(health=health)
   for h in [3,3.01,4,4.01,8]:add('landing','0,8,0',bits(h))
 # Deadly side order, unknown side, surviving/water/solid side and height below head.
 for hazard in ['none','void','unknown','water','wall','hurt']:
  new(floor=-5 if hazard in ['void','water','hurt'] else -1);walk(health=20 if hazard=='hurt' else 10)
  if hazard=='unknown':add('known','1,0,0','false')
  if hazard=='water':setat(1,-1,0,36)
  if hazard=='wall':setat(1,0,0,1)
  add('drop','0,0,0',bits(0));add('drop','0,0,0',bits(2))
 # Changes permissions, wet neighbors, falling sand, hazards, no callback for rejected cells.
 for reason in ['normal','disabled','floor','floorAllowed','air','water','aboveWater','sideWater','falling','hazard','unknown','unbreakable']:
  new();walk(can_break=reason!='disabled',unbreakable='1,0,0' if reason=='unbreakable' else '-');add('from','0,1,0');target='0,0,0' if reason in ['floor','floorAllowed']else'1,0,0';q=[int(v)for v in target.split(',')];setat(*q,0 if reason=='air' else 18 if reason=='hazard' else 1)
  if reason=='water':setat(*q,36)
  if reason=='aboveWater':setat(q[0],q[1]+1,q[2],36)
  if reason=='sideWater':setat(q[0]+1,q[1],q[2],36)
  if reason=='falling':setat(q[0],q[1]+1,q[2],15)
  if reason=='unknown':add('known',target,'false')
  add('break',target,str(reason=='floorAllowed').lower());add('break',target,'pass');add('break',target,'pass');add('clearChanges');add('break',target,'false')
 for ticks in [-1,0,.5,float('inf'),float('-inf'),float('nan')]:
  new();walk();setat(1,0,0,1);add('ticks',1,bits(ticks));add('from','0,0,0');add('break','1,0,0','false');add('break','1,0,0','pass')
 for side in [None,(1,0,0),(-1,0,0),(0,0,1),(0,0,-1),(0,-1,0)]:
  new(floor=-9);walk();add('from','1,1,0')
  if side:setat(*side,1)
  add('placeAgainst','0,0,0','false');add('placeAgainst','0,0,0','true')
 # Course direction epsilon, offsets, exact target/footprint bounds and ballistic steering.
 for target,start,height,climb in [('3,0,0',[.5,0,.5],0,False),('3,0,3',[.5,0,.5],0,False),('0,0,0',[.499999,0,.5],0,False),('3,0,0',[.5,1,.5],0,True)]:
  new();walk()
  if climb:setat(3,0,0,11)
  add('course',raw(start),target,bits(height))
  for offset in [-.5,0,.5]:add('aim',bits(offset));add('behind',bits(offset))
  for point in [[3-.30000001192092896,0,.5],[3-.3,0,.5],[3,0,.5],[3.5,0,.5],[4.3,0,.5],[3.5,1,.5],[float('nan'),0,.5]]:add('over',raw(point))
  for speed,dy,y in [(0,.42,0),(.3,.42,0),(.7,-.2,1),(-.2,.3,1),(.3,-.1,-.1),(.1,0,4),(0,float('nan'),0)]:
   add('overshoots',bits(3),bits(speed),bits(y),bits(dy))
   for sprint in [False,True]:add('estimate',raw([start[0],y,start[2]]),raw([speed,dy,0]),str(sprint).lower())
 # Observable trajectory/ground checks, obstruction clearance, safety and run-up edge.
 for layout in ['plain','gap','water','hazard','wall','roof','slab']:
  new(floor=-6 if layout=='gap'else -1);walk();add('course',raw([.5,0,.5]),'4,0,0',bits(0))
  if layout=='water':setat(2,0,0,36)
  if layout=='hazard':setat(2,0,0,18)
  if layout=='wall':setat(2,1,0,1)
  if layout=='roof':setat(2,2,0,1)
  if layout=='slab':setat(2,0,0,7)
  add('way',raw([.5,0,.5]));add('safe');add('edge');add('runUp',bits(1.5));add('canStand',raw([.5,0,.5]));add('canStand',raw([.5,.5,.5]))
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/name for name in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[AGENT/'pathfinder/walk'/name for name in ['WalkAbilities.kt','Costs.kt','Water.kt','Nodes.kt','Changes.kt','Course.kt']]+[AGENT/'simulation/MotionTraits.kt']
 for i,file in enumerate(original):shutil.copyfile(file,out/f'{i:02}_{file.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8');extra=(HERE/'navigation-source-fixture/WalkBase.kt').read_text(encoding='utf-8');imports,code=extra.split('// fixture code\n',1)
 base=base.replace('package fixture','package fixture\n'+imports).replace('BlockKind.FIRE)','BlockKind.FIRE,BlockKind.FALLING,BlockKind.SLIME)').replace('override fun canBeReplaced()=isAir','override fun canBeReplaced()=replaced[id]?:isAir').replace('states[id]=state','states[id]=state;replaced[id]=a.getOrNull(16)?.toBoolean()?:state.isAir').replace('.6f,1f,1f,Float.fromBits',"if(a.size>13)Float.fromBits(a[13].toUInt().toInt())else .6f,if(a.size>14)Float.fromBits(a[14].toUInt().toInt())else 1f,if(a.size>15)Float.fromBits(a[15].toUInt().toInt())else 1f,Float.fromBits").replace('fun main(){',code+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)')
 (out/'Fixture.kt').write_text(base,encoding='utf-8')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8');extra_rs=(HERE/'navigation-source-fixture/walk_base.rs').read_text(encoding='utf-8')
 fixture=fixture.replace('struct Registry {','struct Registry {\n walk_facts:HashMap<String,(f32,f32,f32,bool)>,walk_ticks:HashMap<i32,f64>,').replace('let mut r = registry.borrow_mut();','let mut r = registry.borrow_mut();\n r.walk_facts.insert(block.clone(),(f32::from_bits(a[13].parse().unwrap()),f32::from_bits(a[14].parse().unwrap()),f32::from_bits(a[15].parse().unwrap()),a[16].parse().unwrap()));').replace('#[test]',extra_rs+'\n#[test]',1).replace('let mut results = Vec::new();','let mut walk_fixture=WalkFixture::default();let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut walk_fixture),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_walk_foundations_and_course')
 (out/'walk_base_oracle.rs').write_text(fixture,encoding='utf-8')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-16000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')}
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod walk_base_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-walk-base-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Terrain.kt','terrain.rs','WalkBase.kt','walk_base.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged WalkAbilities, Costs, Water, Nodes, Changes and Course source units plus actual Terrain sources; float costs and air accounting, voxel stands and room, survival/drop decisions, mining/placing restrictions, full jump course estimates and query order','boundaries':['world/registry block facts remain observable controlled adapter seams','Course consumes a real JVM PlayerMotion value capture; full PlayerMotion simulation is tested separately','complete walk Moves/RouteSearch, controllers and native body are not yet integrated'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
