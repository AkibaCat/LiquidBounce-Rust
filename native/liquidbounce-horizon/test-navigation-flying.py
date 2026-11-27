#!/usr/bin/env python3
"""Unchanged Flying/Follower/Replanner with a controlled observable search provider."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,struct,subprocess,time
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'
def bits(v):return struct.unpack('<Q',struct.pack('<d',v))[0]
def fbits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def raw(v):return ','.join(str(bits(float(x)))for x in v)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*p):commands.append(' '.join(map(str,p)))
 def player(t=0,pos=(0,0,0),delta=(0,0,0),yaw=0,sprint=False,horizontal=False,vertical=False,ground=False,creative=False,mayfly=False):add('player',t,raw(pos),raw(delta),fbits(yaw),*[str(v).lower()for v in [sprint,horizontal,vertical,ground,creative,mayfly]],fbits(.05))
 def base(control='DIRECT',speed=1,vertical=.4,flying=True):add('base',control,bits(speed),bits(vertical),str(flying).lower())
 def program(code='S1,F',points=((0,0,0),(8,0,0)),reach=True):add('program',code,';'.join(map(raw,points)),str(reach).lower())
 def new(code='S1,F',points=((0,0,0),(8,0,0)),control='DIRECT',speed=1,vertical=.4,flying=True,reach=True):add('new');player();base(control,speed,vertical,flying);program(code,points,reach)
 def nav(goal=(8,0,0),radius=.5,length=64,yaw=0):add('nav',raw(goal),bits(radius),bits(length),fbits(yaw))
 def cycle(n=1,goal=(8,0,0),radius=.5,length=64):
  for _ in range(n):add('run');nav(goal,radius,length)
 # Initial request and real background slice/poll/preview transitions.
 new();nav();nav();nav();cycle(4)
 for x in [1.499999,1.5,1.500001,3,3.001]:player(pos=(0,0,0));nav((8+x,0,0));cycle(2,(8+x,0,0))
 # AlreadyThere accepts the exact current position plus the target.
 for pos,goal in [((0,0,0),(0,0,0)),((.5,.2,.5),(.7,.3,.4)),((0,0,0),(3,2,1))]:
  new('A');player(pos=pos);nav(goal);nav(goal);cycle(3,goal)
 # Failed searches, retry at twenty ticks, and thrown worker exceptions.
 for code in ['ENO_PATH','ETOO_FAR','ETIMED_OUT','X']:
  new(code);nav();nav();cycle(2);program('F');cycle(26)
 # A live block change invalidates the same terrain cache after 20 polls.
 for known in [False,True]:
  new('S1');add('known',str(known).lower());nav();nav();cycle(2);add('state','0,0,0',1);add('change','0,0,0');cycle(23);add('level');cycle(3)
 # Cancelled dependent futures leave their already queued worker alive.
 new('S1');nav();nav();add('halt');add('run');nav();nav();cycle();add('level');add('run');nav()
 # Missing level prevents requests without disturbing the flying tick report.
 new('F');add('world','false');nav();nav();add('world','true');nav();nav();cycle(3);add('world','false');nav();add('halt')
 # Stage lead starts the next segment while following an incomplete route.
 new('F',((0,0,0),(20,0,0)),reach=False);nav((100,0,0));nav((100,0,0));cycle(3,(100,0,0));player(pos=(9,0,0));nav((100,0,0));cycle(3,(100,0,0));player(pos=(20,0,0));nav((100,0,0));cycle(3,(100,0,0))
 # Exact half range, changed radius, changed acceleration control and unknown goal.
 for goal in [(31.999999,0,0),(32,0,0),(4,400,0)]:
  new('F');nav(goal);nav(goal);cycle(3,goal);nav(goal,1);base('MOMENTUM',.1,.3);nav(goal,1);cycle(3,goal,1)
 # Off path requests are urgent even before the normal interval expires.
 for offset in [2.5,2.500001,4,4.000001]:
  new('F');nav();nav();cycle(2);player(4,(3,0,offset));nav();cycle(3)
 # Full follower transitions remain observable inside the orchestrator.
 for control in ['DIRECT','MOMENTUM']:
  for path in [((0,0,0),(4,0,0),(4,0,4)),((0,0,0),(4,1,0),(8,1,1))]:
   new('F',path,control,.6);nav();nav();cycle(2)
   for i,pos in enumerate([(0,0,0),(1,0,0),(3.8,0,0),(4,0,0),(4,0,3),(8,1,1)]):player(i*3,pos);nav(yaw=33.25)
 # Stall result triggers a replan; halt resets control state but retains measurements.
 new('F');nav();nav();cycle(2)
 for t in range(70):player(t*3);nav()
 add('halt');nav();nav();cycle(3)
 # Creative takeoff/landing and resumed routes use the same retained handling.
 new('F',((.5,.8,.5),(.5,0,.5)),'MOMENTUM',.2,.4);player(pos=(.5,.8,.5),creative=True);nav((.5,0,.5));nav((.5,0,.5));cycle(2,(.5,0,.5));base('MOMENTUM',.2,.4,False);player(5,(.5,0,.5),ground=True);nav((.5,0,.5));add('halt')
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'navigation/flight'/n for n in ['Flying.kt','Handling.kt','Follower.kt']]+[AGENT/'navigation'/n for n in ['ProgressWatch.kt','FollowState.kt']]+[AGENT/'pathfinder/flight/FlyAbilities.kt']+[AGENT/'pathfinder/route'/n for n in ['BackgroundSearch.kt','Replanner.kt','Destination.kt']]+[AGENT/'math'/n for n in ['InputExtensions.kt','VectorExtensions.kt']]
 for i,p in enumerate(original):shutil.copyfile(p,out/f'{i:02}_{p.name}')
 for source,target in [('Flying.kt','Fixture.kt'),('FlyingContext.kt','Context.kt'),('FlyingSearch.kt','SearchProvider.kt')]:shutil.copyfile(HERE/'navigation-source-fixture'/source,out/target)
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),next(kotlin.glob('slf4j-api-*.jar')),ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar',binary]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-15000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')}
 shutil.copyfile(HERE/'navigation-source-fixture/flying.rs',out/'navigation/flying_oracle.rs')
 with(out/'navigation/flying.rs').open('a',encoding='utf-8')as file:file.write('\n#[cfg(test)]#[path="flying_oracle.rs"]mod flying_oracle;\n')
 source=(HERE/'navigation-source-fixture/flight_follower.rs').read_text(encoding='utf-8');helpers=source[source.index('fn raw(v:f64)'):source.index('fn acceleration')]+source[source.index('fn state(f:'):source.index('#[test]')];helpers=helpers.replace('fn state(','pub(crate) fn fixture_state(').replace('fn raw(', 'fn fixture_raw(').replace('fn vector(', 'fn fixture_vector(').replace('fn vecs(', 'fn fixture_vecs(').replace('raw(', 'fixture_raw(').replace('vector(', 'fixture_vector(').replace('vecs(', 'fixture_vecs(').replace('fixture_fixture_','fixture_').replace('map(raw)','map(fixture_raw)').replace(',vector)',',fixture_vector)').replace('.map(vector)', '.map(fixture_vector)')
 with(out/'navigation/flight_follower.rs').open('a',encoding='utf-8')as file:file.write('\n#[cfg(test)]pub(crate) use fixture_helpers::fixture_state;\n#[cfg(test)]mod fixture_helpers {use super::*;'+helpers+'}\n')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-flying-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,Path(__file__),*[HERE/'navigation-source-fixture'/p for p in ['Flying.kt','FlyingContext.kt','FlyingSearch.kt','flying.rs']]];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged complete Flying/Follower/Handling/Replanner/BackgroundSearch/StageDestination; exact state, route reports, main-thread snapshot/known/invalidation order, queued worker lifecycle and observable controlled owning search provider','boundaries':['search provider returns controlled outcomes; full path generation has its separate original-source oracle','report callback is observable context seam; renderer deduplication is separate','native Player/world/snapshot binding and complete AutoWalk remain unintegrated'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
