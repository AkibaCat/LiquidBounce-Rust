#!/usr/bin/env python3
"""Untouched Agent collision/motion-trait source differential with explicit grids."""
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
 shapes={2:[0,0,0,1,.5,1],3:[0,0,0,1,.6,1],4:[0,0,0,1,.60000001,1],5:[.375,0,.375,.625,1.5,.625],6:[0,0,0,1,.5,1,0,.5,0,.5,1,1],7:[1,1,1,0,0,0],8:[.5,0,.5,.5,.5,.5],9:[0,0,0,1,1,1,0,1,0,1,1.5,1],10:[0,0,0,1,-0.,1],11:[]}
 add('shape',0,'E');add('shape',1,'F')
 for ident,boxes in shapes.items():add('shape',ident,raw(boxes) if boxes else 'E')
 # Direct clipping, axis order, near-zero movement and epsilon boundaries.
 body=[.2,1,.2,.8,2.8,.8]
 for obstacle,delta,ground in itertools.product([1,2,3,4,5,6,7,8,9,10,11],[[1,-.1,1],[.2,-.1,.8],[.8,-.1,.2],[0,0,0],[1e-8,-1e-8,-1e-8],[-1,-1,-1],[1,1,0]],[False,True]):
  add('new',0,1);add('set','1,1,0',obstacle);add('set','0,1,1',obstacle);add('collide',raw([.5,1,.5]),str(ground).lower(),raw(delta));add('collide',raw([.5,1,.5]),str(ground).lower(),raw(delta))
 for shape in range(12):
  for y in [0.,.5,1.,1.5]:
   add('new',-10,0);add('set','0,0,0',shape);add('collect',raw([-.2,-.2,-.2,1.2,3,1.2]));add('steps',raw([.2,y,.2,.8,y+1.8,.8]),f(.6),f(0.));
   for axis in range(3):add('offset',axis,raw(body),d(-2.));add('offset',axis,raw(body),d(2.))
   add('move',raw([.3,-.2,.7]),raw(body));add('intersects','0,0,0',raw([0,0,0,1,1,1]));add('footing','0,0,0',d(.5),d(.5),d(.3))
 # Closest support order and the retained coordinates after clearing/missing support.
 for shape,x,z in itertools.product([1,2,5,6,8,9],[.0,.5,.7,1.0,-.3],[.0,.5,1.0]):
  add('new',-10,0);add('set','0,0,0',shape);add('set','1,0,0',1);add('support',raw([x,1.,z]),d(0.),d(0.));add('clear');add('support',raw([x,1.5,z]),d(.2),d(-.1))
 # Real canFall controls both edge loops, including corners and signed-zero inputs.
 for shape,x,delta in itertools.product([1,2,5,6,8],[.5,.95,1.2,-.1],[[.6,.2],[-.6,-.2],[.05,-.05],[.0,-.0],[.3,.8]]):
  add('new',-10,0);add('set','0,0,0',shape);add('fall',raw([x,1,.5]),d(delta[0]),d(delta[1]),d(.6));add('edge',raw([x,1,.5]),d(delta[0]),d(delta[1]));add('edge',raw([x,1,.5]),d(delta[0]),d(delta[1]))
 # Cache is persistent across grid edits, shared by all collider operations.
 add('new',-10,0);add('set','0,0,0',1);add('intersects','0,0,0',raw([0,0,0,1,1,1]));add('set','0,0,0',0);add('intersects','0,0,0',raw([0,0,0,1,1,1]));add('support',raw([.5,1,.5]),d(0.),d(0.))
 for speed,strength,boost,mechanics in itertools.product([.0,.1,.13,.33333334],[.0,.42,.5,1.0],[-.1,.0,.10000001,.5],['modern','1.8']):add('traits',f(speed),f(strength),f(boost),mechanics)
 for rise in [-1.,-.0,.0,.003,.42,.42000001668930054,.5,1.,4.,float('nan')]:add('rise',d(rise))
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 names=['BlockColliders.kt','BlockGrid.kt','Collisions.kt','EdgeBackOff.kt','Support.kt','MotionTraits.kt','PlayerMotion.kt','BlockEffects.kt','Motion1_8.kt']
 originals=[AGENT/'simulation'/name for name in names]
 for src in originals:shutil.copyfile(src,out/src.name)
 shutil.copyfile(HERE/'simulation-source-fixture/Collision.kt',out/'Fixture.kt')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar'
 cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append(dict(name=name,exitCode=r.returncode,seconds=round(time.monotonic()-began,3)));(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr,end='');return r.returncode
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*map(str,out.glob('*.kt'))])
 if not result:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'net.ccbluex.agent.simulation.FixtureKt'])
 nav=ROOT/'native/liquidbounce-behaviors/src/navigation';sources={name:nav/(name+'.rs')for name in ['geometry','aabb','voxel_shape']}
 for name in ['block_grid','block_colliders','collisions','edge_backoff','support','motion_traits']:sources[name]=nav/'simulation'/(name+'.rs')
 lib=f'#![allow(dead_code)]\n#[path="{(nav/"mod.rs").as_posix()}"]pub mod navigation;\n'
 lib+=f'#[cfg(test)]#[path="{(HERE/"simulation-source-fixture/collision.rs").as_posix()}"]mod tests;\n'
 (out/'lib.rs').write_text(lib,encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-simulation-collision-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if not result:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/simulation-collision-target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if not result:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*originals,binary,fastutil,Path(__file__),HERE/'simulation-source-fixture/Collision.kt',HERE/'simulation-source-fixture/collision.rs',nav/'simulation/mod.rs',*sources.values()]
 report=dict(passed=len(records)==4 and all(r['exitCode']==0 for r in records),commands=records,steps=len(commands),sourceSha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},unchangedOriginalKotlin=all(p.read_bytes()==(out/p.name).read_bytes()for p in originals),scope='actual BlockColliders/Collisions/Support/EdgeBackOff/MotionTraits sources, exact outputs and ordered cached shape queries',boundaries=['controlled shape-grid provider; no block state/game registry native binding','PlayerMotion/BlockEffects/Motion1_8 compiled as unchanged dependencies but tick/reset behavior not exercised','SafeWalk/Parkour/full client differential not_run'])
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(passed=report['passed'],steps=len(commands))));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
