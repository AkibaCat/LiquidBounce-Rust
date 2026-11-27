#!/usr/bin/env python3
"""AABB/Clip/VoxelShape source parity for navigation's actual collision primitives."""
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
def raw(values):return ','.join(str(bits(float(v)))for v in values)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);records=[];commands=[]
 def add(*p):commands.append(' '.join(map(str,p)))
 boxes=[[0,0,0,1,1,1],[-1,-2,-3,4,5,6],[1,1,1,0,0,0],[0,0,0,0,0,0],[-0.0,0,-0.0,1,1,1],[0,0,0,1,0.0625,1],[0.4,0,0.4,0.6,1.5,0.6],[float('nan'),0,0,1,1,1],[-float('inf'),-1,-1,float('inf'),1,1],[1e300,-1e300,0,1e300,1e300,1]]
 points=[[0,0,0],[1,1,1],[.5,.5,.5],[-1,0,.5],[2,0,.5],[0,-1,0],[0,2,0],[0,0,-1],[0,0,2],[float('nan'),float('inf'),float('-inf')]]
 offsets=[[0,0,0],[1,2,3],[-1,-2,-3],[1e-8,-1e-8,1e-7],[1e300,0,-1e300]]
 for i,box in enumerate(boxes):
  for j,point in enumerate(points):add('box',raw(box),raw(boxes[(i+j+1)%len(boxes)]),raw(point),raw(offsets[j%len(offsets)]))
 # Positive/negative near-zero motion, exact touches and each collision axis.
 shapes=[[],boxes[0],boxes[1],boxes[2],boxes[3],boxes[4],boxes[5],boxes[6],boxes[7],boxes[8],boxes[0]+boxes[6],boxes[6]+boxes[0]]
 distances=[-2.0,-1.0,-1e-7,-1e-8,-0.0,0.0,1e-8,1e-7,1.0,2.0,float('nan'),float('inf')]
 for i,shape in enumerate(shapes):
  for axis in range(3):
   for j,distance in enumerate(distances):
    body=[-.3,-.9,-.3,.3,.9,.3];body[axis]=-1.6 if distance>=0 else 1.1;body[axis+3]=-.1 if distance>=0 else 1.7
    add('shape',raw(shape)or'-',axis,raw(body),bits(distance),raw(offsets[j%len(offsets)]),raw(shape)or'-')
 # Multi-box nearest hit, equal-time faces, endpoint/inside rejection and tolerance boundaries.
 for point in points[:9]:
  for end in [[.5,.5,.5],[2,2,2],[-2,-2,-2],point]:add('ray',raw(point),raw(end),';'.join(raw(b)for b in [boxes[0],boxes[6],boxes[1]]))
 for epsilon in [0,0.5e-7,1e-7,1.5e-7,-0.5e-7,-1e-7,-1.5e-7]:add('ray',raw([-1,epsilon,.5]),raw([2,epsilon,.5]),raw(boxes[0]))
 for box in boxes[:7]:add('identity',raw(box))
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=[AGENT/'math/AABB.kt',AGENT/'world/block/VoxelShape.kt']
 for p in original:shutil.copyfile(p,out/p.name)
 shutil.copyfile(HERE/'navigation-source-fixture/Shapes.kt',out/'Fixture.kt');kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-began,3)});(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
  if r.returncode:print((r.stdout+r.stderr)[-11000:])
  return r
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(out/'classes'),*[str(p)for p in out.glob('*.kt')]])
 if result.returncode==0:result=run('oracle',[str(java),'-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 shutil.copytree(ROOT/'native/liquidbounce-behaviors/src/navigation',out/'navigation');shutil.copyfile(HERE/'navigation-source-fixture/shapes.rs',out/'shapes_oracle.rs');snapshot={str(p.relative_to(out/'navigation')):hashlib.sha256(p.read_bytes()).hexdigest()for p in(out/'navigation').glob('*.rs')}
 (out/'lib.rs').write_text('#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod shapes_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-navigation-shape-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if result.returncode==0:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-pose-hook-tests/target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if result.returncode==0:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,Path(__file__),HERE/'navigation-source-fixture/Shapes.kt',HERE/'navigation-source-fixture/shapes.rs'];report={'passed':len(records)==4 and all(r['exitCode']==0 for r in records),'cases':len(commands),'commands':records,'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'compiledNavigationSnapshotSha256':snapshot,'scope':'two unchanged Agent source files, AABB normalization/operations, ordered ray clipping, raw shape boxes, collision axes/tolerances and identity','boundaries':['primitives only; terrain sampling, player motion and route/controller integration are not exercised'],'fullAutoWalkImplemented':False,'fullGameIntegrationPassed':False}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
