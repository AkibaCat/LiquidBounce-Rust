#!/usr/bin/env python3
"""Unchanged glide abilities, rockets, motion, Landing and Cruise oracle."""
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
 def add(*a):commands.append(' '.join(map(str,a)))
 def stack(id,glide=False,damageable=False,maximum=0,damage=0,rocket=False,count=0,flight='-'):add('stack',id,str(glide).lower(),str(damageable).lower(),maximum,damage,str(rocket).lower(),count,flight)
 for id,g,damageable,maximum,damage in [(0,False,False,0,0),(1,True,False,0,0),(2,True,True,432,0),(3,True,True,432,430),(4,True,True,432,432),(5,True,True,2147483647,-2147483648)]:stack(id,g,damageable,maximum,damage)
 for id,count,flight in [(10,64,'-'),(11,2,3),(12,0,0),(13,-1,-2),(14,2147483647,2147483647),(15,1,-2147483648)]:stack(id,rocket=True,count=count,flight=flight)
 for id in range(6):
  for flag in ['false','true']:
   for selected in ['-','0','10','10,11,0','11,10','12,13','14,15','10,14,15','15,14']:add('abilities',id,flag,selected)
 for left in [-2147483648,-1,0,1,4,2147483647]:
  for burn in [-3,0,1,2,20,2147483647]:
   add('rocket',left,burn);add('burn');add('fire')
   for _ in range(23 if burn==20 else 4):add('burn')
   add('fire');add('burn')
 for yaw in [-360,-180,-135,-90,-45,-0.0,0.0,33.25,90,179.999,180,359.99,721]:
  for pitch in [-91,-90,-89,-45,-0.0,0.0,10,45,89,90,91]:
   for movement in [(0,0,0),(.3,-.2,.4),(-1,1,2),(2,-3,-1)]:
    for boost in ['false','true']:add('motion',raw(movement),fbits(yaw),fbits(pitch),boost)
 for x in [-10,-1,-0.0,0.0,1,10,float('inf'),float('nan')]:
  for z in [-1,-0.0,0.0,1]:add('angles',bits(x),bits(z),bits(z),bits(x))
 # Persistent sweep state, approach early return and rocket countdown.
 for target in [(0,0,0),(20,0,5),(-20,40,2),(50,-20,0)]:
  for rockets,burn,approach in [(0,0,0),(2,20,0),(3,1,16),(1,0,16)]:
   add('landing',raw(target),rockets,burn,bits(approach))
   for pos,delta in [((0,10,0),(0,0,0)),((0,1000,0),(0,-.1,0)),((100,5,0),(1,-.2,0)),((1,0,1),(0,-1,0)),((0,-10,0),(.1,.2,.3))]:add('land',raw(pos),raw(delta))
   add('position',raw((0,20,0)),raw((.3,-.1,.3)))
   for _ in range(25):add('step','L')
 # Column sample order, unknown clearance, integer coordinates and zero-length.
 for mode in range(4):
  add('height',mode)
  for start,goal in [((0,0,0),(0,0,0)),((-.5,2,-.5),(7.5,3,3.5)),((0,20,0),(4,0,0)),((0,10,0),(4.000001,0,0)),((-10,30,2),(50,-20,5))]:add('altitude',raw(start),raw(goal))
 for distance in [-1,0,20,64,128,1000,float('inf'),float('nan')]:
  for climb in [-5,0,2,40,float('nan')]:
   for burn in [-1,0,1,20,40]:add('needed',bits(distance),bits(climb),burn)
 for mode in range(4):
  add('height',mode)
  for rockets,burn,ceiling in [(0,0,320),(1,20,50),(4,2,320)]:
   for fallback in ['-',raw((5,0,2))]:
    add('fallback',fallback);add('cruise',raw((200,0,30)),rockets,burn,ceiling)
    for pos,delta in [((0,0,0),(0,0,0)),((0,20,0),(2,0,0)),((30,20,10),(2,-.2,0))]:add('sail',raw(pos),raw(delta))
    add('position',raw((0,20,0)),raw((.3,-.1,.3)))
    for _ in range(25):add('step','C')
    add('sail',raw((150,2,30)),raw((.5,-.1,0)))
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/glide'/name for name in ['GlideAbilities.kt','Rockets.kt','Steering.kt','Landing.kt','Cruise.kt']]+[AGENT/'simulation/ElytraMotion.kt']
 for i,p in enumerate(original):shutil.copyfile(p,out/f'{i:02}_{p.name}')
 shutil.copyfile(HERE/'navigation-source-fixture/Glide.kt',out/'Fixture.kt')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-15000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').rglob('*.rs')}
 shutil.copyfile(HERE/'navigation-source-fixture/glide.rs',out/'navigation/glide/oracle.rs')
 with(out/'navigation/glide/mod.rs').open('a',encoding='utf-8')as f:f.write('\n#[cfg(test)]mod oracle;\n')
 with(out/'navigation/glide/landing.rs').open('a',encoding='utf-8')as f:f.write('''
#[cfg(test)]impl Landing {pub(crate) fn fixture_state(&self)->String {
 fn raw(v:f64)->u64 {if v.is_nan(){0x7ff8_0000_0000_0000}else{v.to_bits()}}
 format!("{},{},{},{},{}:{},{},{}",raw(self.sweep.best_error),self.sweep.best_x_rot.to_bits(),raw(self.sweep.best_along),self.sweep.farthest_x_rot.to_bits(),raw(self.sweep.farthest_along),self.rockets.left(),self.rockets.boost_ticks(),self.rockets.can_fire())
}}
''')
 with(out/'navigation/glide/cruise.rs').open('a',encoding='utf-8')as f:f.write('''
#[cfg(test)]impl<H:ColumnHeights,F:FnMut(Vec3)->Option<Vec3>> Cruise<H,F> {pub(crate) fn fixture_state(&self)->String {
 fn raw(v:f64)->String {(if v.is_nan(){0x7ff8_0000_0000_0000}else{v.to_bits()}).to_string()}
 let target=self.target();let r=self.rockets();format!("{},{}:{}:{}:{},{},{}",raw(self.altitude),self.ticks,self.landing.as_ref().map_or_else(||"-".into(),Landing::fixture_state),[target.x,target.y,target.z].map(raw).join(","),r.left(),r.boost_ticks(),r.can_fire())
}}
''')
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-glide-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,Path(__file__),HERE/'navigation-source-fixture/Glide.kt',HERE/'navigation-source-fixture/glide.rs'];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'unchanged complete GlideAbilities, Rockets, ElytraMotion, Landing, Cruise; exact return bits/private sweeps/rocket state/live item getter/column and fallback query order','boundaries':['native equipment/rocket facts and column heights are explicit providers','complete glide route search, Passage and runtime Gliding remain separate work'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
