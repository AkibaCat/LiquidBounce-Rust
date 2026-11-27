#!/usr/bin/env python3
"""Unchanged RoofBounce/Contacts plus complete Terrain and ordered live host calls."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,struct,subprocess,time,itertools
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
 def new(floor=-1):add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-16,80,bits(-100),bits(100),'false',floor,1);add('runtime')
 def player(pos=(.5,0,.5),ground=True,flying=False,recasts=True,h=False,v=False,below=False,box=None):add('player',raw(pos),*[str(x).lower()for x in [ground,flying,recasts,h,v,below]],raw(box or [pos[0]-.3,pos[1],pos[2]-.3,pos[0]+.3,pos[1]+1.8,pos[2]+.3]))
 def roof(goal=(60.5,0,.5),ability=True,yaw=0):add('roof',raw(goal),str(ability).lower(),fbits(yaw))
 # Flat floor scans, side strips, strict thresholds and unknown reach.
 for goal in [(60.5,0,.5),(.5,0,60.5),(60.5,0,60.5),(-60.5,0,-30.5),(.6,0,.6)]:
  for look in [.49,.5,7.9,8,12,24,35]:new();add('line',raw((.5,0,.5)),raw(goal),bits(look))
 for obstruction in [0,1,7,36]:
  for y in [-1,0,1,2]:new();add('set','8,'+str(y)+',0',obstruction);add('line',raw((.5,0,.5)),raw((40.5,0,.5)),bits(24))
 for unknown in [0,7,8,9,24]:new();add('known',str(unknown)+',0,0','false');add('line',raw((.5,0,.5)),raw((40.5,0,.5)),bits(24))
 # Starting, preserving the original goal bearing, stop/floor loss and alternating open presses.
 for ground,flying,recasts,ability in itertools.product([False,True],repeat=4):new();player(ground=ground,flying=flying,recasts=recasts);roof(ability=ability)
 for distance in [11.999999,12,31.999999,32,32.000001,60]:
  new();player();roof((distance+.5,0,.5));player(pos=(5.5,.5,.5),ground=False);roof((distance+.5,0,10.5));player(pos=(5.5,-.500001,.5),ground=False);roof((distance+.5,0,.5));add('halt')
 new();player();roof()
 for tick in range(16):player(pos=(tick+.5,.25,.5),ground=tick%3==0,flying=tick%4==0);roof((60.5,0,8.5),yaw=33.25)
 for yaw in [-180,-160,-110,-90,-70,-20,0,20,70,90,110,160,180]:new();player();roof(yaw=yaw)
 # Contact normals, repeated touching, recontact and all combinations of collision flags.
 for flags in itertools.product([False,True],repeat=4):
  new();player(pos=(.5,0,.5),flying=flags[0],h=flags[1],v=flags[2],below=flags[3],box=[0,0,0,1,1,1])
  for pos in ['0,1,0','-1,0,0','1,0,0','0,0,-1','0,0,1']:add('set',pos,1)
  add('contacts');add('contacts');player(flying=False);add('contacts');player(flying=True,h=True,v=True,below=False,box=[0,0,0,1,1,1]);add('contacts')
 for bounds in [[-.2,0,-.2,.8,1,.8],[0,.5,0,1,1.5,1],[1,0,1,2,1,2]]:
  new();player(flying=True,h=True,v=True,below=True,box=bounds);add('set','-1,0,0',7);add('set','0,0,-1',7);add('contacts')
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/n for n in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[AGENT/'navigation/glide'/n for n in ['RoofBounce.kt','Contacts.kt']]+[AGENT/'navigation/Progress.kt',AGENT/'math/VectorExtensions.kt',AGENT/'math/InputExtensions.kt',AGENT/'world/LevelBlocks.kt']
 for i,p in enumerate(original):shutil.copyfile(p,out/f'{i:02}_{p.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8');imports,extra=(HERE/'navigation-source-fixture/GlideRuntimeBase.kt').read_text(encoding='utf-8').split('// fixture code\n',1);base=base.replace('package fixture','package fixture\n'+imports).replace('fun main(){',extra+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)');(out/'Fixture.kt').write_text(base,encoding='utf-8');shutil.copyfile(HERE/'navigation-source-fixture/GlideRuntimeContext.kt',out/'Context.kt')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8');fixture=fixture.replace('#[test]',(HERE/'navigation-source-fixture/glide_runtime_base.rs').read_text(encoding='utf-8')+'\n#[test]',1).replace('let mut results = Vec::new();','let mut runtime:Option<Runtime>=None;let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut runtime),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_roof_bounce_and_contacts')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-15000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-DdirectionHashes='+str(out/'direction-hashes.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')};(out/'navigation/glide/roof_oracle.rs').write_text(fixture,encoding='utf-8')
 with(out/'navigation/glide/roof_bounce.rs').open('a',encoding='utf-8')as f:f.write('\n#[cfg(test)]#[path="roof_oracle.rs"]mod oracle;\n')
 with(out/'navigation/glide/contacts.rs').open('a',encoding='utf-8')as f:f.write('\n#[cfg(test)]impl Contacts{pub(crate) fn fixture_state(&self)->String{self.touching.iter().map(|t|format!("{},{},{}:{:?}",t.pos.x,t.pos.y,t.pos.z,t.direction).to_uppercase()).collect::<Vec<_>>().join(";")}}\n')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-glide-runtime-base-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Terrain.kt','terrain.rs','GlideRuntimeBase.kt','GlideRuntimeContext.kt','glide_runtime_base.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged complete RoofBounce, Contacts, TargetProgress and Terrain; exact private bounce/contact state, live actor/world query order, report and JVM HashSet callback order','boundaries':['Direction enum identity hashes are captured from the original JVM as host facts','runtime Gliding/Airways and native binding remain separate'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
