#!/usr/bin/env python3
"""Full land PlayerMotion modern/1.8 differential against unchanged Agent sources."""
import argparse,hashlib,itertools,json,os,shutil,struct,subprocess,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'
def d(v):return str(struct.unpack('<Q',struct.pack('<d',v))[0])
def f(v):return str(struct.unpack('<I',struct.pack('<f',v))[0])
def raw(v):return ','.join(map(d,v))
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);commands=[];records=[]
 def add(*parts):commands.append(' '.join(map(str,parts)))
 def boolean(v):return str(v).lower()
 shapes={2:[0,0,0,1,.5,1],3:[.375,0,.375,.625,1.5,.625],4:[0,0,0,1,.5,1,0,.5,0,.5,1,1],5:[0,0,0,1,.5625,1],6:[0,0,0,1,.875,1]}
 add('shape',0,'E');add('shape',1,'F')
 for ident,boxes in shapes.items():add('shape',ident,raw(boxes))
 def define(id,block,shape=1,flags=0,fluid=False,air=False,friction=.6,speed=1.,jump=1.,bounce=0.,reduction=0.):add('define',id,block,flags,boolean(fluid),boolean(air),shape,f(friction),f(speed),f(jump),f(bounce),f(reduction))
 define(0,'air',shape=0,air=True);define(1,'stone');define(2,'ice',friction=.98);define(3,'blue_ice',friction=.989)
 define(4,'honey_block',speed=.4,jump=.5);define(5,'soul_sand',shape=6,speed=.4);define(6,'slime_block',flags=1,bounce=1.)
 define(7,'red_bed',shape=5,bounce=.66);define(8,'hay_block',reduction=.8);define(9,'oak_fence',shape=3,flags=2)
 define(10,'cobblestone_wall',shape=3,flags=4);define(11,'oak_fence_gate',shape=3,flags=8);define(12,'cobweb',shape=0)
 define(13,'water',shape=0,fluid=True);define(14,'void_air',shape=0,air=True);define(15,'cave_air',shape=0,air=True)
 define(16,'fixture_lowfriction',friction=.4);define(17,'fixture_jump',jump=1.5);define(18,'fixture_partialspeed',speed=.9)
 resting=-(.08*struct.unpack('<f',struct.pack('<f',.98))[0])
 def new(mechanics,floor=1,boost=0.,speed=.1,strength=.42):add('new',0,floor,mechanics,f(speed),f(strength),f(boost))
 def reset(pos=(.5,1.,.5),delta=(0.,resting,0.),ground=True,sprint=False,delay=0):add('reset',raw(pos),raw(delta),boolean(ground),boolean(sprint),delay)
 def tick(yaw=0.,forward=1,strafe=0,jump=False,sprint=False,sneak=False):add('tick',f(yaw),forward,strafe,boolean(jump),boolean(sprint),boolean(sneak))
 # Repeated grounded and airborne ticks, speed factors, bounce/reduction, and crouching one-tick lag.
 for mechanics,floor,pattern in itertools.product(['modern','1.8'],[1,2,3,4,5,6,7,8,9,10,11,16,17,18],range(4)):
  top={5:.875,7:.5625,9:1.5,10:1.5,11:1.5}.get(floor,1.)
  new(mechanics,floor);reset(pos=(.5,top,.5));add('stands','0,0,0');add('footing','0,0,0')
  for t in range(36):
   tick(yaw=[0.,45.,90.,-180.,270.,540.][(t//6)%6],forward=1 if t<28 else 0,strafe=1 if pattern==1 else (-1 if pattern==2 else 0),jump=(pattern>=2 and t<25),sprint=t<20,sneak=(pattern==3 and 8<=t<18))
 # Falling onto each block with an upward velocity floor, ceiling collision and nonstandard jump traits.
 for mechanics,floor,boost in itertools.product(['modern','1.8'],[1,4,5,6,7,8],[0.,.1,.5]):
  new(mechanics,floor,boost);add('set','0,3,0',1);reset(pos=(.5,7.,.5),delta=(.01,-.25,-.02),ground=False)
  for t in range(60):tick(yaw=37.125,forward=0,strafe=0,jump=t>=35,sprint=False,sneak=48<=t<52)
 # Stepping and colliding against partial blocks; diagonal axis order differs by mechanics.
 for mechanics,obstacle,yaw in itertools.product(['modern','1.8'],[1,4,5,9],[0.,45.,90.,-90.]):
  new(mechanics);add('set','0,1,1',obstacle);add('set','1,1,0',obstacle);reset(delta=(.25,resting,.35))
  for t in range(24):tick(yaw=yaw,forward=1,strafe=1,jump=t==12,sprint=True)
 # Original touchBlocks and shape cache, including multiple soul-sand effects in one move.
 for mechanics,state in itertools.product(['modern','1.8'],[4,5,12,13,14,15,18]):
  new(mechanics);add('set','0,1,0',state);add('set','0,2,0',state);reset()
  for t in range(20):tick(yaw=0.,forward=1,strafe=0,jump=t<10,sprint=True,sneak=t%7==0)
 # Backing off open edges, copy/cache sharing, and reset's intentionally retained fields.
 for mechanics,delta in itertools.product(['modern','1.8'],[[.3,.2],[-.3,-.2],[.05,-.05]]):
  new(mechanics,0);add('set','0,0,0',1);reset(pos=(.95,1.,.5),delta=(delta[0],resting,delta[1]));add('fields','true','true',d(123.),d(4.),7);add('copy');add('set','0,0,0',0)
  for t in range(8):tick(yaw=23.,forward=1,jump=t%2==0,sprint=True,sneak=True)
  add('swap');tick(sneak=False);add('copyFrom');reset();tick();add('footing','0,0,0')
 # Actual Player overload through a JVM interface proxy: getter order straddles support queries.
 for mechanics,ground,crouch,support in itertools.product(['modern','1.8'],[False,True],[False,True],['-','4,5,6']):
  new(mechanics);add('fields','true','true',d(9.),d(4.),7)
  add('resetPlayer',raw([.5,1.,.5]),raw([.01,resting,.02]),boolean(ground),'true',3,boolean(crouch),support);tick(sneak=True)
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 names=['BlockColliders.kt','BlockGrid.kt','Collisions.kt','EdgeBackOff.kt','Support.kt','MotionTraits.kt','PlayerMotion.kt','BlockEffects.kt','Motion1_8.kt'];originals=[AGENT/'simulation'/name for name in names]
 for src in originals:shutil.copyfile(src,out/src.name)
 shutil.copyfile(HERE/'simulation-source-fixture/Motion.kt',out/'Fixture.kt')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append(dict(name=name,exitCode=r.returncode,seconds=round(time.monotonic()-began,3)));(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr,end='');return r.returncode
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*map(str,out.glob('*.kt'))])
 if not result:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'net.ccbluex.agent.simulation.FixtureKt'])
 nav=ROOT/'native/liquidbounce-behaviors/src/navigation';lib=f'#![allow(dead_code)]\n#[path="{(nav/"mod.rs").as_posix()}"]pub mod navigation;\n#[cfg(test)]#[path="{(HERE/"simulation-source-fixture/motion.rs").as_posix()}"]mod tests;\n'
 (out/'lib.rs').write_text(lib,encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-simulation-motion-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if not result:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/simulation-motion-target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if not result:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*originals,binary,fastutil,Path(__file__),HERE/'simulation-source-fixture/Motion.kt',HERE/'simulation-source-fixture/motion.rs',*sorted((nav/'simulation').glob('*'))]
 report=dict(passed=len(records)==4 and all(r['exitCode']==0 for r in records),commands=records,steps=len(commands),sourceSha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files if p.is_file()},unchangedOriginalKotlin=all(p.read_bytes()==(out/p.name).read_bytes()for p in originals),scope='all nine unchanged land simulation sources, modern and 1.8 sequential full state and ordered block/shape queries',boundaries=['controlled exact block/registry/shape facts, native registry/world binding not_run','actual vanilla SimulatedPlayer and SafeWalk full-client differential not_run'])
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(passed=report['passed'],steps=len(commands))));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
