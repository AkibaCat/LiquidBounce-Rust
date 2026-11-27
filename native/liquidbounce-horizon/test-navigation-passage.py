#!/usr/bin/env python3
"""Unchanged Passage model-predictive controller and exact Terrain query order."""
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
 for id,shape in [(0,'E'),(1,'F'),(2,raw([0,0,0,1,.5,1]))]:add('shapeDef',id,shape)
 for id,name,shape,fluid in [(0,'air',0,0),(1,'stone',1,0),(2,'void_air',0,0),(7,'fixture_slab',2,0),(36,'water',0,1)]:add('stateDef',id,name,0,fluid,-1,-1,-1,0,shape,shape,-1,-1)
 def new(floor=-5):add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-16,80,bits(-100),bits(100),'false',floor,1)
 def passage(target=(20.5,2,.5),rockets=0,burn=20,clearance=.5,lands=False):add('passage',raw(target),rockets,burn,bits(clearance),str(lands).lower())
 def follow(points,reach=True,override='-'):add('follow',';'.join(map(raw,points)),str(reach).lower(),override)
 for lands in [False,True]:
  for reach in [False,True]:
   for points in [[(.5,2,.5)],[(.5,2,.5),(10.5,2,.5)],[(.5,2,.5),(10.5,2,.5),(10.5,5,10.5)],[(.5,2,.5),(.5,2,.5)],[(.5,2,.5),(20.5,2,.5)],[(.5,2,.5),(20,2,.5)]]:
    new();passage(lands=lands);follow(points,reach)
    for position in [(0,2,0),(8,2,2),(15,4,8),(50,10,10)]:
     add('project',raw(position),0)
     for speed in [0,.6,1.6,4]:add('look',raw(position),0,bits(speed))
 # Default route initialization and persistent target-through or landing routes.
 for lands in [False,True]:
  for rockets in [0,2]:
   new();passage(rockets=rockets,burn=2,lands=lands)
   for position,motion in [((.5,5,.5),(0,0,0)),((1.5,4,.5),(.6,-.1,0)),((10.5,2,.5),(1,-.2,0)),((20.5,2,.5),(.5,-.1,0))]:add('steer',raw(position),raw(motion))
 # Obstacles, water contacts and shell-only contact change the rollout choices.
 for block in [1,7,36]:
  for clearance in [0,.5,1.5]:
   new(floor=-1);add('set','3,2,0',block);add('set','4,2,0',block);passage(target=(10.5,2,.5),rockets=1,clearance=clearance);follow([(.5,2,.5),(5.5,2,3.5),(10.5,2,.5)])
   for yaw,pitch in [(-90,0),(-45,-8),(0,18)]:add('rollout',raw((.5,2,.5)),raw((.5,-.1,0)),0,fbits(yaw),fbits(pitch))
   add('steer',raw((.5,2,.5)),raw((.5,-.1,0)));add('lands','true');add('steer',raw((8.5,1,.5)),raw((.3,-.2,0)))
 # Sliding projection window and nearest tie keep first segment.
 new();passage(target=(60.5,2,.5));follow([(x+.5,2,(x%3)*2+.5)for x in range(35)])
 for start in [0,1,10,24,34,60]:add('project',raw((30.5,2,2.5)),start)
 follow([(.5,2,.5),(5.5,2,.5)],False,'true');add('steer',raw((4.5,2,.5)),raw((.2,-.2,0)))
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/n for n in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[AGENT/'pathfinder/glide'/n for n in ['Passage.kt','AirwaySearch.kt','Landing.kt','Rockets.kt','Steering.kt']]+[AGENT/'simulation/ElytraMotion.kt',AGENT/'math/VectorExtensions.kt']
 for i,p in enumerate(original):shutil.copyfile(p,out/f'{i:02}_{p.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8');imports,extra=(HERE/'navigation-source-fixture/Passage.kt').read_text(encoding='utf-8').split('// fixture code\n',1);base=base.replace('package fixture','package fixture\n'+imports).replace('fun main(){',extra+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)');(out/'Fixture.kt').write_text(base,encoding='utf-8')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8');fixture=fixture.replace('#[test]',(HERE/'navigation-source-fixture/passage.rs').read_text(encoding='utf-8')+'\n#[test]',1).replace('let mut results = Vec::new();','let mut passage:Option<Passage>=None;let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut passage),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_complete_passage_controller')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-15000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')};(out/'navigation/glide/passage_oracle.rs').write_text(fixture,encoding='utf-8')
 with(out/'navigation/glide/passage.rs').open('a',encoding='utf-8')as f:f.write('\n#[cfg(test)]#[path="passage_oracle.rs"]mod oracle;\n')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-passage-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Terrain.kt','terrain.rs','Passage.kt','passage.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged complete Passage, ElytraMotion and Terrain; exact controls/private route-arcs/projection/rollout-cost/rocket state and ordered snapshot/world queries','boundaries':['world and block registry facts are explicit providers','runtime Gliding/Contacts/Airways and native binding remain separate'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
