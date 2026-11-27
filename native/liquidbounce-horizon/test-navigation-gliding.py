#!/usr/bin/env python3
"""Unchanged Gliding/Airways/controllers and actual Terrain, controlled owning searches."""
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
 add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-32,160,bits(-1000),bits(1000),'false',-1,1)
 for id,glide,damage,maximum,damaged,rocket,count,flight in [(0,False,False,0,0,False,0,'-'),(1,True,True,432,10,False,1,'-'),(2,False,False,0,0,True,8,1),(3,False,False,0,0,True,4,3),(4,True,True,5,4,False,1,'-')]:add('stack',id,*[str(v).lower()if isinstance(v,bool)else v for v in [glide,damage,maximum,damaged,rocket,count,flight]])
 def player(pos=(.5,10,.5),ground=True,flying=False,delta=(0,0,0),h=False,v=False,below=False):add('player',raw(pos),*[str(x).lower()for x in [ground,flying,False,h,v,below]],raw([pos[0]-.3,pos[1],pos[2]-.3,pos[0]+.3,pos[1]+.6,pos[2]+.3]));add('delta',raw(delta))
 def program(plan='F',air='F',points=((.5,10,.5),(40.5,0,.5)),rockets=0,approach=0,reach=True):add('program',plan,air,';'.join(map(raw,points)),rockets,bits(approach),str(reach).lower())
 def new(floor=-1):add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-32,160,bits(-1000),bits(1000),'false',floor,1);add('runtime');add('equipment',0,1,','.join(['0']*9));player();program()
 def nav(goal=(40.5,0,.5),radius=.5,ability='0,20,8000',direct=False,yaw=0):add('nav',raw(goal),bits(radius),ability,str(direct).lower(),fbits(yaw))
 def cycle(n=1,**kw):
  for _ in range(n):add('run');nav(**kw)
 # Capture builds all ten inventory entries before asking the chest stack.
 for can in [False,True]:
  for rockets in [False,True]:
   for chest in [0,1,4]:new();add('equipment',2,chest,','.join(map(str,[0,2,0,3,0,0,0,0,0])));add('options',str(can).lower(),str(rockets).lower(),bits(0),3);add('capture')
 # Planning, source worker slice/poll and takeoff presses/open cooldown.
 new();program('S1,F');nav();nav();cycle(3)
 for tick in range(13):player(pos=(tick*.3+.5,10.5,.5),ground=False,flying=tick>=11,delta=(.3,-.1,0));nav()
 player(pos=(40.5,0,.5),ground=True,flying=True);nav();nav();add('glideHalt')
 # Stale route starts and exact tolerance: do not jump from a result for another position.
 for offset in [.999999,1,1.000001]:new();nav();nav();add('run');player(pos=(.5+offset,10,.5));nav();nav()
 # Search failures/exception and cooldown including unfinished takeoff.
 for code in ['ENO_PATH','A','X']:
  new();program(code);nav();nav();cycle(2);program('F');cycle(42)
 new();nav();nav();cycle(2)
 for _ in range(25):nav()
 # Falling is taken over only where the allowed drop has no solid/fluid landing.
 for state in [0,1,7,36]:
  new(floor=-20);player(pos=(.5,10,.5),ground=False,delta=(.1,-.2,0));add('set','0,7,0',state);nav();nav();add('glideHalt')
 new(floor=-20);player(ground=False,flying=True,delta=(.7,-.1,0));nav();nav();cycle(2)
 for goal in [(41.999999,0,.5),(42,0,.5),(42.000001,0,.5),(42.000001,1,.5)]:nav(goal=goal)
 add('glideHalt');add('run');nav(ability='-')
 # AlreadyThere, failure and stage routes within an airborne passage.
 for code in ['A','ENO_PATH','S1,F']:
  new(floor=-20);program(air=code);player(ground=False,flying=True,delta=(.8,-.1,0));nav();cycle(4);add('glideLevel');cycle(2);add('glideHalt')
 new(floor=-20);program(air='F',points=((.5,10,.5),(200.5,10,.5)),reach=False);player(ground=False,flying=True,delta=(1,-.1,0));nav(goal=(500.5,0,.5));cycle(3,goal=(500.5,0,.5));player(pos=(145.5,10,.5),ground=False,flying=True,delta=(1,-.1,0));nav(goal=(500.5,0,.5));cycle(2,goal=(500.5,0,.5))
 # Rocket cruise and fire/release order in offhand and the first matching hotbar slot.
 for offhand,slots in [(2,[0]*9),(0,[0,0,3,2,0,0,0,0,0]),(0,[0]*9)]:
  new(floor=-20);add('equipment',offhand,1,','.join(map(str,slots)));nav(goal=(300.5,0,.5),ability='30,20,8000');player(pos=(.5,11,.5),ground=False,flying=True,delta=(.1,-.1,0));nav(goal=(300.5,0,.5),ability='30,20,8000');nav(goal=(300.5,0,.5),ability='30,20,8000');add('glideHalt')
 # Ceiling, unavailable dimensions and direct movement retain original early exits.
 for ceiling,present,direct in [(True,True,False),(False,False,False),(False,True,True)]:new();add('environment',str(ceiling).lower(),0,str(present).lower());nav(goal=(300.5,0,.5),ability='30,20,8000',direct=direct);add('glideHalt')
 # Ground/roof cache invalidation at the same background terrain object.
 new();program('S1');nav();nav();add('set','0,0,0',1);add('glideChange','0,0,0');cycle(23);add('glideLevel');add('run');add('glideHalt')
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/n for n in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]+[AGENT/'navigation/glide'/n for n in ['Gliding.kt','Airways.kt','Contacts.kt','RoofBounce.kt','GlideCapture.kt']]+[AGENT/'pathfinder/glide'/n for n in ['Passage.kt','Landing.kt','Cruise.kt','Rockets.kt','Steering.kt','GlideAbilities.kt']]+[AGENT/'pathfinder/route'/n for n in ['BackgroundSearch.kt','Replanner.kt','Destination.kt']]+[AGENT/'navigation/Progress.kt',AGENT/'math/VectorExtensions.kt',AGENT/'math/InputExtensions.kt',AGENT/'world/LevelBlocks.kt',AGENT/'simulation/ElytraMotion.kt']
 for idx,p in enumerate(original):shutil.copyfile(p,out/f'{idx:02}_{p.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text(encoding='utf-8');imports,base_extra=(HERE/'navigation-source-fixture/GlideRuntimeBase.kt').read_text(encoding='utf-8').split('// fixture code\n',1);more_imports,extra=(HERE/'navigation-source-fixture/Gliding.kt').read_text(encoding='utf-8').split('// fixture code\n',1)
 base_extra=base_extra.replace('private fun extra(','private fun baseExtra(')
 base_extra=base_extra.replace('"getHeight"->runtimeWorld.height','"getHeight"->if(a.isNullOrEmpty())runtimeWorld.height else runtimeHeight(a[a.size-2]as Int,a[a.size-1]as Int)')
 base_extra=base_extra.replace('"hasChunkAt"->{val x=', '"hasChunkAt"->{if(a!!.size==1){val p=a[0]as BlockPos;events+="J:${p.raw()}";BlockPos(p.x,0,p.z)!in runtimeWorld.unknown}else{val x=').replace('BlockPos(x,0,z)!in runtimeWorld.unknown}', 'BlockPos(x,0,z)!in runtimeWorld.unknown}}')
 base_extra=base_extra.replace('else->error("Level.${m.name}")','"hasCeiling"->{events+="E";runtimeCeiling};"snapshot"->{events+="N:"+a!!.joinToString(",");snapshotRuntime()};else->error("Level.${m.name}")')
 base_extra=base_extra.replace('else->error("Player.${m.name}")','"getDeltaMovement"->{events+="d";runtimeDelta};"blockPosition"->{events+="q";BlockPos.containing(runtimePosition)};"getInventory"->{events+="i";runtimeInventory};"getOffhandItem"->{events+="o";runtimeStacks.getValue(runtimeOffhand)};"getItemBySlot"->{events+="c";runtimeStacks.getValue(runtimeChest)};else->error("Player.${m.name}")')
 base_extra=base_extra.replace('else->error("Body.${m.name}")','"releaseHotbarSlot"->{events+="R";Unit};"selectHotbarSlot"->{events+="B:${a!![0]}";Unit};"useItem"->{events+="U:${a!![0]}";true};else->error("Body.${m.name}")')
 stack=(HERE/'navigation-source-fixture/Glide.kt').read_text(encoding='utf-8');stack=stack[stack.index('private class Stack'):stack.index('fun main(){')]
 base=base.replace('package fixture','package fixture\n'+imports+more_imports).replace('fun main(){',stack+'\n'+base_extra+'\n'+extra+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)');(out/'Fixture.kt').write_text(base,encoding='utf-8')
 context=(HERE/'navigation-source-fixture/GlideRuntimeContext.kt').read_text(encoding='utf-8').replace('val elytraRockets:Boolean=true)','val elytraRockets:Boolean=true,val glideClearance:Double=0.0)').replace('fun options()=current','fun options():RuntimeOptions{fixture.RuntimeProbe.events+="O";return current}')+'\ninternal fun NavigationContext.safeFall():Int=fixture.runtimeFall()\n';(out/'Context.kt').write_text(context,encoding='utf-8');shutil.copyfile(HERE/'navigation-source-fixture/GlidingSearch.kt',out/'SearchProvider.kt')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text(encoding='utf-8').replace('cell::RefCell','sync::Mutex').replace('rc::Rc','sync::Arc').replace('RefCell','Mutex').replace('Rc','Arc').replace('.borrow_mut()','.lock().unwrap()').replace('.borrow()','.lock().unwrap()').replace('struct Data {','#[derive(Clone)]struct Data {').replace('pos.y <= self.max_y()', 'pos.y <= data.min+data.count-1')
 fixture=fixture.replace('self.registry.lock().unwrap().blocks[&self.def().block]', '{let key=self.def().block;self.registry.lock().unwrap().blocks[&key]}').replace('pos.y <= data.min+data.count-1', 'pos.y < data.min+data.count')
 fixture=fixture.replace('#[test]',(HERE/'navigation-source-fixture/gliding.rs').read_text(encoding='utf-8')+'\n#[test]',1).replace('let mut results = Vec::new();','let mut runtime:Option<Runtime>=None;let stacks=Arc::new(Mutex::new(HashMap::new()));let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut runtime,&stacks),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_complete_gliding_airways_and_capture')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),next(kotlin.glob('slf4j-api-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-15000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-DdirectionHashes='+str(out/'direction-hashes.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')};(out/'navigation/glide/gliding_oracle.rs').write_text(fixture,encoding='utf-8')
 with(out/'navigation/glide/gliding.rs').open('a',encoding='utf-8')as file:file.write('\n#[cfg(test)]#[path="gliding_oracle.rs"]mod oracle;\n')
 path=out/'navigation/glide/airways.rs';source=path.read_text(encoding='utf-8').replace('struct Clearance(','pub(crate) struct Clearance(')
 for name in ['routes','has_route','reaches_goal','found_at']:source=source.replace('    '+name+':','    pub(crate) '+name+':',1)
 path.write_text(source,encoding='utf-8')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-gliding-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Terrain.kt','terrain.rs','GlideRuntimeBase.kt','GlideRuntimeContext.kt','Glide.kt','Gliding.kt','GlidingSearch.kt','gliding.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged complete Gliding/Airways/Contacts/Capture and controllers; exact private state, outputs, actor/world query order, owned background worker lifecycle and snapshot cache invalidation','boundaries':['search providers return controlled outcomes; full path generation has separate original-source oracles','Direction identities are explicit host facts and use no JVM in production','native binding and complete AutoWalk remain separate'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
