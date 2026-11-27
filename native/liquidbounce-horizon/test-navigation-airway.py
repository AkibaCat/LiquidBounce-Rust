#!/usr/bin/env python3
"""Unchanged complete AirwaySearch over the verified observable Terrain fixture."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,struct,subprocess,time
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'
def bits(v):return struct.unpack('<Q',struct.pack('<d',v))[0]
def raw(v):return ','.join(str(bits(float(x)))for x in v)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*a):commands.append(' '.join(map(str,a)))
 for id,shape in [(0,'E'),(1,'F'),(2,raw([0,0,0,1,.5,1]))]:add('shapeDef',id,shape)
 for id,name,shape,fluid in [(0,'air',0,0),(1,'stone',1,0),(2,'void_air',0,0),(7,'fixture_slab',2,0),(36,'water',0,1)]:add('stateDef',id,name,0,fluid,-1,-1,-1,0,shape,shape,-1,-1)
 def new(floor=-5):add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-8,25,bits(-20),bits(20),'false',floor,1)
 def air(start=(.5,2,.5),goal=(6.5,2,.5),radius=.1,clearance=.5,stage=None,length=32,visited=4000,slice=20,nanos=9223372036854775807):add('air',raw(start),raw(goal),bits(radius),bits(clearance),'-'if stage is None else bits(stage),bits(length),visited,slice,nanos)
 def advance(n=8):
  add('preview')
  for _ in range(n):add('advance')
 for clearance in [0,.25,.5,1,1.5]:
  for obstacle in [None,(1,2,0),(1,3,0),(1,2,1)]:
   new()
   if obstacle:add('set',','.join(map(str,obstacle)),1)
   add('set','-1,2,0',36);air(clearance=clearance)
   for node in ['0,1,0','1,1,0','2,1,1','0,1,0']:add('open',node);add('node',node)
   advance(3)
 for clearance in [0,.5,1.5]:
  new();air(clearance=clearance)
  for end in [(.5,2,.5),(1,2,.5),(2,2,.5),(2,3,1.5),(.5,4,.5)]:add('line',raw((.5,2,.5)),raw(end),bits(clearance));add('glideBox',raw(end),bits(clearance))
 # Stage unknown checks use the feet height, with the wider horizontal edge reach.
 for stage in [None,4,16]:
  for unknown in ['1,2,0','3,2,0','4,2,0','0,3,0']:
   new();add('known',unknown,'false');air(goal=(15.5,2,.5),stage=stage)
   for node in ['0,1,0','2,1,0','3,1,0']:add('node',node)
   advance(5)
 # Real search paths around wall corners, tight water and clearance, partial routes and budgets.
 for clearance,visited,slice,nanos in [(.25,4000,5,9223372036854775807),(.5,4000,5,9223372036854775807),(1,50,2,9223372036854775807),(.5,1,1,9223372036854775807),(.5,4000,5,0)]:
  new()
  for y in range(0,5):
   for z in [-1,0,1]:add('set',f'2,{y},{z}',1)
  air(clearance=clearance,visited=visited,slice=slice,nanos=nanos);advance(25)
 new();air(goal=(.5,2,.5));advance(2)
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/n for n in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[AGENT/'pathfinder/glide'/n for n in ['AirwaySearch.kt','Passage.kt','Landing.kt','Rockets.kt','Steering.kt']]+[AGENT/'pathfinder/route'/n for n in ['AStar.kt','AStarSearch.kt','Limits.kt','Outcome.kt','Failure.kt','Destination.kt']]+[AGENT/'simulation/ElytraMotion.kt',AGENT/'math/VectorExtensions.kt']
 for i,p in enumerate(original):shutil.copyfile(p,out/f'{i:02}_{p.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8');imports,extra=(HERE/'navigation-source-fixture/Airway.kt').read_text(encoding='utf-8').split('// fixture code\n',1);base=base.replace('package fixture','package fixture\n'+imports).replace('fun main(){',extra+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)');(out/'Fixture.kt').write_text(base,encoding='utf-8')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8');fixture=fixture.replace('#[test]',(HERE/'navigation-source-fixture/airway.rs').read_text(encoding='utf-8')+'\n#[test]',1).replace('let mut results = Vec::new();','let mut air_search:Option<AirSearch>=None;let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut air_search,expected),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_complete_airway_search');(out/'airway_oracle.rs').write_text(fixture,encoding='utf-8')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-15000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')}
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod airway_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-airway-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Terrain.kt','terrain.rs','Airway.kt','airway.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged complete AirwaySearch, AStar and Terrain; exact ordered neighbors/costs/room caches/stages, full search paths/previews/simplification and world queries','boundaries':['world and block registry facts are explicit providers','JVM computeNanos is measured and fed to Rust clock seam','Passage controller and runtime Gliding remain separate'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
