#!/usr/bin/env python3
"""Untouched SafeWalk + RefreshableRangeValue with declared live-query providers."""
import hashlib
import itertools
import json
import os
import shutil
import struct
import subprocess
from pathlib import Path

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
OUT=ROOT/'build/safe-walk-tests'

def d(v):return str(struct.unpack('>Q',struct.pack('>d',v))[0])
def f(v):return str(struct.unpack('>I',struct.pack('>f',v))[0])
def vec(v):return list(map(d,v))
def snapshot(speed=.2,ground=True,edge=True,fall=False,pos=(.85,1.,.5),next=(1.1,1.,.5),yaw=0.):
 return dict(op='snapshot',speed=d(speed),ground=ground,edge=edge,fall=fall,pos=vec(pos),next=vec(next),yaw=f(yaw))
def input(mask=1,jump=False,sneak=False):return dict(op='input',mask=mask,jump=jump,sneak=sneak)
def inputs():
 rows=[]
 for mode,speed,mask,jump in itertools.product(['Stop','Invert','Center'],[.0,.05,.051,.2],range(16),[False,True]):
  rows.append(dict(seed='123',steps=[dict(op='settings',edgeMode=mode),snapshot(speed=speed),input(mask,jump),snapshot(speed=speed,edge=False,fall=True),input(mask,jump),input(mask,jump),input(mask,jump)]))
 for ground,edge,sneak,keep,shift in itertools.product([False,True],[False,True],[False,True],[[0,0],[1,1],[3,3],[-1,-1]],[[0,0],[2,3]]):
  rows.append(dict(seed='-1',steps=[dict(op='settings',keep=keep,sneak=shift,jump=True),snapshot(ground=ground,edge=edge),input(5,False,sneak),input(10,True,False),dict(op='disable'),input(),dict(op='mode',value='Safe'),dict(op='safe',value=False),dict(op='mode',value='None'),dict(op='safe',value=False)]))
 for mode,closer,yaw in itertools.product(['Stop','Invert','Center'],[True,False],[-720.,-180.,-70.,0.,70.,180.,540.,float('nan')]):
  rows.append(dict(seed='0',steps=[dict(op='settings',edgeMode=mode,keep=[3,3],sneak=[3,3]),snapshot(yaw=yaw,next=(.6 if closer else 1.3,1.,.5)),dict(op='state',center=vec((.5,1.,.5)),overwrite=2,sneaks=2),input(6,True),snapshot(yaw=yaw,edge=False),input(9),dict(op='running',value=False),input(),dict(op='running',value=True),input()]))
 for ranges in [[[.1,.15],[.1,.15],[.2,.25]],[[.5,.1],[.7,.3],[0.,0.]],[[0.,0.],[-0.,0.],[.1,.15]]]:
  steps=[]
  for lo,hi in ranges:steps.extend([dict(op='settings',distance=[f(lo),f(hi)]),snapshot(),input()])
  rows.append(dict(seed='81985529216486895',steps=steps))
 rows.append(dict(seed='123',steps=[dict(op='settings',sneak=[5,5],keep=[3,3]),snapshot(),input(),dict(op='running',value=False),dict(op='mode',value='Safe'),dict(op='mode',value='OnEdge'),dict(op='running',value=True),snapshot(ground=False,edge=False),input(),input(),dict(op='disable'),input()]))
 return rows

def block(text,anchor):
 start=text.index(anchor);cursor=text.index('{',start)+1;depth=1
 while depth:
  if text[cursor]=='{':depth+=1
  elif text[cursor]=='}':depth-=1
  cursor+=1
 return text[start:cursor]

def main():
 work=OUT/'original';classes=work/'classes';classes.mkdir(parents=True,exist_ok=True)
 base=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
 originals=[base/'features/module/modules/movement/ModuleSafeWalk.kt',base/'config/utils/RefreshableRangeValue.kt']
 for p in originals:shutil.copyfile(p,work/p.name)
 for p in (HERE/'oracles/safe-walk').glob('*.kt'):shutil.copyfile(p,work/p.name)
 movement=(base/'utils/movement/MovementUtils.kt').read_text(encoding='utf-8')
 functions='package net.ccbluex.liquidbounce.utils.movement\nimport net.minecraft.util.Mth\nimport net.minecraft.world.phys.Vec3\nimport net.ccbluex.liquidbounce.utils.math.yaw\nimport net.ccbluex.liquidbounce.utils.aiming.RotationManager\nimport net.ccbluex.liquidbounce.utils.client.player\n'
 functions+=block(movement,'fun getDegreesRelativeToView(')+'\n'+block(movement,'fun getDirectionalInputForDegrees(')+'\n'
 (work/'MovementFunctions.kt').write_text(functions,encoding='utf-8')
 arrays=(base/'utils/kotlin/ArrayExtensions.kt').read_text(encoding='utf-8')
 (work/'ArrayRandom.kt').write_text('package net.ccbluex.liquidbounce.utils.kotlin\nimport java.util.concurrent.ThreadLocalRandom\nimport kotlin.math.nextUp\n'+block(arrays,'fun ClosedFloatingPointRange<Float>.random()')+'\n',encoding='utf-8')
 vectors=inputs();(OUT/'inputs.json').write_text(json.dumps(vectors),encoding='utf-8')
 cache=Path(os.environ['USERPROFILE'])/'.gradle/caches';gradle=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';jdk=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
 jars=[*gradle.glob('kotlin-stdlib-*.jar'),*(cache/'modules-2/files-2.1/com.google.code.gson').rglob('*.jar')]
 cp=os.pathsep.join(map(str,[classes,*jars]));commands=[]
 for name,argv in [('compile',[str(jdk/'java.exe'),'-cp',str(gradle/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(classes),*map(str,work.glob('*.kt'))]),('run',[str(jdk/'java.exe'),'--add-opens','java.base/java.lang=ALL-UNNAMED','-cp',cp,'fixture.Fixture',str(OUT/'inputs.json'),str(OUT/'oracle.json')])]:
  r=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace');(OUT/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr,end='');commands.append(dict(name=name,exitCode=r.returncode))
  if r.returncode:raise SystemExit(r.returncode)
 metadata=dict(cases=len(vectors),commands=commands,unchangedOriginalKotlin=all(p.read_bytes()==(work/p.name).read_bytes()for p in originals),originalSources={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in originals},exactOriginalFunctionFragments=['MovementUtils.getDegreesRelativeToView','MovementUtils.getDirectionalInputForDegrees','ArrayExtensions.FloatRange.random'],oracleSha256=hashlib.sha256((OUT/'oracle.json').read_bytes()).hexdigest(),boundaries=['unchanged ModuleSafeWalk and RefreshableRangeValue with real JDK25 ThreadLocalRandom/Kotlin Random.Default, per-step underlying seed recorded','live player scalars, geometry/simulation query results and debug sinks are explicit providers','module/group registration and mode lifecycle are explicit providers','not complete original-client execution or validation of live simulation/geometry binding'])
 (OUT/'oracle-metadata.json').write_text(json.dumps(metadata,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(cases=len(vectors),passed=True)))

if __name__=='__main__':main()
