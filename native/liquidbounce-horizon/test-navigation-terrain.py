#!/usr/bin/env python3
"""Terrain source parity. World/registry facts are controlled observable adapter seams."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import struct
import subprocess
import time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'
def bits(v):return struct.unpack('<Q',struct.pack('<d',v))[0]
def fbits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def raw(values):return ','.join(str(bits(float(v)))for v in values)
def pos(x,y=0,z=0):return f'{x},{y},{z}'
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*p):commands.append(' '.join(map(str,p)))
 shape_boxes={2:[0,0,0,1,1,1],3:[0,0,0,1,.5,1],4:[0,0,0,1,.875,1],5:[0,0,0,1,.9375,1],6:[0,0,0,1,.5625,1],7:[0,0,0,1,.0625,1],8:[.3,0,.3,.7,1.5,.7],9:[0,0,.375,1,1.5,.625],10:[0,0,0,1,.3125,1]+[0,0,0,.125,1,1]+[.875,0,0,1,1,1]+[0,0,0,1,1,.125]+[0,0,.875,1,1,1],11:[0,0,0,1,.4,1]+[0,0,0,.125,1,1],12:[0,0,0,1,1,.1875],13:[0,0,0,.1875,1,1],14:[0,0,0,1,.1875,1],15:[0,0,0,.1875,1,1],16:[0,0,0,1,float('nan'),1],17:[0,.5,0,1,1,1],18:[.8,0,0,1,.5,1],19:[.800000011920929,0,0,1,.5,1],20:[1,1,1,0,0,0],21:[.5,0,.5,.5,.5,.5],22:[],23:[0,0,0,1,1.25,1]}
 add('shapeDef',0,'E');add('shapeDef',1,'F')
 for id,box in shape_boxes.items():add('shapeDef',id,raw(box)or'-')
 # State id/block/flags/fluid/open/half/facing/bounce/closed/opened/paired/neighbor-dependent shape.
 def state(id,block,shape=0,flags=0,fluid=0,opened=-1,half=-1,facing=-1,bounce=0,opened_shape=None,pair=-1,neighbor=-1):add('stateDef',id,block,flags,fluid,opened,half,facing,fbits(bounce),shape,shape if opened_shape is None else opened_shape,pair,neighbor)
 state(0,'air');state(1,'stone',1);state(2,'void_air');state(3,'stone',1);state(4,'stone',2);state(5,'stone',1,fluid=1);state(6,'stone',1,fluid=2)
 for id,s in [(7,3),(8,7),(9,8),(10,10),(11,11),(12,5),(16,4),(17,17),(18,18),(19,19),(41,16),(42,20),(43,21),(44,22),(45,23)]:state(id,'fixture_'+str(id),s)
 state(14,'slime_block',1,bounce=1);state(15,'red_bed',6,bounce=.66)
 for id,block,flags,closed,opened_shape,half in [(20,'oak_door',3,12,13,0),(22,'oak_door',3,12,13,1),(24,'iron_door',1,12,13,0),(26,'oak_fence_gate',4,9,0,-1),(28,'oak_trapdoor',8,14,15,-1),(30,'iron_trapdoor',8,14,15,-1)]:
  state(id,block,closed,flags,opened=0,half=half,opened_shape=opened_shape,pair=id+1);state(id+1,block,closed,flags,opened=1,half=half,opened_shape=opened_shape,pair=id)
 state(32,'ladder',flags=16,facing=2);state(33,'ladder',flags=16,facing=5);state(34,'vine');state(35,'fixture_fire',flags=32);state(36,'water',fluid=1);state(37,'lava',fluid=2);state(38,'cobweb');state(39,'cactus',1);state(40,'fixture_connecting',0,neighbor=8)
 for i,name in enumerate(['ladder','vine','weeping_vines','weeping_vines_plant','twisting_vines','twisting_vines_plant','cave_vines','cave_vines_plant']):state(70+i,name,flags=16 if i==0 else 0,facing=2 if i==0 else -1)
 for i,name in enumerate(['cobweb','sweet_berry_bush','powder_snow','wither_rose','magma_block','campfire','soul_campfire','cactus']):state(80+i,name)
 def new(opens=False,width=struct.unpack('<f',struct.pack('<f',.6))[0],height=struct.unpack('<f',struct.pack('<f',1.8))[0],minimum=-64,count=384,border=(-100,100),legacy=False,floor=-1,floor_state=1):add('new',str(opens).lower(),bits(width),bits(height),minimum,count,bits(border[0]),bits(border[1]),str(legacy).lower(),floor,floor_state)
 def setat(x,y,z,id):add('set',pos(x,y,z),id)
 # Floors, partial floors, footprint boundaries, bowls, headroom and crouch.
 for floor_state in [0,1,7,8,9,10,11,12,14,15,16,17,18,19,38,41,42,43,44,45]:
  for ceiling in [0,1,17]:
   new();setat(0,0,0,floor_state);setat(0,1,0,ceiling)
   add('stand','0,0,0');add('stand','0,0,0');add('stand','0,1,0');add('support',raw([.5,0,.5]));add('support',raw([.5,.5,.5]));add('shape','0,0,0')
 # Width/height are immutable constructor facts, including zero and narrow bodies.
 for width,height in [(0,0),(.2,1.5),(.6,1.8),(1,2),(1.4,.9)]:
  new(width=width,height=height);setat(0,0,0,10);add('limits');add('stand','0,0,0');add('box',raw([.5,0,.5]),bits(height))
 for point in [[-0.0,-0.0,-0.0],[.5,-1,.5],[float('nan'),0,0],[float('inf'),0,float('-inf')]]:add('box',raw(point),bits(1.5))
 # Exact known/loading/border precedence; cached values survive until invalidation.
 new(border=(-.5,.5));add('limits')
 for q in ['0,0,0','1,0,0','0,-65,0','0,320,0']:
  add('state',q);add('shape',q);add('stand',q)
 add('known','0,0,0','false');add('flags','0,0,0');add('stand','0,0,0');add('invalidate','0,0,0',0);add('shape','0,0,0');add('known','0,0,0','true');add('invalidate','0,0,0',1);add('border',bits(-100),bits(100));add('shape','0,0,0')
 for minimum,count,y in [(2147483646,2,2147483647),(-2147483648,2,-2147483648),(100,0,100)]:
  new(minimum=minimum,count=count,floor=y-1 if y>-2147483648 else 2147483647);setat(0,y,0,7);add('limits');add('stand',pos(0,y))
 # All hazard/climbable references and fluid distinctions; ladder facing and grip priority.
 for id in [32,33,34,35,36,37,38,39,*range(70,78),*range(80,88)]:
  new();setat(0,0,0,id);add('flags','0,0,0');add('stand','0,0,0');add('clear',raw([.2,.1,.2,.8,1.7,.8]))
 for side in [(0,0,-1),(0,0,1),(-1,0,0),(1,0,0),(0,1,-1),(0,1,1),(-1,1,0),(1,1,0)]:
  new();setat(0,0,0,34);setat(*side,1);add('flags','0,0,0');setat(0,0,-1,35);add('invalidate','0,0,-1',35);add('flags','0,0,0')
 for id,top in [(14,1),(14,1.0625),(15,.5625)]:
  new(floor_state=id)
  for offset in [-1,0,1e-8,.2,.20000000298023224,.20000001,.5,1,float('nan')]:add('bounce','0,0,0',bits(top-1+offset))
 # Openable kinds, floor-trapdoor exception, iron variants, lower/upper de-duplication.
 body=raw([.2,0,-.2,.8,1.8,.4]);outside=raw([.2,0,-1,.8,1.8,-.4]);from_inside=body
 for opens in [False,True]:
  for id in [20,21,22,23,24,25,26,27,28,29,30,31]:
   for condition in ['floor','climb','ceiling']:
    new(opens=opens);setat(0,0,0,id)
    if condition=='climb':setat(0,-1,0,34)
    if condition=='ceiling':setat(0,1,0,1)
    add('shape','0,0,0');add('shape','0,0,0');add('closed',body,outside,'-');add('closed',body,from_inside,'-');add('closed',body,outside,'0,0,0');add('stand','0,0,0')
 new(opens=True);setat(0,0,0,20);setat(0,1,0,22);add('closed',body,outside,'-');add('change','0,0,0',21);add('change','0,1,0',23);add('closed',body,outside,'-');add('shape','0,0,0');add('shape','0,1,0')
 # Trapdoor dependence at -2/-1/+1; stand invalidation reaches two layers.
 for dy in [-2,-1,1,2]:
  new(opens=True);setat(0,dy,0,28);add('shape',pos(0,dy));add('stand',pos(0,dy));add('stand',pos(0,dy+1));add('change','0,0,0',1);add('shape',pos(0,dy));add('stand',pos(0,dy));add('invalidate','0,0,0',1)
 for old,new_id in [(1,1),(1,3),(1,4),(1,5),(1,6),(36,37),(20,21),(21,20),(28,29),(35,0),(0,38),(40,40)]:
  for sample in ['state','shape','stand']:
   new(opens=True);setat(0,0,0,old);add(sample,'0,0,0');add('change','0,0,0',new_id);add(sample,'0,0,0');add('invalidate','0,0,0',new_id)
 new();add('invalidate','5,5,5',1);setat(0,0,0,40);add('shape','0,0,0');setat(1,0,0,1);add('shape','0,0,0');add('invalidate','0,0,0',4);add('shape','0,0,0')
 # Block iteration x/z/y order, touching boundaries, liquid early return and sweep axis order.
 for legacy in [False,True]:
  for obstacle in [(1,0,0),(0,0,1),(1,0,1),(0,2,0)]:
   new(legacy=legacy);setat(*obstacle,1)
   for endpoint in [[1.5,0,2.5],[2.5,0,1.5],[1.5,0,1.5],[.5,1,.5],[.5,2,.5],[-.5,0,-.5],[.5,0,.5]]:add('sweep',raw([.5,0,.5]),raw(endpoint))
 for id in [0,1,9,10,36,37,38]:
  new();setat(0,0,0,id)
  for box in [[0,0,0,1,1,1],[.2,0,.2,.8,1.8,.8],[.2,.3125,.2,.8,1.8,.8],[1,0,0,2,1,1],[0,1,0,1,2,1],[0,0,0,0,0,0],[-.1,-.1,-.1,1.1,1.1,1.1]]:add('clear',raw(box))
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'pathfinder/terrain'/name for name in ['Terrain.kt','StandHeight.kt','BlockShapes.kt','BlockKinds.kt','DestinationCheck.kt']]
 for file in original:shutil.copyfile(file,out/file.name)
 shutil.copyfile(HERE/'navigation-source-fixture/Terrain.kt',out/'Fixture.kt');kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-13000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');shutil.copyfile(HERE/'navigation-source-fixture/terrain.rs',out/'terrain_oracle.rs');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').glob('*.rs')}
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod terrain_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-terrain-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,Path(__file__),HERE/'navigation-source-fixture/Terrain.kt',HERE/'navigation-source-fixture/terrain.rs'];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'five unchanged Terrain sources; known/border precedence, cached block/shape facts, stand/crouch/partial footprint/bowls, doors/gates/trapdoors, hazards/climb/restitution, ordered movement collision and invalidation','boundaries':['controlled observable block/registry/world adapter; real MC registry extraction and native snapshot are not exercised','terrain only; search controllers, player simulation and native route input remain incomplete'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'cases':len(commands),'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
