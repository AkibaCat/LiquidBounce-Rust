#!/usr/bin/env python3
"""Unchanged complete glide RouteSearch over the verified observable Terrain fixture."""
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
 def new(floor=-1):add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-64,384,bits(-200),bits(200),'false',floor,1)
 def glide(start=(.5,0,.5),goal=(10.5,0,.5),radius=.5,rockets=0,burn=20,ticks=2400):add('glide',raw(start),raw(goal),bits(radius),rockets,burn,ticks)
 def advance(n=4):
  add('preview')
  for _ in range(n):add('advance');add('preview')
 for start in [(0,0,0),(-.5,-2,.5),(3.5,1000,-.5)]:
  for goal in [(0,0,0),(4,3,4),(48.00001,-10,0),(-120,100,8)]:new();add('region',raw(start),raw(goal))
 for length in [0,1,2,3,10,200,2400]:new();glide(ticks=length);advance(2)
 for start,goal,rockets,burn in [((.5,0,.5),(0.5,0,.5),0,20),((.5,0,.5),(3.5,0,.5),0,20),((.5,0,.5),(10.5,0,.5),0,20),((.5,20,.5),(30.5,0,.5),0,20),((.5,20,.5),(-20.5,0,10.5),0,20),((.5,0,.5),(20.5,0,.5),1,20),((.5,0,.5),(50.5,0,.5),4,20),((.5,0,.5),(10.5,0,.5),2,1)]:new();glide(start,goal,rockets=rockets,burn=burn);advance()
 # Obstructed takeoff, short ceilings, walls, half floors and region edges.
 for obstacle in [(0,0,0),(0,1,0),(1,0,0),(1,2,0),(5,0,0)]:new();add('set',','.join(map(str,obstacle)),1);glide(rockets=1);advance(2)
 new();add('set','0,0,0',7);glide(start=(.5,.5,.5),goal=(10.5,0,.5));advance(2)
 new();add('set','1,0,0',36);glide(rockets=1);advance(2)
 new();add('known','1,1,0','false');glide(rockets=1);advance(2)
 new(floor=-64);glide(goal=(40.5,-63,.5));advance(5)
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/n for n in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[AGENT/'pathfinder/glide'/n for n in ['GlideAbilities.kt','Rockets.kt','Steering.kt','Landing.kt','RouteSearch.kt']]+[AGENT/'simulation/ElytraMotion.kt',AGENT/'math/VectorExtensions.kt']
 for i,p in enumerate(original):shutil.copyfile(p,out/f'{i:02}_{p.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8');imports,extra=(HERE/'navigation-source-fixture/GlideRoute.kt').read_text(encoding='utf-8').split('// fixture code\n',1);base=base.replace('package fixture','package fixture\n'+imports).replace('fun main(){',extra+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)');(out/'Fixture.kt').write_text(base,encoding='utf-8')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8');fixture=fixture.replace('#[test]',(HERE/'navigation-source-fixture/glide_route.rs').read_text(encoding='utf-8')+'\n#[test]',1).replace('let mut results = Vec::new();','let mut glide_search:Option<GlideSearch>=None;let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut glide_search),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_complete_glide_route_search');(out/'glide_route_oracle.rs').write_text(fixture,encoding='utf-8')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-15000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')}
 with(out/'navigation/glide/route_search.rs').open('a',encoding='utf-8')as f:f.write('''
#[cfg(test)]impl<B:TerrainState,L:TerrainLevel<B>> RouteSearch<B,L> {pub(crate) fn fixture_state(&self)->String {
 let a=&self.attempt;let r=&a.control.rockets;let v=|p:Vec3|[p.x,p.y,p.z].map(|x|x.to_bits().to_string()).join(",");format!("{}:{}:{}:{}:{}:{}:{}",a.approach.to_bits(),a.tick,v(a.position),v(a.motion),r.left(),r.boost_ticks(),r.can_fire())
}}
''')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod glide_route_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-glide-route-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Terrain.kt','terrain.rs','GlideRoute.kt','glide_route.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged complete glide RouteSearch, Landing, ElytraMotion and Terrain; exact route and internal attempt state, previews, retries, query sequence and limits','boundaries':['world and block registry facts are explicit providers','Passage/AirwaySearch and runtime Gliding remain separate'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
