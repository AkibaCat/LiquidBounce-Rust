#!/usr/bin/env python3
"""Unchanged Agent jump, Leap, Landings and AssistedJump differential."""
from pathlib import Path
import argparse,hashlib,itertools,json,math,os,random,shutil,struct,subprocess,time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'
def bits(v):return struct.unpack('<Q',struct.pack('<d',v))[0]
def fbits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def raw(v):return ','.join(str(bits(float(x)))for x in v)
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);args=p.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);commands=[];records=[]
 def add(*a):commands.append(' '.join(map(str,a)))
 def b(v):return str(v).lower()
 for ident,shape in [(0,'E'),(1,'F'),(2,raw([0,0,0,1,.5,1])),(3,raw([.375,0,.375,.625,1.5,.625])),(4,raw([0,0,0,.125,1,1])),(5,raw([0,0,0,1,.5625,1])),(6,raw([0,0,0,1,.875,1]))]:add('shapeDef',ident,shape)
 def state(ident,name,shape=1,flags=0,fluid=0,friction=.6,speed=1,jump=1,bounce=0,fall=0):add('stateDef',ident,name,flags,fluid,-1,-1,-1,fbits(bounce),shape,shape,-1,-1,fbits(friction),fbits(speed),fbits(jump),b(name in ['air','void_air']),fbits(fall))
 state(0,'air',0);state(1,'stone');state(2,'void_air',0);state(3,'slime_block',flags=128,bounce=1);state(4,'red_bed',5,bounce=.66);state(5,'honey_block',speed=.4,jump=.5);state(6,'ice',friction=.98);state(7,'blue_ice',friction=.989);state(8,'fixture_slab',2);state(9,'oak_fence',3,256);state(10,'cobblestone_wall',3,512);state(11,'oak_fence_gate',3,4);state(12,'water',0,fluid=1);state(13,'ladder',4,16);state(14,'cobweb',0);state(15,'hay_block',fall=.8);state(16,'soul_sand',6,speed=.4)
 resting=-(.08*struct.unpack('<f',struct.pack('<f',.98))[0])
 def new(legacy=False,floor=-4,floor_state=1):add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-16,48,bits(-24.),bits(24.),b(legacy),floor,floor_state)
 def at(x,y,z,ident=1):add('set',f'{x},{y},{z}',ident)
 def ability(risky=False,sprint=True,safe=3,speed=.1,strength=.42,boost=0,holds=False):add('jnew',b(risky),b(sprint),safe,fbits(speed),fbits(strength),fbits(boost),b(holds))
 def motion(pos=(.5,0,.5),delta=(0,resting,0),ground=True,sprint=False,delay=0,crouch=False,sneak=False):add('motion',raw(pos),raw(delta),b(ground),b(sprint),delay,b(crouch),b(sneak))
 def jdef(to=(3.5,0,.5),takeoff=2,hops=0,jumps=True):add('jdef',raw([.5,0,.5]),takeoff,b(jumps),12,bits(.4),hops,raw(to))
 new()
 # Direct JDK atan2 and both original yaw conversion orders, finite bit patterns.
 rng=random.Random(0x26_03)
 for y,x in itertools.product([0.,-0.,1.,-1.,math.inf,-math.inf,1.e-300,1.e300],repeat=2):add('math',bits(y),bits(x))
 for _ in range(1000):add('math',bits(math.ldexp(rng.uniform(-1,1),rng.randrange(-1000,1000))),bits(math.ldexp(rng.uniform(-1,1),rng.randrange(-1000,1000))))
 # Retained planner cache/pools, all public plan/live/air/takeoff paths.
 for legacy,risky,sprint,gap in itertools.product([False,True],[False,True],[False,True],[2,3,4,5]):
  new(legacy);ability(risky,sprint)
  for x in range(-3,1):
   for z in [-1,0,1]:at(x,-1,z)
  at(gap,-1,0)
  motion(delta=(.18,resting,.01),sprint=sprint)
  add('plan','0,0,0',bits(0.),f'{gap},0,0',bits(0.),'false');add('duration',raw([.5,0,.5]));add('takeOff','-',f'{gap},0,0',bits(0.))
  motion(delta=(.15,resting,0),sprint=sprint)
  add('refine','0,0,0',bits(0.),f'{gap},0,0',bits(0.));add('here',fbits(-90),f'{gap},0,0',bits(0.),raw([gap+.8,0,.8]));add('now',fbits(-90),f'{gap},0,0',bits(0.));add('last',fbits(-90),f'{gap},0,0',bits(0.))
  motion(pos=(1.1,.7,.5),delta=(.2,.1,0),ground=False,sprint=sprint)
  for keep in [False,True]:add('air',fbits(-90),f'{gap},0,0',bits(0.),b(keep))
 # Obstacles, side aims, catchable ladder, rise/drop, block effects, momentum.
 for legacy,kind in itertools.product([False,True],['wall','post','ladder','drop','rise','slab','slime','bed','honey','ice','soul','boost','speed','hops']):
  new(legacy,floor=-10);ability(True,True,safe=4,boost=.2 if kind=='boost'else 0,speed=.14 if kind=='speed'else .1)
  for x in range(-8,1):
   for z in [-1,0,1]:at(x,-1,z,{'honey':5,'ice':6,'soul':16}.get(kind,1))
  gap=6 if kind=='hops'else 3;ty=-2 if kind=='drop'else 1 if kind=='rise'else 0;top=ty+.5 if kind=='slab'else ty
  at(gap,ty-1,0,3 if kind=='slime'else 4 if kind=='bed'else 1)
  if kind=='slab':at(gap,ty,0,8)
  if kind=='wall':
   for z in [-1,0,1]:at(1,0,z);at(1,1,z)
  if kind=='post':at(1,0,0,9)
  if kind=='ladder':at(gap,ty-1,0,0);at(gap,ty,0,13);at(gap,ty+1,0,13)
  add('plan','0,0,0',bits(0.),f'{gap},{ty},0',bits(top),'true');add('duration',raw([.5,0,.5]));motion();add('takeOff','-',f'{gap},{ty},0',bits(top))
  for up in [-.1,0,.42,.8,1.5]:add('lift',bits(up))
  add('targets','0,0,0',bits(0.),4,12,bits(0.),'true',3,'true')
 # Candidate callback short circuit and interleaved full plan, bounce reach/rise.
 for up,limit,around in [(0.,1,False),(0.,99,True),(.8,4,True),(1.2,2,False)]:
  new(floor=-14);ability(True,holds=True);at(0,-1,0)
  for x,y,z,ident in [(2,-1,0,1),(3,-1,0,3),(4,0,0,1),(-3,-1,0,1),(0,-2,3,4),(0,2,5,1),(0,-6,7,3)]:at(x,y,z,ident)
  add('targets','0,0,0',bits(0.),4,12,bits(up),b(around),limit,'true')
 # Leap retained lifecycle, turning in place, approach, refinement and retreat.
 for legacy,scene in itertools.product([False,True],['gap','blocked','flat','runup','fall']):
  new(legacy,floor=-8);ability(True)
  for x in range(-3,1):at(x,-1,0)
  at(3,-1,0)
  if scene=='blocked':at(1,0,0);at(1,1,0)
  if scene=='flat':
   for x in [1,2]:at(x,-1,0)
  jdef(hops=1 if scene=='runup'else 0)
  for tick in range(9):
   motion(pos=(.5,.7,.5)if scene=='fall'else(.5,0,.5),delta=(0,.1,0)if scene=='fall'else(0,resting,0),ground=scene!='fall')
   add('leap',fbits(90 if tick<3 else -90),'0,0,0','3,0,0',bits(0.))
 # Assisted selection, retained target identity, trace, airborne steering,
 # release/reset and bounce/step-down brake paths.
 for legacy,scene in itertools.product([False,True],['gap','slime','bed','bounce','runup','none','step','tiny']):
  new(legacy,floor=-12);ability(True,holds=True)
  for x in range(-3,1):at(x,-1,0)
  if scene!='none':at(3,-1,0,3 if scene in ['slime','bounce']else 4 if scene=='bed'else 1)
  if scene=='bounce':at(6,0,0);at(0,-1,0,3)
  if scene=='runup':at(1,0,0,9)
  if scene=='step':at(0,-1,0,4);at(1,-2,0)
  if scene=='tiny':at(0,-1,0,8)
  for tick in range(3):
   motion(pos=(.8,0,.5),delta=(.18,.7 if scene=='bounce'else resting,0),sprint=True)
   add('assist',fbits(-90),fbits(-90),fbits(45),1,0,'false','true','false')
   add('trace',fbits(-90),fbits(-90),fbits(45))
  motion(pos=(1.2,.7,.5),delta=(.25,.1,0),ground=False,sprint=True);add('assist',fbits(-90),fbits(-90),fbits(45),1,0,'false','true','false');add('release');add('resetAssist')
 # Landing helpers, settle steer and yaw hold on/near/past a target.
 for ground,settles,takes_off,pos in itertools.product([False,True],[False,True],[False,True],[(3.5,0,.5),(3.8,.2,.5),(2.,1.,.5)]):
  new();ability(True,holds=True);at(3,-1,0,3);motion(pos=pos,delta=(.2,-.2,0),ground=ground);jdef();add('landing','3,0,0',bits(0.),'true',b(takes_off),b(settles),'-',fbits(-90))
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/n for n in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[q for q in sorted((AGENT/'pathfinder/walk').glob('*.kt'))if q.name not in ['BreakTimes.kt','StraightWalk.kt']]+[AGENT/'pathfinder/route'/n for n in ['AStar.kt','AStarSearch.kt','Limits.kt','Outcome.kt','Failure.kt','Search.kt']]+[AGENT/'simulation'/n for n in ['BlockColliders.kt','BlockEffects.kt','BlockGrid.kt','Collisions.kt','EdgeBackOff.kt','Motion1_8.kt','MotionTraits.kt','PlayerMotion.kt','Support.kt']]+[AGENT/'navigation/walk'/n for n in ['AssistedJump.kt','Landings.kt']]
 for i,q in enumerate(original):shutil.copyfile(q,out/f'{i:02}_{q.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text();imports,code=(HERE/'jump-source-fixture/JumpChain.kt').read_text().split('// fixture code\n',1)
 base=base.replace('package fixture','package fixture\n'+imports).replace('BlockKind.FIRE)','BlockKind.FIRE,BlockKind.FALLING,BlockKind.SLIME,BlockKind.FENCE,BlockKind.WALL)').replace('override fun canBeReplaced()=isAir','override fun canBeReplaced()=replaced[id]?:isAir').replace('states[id]=state','states[id]=state;replaced[id]=a[16].toBoolean()').replace('.6f,1f,1f,Float.fromBits','Float.fromBits(a[13].toUInt().toInt()),Float.fromBits(a[14].toUInt().toInt()),Float.fromBits(a[15].toUInt().toInt()),Float.fromBits').replace('0f)','Float.fromBits(a[17].toUInt().toInt()))').replace('fun main(){',code+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)')
 (out/'Fixture.kt').write_text(base,encoding='utf-8')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text();extra=(HERE/'jump-source-fixture/jump_chain.rs').read_text()
 fixture=fixture.replace('struct Registry {','struct Registry {\n walk_facts:HashMap<String,(f32,f32,f32,bool)>,walk_fall:HashMap<String,f32>,').replace('let mut r = registry.borrow_mut();','let mut r = registry.borrow_mut();\n r.walk_facts.insert(block.clone(),(f32::from_bits(a[13].parse().unwrap()),f32::from_bits(a[14].parse().unwrap()),f32::from_bits(a[15].parse().unwrap()),a[16].parse().unwrap()));r.walk_fall.insert(block.clone(),f32::from_bits(a[17].parse().unwrap()));').replace('#[test]',extra+'\n#[test]',1).replace('let mut results = Vec::new();','let mut jump_fixture=JumpFixture::default();let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut jump_fixture),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_jump_leap_and_assisted_jump_full_state_and_ordered_world_queries')
 fixture=fixture.replace('"case {index}: {command}\\nactual: {result}\\nexpected: {expected}"','"case {index}: {command}\\nactual: {}\\nexpected: {}", result.chars().take(2000).collect::<String>(), expected.chars().take(2000).collect::<String>()')
 (out/'jump_chain_oracle.rs').write_text(fixture,encoding='utf-8')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append(dict(name=name,exitCode=r.returncode,seconds=round(time.monotonic()-began,3)));(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8');print((r.stdout+r.stderr)[-12000:]if r.returncode else r.stdout+r.stderr,end='');return r.returncode
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(q)for q in out.glob('*.kt')]])
 if not result:result=run('oracle',[str(java),'-Xmx2g','-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 nav=ROOT/'native/liquidbounce-behaviors/src/navigation';shutil.copytree(nav,out/'navigation');snapshot={str(q.relative_to(out/'navigation')):hashlib.sha256(q.read_bytes()).hexdigest()for q in(out/'navigation').rglob('*')if q.is_file()}
 (out/'lib.rs').write_text('#![allow(dead_code)]\n#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod jump_chain_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-jump-chain-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if not result:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/jump-chain-target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if not result:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),HERE/'jump-source-fixture/JumpChain.kt',HERE/'jump-source-fixture/jump_chain.rs',HERE/'navigation-source-fixture/Terrain.kt',HERE/'navigation-source-fixture/terrain.rs']
 report=dict(passed=len(records)==4 and all(r['exitCode']==0 for r in records),commands=records,steps=len(commands),sourceSha256={str(q.relative_to(ROOT)):hashlib.sha256(q.read_bytes()).hexdigest()for q in files},compiledNavigationSnapshotSha256=snapshot,unchangedOriginalKotlin=all(q.read_bytes()==(out/f'{i:02}_{q.name}').read_bytes()for i,q in enumerate(original)),scope='complete Jump/Jumps/JumpTargets/Airtime/RunUp/Leap/AssistedJump/Landings public behavior, sequential retained state, exact numeric results and ordered actual Terrain/world queries',boundaries=['controlled exact block/registry/shape facts, native game registry/world adapter not_run','SafeWalk actual vanilla SimulatedPlayer not implemented here','Parkour/native full-client differential not_run'])
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(passed=report['passed'],steps=len(commands))));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
