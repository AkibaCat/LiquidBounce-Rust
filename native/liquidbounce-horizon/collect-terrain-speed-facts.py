#!/usr/bin/env python3
"""Run all unchanged TerrainSpeed Kotlin groups in an explicit event/provider shell."""
import hashlib
import itertools
import json
import os
from pathlib import Path
import runpy
import shutil
import struct
import subprocess
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = ROOT / 'build/terrain-speed-tests'


def mutable_vec3(data):
    """Apply only original accesswidener's mutable Vec3 x/y/z fields."""
    data=bytearray(data)
    u2=lambda offset:struct.unpack_from('>H',data,offset)[0]
    count=u2(8);cursor=10;index=1;utf={}
    while index<count:
        tag=data[cursor];cursor+=1
        if tag==1:
            size=u2(cursor);cursor+=2;utf[index]=bytes(data[cursor:cursor+size]).decode();cursor+=size
        elif tag in [3,4,9,10,11,12,17,18]:cursor+=4
        elif tag in [5,6]:cursor+=8;index+=1
        elif tag in [7,8,16,19,20]:cursor+=2
        elif tag==15:cursor+=3
        else:raise ValueError(f'unknown constant pool tag {tag}')
        index+=1
    cursor+=6
    interfaces=u2(cursor);cursor+=2+interfaces*2
    fields=u2(cursor);cursor+=2;patched=[]
    for _ in range(fields):
        access=cursor;name=utf[u2(cursor+2)];desc=utf[u2(cursor+4)];attributes=u2(cursor+6);cursor+=8
        if name in ['x','y','z'] and desc=='D':
            struct.pack_into('>H',data,access,u2(access)&~0x10);patched.append(name)
        for _ in range(attributes):
            size=struct.unpack_from('>I',data,cursor+2)[0];cursor+=6+size
    assert patched==['x','y','z'],patched
    return bytes(data)


def inputs():
    base=dict(event='tick',running=True,velocity=[-0.0,0.2,-0.4],position=[2.25,-3.5,4.75],
        collision=True,climbable=True,water=True,direction=2,yaw=0,input=1,moving=True,
        forward=True,sprintKey=False,jumpKey=False,sneakKey=False,pattern=0,
        climbEnabled=True,climbMode='Motion',climbMotion=0.2872,iceEnabled=True,
        slipperiness=0.6,iceMotionEnabled=False,iceMotion=0.5,waterEnabled=True,
        autoSwim=True,baseHorizontal=0.44,baseVertical=0.44,sprintEnabled=True,
        sprintHorizontal=1.0,sprintVertical=1.0,mover=0,movement=[0.3,-0.0,-0.4],jump=False,
        block=0,friction=0.98,repeat=1)
    result=[]
    for mode in ['Motion','Clip']:
        for index,flags in enumerate(itertools.product([False,True],repeat=5)):
            for direction in range(6):
                for pattern in range(6):
                    row=base|dict(event='move',climbMode=mode,running=flags[0],climbEnabled=flags[1],collision=flags[2],climbable=flags[3],forward=flags[4],direction=direction,pattern=pattern,
                        climbMotion=[0.2872,-2.5,-0.0,1e-30,8.25][(index+pattern)%5],mover=(index+pattern)%5)
                    result.append(row)
    for flags in itertools.product([False,True],repeat=6):
        result.append(base|dict(event='input',running=flags[0],waterEnabled=flags[1],autoSwim=flags[2],water=flags[3],sneakKey=flags[4],jump=flags[5]))
    for index,flags in enumerate(itertools.product([False,True],repeat=8)):
        for input_flags in range(16):
            variant=(index+input_flags)%5
            result.append(base|dict(event='tick',running=flags[0],waterEnabled=flags[1],water=flags[2],sprintKey=flags[3],sprintEnabled=flags[4],jumpKey=flags[5],sneakKey=flags[6],moving=flags[7],input=input_flags,
                yaw=[0,-90,361.25,-725.5,1e9][variant],baseHorizontal=[0.44,-2.5,0,1e-30,12.5][variant],baseVertical=[0.44,7.25,-0.0,-3.75,1e-30][variant],sprintHorizontal=[1,-1.5,3.25,-0.0,8][variant],sprintVertical=[1,-7.5,0,1e-30,2.25][variant]))
    for flags in itertools.product([False,True],repeat=3):
        for block in range(6):
            for repeat in [1,2,4]:
                for motion,slipperiness in [(0.5,0.6),(-2.5,1.25),(-0.0,-3.75),(1e-30,0.99)]:
                    result.append(base|dict(event='friction',running=flags[0],iceEnabled=flags[1],iceMotionEnabled=flags[2],block=block,repeat=repeat,iceMotion=motion,slipperiness=slipperiness))
    return result


