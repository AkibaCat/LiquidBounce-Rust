#!/usr/bin/env python3
"""Full unchanged ride Vehicle/Moves/RouteSearch and actual Terrain/AStar."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,struct,subprocess,time
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'
def bits(v):return struct.unpack('<Q',struct.pack('<d',v))[0]
def fbits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def raw(v):return ','.join(str(bits(float(x)))for x in v)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*a):commands.append(' '.join(map(str,a)))
 for id,shape in [(0,'E'),(1,'F'),(2,raw([0,0,0,1,.5,1])),(3,raw([0,0,0,.125,1,1])),(4,raw([0,0,0,1,1.5,1]))]:add('shapeDef',id,shape)
 for id,name,shape,fluid,friction in [(0,'air',0,0,.6),(1,'stone',1,0,.6),(2,'void_air',0,0,.6),(3,'ice',1,0,.98),(4,'fixture_boundary_ice',1,0,.9),(5,'fixture_below_ice',1,0,.899999),(7,'fixture_slab',2,0,.6),(8,'fixture_thin',3,0,.6),(9,'fixture_tall',4,0,.6),(36,'water',0,1,.6)]:add('stateDef',id,name,0,fluid,-1,-1,-1,0,shape,shape,-1,-1,fbits(friction))
 def new(floor=-1,state=1):add('new','false',bits(.6),bits(1.8),-8,32,bits(-32),bits(32),'false',floor,state)
 def setat(x,y,z,state):add('set',f'{x},{y},{z}',state)
 def vehicle(kind='M',width=1.4,height=2.2,step=1,jump=1.5,safe=3,speed=.28,ice=1):add('vehicle',kind,raw([width,height,step,jump,speed,ice if kind=='B'else speed]),safe)
 def path(start=(.5,0,.5),kind='B',goal='4,0,0',radius=0,length=32,visited=2000,slice=12,nanos=9223372036854775807):add('path',raw(start),kind,goal,bits(radius),bits(length),visited,slice,nanos)
 def advance(n=10):add('preview');[add('advance')for _ in range(n)]
 new()
 for kind in ['B','H','C','O']:
  for jump in [False,True]:
   for strength,boost,safe in [(0,0,3),(.42,0,3),(.9,.2,4.9),(.3,-.1,-.4),(1.1,0,2147483648),(-.2,0,float('nan'))]:add('capture',kind,raw([2.7,2,1.1,1.62,1.4,1,strength,boost,safe,.25]),str(jump).lower())
 for kind in ['M','B']:
  for layout in ['flat','water','deep','ice','boundary','below','half','rise','tall','side','corner','roof','drop','unknown','empty']:
   new(floor=-7 if layout=='empty'else -1,state=3 if layout=='ice'else 4 if layout=='boundary'else 5 if layout=='below'else 1)
   if layout in ['water','deep']:
    for x in range(-2,4):
     for z in range(-2,3):setat(x,0,z,36)
    if layout=='deep':setat(0,1,0,36)
   if layout=='half':setat(1,0,0,7)
   if layout=='rise':setat(1,0,0,1)
   if layout=='tall':setat(1,0,0,9)
   if layout=='side':setat(1,0,0,1);setat(1,1,0,1)
   if layout=='corner':setat(1,0,1,1);setat(1,1,1,1)
   if layout=='roof':setat(0,2,0,1)
   if layout=='drop':setat(1,-1,0,0);setat(1,-3,0,1)
   if layout=='unknown':add('known','0,0,0','false')
   vehicle(kind)
   for node in ['0,0,0','1,0,0','0,1,0','0,-1,0','0,0,0']:add('rideSpot',node)
   add('rideMoves','0,0,0');add('rideMoves','0,0,0')
   for to in [(3.5,0,.5),(1.5,.6,.5),(1.5,.600001,.5),(2.5,0,2.5),(.5,0,.5)]:add('straight',raw([.5,0,.5]),raw(to));add('passes',raw([.5,0,.5]),raw(to))
 # Wide vehicles can rest against a wall at side/corner anchors; safe-fall and jump limits are exact.
 for width in [.4,1,1.00001,1.4,2.8]:
  for step,jump,safe in [(0,0,0),(.6,0,1),(1,1.2499,3),(.6,2.5,5)]:
   new();setat(1,0,0,7);setat(0,0,1,1);setat(1,0,1,9);vehicle(width=width,step=step,jump=jump,safe=safe);add('rideMoves','0,0,0')
   for rise in [-4,-1,0,step,step+.0000999,step+.0001,step+.0001001,2.5]:add('cost',raw([0,0,0]),bits(.28),raw([1,rise,1]),bits(.35))
 # Mount routes, start adjustment, no-path closest routes, goals without ends, and search limits.
 for layout,start,goal in [('flat',(.5,0,.5),'4,0,0'),('above',(.5,-1,.5),'4,0,0'),('below',(.5,1,.5),'4,0,0'),('same',(.5,0,.5),'0,0,0'),('slab',(.5,.5,.5),'4,0,0'),('wall',(.5,0,.5),'4,0,0'),('empty',(.5,0,.5),'4,0,0')]:
  new(floor=-7 if layout=='empty'else -1)
  if layout=='slab':setat(0,0,0,7)
  if layout=='wall':
   for z in range(-3,4):setat(2,0,z,1);setat(2,1,z,1);setat(2,2,z,1)
  vehicle();path(start=start,goal=goal);advance(12)
 for kind in ['B','N','G']:
  new();vehicle();path(kind=kind,goal=raw([4.2,0,.5])if kind=='N'else '4,0,0',radius=.7);advance(8)
 for visited,slice,nanos in [(1,1,9223372036854775807),(0,1,9223372036854775807),(200,0,9223372036854775807),(200,12,0),(200,12,9223372036854775807)]:new();vehicle();path(visited=visited,slice=slice,nanos=nanos);advance(8)
 # Boat goals include the rider's block above its water node and incomplete water crossings.
 for ice,short,goaly in [(False,False,0),(False,False,1),(False,True,1),(True,False,0)]:
  new(state=3 if ice else 1)
  if not ice:
   for x in range(-2,3 if short else 7):
    for z in range(-2,3):setat(x,0,z,36)
  vehicle('B',height=2.1,step=0,jump=0,safe=1,speed=.35);path(start=(.5,.5 if not ice else 0,.5),goal=f'4,{goaly},0');advance(14)
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/n for n in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[AGENT/'pathfinder/ride'/n for n in ['Vehicle.kt','Moves.kt','RouteSearch.kt']]+[AGENT/'pathfinder/route'/n for n in ['AStar.kt','AStarSearch.kt','Destination.kt','Limits.kt','Outcome.kt','Failure.kt','Search.kt']]+[AGENT/'pathfinder/walk/Costs.kt',AGENT/'math/VectorExtensions.kt']
 for idx,p in enumerate(original):shutil.copyfile(p,out/f'{idx:02}_{p.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8');imports,extra=(HERE/'navigation-source-fixture/Ride.kt').read_text(encoding='utf-8').split('// fixture code\n',1)
 base=base.replace('package fixture','package fixture\n'+imports).replace('{false},.6f,1f,1f','{false},Float.fromBits(a[13].toUInt().toInt()),1f,1f').replace('fun main(){',extra+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)');(out/'Fixture.kt').write_text(base,encoding='utf-8')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8').replace('struct Registry {','struct Registry {\nride_friction:HashMap<String,f32>,').replace('let mut r = registry.borrow_mut();','let mut r = registry.borrow_mut();\nr.ride_friction.insert(block.clone(),f32::from_bits(a[13].parse().unwrap()));').replace('#[test]',(HERE/'navigation-source-fixture/ride.rs').read_text(encoding='utf-8')+'\n#[test]',1).replace('let mut results = Vec::new();','let mut ride=RideFixture::default();let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut ride,expected),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_complete_ride_vehicle_moves_and_routes');(out/'ride_oracle.rs').write_text(fixture,encoding='utf-8')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-18000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Xmx2g','-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')};(out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod ride_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-ride-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Terrain.kt','terrain.rs','Ride.kt','ride.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged complete Vehicle/Moves/RouteSearch and actual Terrain/AStar; exact routes, edges/costs, spot caches and player/world queries','boundaries':['JVM computeNanos fed to Rust clock seam','live Riding/Boating/Steering/native binding remain separate'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False};(out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