def main():
    OUT.mkdir(parents=True,exist_ok=True)
    work=OUT/'original';work.mkdir(exist_ok=True)
    classes=work/'classes';classes.mkdir(exist_ok=True)
    upstream=ROOT/'upstream/LiquidBounce/src/main'
    kotlin_base=upstream/'kotlin/net/ccbluex/liquidbounce'
    terrain=kotlin_base/'features/module/modules/movement/terrainspeed'
    originals=[terrain/'ModuleTerrainSpeed.kt',terrain/'fastclimb/FastClimb.kt',terrain/'icespeed/IceSpeed.kt',terrain/'waterspeed/WaterSpeed.kt',kotlin_base/'utils/movement/DirectionalInput.kt']
    for source in originals:shutil.copyfile(source,work/source.name)
    shell=HERE/'oracles/terrain-speed'
    for source in shell.iterdir():shutil.copyfile(source,work/source.name)
    block=runpy.run_path(str(HERE/'test-attack-patch.py'))['block']
    entity=kotlin_base/'utils/entity/EntityExtensions.kt';source=entity.read_text(encoding='utf-8')
    helpers='''package net.ccbluex.liquidbounce.utils.entity
import fixture.FixtureState
import net.ccbluex.liquidbounce.features.addon.AddonApi
import net.ccbluex.liquidbounce.features.module.FixtureInput
import net.ccbluex.liquidbounce.utils.movement.DirectionalInput
import net.minecraft.client.player.ClientInput
import net.minecraft.client.player.LocalPlayer
import net.minecraft.world.phys.Vec2
import net.minecraft.world.phys.Vec3
import kotlin.math.sin
import kotlin.math.cos
val player get()=FixtureState.player
val ClientInput.untransformed get()=(this as FixtureInput).original
'''
    helpers+=source[source.index('val LocalPlayer.moving'):source.index('val ClientInput.untransformed')]
    extracted={anchor:block(source,anchor) for anchor in ['fun LocalPlayer.getMovementDirectionOfInput(', 'fun getMovementDirectionOfInput(', 'fun Vec3.withStrafe(']}
    helpers+='\n'.join(extracted.values())+'\n'
    (work/'EntityHelpers.kt').write_text(helpers,encoding='utf-8')
    math=kotlin_base/'utils/math/MinecraftVectorExtensions.kt'
    copy=next(line for line in math.read_text(encoding='utf-8').splitlines() if line.startswith('inline fun Vec3.copy('))
    (work/'Math.kt').write_text('package net.ccbluex.liquidbounce.utils.math\nimport net.minecraft.world.phys.Vec3\n'+copy+'\n',encoding='utf-8')
    (work/'AddonApi.kt').write_text('package net.ccbluex.liquidbounce.features.addon\nannotation class AddonApi\n',encoding='utf-8')
    (work/'Blocks.kt').write_text('package net.ccbluex.liquidbounce.utils.block\nfun net.minecraft.core.BlockPos.getBlock()=fixture.FixtureState.block(this)\n',encoding='utf-8')
    (work/'Fastutil.kt').write_text('''package net.ccbluex.fastutil
fun <T> referenceHashSetOf(vararg values:T):Set<T> = java.util.Collections.newSetFromMap(java.util.IdentityHashMap<T,Boolean>()).apply{addAll(values)}
''',encoding='utf-8')
    cases=inputs();(OUT/'inputs.json').write_text(json.dumps(cases),encoding='utf-8')
    cache=Path(os.environ['USERPROFILE'])/'.gradle/caches'
    minecraft=cache/'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    widener=upstream/'resources/liquidbounce.accesswidener'
    for axis in ['x','y','z']:assert f'mutable field net/minecraft/world/phys/Vec3 {axis} D' in widener.read_text(encoding='utf-8')
    with zipfile.ZipFile(minecraft) as archive:original_vec3=archive.read('net/minecraft/world/phys/Vec3.class')
    patched_vec3=mutable_vec3(original_vec3)
    vecpath=classes/'net/minecraft/world/phys/Vec3.class';vecpath.parent.mkdir(parents=True,exist_ok=True);vecpath.write_bytes(patched_vec3)
    jars=[minecraft]
    for group in ['com.mojang','com.google.guava','com.google.code.gson','it.unimi.dsi','org.slf4j','org.joml','io.netty','org.apache.logging.log4j','org.apache.commons','org.lwjgl']:
        jars.extend((cache/'modules-2/files-2.1'/group).rglob('*.jar'))
    jdk=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    jars.append(next(kotlin.glob('kotlin-stdlib-*.jar')))
    classpath=os.pathsep.join(map(str,[classes,*jars]))
    commands=[
        ('java-compile',[str(jdk/'javac.exe'),'-cp',classpath,'-d',str(classes),str(work/'FixtureState.java')]),
        ('kotlin-compile',[str(jdk/'java.exe'),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',classpath,'-d',str(classes),*map(str,sorted(work.glob('*.kt')))]),
        ('original-oracle',[str(jdk/'java.exe'),'-cp',classpath,'MainKt',str(OUT/'inputs.json'),str(OUT/'oracle.json'),str(OUT/'minecraft-facts.json')]),
    ]
    for name,command in commands:
        result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace')
        (OUT/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
        if result.returncode:raise SystemExit(result.stdout+result.stderr)
    rows=json.loads((OUT/'oracle.json').read_text(encoding='utf-8'))
    source_files=[*originals,entity,math,widener,Path(__file__),*shell.iterdir()]
    metadata={'cases':len(rows),'eventCounts':{event:sum(row['input']['event']==event for row in rows) for event in ['move','input','tick','friction']},
        'unchangedOriginalKotlin':all(p.read_bytes()==(work/p.name).read_bytes() for p in originals),
        'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in source_files},
        'extractedHelpersSha256':{anchor:hashlib.sha256(text.encode()).hexdigest() for anchor,text in extracted.items()},
        'originalVec3ClassSha256':hashlib.sha256(original_vec3).hexdigest(),'accessWidenedVec3ClassSha256':hashlib.sha256(patched_vec3).hexdigest(),
        'minecraftSha1':hashlib.sha1(minecraft.read_bytes()).hexdigest(),
        'boundaries':['event registration/running, settings, key and block providers are shells','actual MC LocalPlayer subclass, Vec3, Input, BlockPos, block classes and Direction','actual Entity.absSnapTo executes; setPos is recording storage provider','81 actual MC absSnapTo boundary cases; 15 Direction.fromYRot cases; complete LadderBlock/VineBlock registry class identity set','Vec3 field mutability matches original exact accesswidener; method bytecode unchanged','not original full-client execution']}
    (OUT/'oracle-metadata.json').write_text(json.dumps(metadata,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({key:metadata[key] for key in ['cases','eventCounts','unchangedOriginalKotlin']}))


if __name__=='__main__':main()
